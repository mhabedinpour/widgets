use crate::drawer::GlobalDrawer;
use crate::http::GlobalHttpClient;
use crate::timer::GlobalTimer;
use crate::widget::executor::Context;
use crate::widget::{Widget, WidgetEvent, WidgetId};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

pub struct WidgetManager<D: GlobalDrawer, T: GlobalTimer, H: GlobalHttpClient> {
    widgets: BTreeMap<WidgetId, Widget>,
    drawer: D,
    timer: T,
    http: H,
}

impl<D: GlobalDrawer, T: GlobalTimer, H: GlobalHttpClient> WidgetManager<D, T, H> {
    pub fn new(drawer: D, timer: T, http: H) -> Self {
        Self {
            widgets: BTreeMap::new(),
            drawer,
            timer,
            http,
        }
    }

    pub fn add_widget(&mut self, id: WidgetId, mut widget: Widget) {
        widget.executor.set_ctx(Context {
            drawer: self.drawer.scoped(widget.placement),
            timer: self.timer.scoped(id),
            http: self.http.scoped(id),
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
        self.drawer.flush();
    }

    pub fn poll_events(&mut self) {
        let mut events: BTreeMap<WidgetId, Vec<WidgetEvent>> = BTreeMap::new();
        self.handle_timer_events(&mut events);
        self.handle_http_events(&mut events);

        let flush = events.len() > 0;

        for (id, event_list) in events {
            self.widgets
                .get_mut(&id)
                .unwrap()
                .executor
                .render(Some(event_list));
        }

        if flush {
            self.drawer.flush();
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

    pub fn handle_http_events(&mut self, events: &mut BTreeMap<WidgetId, Vec<WidgetEvent>>) {
        let responses = self.http.poll();
        for (widget_id, response) in responses {
            let success = response.headers.is_some();
            events
                .entry(widget_id)
                .or_default()
                .push(WidgetEvent::HttpResponse {
                    request_id: response.request_id,
                    headers: response.headers.unwrap_or("".parse().unwrap()),
                    body: response.body.unwrap_or("".parse().unwrap()),
                    success,
                });
        }
    }
}
