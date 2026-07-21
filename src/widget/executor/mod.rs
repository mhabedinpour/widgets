pub mod wasm;

use crate::console::Console;
use crate::drawer::Drawer;
use crate::http::Http;
use crate::network::Network;
use crate::time::Time;
use crate::widget::{WidgetConfig, WidgetEvent};
use alloc::boxed::Box;
use alloc::vec::Vec;

pub struct Context {
    pub drawer: Box<dyn Drawer>,
    pub time: Box<dyn Time>,
    pub http: Box<dyn Http>,
    pub console: Box<dyn Console>,
    pub network: Box<dyn Network>,
    pub config: WidgetConfig,
}

pub trait Executor {
    fn set_ctx(&mut self, ctx: Context);
    fn render(&mut self, events: Option<Vec<WidgetEvent>>);
}
