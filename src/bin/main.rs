#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]
#![feature(type_alias_impl_trait)]
#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
use alloc::boxed::Box;
use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::Pin;
use esp_hal::interrupt::software::{SoftwareInterrupt, SoftwareInterruptControl};
use esp_hal::peripherals::CPU_CTRL;
use esp_hal::system::Stack;
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hub75::Hub75Pins16;
use log::info;
use static_cell::make_static;
use widgets::backend::i2s::{I2S64x64Flusher, I2s64x64};
use widgets::drawer::{Point, Rect, Size};
use widgets::widget::executor::test::TestExec;
use widgets::widget::manager::WidgetManager;
use widgets::widget::{Widget, WidgetId};

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

fn flush_task(mut flusher: I2S64x64Flusher<'static>) -> ! {
    loop {
        flusher.flush().expect("Failed to flush display");
    }
}

fn init_display_flusher(
    cpu_ctrl: CPU_CTRL,
    sw_int: SoftwareInterrupt<'static, 1>,
    flusher: I2S64x64Flusher<'static>,
) {
    let app_core_stack: &'static mut Stack<16384> = make_static!(Stack::<16384>::new());
    esp_rtos::start_second_core(cpu_ctrl, sw_int, app_core_stack, move || {
        flush_task(flusher);
    });
}

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // The following pins are used to bootstrap the chip. They are available
    // for use, but check the datasheet of the module for more information on them.
    // - GPIO0
    // - GPIO2
    // - GPIO5
    // - GPIO12
    // - GPIO15
    // These GPIO pins are in use by some feature of the module and should not be used.
    let _ = peripherals.GPIO6;
    let _ = peripherals.GPIO7;
    let _ = peripherals.GPIO8;
    let _ = peripherals.GPIO9;
    let _ = peripherals.GPIO10;
    let _ = peripherals.GPIO11;
    let _ = peripherals.GPIO16;
    let _ = peripherals.GPIO20;

    esp_alloc::heap_allocator!(size: 32 * 1024);
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    info!("Embassy initialized!");

    let (mut _wifi_controller, _interfaces) =
        esp_radio::wifi::new(peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");

    let pins = Hub75Pins16 {
        red1: peripherals.GPIO25.degrade(),
        grn1: peripherals.GPIO26.degrade(),
        blu1: peripherals.GPIO27.degrade(),
        red2: peripherals.GPIO14.degrade(),
        grn2: peripherals.GPIO12.degrade(),
        blu2: peripherals.GPIO13.degrade(),
        addr0: peripherals.GPIO19.degrade(),
        addr1: peripherals.GPIO18.degrade(),
        addr2: peripherals.GPIO5.degrade(),
        addr3: peripherals.GPIO17.degrade(),
        addr4: peripherals.GPIO21.degrade(),
        blank: peripherals.GPIO15.degrade(),
        clock: peripherals.GPIO16.degrade(),
        latch: peripherals.GPIO4.degrade(),
    };
    let i2s = peripherals.I2S0.into();
    let dma = peripherals.DMA_I2S0;
    let freq = Rate::from_mhz(16);

    let mut backend = I2s64x64::new(pins, i2s, dma, freq);
    let mut manager = WidgetManager::new(&mut backend.1);

    init_display_flusher(
        peripherals.CPU_CTRL,
        sw_interrupt.software_interrupt1,
        backend.0,
    );

    manager.add_widget(
        WidgetId::new(1),
        Widget {
            placement: Rect::new(Point::new(0, 10), Size::new(14, 20)),
            executor: Box::new(TestExec {}),
        },
    );
    manager.add_widget(
        WidgetId::new(2),
        Widget {
            placement: Rect::new(Point::new(0, 30), Size::new(14, 20)),
            executor: Box::new(TestExec {}),
        },
    );
    manager.add_widget(
        WidgetId::new(3),
        Widget {
            placement: Rect::new(Point::new(0, 50), Size::new(14, 20)),
            executor: Box::new(TestExec {}),
        },
    );
    manager.add_widget(
        WidgetId::new(4),
        Widget {
            placement: Rect::new(Point::new(30, 10), Size::new(14, 20)),
            executor: Box::new(TestExec {}),
        },
    );
    manager.add_widget(
        WidgetId::new(5),
        Widget {
            placement: Rect::new(Point::new(30, 30), Size::new(14, 20)),
            executor: Box::new(TestExec {}),
        },
    );
    manager.add_widget(
        WidgetId::new(6),
        Widget {
            placement: Rect::new(Point::new(30, 50), Size::new(14, 20)),
            executor: Box::new(TestExec {}),
        },
    );
    manager.add_widget(
        WidgetId::new(7),
        Widget {
            placement: Rect::new(Point::new(50, 10), Size::new(14, 20)),
            executor: Box::new(TestExec {}),
        },
    );
    manager.add_widget(
        WidgetId::new(8),
        Widget {
            placement: Rect::new(Point::new(50, 30), Size::new(14, 20)),
            executor: Box::new(TestExec {}),
        },
    );
    manager.add_widget(
        WidgetId::new(9),
        Widget {
            placement: Rect::new(Point::new(50, 50), Size::new(14, 20)),
            executor: Box::new(TestExec {}),
        },
    );

    manager.render();

    loop {
        Timer::after_millis(100).await;
    }
}
