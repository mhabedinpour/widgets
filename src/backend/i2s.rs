use crate::drawer::{Drawer, EmbeddedGraphicsDrawer, GlobalDrawer, Rect};
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::sync::Arc;
use core::cell::RefCell;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use esp_hal::Blocking;
use esp_hal::dma::DmaChannelFor;
use esp_hal::i2s::AnyI2s;
use esp_hal::time::Rate;
use esp_hub75::Hub75Error;
use esp_hub75::framebuffer::bitplane::plain::DmaFrameBuffer;
use esp_hub75::framebuffer::compute_rows;
use esp_hub75::{Hub75, Hub75Pins16};

const PLANES: usize = 7;

macro_rules! define_i2s_backend {
    ($name:ident, $flusher_name:ident, $drawer_name:ident, $fb_name:ident, $rows:expr, $cols:expr) => {
        pub type $fb_name = DmaFrameBuffer<{ compute_rows($rows) }, $cols, PLANES>;
        pub struct $name<'a>(pub $flusher_name<'a>, pub $drawer_name);
        pub type FBChannel = Channel<CriticalSectionRawMutex, Box<$fb_name>, 1>;

        pub struct $flusher_name<'a> {
            hub75: Option<Hub75<'a, Blocking>>,
            fb: Option<Box<$fb_name>>,
            channel: Arc<FBChannel>,
        }

        unsafe impl Send for $flusher_name<'_> {}

        impl<'a> $flusher_name<'a> {
            pub fn flush(&mut self) -> Result<(), Hub75Error> {
                if let Ok(new_fb) = self.channel.try_receive() {
                    self.fb = Some(new_fb);
                }

                if self.fb.is_none() {
                    return Ok(());
                }

                let h75 = self.hub75.take().unwrap();
                let xfer = h75.render(self.fb.as_ref().unwrap().as_ref());
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
            channel: Arc<FBChannel>,
        }

        impl GlobalDrawer for $drawer_name {
            fn scoped(&self, bounds: Rect) -> Box<dyn Drawer> {
                Box::new(EmbeddedGraphicsDrawer::new(Rc::clone(&self.fb), bounds))
            }

            fn flush(&self) {
                let _ = self.channel.try_receive();
                let _ = self.channel.try_send(Box::from(self.fb.borrow().clone()));
            }
        }

        impl<'a> $name<'a> {
            pub fn new(
                pins: Hub75Pins16<'a>,
                i2s: AnyI2s<'a>,
                dma: impl DmaChannelFor<AnyI2s<'a>>,
                freq: Rate,
            ) -> Self {
                let tx_descriptors = esp_hub75::hub75_dma_descriptors!($fb_name);

                let hub75 = Hub75::new(i2s, pins, dma, tx_descriptors, freq)
                    .expect("failed to create Hub75!");

                let fb = Rc::new(RefCell::new($fb_name::new()));
                let ch = Arc::new(FBChannel::new());

                $name(
                    $flusher_name {
                        hub75: Some(hub75),
                        fb: None,
                        channel: ch.clone(),
                    },
                    $drawer_name { fb, channel: ch },
                )
            }
        }
    };
}

define_i2s_backend!(
    I2s64x64,
    I2S64x64Flusher,
    I2S64x64Drawer,
    FrameBuffer64x64,
    64,
    64
);
