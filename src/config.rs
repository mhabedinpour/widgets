use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub wifi: WifiConfig,
    pub display: DisplayConfig,
    pub widgets: Vec<WidgetEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WifiConfig {
    pub ssid: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DisplayConfig {
    pub freq_mhz: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WidgetEntry {
    pub id: usize,
    pub r#type: String,
    #[serde(default)]
    pub x: u32,
    #[serde(default)]
    pub y: u32,
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub config: BTreeMap<String, String>,
}
