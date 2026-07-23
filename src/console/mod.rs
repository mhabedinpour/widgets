pub mod logger;

pub use logger::ConsoleLogger;

use crate::widget::WidgetId;
use alloc::boxed::Box;
use alloc::string::String;

/// @wasm required="message"
pub struct LogData {
    pub message: String,
}

/// @wasm
pub trait Console {
    /// @wasm builder_name="info"
    fn log_info(&self, data: LogData);
    /// @wasm builder_name="error"
    fn log_error(&self, data: LogData);
}

pub trait GlobalConsole {
    fn scoped(&self, widget_id: WidgetId) -> Box<dyn Console>;
}
