pub mod wasm;

use crate::drawer::Drawer;
use crate::http::Http;
use crate::time::Time;
use crate::widget::WidgetEvent;
use alloc::boxed::Box;
use alloc::vec::Vec;

pub struct Context {
    pub drawer: Box<dyn Drawer>,
    pub time: Box<dyn Time>,
    pub http: Box<dyn Http>,
}

pub trait Executor {
    fn set_ctx(&mut self, ctx: Context);
    fn render(&mut self, events: Option<Vec<WidgetEvent>>);
}
