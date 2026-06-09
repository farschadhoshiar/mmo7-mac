use crate::hid::report::RawReport;
use hidapi::{HidApi, HidDevice};
use std::ffi::CString;
use std::sync::Arc;
use std::thread;
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
const CHANNEL_CAPACITY: usize = 1024;

const CANDIDATES: &[(u16, u16)] = &[
    (VENDOR_ID_MADCATZ, PRODUCT_ID_MMO7_PLUS),
    (VENDOR_ID_MADCATZ, PRODUCT_ID_MMO7_LEGACY_MC),
    (VENDOR_ID_SAITEK, PRODUCT_ID_MMO7_LEGACY_SAITEK),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceInfo {
    pub id: u8,
    pub vid: u16,
    pub pid: u16,
    pub usage_page: u16,
    pub usage: u16,
    pub interface_number: i32,
    pub product_name: String,
}

impl InterfaceInfo {
    pub fn role_hint(&self) -> &'static str {
        match (self.usage_page, self.usage) {
            (0x01, 0x02) => "mouse",
            (0x01, 0x06) => "keyboard",
            (0x01, 0x80) => "syscontrol",
            (0x0C, _) => "consumer",
            (page, _) if page >= 0xFF00 => "vendor",
            _ => "other",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Searching,
    Connected { interfaces: Vec<InterfaceInfo> },
}

pub struct DeviceHandles {
    pub reports: mpsc::Receiver<RawReport>,
    pub state: watch::Receiver<ConnectionState>,
}

pub fn spawn_reader() -> DeviceHandles {
    let (report_tx, report_rx) = mpsc::channel::<RawReport>(CHANNEL_CAPACITY);
    let (state_tx, state_rx) = watch::channel(ConnectionState::Searching);

    tokio::task::spawn_blocking(move || supervisor(report_tx, state_tx));

    DeviceHandles { reports: report_rx, state: state_rx }
}

struct DeviceSnapshot {
    path: CString,
    vid: u16,
    pid: u16,
    usage_page: u16,
    usage: u16,
    interface_number: i32,
    product_name: String,
}

fn supervisor(report_tx: mpsc::Sender<RawReport>, state_tx: watch::Sender<ConnectionState>) {
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

        let snapshots: Vec<DeviceSnapshot> = api
            .device_list()
            .filter(|d| {
                CANDIDATES
                    .iter()
                    .any(|&(v, p)| d.vendor_id() == v && d.product_id() == p)
            })
            .map(|d| DeviceSnapshot {
                path: d.path().to_owned(),
                vid: d.vendor_id(),
                pid: d.product_id(),
                usage_page: d.usage_page(),
                usage: d.usage(),
                interface_number: d.interface_number(),
                product_name: d.product_string().unwrap_or("").to_string(),
            })
            .collect();

        if snapshots.is_empty() {
            thread::sleep(RECONNECT_DELAY);
            continue;
        }

        let mut opened: Vec<(HidDevice, Arc<InterfaceInfo>)> = Vec::new();
        for (i, snap) in snapshots.iter().enumerate() {
            match api.open_path(&snap.path) {
                Ok(dev) => {
                    let info = Arc::new(InterfaceInfo {
                        id: i as u8,
                        vid: snap.vid,
                        pid: snap.pid,
                        usage_page: snap.usage_page,
                        usage: snap.usage,
                        interface_number: snap.interface_number,
                        product_name: snap.product_name.clone(),
                    });
                    opened.push((dev, info));
                }
                Err(e) => debug!(
                    "open_path failed for iface {} ({:04X}:{:04X} up={:04X} u={:04X}): {e}",
                    i, snap.vid, snap.pid, snap.usage_page, snap.usage
                ),
            }
        }

        if opened.is_empty() {
            thread::sleep(RECONNECT_DELAY);
            continue;
        }

        let interfaces: Vec<InterfaceInfo> =
            opened.iter().map(|(_, info)| (**info).clone()).collect();
        info!("opened {} interfaces", opened.len());
        let _ = state_tx.send(ConnectionState::Connected {
            interfaces: interfaces.clone(),
        });

        let mut handles = Vec::with_capacity(opened.len());
        for (device, info) in opened {
            let tx = report_tx.clone();
            handles.push(thread::spawn(move || pump(device, info, tx)));
        }
        for h in handles {
            let _ = h.join();
        }
        warn!("all interfaces disconnected");
    }
}

fn pump(device: HidDevice, info: Arc<InterfaceInfo>, tx: mpsc::Sender<RawReport>) {
    let mut buf = [0u8; MAX_REPORT_LEN];
    loop {
        if tx.is_closed() {
            return;
        }
        match device.read_timeout(&mut buf, READ_TIMEOUT_MS) {
            Ok(0) => continue,
            Ok(n) => {
                let report = RawReport::new(info.clone(), buf[..n].to_vec());
                if tx.blocking_send(report).is_err() {
                    return;
                }
            }
            Err(e) => {
                debug!("read error on iface {}: {e}", info.id);
                return;
            }
        }
    }
}
