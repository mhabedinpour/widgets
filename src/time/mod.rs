pub mod timer;

use crate::widget::WidgetId;
use alloc::boxed::Box;
use alloc::vec::Vec;
use embassy_time::Duration;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TimerId(pub u32);

/// @wasm required="duration"
#[derive(Clone, Copy, Debug)]
pub struct CreateTimeoutData {
    pub duration: Duration,
    pub recurring: bool,
}

/// @wasm required="id"
#[derive(Clone, Copy, Debug)]
pub struct DeleteTimeoutData {
    pub id: TimerId,
}

/// @wasm
pub trait Time {
    /// @wasm builder_name="setTimeout"
    fn create_timeout(&mut self, data: CreateTimeoutData) -> TimerId;
    /// @wasm builder_name="clearTimeout"
    fn delete_timeout(&mut self, data: DeleteTimeoutData);
    /// @wasm builder_name="getUnixTimestamp"
    fn get_unix_timestamp(&mut self) -> i64;
}

pub trait GlobalTime {
    fn poll(&mut self) -> Vec<(WidgetId, TimerId)>;
    fn scoped(&self, widget_id: WidgetId) -> Box<dyn Time>;
}
