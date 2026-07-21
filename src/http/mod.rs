pub mod service;

pub use service::{HttpService, WidgetHttpClient};

use crate::widget::WidgetId;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RequestId(pub u32);

/// @wasm required="method,url,body,headers"
pub struct HttpRequestData {
    pub method: String,
    pub url: String,
    pub body: String,
    pub headers: Vec<(String, String)>,
}

pub struct HttpResponse {
    pub request_id: RequestId,
    pub headers: Option<Vec<(String, String)>>,
    pub body: Option<String>,
    pub error: Option<HttpError>,
}

#[derive(Debug)]
pub enum HttpError {
    UnsupportedScheme,
    InvalidPort,
    DnsQueryFailed,
    NoDnsResult,
    ConnectFailed,
    WriteFailed,
    FlushFailed,
    TlsHandshakeFailed,
}

impl core::fmt::Display for HttpError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::UnsupportedScheme  => "unsupported scheme",
            Self::InvalidPort        => "invalid port",
            Self::DnsQueryFailed     => "DNS query failed",
            Self::NoDnsResult        => "no DNS result",
            Self::ConnectFailed      => "connect failed",
            Self::WriteFailed        => "write failed",
            Self::FlushFailed        => "flush failed",
            Self::TlsHandshakeFailed => "TLS handshake failed",
        })
    }
}

pub struct ParsedUrl<'a> {
    pub is_https: bool,
    pub host: &'a str,
    pub port: u16,
    pub path: &'a str,
}

pub fn parse_url(url: &str) -> Result<ParsedUrl<'_>, HttpError> {
    let (is_https, rest) = if let Some(r) = url.strip_prefix("https://") {
        (true, r)
    } else if let Some(r) = url.strip_prefix("http://") {
        (false, r)
    } else {
        return Err(HttpError::UnsupportedScheme);
    };

    let default_port: u16 = if is_https { 443 } else { 80 };

    let (host_port, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None    => (rest, "/"),
    };

    let (host, port) = match host_port.rfind(':') {
        Some(i) => {
            let port = host_port[i + 1..]
                .parse::<u16>()
                .map_err(|_| HttpError::InvalidPort)?;
            (&host_port[..i], port)
        }
        None => (host_port, default_port),
    };

    Ok(ParsedUrl { is_https, host, port, path })
}

/// @wasm
pub trait Http {
    /// @wasm builder_name="fetch"
    fn send_request(&mut self, data: HttpRequestData) -> RequestId;
}

pub trait GlobalHttpClient {
    fn poll(&mut self) -> Vec<(WidgetId, HttpResponse)>;
    fn scoped(&self, widget_id: WidgetId) -> Box<dyn Http>;
}
