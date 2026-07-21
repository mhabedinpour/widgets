use crate::time::{CreateTimeoutData, DeleteTimeoutData, GlobalTime, Time, TimerId};
use crate::widget::WidgetId;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;
use embassy_time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimerKey {
    pub widget_id: WidgetId,
    pub timer_id: TimerId,
}

pub struct TimerEntry {
    pub expiration: Instant,
    /// `Some(period)` for recurring timers; `None` for one-shot.
    pub recurring: Option<Duration>,
}

pub struct TimeService {
    timers: Rc<RefCell<BTreeMap<TimerKey, TimerEntry>>>,
}

impl TimeService {
    pub fn new() -> Self {
        Self {
            timers: Rc::new(RefCell::new(BTreeMap::new())),
        }
    }

    pub fn schedule(&self, widget_id: WidgetId, timer_id: TimerId, duration: Duration, recurring: Option<Duration>) {
        let expiration = Instant::now() + duration;
        self.timers.borrow_mut().insert(
            TimerKey { widget_id, timer_id },
            TimerEntry { expiration, recurring },
        );
    }

    pub fn cancel(&self, widget_id: WidgetId, timer_id: TimerId) {
        self.timers.borrow_mut().remove(&TimerKey { widget_id, timer_id });
    }
}

impl GlobalTime for TimeService {
    fn poll(&mut self) -> Vec<(WidgetId, TimerId)> {
        let now = Instant::now();
        let mut expired = Vec::new();
        let mut timers = self.timers.borrow_mut();

        // Collect all expired keys first — map is ordered by key, not expiration,
        // so we must scan every entry rather than break early.
        let expired_keys: Vec<(TimerKey, Option<Duration>)> = timers
            .iter()
            .filter(|(_, entry)| entry.expiration <= now)
            .map(|(key, entry)| (*key, entry.recurring))
            .collect();

        for (key, recurring) in expired_keys {
            expired.push((key.widget_id, key.timer_id));
            match recurring {
                Some(period) => {
                    if let Some(entry) = timers.get_mut(&key) {
                        entry.expiration = now + period;
                    }
                }
                None => {
                    timers.remove(&key);
                }
            }
        }

        expired
    }

    fn scoped(&self, widget_id: WidgetId) -> Box<dyn Time> {
        Box::new(WidgetTime::new(widget_id, Rc::clone(&self.timers)))
    }
}

pub struct WidgetTime {
    pub widget_id: WidgetId,
    next_id: u32,
    timers: Rc<RefCell<BTreeMap<TimerKey, TimerEntry>>>,
}

impl WidgetTime {
    pub(crate) fn new(
        widget_id: WidgetId,
        timers: Rc<RefCell<BTreeMap<TimerKey, TimerEntry>>>,
    ) -> Self {
        Self {
            widget_id,
            next_id: 0,
            timers,
        }
    }
}

impl Time for WidgetTime {
    fn create_timeout(&mut self, data: CreateTimeoutData) -> TimerId {
        let id = TimerId(self.next_id);
        self.next_id += 1;
        let recurring = if data.recurring { Some(data.duration) } else { None };
        self.timers.borrow_mut().insert(
            TimerKey { widget_id: self.widget_id, timer_id: id },
            TimerEntry { expiration: Instant::now() + data.duration, recurring },
        );
        id
    }

    fn delete_timeout(&mut self, data: DeleteTimeoutData) {
        self.timers.borrow_mut().remove(&TimerKey {
            widget_id: self.widget_id,
            timer_id: data.id,
        });
    }

    fn get_unix_timestamp(&mut self) -> i64 {
        crate::time_sync::get_unix_timestamp().unwrap_or(0)
    }
}
