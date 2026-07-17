use crate::drawer::GlobalDrawer;
use crate::timer::GlobalTimer;
use crate::widget::executor::Context;
use crate::widget::{Widget, WidgetEvent, WidgetId};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

pub struct WidgetManager<D: GlobalDrawer, T: GlobalTimer> {
    widgets: BTreeMap<WidgetId, Widget>,
    drawer: D,
    timer: T,
}

impl<D: GlobalDrawer, T: GlobalTimer> WidgetManager<D, T> {
    pub fn new(drawer: D, timer: T) -> Self {
        Self {
            widgets: BTreeMap::new(),
            drawer,
            timer,
        }
    }

    pub fn add_widget(&mut self, id: WidgetId, mut widget: Widget) {
        widget.executor.set_ctx(Context {
            drawer: self.drawer.scoped(widget.placement),
            timer: self.timer.scoped(id),
        });

        self.widgets.insert(id, widget);
    }

    pub fn remove_widget(&mut self, id: WidgetId) {
        self.widgets.remove(&id);
    }

    pub fn render(&mut self) {
        for (_, widget) in self.widgets.iter_mut() {
            widget.executor.render(None);
        }
    }

    pub fn poll_events(&mut self) {
        let mut events: BTreeMap<WidgetId, Vec<WidgetEvent>> = BTreeMap::new();
        self.handle_timer_events(&mut events);

        for (id, event_list) in events {
            self.widgets
                .get_mut(&id)
                .unwrap()
                .executor
                .render(Some(event_list));
        }
    }

    pub fn handle_timer_events(&mut self, events: &mut BTreeMap<WidgetId, Vec<WidgetEvent>>) {
        let expired = self.timer.poll();
        for (widget_id, timer_id) in expired {
            events
                .entry(widget_id)
                .or_default()
                .push(WidgetEvent::TimerInterrupt { timer_id });
        }
    }
}
