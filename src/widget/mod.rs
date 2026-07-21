use crate::drawer::Rect;
use crate::http::RequestId;
use crate::time::TimerId;
use crate::widget::executor::Executor;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

pub mod executor;
pub mod manager;

pub enum WidgetEvent {
    TimerInterrupt { timer_id: TimerId },
    HttpResponse { request_id: RequestId, headers: Vec<(String, String)>, body: String, success: bool },
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
