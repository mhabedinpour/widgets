use crate::console::GlobalConsole;
use crate::drawer::GlobalDrawer;
use crate::http::GlobalHttpClient;
use crate::network::GlobalNetwork;
use crate::time::GlobalTime;
use crate::widget::executor::Context;
use crate::widget::{Widget, WidgetConfig, WidgetEvent, WidgetId};
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

pub struct WidgetManager {
    widgets: BTreeMap<WidgetId, Widget>,
    drawer: Box<dyn GlobalDrawer>,
    time: Box<dyn GlobalTime>,
    http: Box<dyn GlobalHttpClient>,
    console: Box<dyn GlobalConsole>,
    network: Box<dyn GlobalNetwork>,
}

impl WidgetManager {
    pub fn new(
        drawer: Box<dyn GlobalDrawer>,
        time: Box<dyn GlobalTime>,
        http: Box<dyn GlobalHttpClient>,
        console: Box<dyn GlobalConsole>,
        network: Box<dyn GlobalNetwork>,
    ) -> Self {
        Self {
            widgets: BTreeMap::new(),
            drawer,
            time,
            http,
            console,
            network,
        }
    }

    pub fn add_widget(&mut self, id: WidgetId, mut widget: Widget) {
        widget.executor.set_ctx(Context {
            drawer: self.drawer.scoped(widget.placement),
            time: self.time.scoped(id),
            http: self.http.scoped(id),
            console: self.console.scoped(id),
            network: self.network.scoped(id),
            config: widget.config.clone(),
        });

        self.widgets.insert(id, widget);
    }

    pub fn widgets(&self) -> &BTreeMap<WidgetId, Widget> {
        &self.widgets
    }

    pub fn update_widget_config(&mut self, id: WidgetId, new_config: WidgetConfig) -> bool {
        if let Some(widget) = self.widgets.get_mut(&id) {
            // TODO: update context
            widget.config = new_config;
            true
        } else {
            false
        }
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
        let expired = self.time.poll();
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
                    headers: response.headers.unwrap_or(Vec::new()),
                    body: response.body.unwrap_or("".parse().unwrap()),
                    success,
                });
        }
    }
}
