#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]
#![feature(type_alias_impl_trait)]
#![feature(allocator_api)]
extern crate alloc;
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
use widgets::backend::lcd::{LCDCAM64x64, LCDCAM64x64Flusher};
use widgets::compiled_widgets;
use widgets::drawer::{Point, Rect, Size};
use widgets::timer::timer::TimerService;
use widgets::widget::executor::wasm::WasmExecutor;
use widgets::widget::manager::WidgetManager;
use widgets::widget::{Widget, WidgetId};

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

fn flush_task(mut flusher: LCDCAM64x64Flusher<'static>) -> ! {
    loop {
        flusher.flush().expect("Failed to flush display");
    }
}

fn init_display_flusher(
    cpu_ctrl: CPU_CTRL,
    sw_int: SoftwareInterrupt<'static, 1>,
    flusher: LCDCAM64x64Flusher<'static>,
) {
    let app_core_stack: &'static mut Stack<2048> = make_static!(Stack::<2048>::new());
    esp_rtos::start_second_core(cpu_ctrl, sw_int, app_core_stack, move || {
        flush_task(flusher);
    });
}

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 96 * 1024);
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 64000);
    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    info!("Embassy initialized!");

    let (mut _wifi_controller, _interfaces) =
        esp_radio::wifi::new(peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");

    let pins = Hub75Pins16 {
        red1: peripherals.GPIO5.degrade(),
        grn1: peripherals.GPIO6.degrade(),
        blu1: peripherals.GPIO7.degrade(),
        red2: peripherals.GPIO15.degrade(),
        grn2: peripherals.GPIO16.degrade(),
        blu2: peripherals.GPIO17.degrade(),
        addr0: peripherals.GPIO8.degrade(),
        addr1: peripherals.GPIO3.degrade(),
        addr2: peripherals.GPIO46.degrade(),
        addr3: peripherals.GPIO9.degrade(),
        addr4: peripherals.GPIO18.degrade(),
        blank: peripherals.GPIO12.degrade(),
        clock: peripherals.GPIO10.degrade(),
        latch: peripherals.GPIO11.degrade(),
    };
    let freq = Rate::from_mhz(20);

    let backend = LCDCAM64x64::new(pins, peripherals.LCD_CAM, peripherals.DMA_CH0, freq);
    let mut manager = WidgetManager::new(backend.1, TimerService::new());

    init_display_flusher(
        peripherals.CPU_CTRL,
        sw_interrupt.software_interrupt1,
        backend.0,
    );

    manager.add_widget(
        WidgetId::new(1),
        Widget {
            placement: Rect::new(Point::new(0, 0), Size::new(64, 64)),
            executor: Box::new(WasmExecutor::new(compiled_widgets::SAMPLE).unwrap()),
        },
    );

    manager.render();

    loop {
        manager.poll_events();
        Timer::after_millis(10).await;
    }
}
