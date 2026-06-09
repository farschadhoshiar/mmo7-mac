use crate::hid::device::InterfaceInfo;
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct RawReport {
    pub iface: Arc<InterfaceInfo>,
    pub ts: Instant,
    pub bytes: Vec<u8>,
}

impl RawReport {
    pub fn new(iface: Arc<InterfaceInfo>, bytes: Vec<u8>) -> Self {
        Self {
            iface,
            ts: Instant::now(),
            bytes,
        }
    }

    pub fn hex(&self) -> String {
        use std::fmt::Write;
        let mut s = String::with_capacity(self.bytes.len() * 3);
        for (i, b) in self.bytes.iter().enumerate() {
            if i > 0 {
                s.push(' ');
            }
            let _ = write!(s, "{:02X}", b);
        }
        s
    }

    pub fn ascii(&self) -> String {
        self.bytes
            .iter()
            .map(|b| if b.is_ascii_graphic() { *b as char } else { '·' })
            .collect()
    }
}
