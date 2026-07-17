use crate::timer::{GlobalTimer, TimeoutData, Timer, TimerId};
use crate::widget::WidgetId;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;
use embassy_time::{Duration, Instant};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct TimerKey {
    pub widget_id: WidgetId,
    pub timer_id: TimerId,
}

pub struct TimerService {
    timers: Rc<RefCell<BTreeMap<TimerKey, Instant>>>,
}

impl TimerService {
    pub fn new() -> Self {
        Self {
            timers: Rc::new(RefCell::new(BTreeMap::new())),
        }
    }

    pub fn schedule(&self, widget_id: WidgetId, timer_id: TimerId, duration: Duration) {
        let expiration = Instant::now() + duration;
        self.timers.borrow_mut().insert(
            TimerKey {
                widget_id,
                timer_id,
            },
            expiration,
        );
    }

    pub fn cancel(&self, widget_id: WidgetId, timer_id: TimerId) {
        self.timers.borrow_mut().remove(&TimerKey {
            widget_id,
            timer_id,
        });
    }
}

impl GlobalTimer for TimerService {
    fn poll(&mut self) -> Vec<(WidgetId, TimerId)> {
        let now = Instant::now();
        let mut expired = Vec::new();
        let mut timers = self.timers.borrow_mut();

        while let Some(entry) = timers.first_entry() {
            if *entry.get() <= now {
                let key = entry.remove_entry().0;
                expired.push((key.widget_id, key.timer_id));
            } else {
                break;
            }
        }

        expired
    }

    fn scoped(&self, widget_id: WidgetId) -> Box<dyn Timer> {
        Box::new(WidgetTimer::new(widget_id, Rc::clone(&self.timers)))
    }
}

pub struct WidgetTimer {
    pub widget_id: WidgetId,
    timers: Rc<RefCell<BTreeMap<TimerKey, Instant>>>,
}

impl WidgetTimer {
    pub(crate) fn new(
        widget_id: WidgetId,
        timers: Rc<RefCell<BTreeMap<TimerKey, Instant>>>,
    ) -> Self {
        Self { widget_id, timers }
    }
}

impl Timer for WidgetTimer {
    fn create_timeout(&mut self, data: TimeoutData) {
        let expiration = Instant::now() + data.duration;
        self.timers.borrow_mut().insert(
            TimerKey {
                widget_id: self.widget_id,
                timer_id: data.id,
            },
            expiration,
        );
    }

    fn delete_timeout(&mut self, data: TimeoutData) {
        self.timers.borrow_mut().remove(&TimerKey {
            widget_id: self.widget_id,
            timer_id: data.id,
        });
    }
}
