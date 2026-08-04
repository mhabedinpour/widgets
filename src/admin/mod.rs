mod file;
pub mod server;

use alloc::collections::BTreeMap;
use alloc::string::String;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
pub struct SystemStatusResponse {
    pub status: &'static str,
    pub uptime_ms: u64,
    pub ip: String,
    pub free_heap: usize,
    pub free_psram: usize,
    pub widget_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WidgetItem {
    pub id: usize,
    pub r#type: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub config: BTreeMap<String, String>,
}
