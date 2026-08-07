use crate::drawer::{Point, Rect, Size};
use crate::storage::FS;
use crate::widget::executor::wasm::WasmExecutor;
use crate::widget::{Widget, WidgetId};
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cell::RefCell;
use littlefs2::io::Write;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub wifi: WifiConfig,
    pub display: DisplayConfig,
    pub widgets: RefCell<Vec<WidgetEntry>>,
}

const CONFIG_FILE: &str = "/config.json";

impl Config {
    pub fn save(&self, fs: FS) -> Result<(), littlefs2::io::Error> {
        let json = serde_json::to_vec(self).map_err(|_| littlefs2::io::Error::IO)?;
        let path = littlefs2::path::PathBuf::try_from(CONFIG_FILE).unwrap();
        fs.open_file_with_options_and_then(
            |opts| {
                opts.truncate(true);

                opts
            },
            &path,
            |file| {
                file.write_all(&json)?;
                Ok(())
            },
        )
    }

    pub fn load(fs: FS) -> Result<Arc<Config>, littlefs2::io::Error> {
        let mut config_bytes = Vec::new();
        fs.open_file_and_then(
            &littlefs2::path::PathBuf::try_from(CONFIG_FILE).unwrap(),
            |file| {
                let len = file.len()?;
                config_bytes.resize(len, 0);
                file.read(&mut config_bytes)?;
                Ok(())
            },
        )?;

        Ok(Arc::new(
            serde_json::from_slice(&config_bytes).expect("Failed to parse config.json"),
        ))
    }

    pub fn set_widgets(&self, widgets: Vec<WidgetEntry>) {
        self.widgets.replace(widgets);
    }
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

impl WidgetEntry {
    pub fn validate(&self) -> Result<(), String> {
        if self.r#type.is_empty() {
            return Err(String::from("Widget type cannot be empty"));
        }

        if !self
            .r#type
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(String::from("Widget type contains invalid characters"));
        }

        if self.width == 0 || self.height == 0 {
            return Err(String::from("Widget dimensions must be greater than zero"));
        }

        if self.x + self.width > 64 || self.y + self.height > 64 {
            return Err(String::from("Widget is out of display bounds (64x64)"));
        }

        Ok(())
    }

    pub fn as_widget(&self, fs: FS) -> Widget {
        let wasm_path = format!("/widgets/{}.wasm", self.r#type);
        let executor = Box::new(WasmExecutor::new(fs, &wasm_path));

        Widget {
            id: WidgetId(self.id),
            placement: Rect::new(
                Point::new(self.x, self.y),
                Size::new(self.width, self.height),
            ),
            r#type: self.r#type.clone(),
            config: self.config.clone(),
            executor,
        }
    }
}

impl From<&Widget> for WidgetEntry {
    fn from(value: &Widget) -> Self {
        WidgetEntry {
            id: value.id.0,
            r#type: value.r#type.clone(),
            x: value.placement.origin.x,
            y: value.placement.origin.y,
            width: value.placement.size.width,
            height: value.placement.size.height,
            config: value.config.clone(),
        }
    }
}
