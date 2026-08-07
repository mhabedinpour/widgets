pub const READ_SIZE: usize = 256;
pub const WRITE_SIZE: usize = 256;
pub const BLOCK_SIZE: usize = 4096;
pub const BLOCK_COUNT: usize = 256;
#[allow(dead_code)]
pub const PARTITION_OFFSET: u32 = 0x300000;
pub type CacheSize = littlefs2::consts::U512;
pub type LookaheadSize = littlefs2::consts::U128;
