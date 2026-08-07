use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use embassy_net::Stack;
use embassy_net::tcp::TcpSocket;
use embassy_time::Instant;
use littlefs2::io::Write;
use log::{error, info};

use super::{ApiResult, SystemStatusResponse, error_response};
use crate::admin::file::FileResponse;
use crate::admin::json::Json;
use crate::config::WidgetEntry;
use crate::storage::FS;
use crate::widget::WidgetId;
use crate::widget::manager::WidgetManager;
use picoserve::response::StatusCode;
use picoserve::routing::{PathRouter, get, parse_path_segment, post, put};
use picoserve::{Config as PicoserveConfig, Router};

pub struct Server {
    manager: Rc<RefCell<WidgetManager>>,
    stack: Stack<'static>,
    fs: FS,
}

impl Server {
    pub fn new(stack: Stack<'static>, fs: FS, manager: Rc<RefCell<WidgetManager>>) -> Self {
        Self { stack, fs, manager }
    }

    fn router(&self) -> Router<impl PathRouter<()>> {
        Router::new()
            .route(
                "/",
                get(move || FileResponse::from(self.fs.clone(), "/admin/index.html")),
            )
            .route("/api/status", get(move || self.handle_status()))
            .route(
                "/api/widgets",
                get(move || self.handle_get_widgets())
                    .post(move |Json(item): Json<WidgetEntry>| self.handle_add_widget(item)),
            )
            .route(
                ("/api/widgets", parse_path_segment::<usize>()),
                put({
                    move |id: usize, Json(item): Json<WidgetEntry>| {
                        self.handle_replace_widget(id, item)
                    }
                })
                .delete(move |id: usize| self.handle_remove_widget(id)),
            )
            .route(
                ("/api/upload", parse_path_segment::<String>()),
                post(move |name: String, body: Vec<u8>| self.handle_upload(name, body)),
            )
            .route("/api/reboot", post(move || self.handle_reboot()))
    }

    async fn handle_status(&self) -> ApiResult<SystemStatusResponse> {
        let ip = self
            .stack
            .config_v4()
            .map(|c| format!("{}", c.address.address()))
            .unwrap_or_else(|| String::from("127.0.0.1"));

        let widget_count = self.manager.borrow().widgets().len();

        let response = SystemStatusResponse {
            status: if self.stack.is_link_up() {
                "ok"
            } else {
                "disconnected"
            },
            uptime_ms: Instant::now().as_millis(),
            ip,
            free_heap: esp_alloc::HEAP.free_caps(esp_alloc::MemoryCapability::Internal.into()),
            free_psram: esp_alloc::HEAP.free_caps(esp_alloc::MemoryCapability::External.into()),
            widget_count,
        };
        Ok(Json(response))
    }

    async fn handle_get_widgets(&self) -> ApiResult<Vec<WidgetEntry>> {
        let items = self
            .manager
            .borrow()
            .widgets()
            .iter()
            .map(|(_id, w)| w.into())
            .collect::<Vec<WidgetEntry>>();
        Ok(Json(items))
    }

    async fn handle_add_widget(&self, item: WidgetEntry) -> ApiResult<usize> {
        if let Err(e) = item.validate() {
            return Err(error_response(StatusCode::BAD_REQUEST, e));
        }

        let mut manager = self.manager.borrow_mut();

        let max_id = manager.widgets().keys().map(|id| id.0).max().unwrap_or(0);
        let id = WidgetId(max_id + 1);
        let mut widget = item.as_widget(self.fs.clone());
        widget.id = id;

        manager.add_widget(widget);
        manager.flush();
        Ok(Json(id.0))
    }

    async fn handle_remove_widget(&self, id: usize) -> ApiResult<&'static str> {
        let mut manager = self.manager.borrow_mut();
        if !manager.widgets().contains_key(&WidgetId(id)) {
            return Err(error_response(
                StatusCode::NOT_FOUND,
                format!("Widget with ID {} not found", id),
            ));
        }

        manager.remove_widget(WidgetId(id));
        manager.flush();
        Ok(Json("ok"))
    }

    async fn handle_replace_widget(&self, id: usize, item: WidgetEntry) -> ApiResult<&'static str> {
        if let Err(e) = item.validate() {
            return Err(error_response(StatusCode::BAD_REQUEST, e));
        }

        let mut manager = self.manager.borrow_mut();
        if !manager.widgets().contains_key(&WidgetId(id)) {
            return Err(error_response(
                StatusCode::NOT_FOUND,
                format!("Widget with ID {} not found", id),
            ));
        }

        let mut widget = item.as_widget(self.fs.clone());
        widget.id = WidgetId(id);

        manager.replace_widget(widget);
        manager.flush();
        Ok(Json("ok"))
    }

    async fn handle_upload(&self, name: String, body: Vec<u8>) -> ApiResult<&'static str> {
        if name.is_empty() || name.contains("..") || name.contains('/') || name.contains('\\') {
            return Err(error_response(
                StatusCode::BAD_REQUEST,
                "Invalid filename: directory traversal or path segments not allowed",
            ));
        }

        if !name.ends_with(".wasm") {
            return Err(error_response(
                StatusCode::BAD_REQUEST,
                "Only .wasm files can be uploaded to the widgets directory",
            ));
        }

        let path_str = format!("/widgets/{}", name);
        let path = match littlefs2::path::PathBuf::try_from(path_str.as_str()) {
            Ok(p) => p,
            Err(_) => return Err(error_response(StatusCode::BAD_REQUEST, "Invalid filename")),
        };

        info!("Uploading {} ({} bytes)", path_str, body.len());

        let res = self.fs.create_file_and_then(&path, |file| {
            file.write_all(&body)?;
            Ok(())
        });

        match res {
            Ok(_) => Ok(Json("ok")),
            Err(e) => {
                error!("Failed to save uploaded file: {:?}", e.code());
                Err(error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to save uploaded file to storage",
                ))
            }
        }
    }

    async fn handle_reboot(&self) -> Json<&'static str> {
        info!("Reboot requested via REST API!");
        esp_hal::system::software_reset()
    }
}

#[embassy_executor::task(pool_size = 1)]
pub async fn admin_api_task(server: Server) -> ! {
    info!("Starting Picoserve REST API server & Admin Dashboard on port 8080...");

    let app = server.router();
    let config = PicoserveConfig::new(picoserve::Timeouts::default());

    let mut rx_buffer = [0u8; 2048];
    let mut tx_buffer = [0u8; 2048];

    loop {
        let mut socket = TcpSocket::new(server.stack, &mut rx_buffer, &mut tx_buffer);
        if let Err(e) = socket.accept(8080).await {
            error!("Admin API accept error: {:?}", e);
            continue;
        }

        let mut http_buffer = [0u8; 2048];
        let _ = picoserve::Server::new(&app, &config, &mut http_buffer)
            .serve(socket)
            .await;
    }
}
