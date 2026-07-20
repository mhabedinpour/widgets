pub mod http;

use crate::widget::{WidgetId};
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

/// @wasm required="id,method,url,body,headers"
pub struct HttpRequestData {
    pub id: u32,
    pub method: String,
    pub url: String,
    pub body: String,
    pub headers: String,
}

pub struct HttpResponse {
    pub request_id: u32,
    pub headers: Option<String>,
    pub body: Option<String>,
}

/// @wasm
pub trait Http {
    /// @wasm builder_name="fetch"
    fn send_request(&mut self, data: HttpRequestData);
}

pub trait GlobalHttpClient {
    fn poll(&mut self) -> Vec<(WidgetId, HttpResponse)>;
    fn scoped(&self, widget_id: WidgetId) -> Box<dyn Http>;
}
