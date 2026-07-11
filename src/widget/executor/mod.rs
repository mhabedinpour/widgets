pub mod test;
pub mod wasm;

use crate::drawer::Drawer;

pub struct Context<'a> {
    pub drawer: &'a mut dyn Drawer,
}

impl<'a> Context<'a> {
    pub fn new(drawer: &'a mut dyn Drawer) -> Self {
        Self { drawer }
    }
}

pub trait Executor {
    fn render(&mut self, ctx: Context);
}
