pub mod wifi;
pub use wifi::NetworkService;

use crate::widget::WidgetId;
use alloc::boxed::Box;

/// @wasm
pub trait Network {
    /// Returns 1 if connected to WiFi, 0 otherwise.
    /// @wasm builder_name="isConnected"
    fn is_connected(&mut self) -> u32;
    /// Returns the internal IPv4 address packed as a big-endian u32.
    /// Returns 0 if DHCP has not yet assigned an address.
    /// @wasm builder_name="getInternalIp"
    fn get_internal_ip(&mut self) -> u32;
}

pub trait GlobalNetwork {
    fn scoped(&self, widget_id: WidgetId) -> Box<dyn Network>;
}
