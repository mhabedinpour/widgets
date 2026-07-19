use crate::drawer::{Drawer, EmbeddedGraphicsDrawer, GlobalDrawer, Rect};
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::sync::Arc;
use core::cell::RefCell;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use esp_hal::Blocking;
use esp_hal::peripherals::LCD_CAM;
use esp_hal::time::Rate;
use esp_hub75::Hub75Error;
use esp_hub75::framebuffer::bitplane::plain::DmaFrameBuffer;
use esp_hub75::framebuffer::compute_rows;
use esp_hub75::{Hub75, Hub75Pins16};

const PLANES: usize = 7;

macro_rules! define_lcd_cam_backend {
    ($name:ident, $flusher_name:ident, $drawer_name:ident, $fb_name:ident, $rows:expr, $cols:expr) => {
        pub type $fb_name = DmaFrameBuffer<{ compute_rows($rows) }, $cols, PLANES>;
        pub struct $name<'a>(pub $flusher_name<'a>, pub $drawer_name);

        pub struct $flusher_name<'a> {
            hub75: Option<Hub75<'a, Blocking>>,
            own_buf: Box<$fb_name, esp_alloc::InternalMemory>,
            shared: Arc<Mutex<CriticalSectionRawMutex, RefCell<$fb_name>>>,
        }

        unsafe impl Send for $flusher_name<'_> {}

        impl<'a> $flusher_name<'a> {
            pub fn flush(&mut self) -> Result<(), Hub75Error> {
                let own_buf = &mut self.own_buf;
                self.shared
                    .lock(|fb| own_buf.as_mut().clone_from(&*fb.borrow()));

                let h75 = self.hub75.take().unwrap();
                let xfer = h75.render(self.own_buf.as_ref());
                match xfer {
                    Ok(xfer) => {
                        let (res, new_hub75) = xfer.wait();
                        self.hub75 = Some(new_hub75);
                        res.map_err(|e| e.into())
                    }
                    Err(e) => {
                        self.hub75 = Some(e.1);
                        Err(e.0)
                    }
                }
            }
        }

        pub struct $drawer_name {
            fb: Rc<RefCell<$fb_name>>,
            shared: Arc<Mutex<CriticalSectionRawMutex, RefCell<$fb_name>>>,
        }

        impl GlobalDrawer for $drawer_name {
            fn scoped(&self, bounds: Rect) -> Box<dyn Drawer> {
                Box::new(EmbeddedGraphicsDrawer::new(Rc::clone(&self.fb), bounds))
            }

            fn flush(&self) {
                self.shared
                    .lock(|shared_fb| shared_fb.borrow_mut().clone_from(&*self.fb.borrow()));
            }
        }

        impl<'a> $name<'a> {
            pub fn new(
                pins: Hub75Pins16<'a>,
                lcd: LCD_CAM<'a>,
                dma: esp_hal::peripherals::DMA_CH0<'a>,
                freq: Rate,
            ) -> Self {
                let tx_descriptors = esp_hub75::hub75_dma_descriptors!($fb_name);

                let hub75 = Hub75::new(lcd, pins, dma, tx_descriptors, freq)
                    .expect("failed to create Hub75!");

                let fb = Rc::new(RefCell::new($fb_name::new()));
                let shared = Arc::new(Mutex::new(RefCell::new($fb_name::new())));

                $name(
                    $flusher_name {
                        hub75: Some(hub75),
                        own_buf: Box::new_in($fb_name::new(), esp_alloc::InternalMemory),
                        shared: shared.clone(),
                    },
                    $drawer_name { fb, shared },
                )
            }
        }
    };
}

define_lcd_cam_backend!(
    LCDCAM64x64,
    LCDCAM64x64Flusher,
    LCDCAM64x64Drawer,
    FrameBuffer64x64,
    64,
    64
);
