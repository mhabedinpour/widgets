use alloc::rc::Rc;
use embedded_storage::ReadStorage;
use embedded_storage::nor_flash::NorFlash;
use esp_hal::peripherals::FLASH;
use esp_storage::FlashStorage;
use littlefs2::driver::Storage;
use littlefs2::fs::{Allocation, Filesystem};
use littlefs2::io::{Error, Result};
use static_cell::StaticCell;

mod constants;
pub use constants::*;

pub struct EspFlashStorage<'a> {
    flash: FlashStorage<'a>,
    partition_offset: u32,
}

impl<'a> EspFlashStorage<'a> {
    pub fn new(flash: FLASH<'a>, partition_offset: u32) -> Self {
        Self {
            flash: FlashStorage::new(flash),
            partition_offset,
        }
    }
}

impl<'a> Storage for EspFlashStorage<'a> {
    const READ_SIZE: usize = READ_SIZE;
    const WRITE_SIZE: usize = WRITE_SIZE;
    const BLOCK_SIZE: usize = BLOCK_SIZE;
    const BLOCK_COUNT: usize = BLOCK_COUNT;
    type CACHE_SIZE = littlefs2::consts::U32;
    type LOOKAHEAD_SIZE = littlefs2::consts::U8;

    fn read(&mut self, off: usize, buf: &mut [u8]) -> Result<usize> {
        let addr = self.partition_offset + off as u32;
        ReadStorage::read(&mut self.flash, addr, buf).map_err(|_| Error::IO)?;
        Ok(buf.len())
    }

    fn write(&mut self, off: usize, data: &[u8]) -> Result<usize> {
        let addr = self.partition_offset + off as u32;
        NorFlash::write(&mut self.flash, addr, data).map_err(|_| Error::IO)?;
        Ok(data.len())
    }

    fn erase(&mut self, off: usize, len: usize) -> Result<usize> {
        let start_addr = self.partition_offset + off as u32;
        let end_addr = start_addr + len as u32;

        NorFlash::erase(&mut self.flash, start_addr, end_addr).map_err(|_| Error::IO)?;
        Ok(len)
    }
}

static FS_ALLOC: StaticCell<Allocation<EspFlashStorage<'static>>> = StaticCell::new();
static STORAGE: StaticCell<EspFlashStorage<'static>> = StaticCell::new();

pub type FS = Rc<Filesystem<'static, EspFlashStorage<'static>>>;

pub fn init(flash: FLASH<'static>) -> FS {
    let storage = STORAGE.init(EspFlashStorage::new(flash, PARTITION_OFFSET));
    let alloc = FS_ALLOC.init(Allocation::new());

    Rc::new(Filesystem::mount(alloc, storage).expect("Failed to mount filesystem"))
}
