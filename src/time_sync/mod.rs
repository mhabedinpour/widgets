pub mod sntp;

pub use sntp::{SntpClient, SntpTimestamp, get_last_sync_instant, get_last_sync_unix, get_unix_timestamp, start};
