pub mod test;

use crate::drawer::Drawer;

pub struct Context<'a> {
    pub drawer: &'a mut dyn Drawer,
}

pub trait Executor {
    fn render(&self, ctx: Context);
}
