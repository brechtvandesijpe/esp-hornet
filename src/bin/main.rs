//! WiFi sniffer example
//!
//! Sniffs for beacon frames.

//% FEATURES: esp-wifi esp-wifi/wifi esp-wifi/sniffer esp-hal/unstable
//% CHIPS: esp32 esp32s2 esp32s3 esp32c2 esp32c3 esp32c6

#![no_std]
#![no_main]

extern crate alloc;

use esp_backtrace as _;
use esp_hal::{clock::CpuClock, rng::Rng, timer::timg::TimerGroup};
use embassy_time::{Duration, Timer};
use esp_wifi::init;
use embassy_executor::Spawner;

esp_bootloader_esp_idf::esp_app_desc!();

mod sting;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);
    
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);
    
    let timg1 = TimerGroup::new(peripherals.TIMG1);
    let esp_wifi_ctrl = init(timg1.timer0, Rng::new(peripherals.RNG)).unwrap();
    let (wifi_controller, interfaces) = esp_wifi::wifi::new(&esp_wifi_ctrl, peripherals.WIFI).unwrap();

    let mut sting_guard = sting::init_sting(wifi_controller, interfaces);

    sting_guard.start_sniff().await;
    Timer::after(Duration::from_millis(10000)).await;
    sting_guard.stop_sniff().await;
    
    loop {}
}