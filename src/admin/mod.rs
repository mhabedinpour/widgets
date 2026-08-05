mod file;
pub mod server;

use alloc::string::String;
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
