use crate::network::{GlobalNetwork, Network};
use crate::widget::WidgetId;
use alloc::boxed::Box;
use embassy_net::Stack;

pub struct NetworkService {
    stack: Stack<'static>,
}

impl NetworkService {
    pub fn new(stack: Stack<'static>) -> Self {
        Self { stack }
    }
}

impl GlobalNetwork for NetworkService {
    fn scoped(&self, _widget_id: WidgetId) -> Box<dyn Network> {
        Box::new(WidgetNetwork { stack: self.stack })
    }
}

struct WidgetNetwork {
    stack: Stack<'static>,
}

impl Network for WidgetNetwork {
    fn is_connected(&mut self) -> u32 {
        if self.stack.is_link_up() { 1 } else { 0 }
    }

    fn get_internal_ip(&mut self) -> u32 {
        self.stack
            .config_v4()
            .map(|c| c.address.address().to_bits())
            .unwrap_or(0)
    }
}
