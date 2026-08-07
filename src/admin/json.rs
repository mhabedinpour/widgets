use embedded_io_async::Read;
use picoserve::request::RequestBody;
use picoserve::response::{Connection, IntoResponse, Response, ResponseWriter};
use picoserve::{ResponseSent, extract::FromRequest, request::RequestParts, response::StatusCode};
use serde::{Deserialize, Serialize};

use crate::admin::ApiErrorResponse;
use crate::admin::error_response;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Json<T>(pub T);

impl<'a, State, T> FromRequest<'a, State> for Json<T>
where
    T: Deserialize<'a>,
{
    type Rejection = ApiErrorResponse;

    async fn from_request<R: Read>(
        _state: &'a State,
        _request_parts: RequestParts<'a>,
        body: RequestBody<'a, R>,
    ) -> Result<Self, Self::Rejection> {
        let bytes = body
            .read_all()
            .await
            .map_err(|_| error_response(StatusCode::BAD_REQUEST, "Failed to read request body"))?;

        let value = serde_json::from_slice(bytes)
            .map_err(|_| error_response(StatusCode::BAD_REQUEST, "Invalid JSON in request body"))?;

        Ok(Json(value))
    }
}

impl<T> IntoResponse for Json<T>
where
    T: Serialize,
{
    async fn write_to<R: Read, W: ResponseWriter<Error = R::Error>>(
        self,
        connection: Connection<'_, R>,
        writer: W,
    ) -> Result<ResponseSent, W::Error> {
        let payload = match serde_json::to_vec(&self.0) {
            Ok(bytes) => bytes,
            Err(_) => {
                return StatusCode::INTERNAL_SERVER_ERROR
                    .write_to(connection, writer)
                    .await;
            }
        };

        Response::ok(payload)
            .with_header("Content-Type", "application/json")
            .write_to(connection, writer)
            .await
    }
}
