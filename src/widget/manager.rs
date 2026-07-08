use crate::drawer::Drawer;
use crate::widget::executor::Context;
use crate::widget::{Widget, WidgetId};
use alloc::collections::BTreeMap;

pub struct WidgetManager<'a> {
    widgets: BTreeMap<WidgetId, Widget>,
    drawer: &'a mut dyn Drawer,
}

impl<'a> WidgetManager<'a> {
    pub fn new(drawer: &'a mut dyn Drawer) -> Self {
        Self {
            widgets: BTreeMap::new(),
            drawer,
        }
    }

    pub fn add_widget(&mut self, id: WidgetId, widget: Widget) {
        self.widgets.insert(id, widget);
    }

    pub fn remove_widget(&mut self, id: WidgetId) {
        self.widgets.remove(&id);
    }

    pub fn render(&mut self) {
        for (_, widget) in self.widgets.iter_mut() {
            self.drawer.with_viewport(widget.placement, &mut |d| {
                widget.executor.render(Context { drawer: d });
            });
        }
    }
}
