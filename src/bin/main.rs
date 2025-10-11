//! WiFi sniffer example
//!
//! Sniffs for beacon frames.

//% FEATURES: esp-wifi esp-wifi/wifi esp-wifi/sniffer esp-hal/unstable
//% CHIPS: esp32 esp32s2 esp32s3 esp32c2 esp32c3 esp32c6

#![no_std]
#![no_main]

extern crate alloc;

use alloc::{
    collections::btree_set::BTreeSet,
    string::{String, ToString},
};
use core::cell::RefCell;

use critical_section::Mutex;
use esp_backtrace as _;
use esp_hal::{clock::CpuClock, main, rng::Rng, timer::timg::TimerGroup};
use esp_println::println;
use esp_wifi::{init, wifi};


esp_bootloader_esp_idf::esp_app_desc!();

// static KNOWN_SSIDS: Mutex<RefCell<BTreeSet<String>>> = Mutex::new(RefCell::new(BTreeSet::new()));

mod sting;

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let esp_wifi_ctrl = init(timg0.timer0, Rng::new(peripherals.RNG)).unwrap();
    let (mut wifi_controller, mut interfaces) = esp_wifi::wifi::new(&esp_wifi_ctrl, peripherals.WIFI).unwrap();

    let _sting_guard = sting::init_sting(wifi_controller, interfaces);

    loop {}
}