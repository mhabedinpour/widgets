use crate::console::{Console, GlobalConsole, LogData};
use crate::widget::WidgetId;
use alloc::boxed::Box;

pub struct ConsoleLogger;

impl ConsoleLogger {
    pub fn new() -> Self {
        Self
    }
}

impl GlobalConsole for ConsoleLogger {
    fn scoped(&self, widget_id: WidgetId) -> Box<dyn Console> {
        Box::new(WidgetConsole { widget_id })
    }
}

struct WidgetConsole {
    widget_id: WidgetId,
}

impl Console for WidgetConsole {
    fn log_info(&self, data: LogData) {
        log::info!("[widget {:?}] {}", self.widget_id, data.message);
    }

    fn log_error(&self, data: LogData) {
        log::error!("[widget {:?}] {}", self.widget_id, data.message);
    }
}
