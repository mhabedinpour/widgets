pub mod timer;

use crate::widget::WidgetId;
use alloc::boxed::Box;
use alloc::vec::Vec;
use embassy_time::Duration;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TimerId(pub u32);

/// @wasm required="id,duration"
#[derive(Clone, Copy, Debug)]
pub struct TimeoutData {
    pub id: TimerId,
    pub duration: Duration,
}

/// @wasm
pub trait Timer {
    // Underlying primitive drawing execution hooks
    /// @wasm builder_name="setTimeout"
    fn create_timeout(&mut self, data: TimeoutData);
    /// @wasm builder_name="clearTimeout"
    fn delete_timeout(&mut self, data: TimeoutData);
}

pub trait GlobalTimer {
    fn poll(&mut self) -> Vec<(WidgetId, TimerId)>;
    fn scoped(&self, widget_id: WidgetId) -> Box<dyn Timer>;
}
