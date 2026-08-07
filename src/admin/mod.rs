mod file;
mod json;
pub mod server;

use crate::admin::json::Json;
use alloc::string::String;
use embedded_io_async::Read;
use picoserve::ResponseSent;
use picoserve::response::{Connection, IntoResponse, Response, ResponseWriter, StatusCode};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct SystemStatusResponse {
    pub status: &'static str,
    pub uptime_ms: u64,
    pub ip: String,
    pub free_heap: usize,
    pub free_psram: usize,
    pub widget_count: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct ApiError {
    pub error: String,
}

pub struct ApiErrorResponse(pub StatusCode, pub ApiError);

impl IntoResponse for ApiErrorResponse {
    async fn write_to<R: Read, W: ResponseWriter<Error = R::Error>>(
        self,
        connection: Connection<'_, R>,
        writer: W,
    ) -> Result<ResponseSent, W::Error> {
        let payload = match serde_json::to_vec(&self.1) {
            Ok(bytes) => bytes,
            Err(_) => {
                return StatusCode::INTERNAL_SERVER_ERROR
                    .write_to(connection, writer)
                    .await;
            }
        };

        Response::new(self.0, payload)
            .with_header("Content-Type", "application/json")
            .write_to(connection, writer)
            .await
    }
}

pub type ApiResult<T> = Result<Json<T>, ApiErrorResponse>;

pub fn error_response(status: StatusCode, message: impl Into<String>) -> ApiErrorResponse {
    ApiErrorResponse(
        status,
        ApiError {
            error: message.into(),
        },
    )
}
