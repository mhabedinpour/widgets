use crate::config::Config;
use crate::console::GlobalConsole;
use crate::drawer::GlobalDrawer;
use crate::http::GlobalHttpClient;
use crate::network::GlobalNetwork;
use crate::storage::FS;
use crate::time::GlobalTime;
use crate::widget::executor::Context;
use crate::widget::{Widget, WidgetEvent, WidgetId};
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;

pub struct WidgetManager {
    widgets: BTreeMap<WidgetId, Widget>,
    drawer: Box<dyn GlobalDrawer>,
    time: Box<dyn GlobalTime>,
    http: Box<dyn GlobalHttpClient>,
    console: Box<dyn GlobalConsole>,
    network: Box<dyn GlobalNetwork>,
    fs: FS,
    config: Arc<Config>,
}

impl WidgetManager {
    pub fn new(
        drawer: Box<dyn GlobalDrawer>,
        time: Box<dyn GlobalTime>,
        http: Box<dyn GlobalHttpClient>,
        console: Box<dyn GlobalConsole>,
        network: Box<dyn GlobalNetwork>,
        fs: FS,
        config: Arc<Config>,
    ) -> Self {
        Self {
            widgets: BTreeMap::new(),
            drawer,
            time,
            http,
            console,
            network,
            fs,
            config,
        }
    }

    pub fn add_widget(&mut self, mut widget: Widget) {
        widget.executor.set_ctx(Context {
            drawer: self.drawer.scoped(widget.placement),
            time: self.time.scoped(widget.id),
            http: self.http.scoped(widget.id),
            console: self.console.scoped(widget.id),
            network: self.network.scoped(widget.id),
            config: widget.config.clone(),
        });

        let id = widget.id;
        self.widgets.insert(widget.id, widget);

        self.widgets.get_mut(&id).unwrap().executor.render(None);
        self.drawer.flush();
    }

    pub fn widgets(&self) -> &BTreeMap<WidgetId, Widget> {
        &self.widgets
    }

    pub fn remove_widget(&mut self, id: WidgetId) {
        self.widgets.remove(&id);
    }

    pub fn replace_widget(&mut self, widget: Widget) {
        self.remove_widget(widget.id);
        self.add_widget(widget);
    }

    pub fn flush(&mut self) {
        self.config
            .set_widgets(self.widgets.iter().map(|(_id, w)| w.into()).collect());

        if let Err(e) = self.config.save(self.fs.clone()) {
            log::error!("Failed to save config: {:?}", e.code());
        }
    }

    pub fn poll_events(&mut self) {
        let mut events: BTreeMap<WidgetId, Vec<WidgetEvent>> = BTreeMap::new();
        self.handle_timer_events(&mut events);
        self.handle_http_events(&mut events);

        let flush = events.len() > 0;

        for (id, event_list) in events {
            if let Some(widget) = self.widgets.get_mut(&id) {
                widget.executor.render(Some(event_list));
            }
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
