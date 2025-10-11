use esp_println::println;
use alloc::{collections::btree_set::BTreeSet, string::{String, ToString}};
use core::cell::RefCell;
use core::marker::PhantomData;
use critical_section::Mutex;
use esp_wifi::{init, wifi};
use ieee80211::{match_frames, mgmt_frame::{BeaconFrame, body::HasElements}};

#[cfg(feature = "sting")]
static KNOWN_SSIDS: Mutex<RefCell<BTreeSet<String>>> = Mutex::new(RefCell::new(BTreeSet::new()));

#[cfg(feature = "sting")]
fn get_security_from_beacon(beacon: &BeaconFrame) -> &'static str {
    let mut has_wpa = false;
    let mut has_rsn = false;
    let mut has_sae = false;

    let mut pos = 0;
    let bytes = beacon.get_elements().bytes;

    while pos + 2 <= bytes.len() {
        let id = bytes[pos];
        let len = bytes[pos + 1] as usize;

        // safety check
        if pos + 2 + len > bytes.len() {
            break;
        }

        let data = &bytes[pos + 2..pos + 2 + len];

        match id {
            48 => { // RSN IE
                has_rsn = true;

                // check AKM suite for SAE (00:0F:AC:08)
                if data.windows(4).any(|b| b == [0x00, 0x0F, 0xAC, 0x08]) {
                    has_sae = true;
                }
            }
            221 => { // Vendor specific (WPA)
                if data.len() >= 4
                    && &data[0..3] == [0x00, 0x50, 0xF2]
                    && data[3] == 0x01
                {
                    has_wpa = true;
                }
            }
            _ => {}
        }

        pos += 2 + len;
    }

    match (has_wpa, has_rsn, has_sae) {
        (_, _, true) => "WPA3",
        (true, true, _) => "WPA/WPA2 mixed",
        (false, true, _) => "WPA2",
        (true, false, _) => "WPA",
        _ => "Open",
    }
}

// a generic no-op function to satisfy the expected for<'a> fn(PromiscuousPkt<'a>) type
#[cfg(feature = "sting")]
fn noop_promiscuous<'a>(_pkt: wifi::PromiscuousPkt<'a>) {}

#[cfg(feature = "sting")]
pub struct StingGuard<'a> {
    // own the controller & interfaces so they live while the guard exists
    wifi_controller: wifi::WifiController<'a>,
    interfaces: wifi::Interfaces<'a>,
}

#[cfg(not(feature = "sting"))]
pub struct StingGuard<'a>(PhantomData<&'a ()>);

#[cfg(feature = "sting")]
impl<'a> Drop for StingGuard<'a> {
    fn drop(&mut self) {
        // cleanup: stop WiFi/sniffer if API allows
        let _ = self.wifi_controller.stop();
        // clear callback by setting a no-op fn (API expects a fn pointer, not Option)
        let _ = self.interfaces.sniffer.set_receive_cb(noop_promiscuous);
    }
}

impl<'a> StingGuard<'a> {
    pub fn start_sniff(&mut self) {
        let _ = self.interfaces.sniffer.set_promiscuous_mode(true);
    }
}

#[cfg(feature = "sting")]
pub fn init_sting<'a>(
    mut wifi_controller: wifi::WifiController<'a>,
    mut interfaces: wifi::Interfaces<'a>,
) -> StingGuard<'a> {
    crate::logger::println!("Sting: ENABLED");
    wifi_controller.set_mode(wifi::WifiMode::Sta).unwrap();
    wifi_controller.start().unwrap();

    // interfaces.sniffer.set_promiscuous_mode(true).unwrap();
    // non-capturing closure coerces to fn pointer; it doesn't capture local state
    interfaces.sniffer.set_receive_cb(|packet| {
        let _ = match_frames! {
            packet.data,
            beacon = BeaconFrame => {
                let Some(ssid) = beacon.ssid() else { return; };
                let handshake = get_security_from_beacon(&beacon);
                if critical_section::with(|cs| {
                    KNOWN_SSIDS.borrow_ref_mut(cs).insert(ssid.to_string())
                }) {
                    println!("Network: {ssid} ({handshake})");
                }
            }
        };
    });

    StingGuard {
        wifi_controller,
        interfaces,
    }
}

#[cfg(not(feature = "sting"))]
pub fn init_sting<'a, A, B>(_a: A, _b: B) -> StingGuard<'a> {
    StingGuard(PhantomData)
}