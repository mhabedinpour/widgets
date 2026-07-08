use crate::drawer::{Color, RectBuilder};
use crate::widget::executor::{Context, Executor};

pub struct TestExec;

impl Executor for TestExec {
    fn render(&self, ctx: Context) {
        RectBuilder::new(0, 0, 10, 10)
            .color(Color::WHITE)
            .stroke(1)
            .draw(ctx.drawer);
    }
}
