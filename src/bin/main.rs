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
use alloc::rc::Rc;
use core::cell::RefCell;
use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::Pin;
use esp_hal::interrupt::software::{SoftwareInterrupt, SoftwareInterruptControl};
use esp_hal::peripherals::CPU_CTRL;
use esp_hal::system::Stack as HalStack;
use esp_hal::time::Rate;
use esp_hal::timer::timg::{MwdtStage, MwdtStageAction, TimerGroup};
use esp_hub75::Hub75Pins16;
use log::info;
use static_cell::StaticCell;
use widgets::admin::server::{Server, admin_api_task};
use widgets::backend::lcd::{LCDCAM64x64, LCDCAM64x64Drawer, LCDCAM64x64Flusher};
use widgets::boot_screen;
use widgets::console::ConsoleLogger;
use widgets::http::HttpService;
use widgets::network::NetworkService;
use widgets::time::timer::TimeService;
use widgets::widget::manager::WidgetManager;

use embassy_net::{Config as NetConfig, DhcpConfig, Runner, StackResources};
use esp_radio::wifi::{Config as WifiConfig, WifiController, sta::StationConfig};
use widgets::config::Config;
use widgets::time_sync::sntp::{TimeSyncService, time_sync_task};

// Generated from config.json at build time.
include!(concat!(env!("OUT_DIR"), "/config.rs"));

static STACK_NET: StaticCell<StackResources<8>> = StaticCell::new();
static FLUSHER_STACK: StaticCell<HalStack<2048>> = StaticCell::new();

// This creates a default app-descriptor required by the esp-idf bootloader.
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
async fn connection_task(
    mut controller: WifiController<'static>,
    wifi_config: widgets::config::WifiConfig,
) {
    loop {
        if !controller.is_connected() {
            info!("Connecting to Wi-Fi...");
            let config = WifiConfig::Station(
                StationConfig::default()
                    .with_ssid(wifi_config.ssid.as_str())
                    .with_password(wifi_config.password.clone()),
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

fn setup_network(
    spawner: Spawner,
    wifi: esp_hal::peripherals::WIFI<'static>,
    wifi_config: widgets::config::WifiConfig,
) -> embassy_net::Stack<'static> {
    let (wifi_controller, interfaces) = esp_radio::wifi::new(wifi, Default::default())
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
    spawner.spawn(connection_task(wifi_controller, wifi_config).unwrap());

    stack
}

fn init_display(
    lcd_cam: esp_hal::peripherals::LCD_CAM<'static>,
    dma_ch0: esp_hal::peripherals::DMA_CH0<'static>,
    cpu_ctrl: CPU_CTRL,
    sw_int: SoftwareInterrupt<'static, 1>,
    pins: Hub75Pins16<'static>,
    display_config: &widgets::config::DisplayConfig,
) -> LCDCAM64x64Drawer {
    let freq = Rate::from_mhz(display_config.freq_mhz);
    let LCDCAM64x64(flusher, drawer) = LCDCAM64x64::new(pins, lcd_cam, dma_ch0, freq);
    init_display_flusher(cpu_ctrl, sw_int, flusher);
    drawer
}

fn register_widgets(
    mgr: &mut WidgetManager,
    app_config: &Config,
    file_system: &widgets::storage::FS,
) {
    for widget in app_config.widgets.borrow().iter() {
        mgr.add_widget(widget.as_widget(file_system.clone()));
    }
}

#[esp_rtos::main]
#[allow(clippy::large_stack_frames)]
async fn main(spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();

    let hal_config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(hal_config);

    esp_alloc::heap_allocator!(size: 90 * 1024);
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 64000);
    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let mut wdt0 = timg0.wdt;
    wdt0.enable();
    wdt0.set_timeout(MwdtStage::Stage0, esp_hal::time::Duration::from_secs(20));
    wdt0.set_stage_action(MwdtStage::Stage0, MwdtStageAction::ResetSystem);

    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    info!("Embassy initialized!");

    let file_system = widgets::storage::init(peripherals.FLASH);
    let app_config = Config::load(file_system.clone()).expect("failed to load config");

    // Init display early so we can show a boot screen while Wi-Fi connects.
    let pins = hub75_pins!(peripherals);
    let drawer = init_display(
        peripherals.LCD_CAM,
        peripherals.DMA_CH0,
        peripherals.CPU_CTRL,
        sw_interrupt.software_interrupt1,
        pins,
        &app_config.display,
    );

    let stack = setup_network(spawner, peripherals.WIFI, app_config.wifi.clone());
    // Animate while waiting for DHCP.
    info!("Waiting for Wi-Fi DHCP...");
    boot_screen::run(&drawer, stack).await;
    info!("Wi-Fi DHCP configured!");
    if let Some(config) = stack.config_v4() {
        info!("IP Address: {:?}", config.address);
    }

    widgets::allocator::use_psram_heap();

    let time_sync: &'static TimeSyncService = Box::leak(Box::new(TimeSyncService::new(stack)));
    spawner.spawn(time_sync_task(time_sync).unwrap());

    let raw_manager = WidgetManager::new(
        Box::new(drawer),
        Box::new(TimeService::new(time_sync)),
        Box::new(HttpService::new(spawner, stack)),
        Box::new(ConsoleLogger::new()),
        Box::new(NetworkService::new(stack)),
        file_system.clone(),
        app_config.clone(),
    );
    let manager = Rc::new(RefCell::new(raw_manager));
    {
        let mut mgr = manager.borrow_mut();
        register_widgets(&mut mgr, &app_config, &file_system);
    }
    manager.borrow_mut().render();

    spawner
        .spawn(admin_api_task(Server::new(stack, file_system.clone(), manager.clone())).unwrap());

    loop {
        wdt0.feed();
        manager.borrow_mut().poll_events();
        Timer::after_millis(10).await;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strspn(
    cs: *const core::ffi::c_char,
    ct: *const core::ffi::c_char,
) -> usize {
    let mut p = cs;
    unsafe {
        while *p != 0 {
            let mut r = ct;
            let mut found = false;
            while *r != 0 {
                if *p == *r {
                    found = true;
                    break;
                }
                r = r.add(1);
            }
            if !found {
                break;
            }
            p = p.add(1);
        }
        p.offset_from(cs) as usize
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strcspn(
    cs: *const core::ffi::c_char,
    ct: *const core::ffi::c_char,
) -> usize {
    let mut p = cs;
    unsafe {
        while *p != 0 {
            let mut r = ct;
            while *r != 0 {
                if *p == *r {
                    return p.offset_from(cs) as usize;
                }
                r = r.add(1);
            }
            p = p.add(1);
        }
        p.offset_from(cs) as usize
    }
}
