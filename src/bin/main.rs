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
#![feature(const_trait_impl)]
#![feature(const_option_ops)]
extern crate alloc;

use alloc::boxed::Box;
use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::Pin;
use esp_hal::interrupt::software::{SoftwareInterrupt, SoftwareInterruptControl};
use esp_hal::peripherals::CPU_CTRL;
use esp_hal::system::Stack as HalStack;
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hub75::Hub75Pins16;
use log::info;
use static_cell::StaticCell;
use widgets::backend::lcd::{LCDCAM64x64, LCDCAM64x64Flusher};
use widgets::compiled_widgets;
use widgets::console::ConsoleLogger;
use widgets::drawer::{Point, Rect, Size};
use widgets::http::HttpService;
use widgets::network::NetworkService;
use widgets::time::timer::TimeService;
use widgets::time_sync::start as start_time_sync;
use widgets::widget::executor::wasm::WasmExecutor;
use widgets::widget::manager::WidgetManager;
use widgets::widget::{Widget, WidgetConfig, WidgetId};

use embassy_net::{Config as NetConfig, DhcpConfig, Runner, StackResources};
use esp_radio::wifi::{Config as WifiConfig, WifiController, sta::StationConfig};

// Generated from config.json at build time.
include!(concat!(env!("OUT_DIR"), "/config.rs"));

static STACK_NET: StaticCell<StackResources<8>> = StaticCell::new();
static FLUSHER_STACK: StaticCell<HalStack<2048>> = StaticCell::new();

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

fn flush_task(mut flusher: LCDCAM64x64Flusher<'static>) -> ! {
    loop {
        flusher.flush().expect("Failed to flush display");
    }
}

#[allow(clippy::large_stack_frames)]
fn init_display_flusher(
    cpu_ctrl: CPU_CTRL,
    sw_int: SoftwareInterrupt<'static, 1>,
    flusher: LCDCAM64x64Flusher<'static>,
) {
    let app_core_stack = FLUSHER_STACK.init(HalStack::<2048>::new());
    esp_rtos::start_second_core(cpu_ctrl, sw_int, app_core_stack, move || {
        flush_task(flusher);
    });
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, esp_radio::wifi::Interface<'static>>) -> ! {
    loop {
        runner.run().await;
    }
}

#[embassy_executor::task]
#[allow(clippy::large_stack_frames)]
async fn connection_task(mut controller: WifiController<'static>) {
    loop {
        if !controller.is_connected() {
            info!("Connecting to Wi-Fi...");
            let config = WifiConfig::Station(
                StationConfig::default()
                    .with_ssid(WIFI_SSID)
                    .with_password(alloc::string::String::from(WIFI_PASSWORD)),
            );
            if let Err(e) = controller.set_config(&config) {
                log::error!("Failed to set Wi-Fi config: {:?}", e);
                Timer::after_millis(5000).await;
                continue;
            }

            match controller.connect_async().await {
                Ok(_) => {
                    info!("Wi-Fi connected!");
                }
                Err(e) => {
                    log::error!("Failed to connect to Wi-Fi: {:?}. Retrying...", e);
                    Timer::after_millis(5000).await;
                    continue;
                }
            }
        }

        // Wait for disconnect
        match controller.wait_for_disconnect_async().await {
            Ok(_) => {
                log::warn!("Wi-Fi disconnected!");
            }
            Err(e) => {
                log::error!("Error waiting for disconnect: {:?}", e);
                Timer::after_millis(1000).await;
            }
        }
    }
}

#[esp_rtos::main]
#[allow(clippy::large_stack_frames)]
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

    let (wifi_controller, interfaces) = esp_radio::wifi::new(peripherals.WIFI, Default::default())
        .expect("Failed to initialize Wi-Fi controller");

    let rng = esp_hal::rng::Rng::new();
    let seed = (rng.random() as u64) << 32 | (rng.random() as u64);
    let (stack, runner) = embassy_net::new(
        interfaces.station,
        NetConfig::dhcpv4(DhcpConfig::default()),
        STACK_NET.init(StackResources::<8>::new()),
        seed,
    );

    spawner.spawn(net_task(runner).unwrap());
    spawner.spawn(connection_task(wifi_controller).unwrap());

    info!("Waiting for Wi-Fi DHCP...");
    stack.wait_config_up().await;
    info!("Wi-Fi DHCP configured!");
    if let Some(config) = stack.config_v4() {
        info!("IP Address: {:?}", config.address);
    }

    start_time_sync(spawner, stack);

    let pins = hub75_pins!(peripherals);
    let freq = Rate::from_mhz(DISPLAY_FREQ_MHZ);

    let backend = LCDCAM64x64::new(pins, peripherals.LCD_CAM, peripherals.DMA_CH0, freq);
    let mut manager = WidgetManager::new(
        backend.1,
        TimeService::new(),
        HttpService::new(spawner, stack),
        ConsoleLogger::new(),
        NetworkService::new(stack),
    );

    init_display_flusher(
        peripherals.CPU_CTRL,
        sw_interrupt.software_interrupt1,
        backend.0,
    );

    add_widgets!(manager);

    manager.render();

    loop {
        manager.poll_events();
        Timer::after_millis(10).await;
    }
}
