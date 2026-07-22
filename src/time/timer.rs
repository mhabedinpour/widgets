use crate::time::{CreateTimeoutData, DeleteTimeoutData, GlobalTime, Time, TimerId};
use crate::time_sync::sntp::TimeSyncService;
use crate::widget::WidgetId;
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BinaryHeap};
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::cmp::Reverse;
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

/// A single entry in the min-heap, ordered by expiration time.
///
/// Stale entries (cancelled or superseded by a reschedule) are detected in
/// [`TimerState::poll`] by comparing the heap expiration against the canonical
/// expiration stored in `active`.
#[derive(Clone, Copy, PartialEq, Eq)]
struct HeapEntry {
    expiration: Instant,
    widget_id: WidgetId,
    timer_id: TimerId,
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        // Primary: earlier expiration first (min-heap via `Reverse` at the call site).
        // Secondary: stable tiebreak so the heap is fully ordered.
        self.expiration
            .cmp(&other.expiration)
            .then(self.widget_id.cmp(&other.widget_id))
            .then(self.timer_id.cmp(&other.timer_id))
    }
}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Inner state shared between [`TimeService`] and every [`WidgetTime`] it spawns.
struct TimerState {
    /// Min-heap: the entry with the earliest expiration is at the top.
    /// Wrapped in `Reverse` because `BinaryHeap` is a max-heap by default.
    heap: BinaryHeap<Reverse<HeapEntry>>,
    /// Canonical set of active timers.
    ///
    /// - Insert / cancel here first; the heap uses lazy deletion.
    /// - Recurring periods are stored here so `poll` can reschedule without
    ///   keeping extra data in `HeapEntry`.
    active: BTreeMap<TimerKey, TimerEntry>,
}

impl TimerState {
    fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
            active: BTreeMap::new(),
        }
    }

    fn insert(
        &mut self,
        widget_id: WidgetId,
        timer_id: TimerId,
        duration: Duration,
        recurring: Option<Duration>,
    ) {
        let expiration = Instant::now() + duration;
        let key = TimerKey {
            widget_id,
            timer_id,
        };
        self.active.insert(
            key,
            TimerEntry {
                expiration,
                recurring,
            },
        );
        self.heap.push(Reverse(HeapEntry {
            expiration,
            widget_id,
            timer_id,
        }));
    }

    /// Lazy cancel: remove from `active`; the heap entry is discarded when it
    /// surfaces during `poll`.
    fn remove(&mut self, widget_id: WidgetId, timer_id: TimerId) {
        self.active.remove(&TimerKey {
            widget_id,
            timer_id,
        });
    }

    fn poll(&mut self) -> Vec<(WidgetId, TimerId)> {
        let now = Instant::now();
        let mut expired = Vec::new();

        loop {
            // Break early: the heap's minimum is the earliest expiration.
            // If it hasn't fired yet, nothing else has either.
            match self.heap.peek() {
                Some(Reverse(e)) if e.expiration > now => break,
                None => break,
                _ => {}
            }

            let Reverse(entry) = self.heap.pop().unwrap();
            let key = TimerKey {
                widget_id: entry.widget_id,
                timer_id: entry.timer_id,
            };

            // Stale-entry check: the heap entry is stale if it was cancelled
            // (key absent) or superseded by a reschedule (expiration mismatch).
            let canonical = self.active.get(&key).map(|e| e.expiration);
            if canonical != Some(entry.expiration) {
                continue;
            }

            expired.push((entry.widget_id, entry.timer_id));

            match self.active.get_mut(&key) {
                Some(timer) if timer.recurring.is_some() => {
                    let period = timer.recurring.unwrap();
                    timer.expiration = now + period;
                    self.heap.push(Reverse(HeapEntry {
                        expiration: timer.expiration,
                        widget_id: entry.widget_id,
                        timer_id: entry.timer_id,
                    }));
                }
                _ => {
                    self.active.remove(&key);
                }
            }
        }

        expired
    }
}

pub struct TimeService {
    state: Rc<RefCell<TimerState>>,
    time_sync: &'static TimeSyncService,
}

impl TimeService {
    pub fn new(time_sync: &'static TimeSyncService) -> Self {
        Self {
            state: Rc::new(RefCell::new(TimerState::new())),
            time_sync,
        }
    }

    pub fn schedule(
        &self,
        widget_id: WidgetId,
        timer_id: TimerId,
        duration: Duration,
        recurring: Option<Duration>,
    ) {
        self.state
            .borrow_mut()
            .insert(widget_id, timer_id, duration, recurring);
    }

    pub fn cancel(&self, widget_id: WidgetId, timer_id: TimerId) {
        self.state.borrow_mut().remove(widget_id, timer_id);
    }
}

impl GlobalTime for TimeService {
    fn poll(&mut self) -> Vec<(WidgetId, TimerId)> {
        self.state.borrow_mut().poll()
    }

    fn scoped(&self, widget_id: WidgetId) -> Box<dyn Time> {
        Box::new(WidgetTime::new(
            widget_id,
            Rc::clone(&self.state),
            self.time_sync,
        ))
    }
}

pub struct WidgetTime {
    pub widget_id: WidgetId,
    next_id: u32,
    state: Rc<RefCell<TimerState>>,
    time_sync: &'static TimeSyncService,
}

impl WidgetTime {
    fn new(
        widget_id: WidgetId,
        state: Rc<RefCell<TimerState>>,
        time_sync: &'static TimeSyncService,
    ) -> Self {
        Self {
            widget_id,
            next_id: 0,
            state,
            time_sync,
        }
    }
}

impl Time for WidgetTime {
    fn create_timeout(&mut self, data: CreateTimeoutData) -> TimerId {
        let id = TimerId(self.next_id);
        self.next_id += 1;
        let recurring = if data.recurring {
            Some(data.duration)
        } else {
            None
        };
        self.state
            .borrow_mut()
            .insert(self.widget_id, id, data.duration, recurring);
        id
    }

    fn delete_timeout(&mut self, data: DeleteTimeoutData) {
        self.state.borrow_mut().remove(self.widget_id, data.id);
    }

    fn get_unix_timestamp(&mut self) -> i64 {
        self.time_sync.get_unix_timestamp().unwrap_or(0)
    }

    fn get_last_sync(&mut self) -> i64 {
        self.time_sync.get_last_sync_unix().unwrap_or(-1)
    }
}
