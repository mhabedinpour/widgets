use crate::drawer::Rect;
use crate::http::Response;
use crate::widget::executor::Executor;
use alloc::boxed::Box;

pub mod executor;
pub mod manager;

pub enum WidgetEvent<'a> {
    TimerInterrupt {
        widget_id: u32,
        timer_id: u32,
    },
    HttpResponse {
        widget_id: u32,
        request_id: u32,
        response: Response<'a>,
    },
}

#[derive(Ord, Eq, PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct WidgetId(usize);

impl WidgetId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

pub struct Widget {
    pub placement: Rect,
    pub executor: Box<dyn Executor>,
}
