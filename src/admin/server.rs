use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use embassy_net::Stack;
use embassy_net::tcp::TcpSocket;
use embassy_time::Instant;
use log::{error, info};

use super::{SystemStatusResponse, WidgetItem};
use crate::admin::file::FileResponse;
use crate::storage::FS;
use crate::widget::manager::WidgetManager;
use picoserve::response::IntoResponse;
use picoserve::response::json::Json;
use picoserve::routing::{PathRouter, get, post};
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
                get(|| FileResponse::from(self.fs.clone(), "/admin/index.html")),
            )
            .route("/api/status", get(|| self.handle_status()))
            .route("/api/widgets", get(|| self.handle_get_widgets()))
            .route("/api/reboot", post(|| self.handle_reboot()))
    }

    async fn handle_status(&self) -> impl IntoResponse {
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
        Json(response)
    }

    async fn handle_get_widgets(&self) -> impl IntoResponse {
        let items = self
            .manager
            .borrow()
            .widgets()
            .iter()
            .map(|(id, w)| WidgetItem {
                id: id.0,
                r#type: w.r#type.clone(),
                x: w.placement.origin.x as i32,
                y: w.placement.origin.y as i32,
                width: w.placement.size.width,
                height: w.placement.size.height,
                config: w.config.clone(),
            })
            .collect::<Vec<_>>();
        Json(items)
    }

    async fn handle_reboot(&self) -> Json<&'static str> {
        info!("Reboot requested via REST API!");
        esp_hal::system::software_reset()
    }
}

#[embassy_executor::task]
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
