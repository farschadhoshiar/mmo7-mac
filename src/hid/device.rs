use crate::hid::report::RawReport;
use hidapi::{HidApi, HidDevice};
use std::time::Duration;
use tokio::sync::{mpsc, watch};
use tracing::{debug, info, warn};

pub const VENDOR_ID_MADCATZ: u16 = 0x0738;
pub const VENDOR_ID_SAITEK: u16 = 0x06a3;
pub const PRODUCT_ID_MMO7_PLUS: u16 = 0x1C02;
pub const PRODUCT_ID_MMO7_LEGACY_MC: u16 = 0x1713;
pub const PRODUCT_ID_MMO7_LEGACY_SAITEK: u16 = 0x0CD0;

const RECONNECT_DELAY: Duration = Duration::from_millis(750);
const READ_TIMEOUT_MS: i32 = 100;
const MAX_REPORT_LEN: usize = 64;
const CHANNEL_CAPACITY: usize = 512;

const CANDIDATES: &[(u16, u16)] = &[
    (VENDOR_ID_MADCATZ, PRODUCT_ID_MMO7_PLUS),
    (VENDOR_ID_MADCATZ, PRODUCT_ID_MMO7_LEGACY_MC),
    (VENDOR_ID_SAITEK, PRODUCT_ID_MMO7_LEGACY_SAITEK),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Searching,
    Connected { vid: u16, pid: u16 },
}

pub struct DeviceHandles {
    pub reports: mpsc::Receiver<RawReport>,
    pub state: watch::Receiver<ConnectionState>,
}

pub fn spawn_reader() -> DeviceHandles {
    let (report_tx, report_rx) = mpsc::channel::<RawReport>(CHANNEL_CAPACITY);
    let (state_tx, state_rx) = watch::channel(ConnectionState::Searching);

    tokio::task::spawn_blocking(move || reader_loop(report_tx, state_tx));

    DeviceHandles { reports: report_rx, state: state_rx }
}

fn reader_loop(report_tx: mpsc::Sender<RawReport>, state_tx: watch::Sender<ConnectionState>) {
    let mut api = match HidApi::new() {
        Ok(api) => api,
        Err(e) => {
            warn!("failed to init hidapi: {e}");
            return;
        }
    };

    loop {
        if report_tx.is_closed() {
            return;
        }

        let _ = state_tx.send(ConnectionState::Searching);

        if let Err(e) = api.refresh_devices() {
            debug!("refresh_devices: {e}");
        }

        let opened = open_first_match(&api);

        let Some((device, vid, pid)) = opened else {
            std::thread::sleep(RECONNECT_DELAY);
            continue;
        };

        info!("connected to {:04X}:{:04X}", vid, pid);
        let _ = state_tx.send(ConnectionState::Connected { vid, pid });

        pump_reports(&device, &report_tx);
        warn!("device {:04X}:{:04X} disconnected", vid, pid);
    }
}

fn open_first_match(api: &HidApi) -> Option<(HidDevice, u16, u16)> {
    for &(vid, pid) in CANDIDATES {
        let available = api
            .device_list()
            .any(|d| d.vendor_id() == vid && d.product_id() == pid);
        if !available {
            continue;
        }
        match api.open(vid, pid) {
            Ok(dev) => return Some((dev, vid, pid)),
            Err(e) => debug!("could not open {:04X}:{:04X}: {e}", vid, pid),
        }
    }
    None
}

fn pump_reports(device: &HidDevice, report_tx: &mpsc::Sender<RawReport>) {
    let mut buf = [0u8; MAX_REPORT_LEN];
    loop {
        if report_tx.is_closed() {
            return;
        }
        match device.read_timeout(&mut buf, READ_TIMEOUT_MS) {
            Ok(0) => continue,
            Ok(n) => {
                let report = RawReport::new(buf[..n].to_vec());
                if report_tx.blocking_send(report).is_err() {
                    return;
                }
            }
            Err(e) => {
                debug!("hid read error: {e}");
                return;
            }
        }
    }
}
