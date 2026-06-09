use crate::hid::device::{ConnectionState, InterfaceInfo};
use crate::hid::report::RawReport;
use crate::wizard::Wizard;
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;

const MAX_REPORTS: usize = 4096;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum View {
    Wizard,
    Sniffer,
}

pub struct App {
    pub running: bool,
    pub view: View,
    pub paused: bool,
    pub follow: bool,
    pub scroll: usize,
    pub connection: ConnectionState,
    pub reports: VecDeque<RawReport>,
    pub started_at: Instant,
    pub total_received: u64,
    pub per_iface_counts: HashMap<u8, u64>,
    pub hidden_ifaces: HashSet<u8>,
    pub wizard: Wizard,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            view: View::Wizard,
            paused: false,
            follow: true,
            scroll: 0,
            connection: ConnectionState::Searching,
            reports: VecDeque::with_capacity(MAX_REPORTS),
            started_at: Instant::now(),
            total_received: 0,
            per_iface_counts: HashMap::new(),
            hidden_ifaces: HashSet::new(),
            wizard: Wizard::new(),
        }
    }

    pub fn interfaces(&self) -> &[InterfaceInfo] {
        match &self.connection {
            ConnectionState::Connected { interfaces } => interfaces,
            ConnectionState::Searching => &[],
        }
    }

    pub fn push_report(&mut self, report: RawReport) {
        self.total_received = self.total_received.wrapping_add(1);
        *self.per_iface_counts.entry(report.iface.id).or_insert(0) += 1;

        self.wizard.on_report(&report);

        if self.paused || self.hidden_ifaces.contains(&report.iface.id) {
            return;
        }
        if self.reports.len() == MAX_REPORTS {
            self.reports.pop_front();
            self.scroll = self.scroll.saturating_sub(1);
        }
        self.reports.push_back(report);
        if self.follow {
            self.scroll = self.reports.len().saturating_sub(1);
        }
    }

    pub fn on_connection_changed(&mut self) {
        self.per_iface_counts.clear();
        self.hidden_ifaces.clear();
    }

    pub fn clear(&mut self) {
        self.reports.clear();
        self.scroll = 0;
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.scroll = self.scroll.saturating_sub(n);
        self.follow = false;
    }

    pub fn scroll_down(&mut self, n: usize) {
        let max = self.reports.len().saturating_sub(1);
        self.scroll = self.scroll.saturating_add(n).min(max);
        self.follow = self.scroll == max;
    }

    pub fn jump_top(&mut self) {
        self.scroll = 0;
        self.follow = false;
    }

    pub fn jump_bottom(&mut self) {
        self.scroll = self.reports.len().saturating_sub(1);
        self.follow = true;
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn toggle_follow(&mut self) {
        self.follow = !self.follow;
        if self.follow {
            self.scroll = self.reports.len().saturating_sub(1);
        }
    }

    pub fn toggle_iface_visibility(&mut self, id: u8) {
        if self.hidden_ifaces.contains(&id) {
            self.hidden_ifaces.remove(&id);
        } else {
            self.hidden_ifaces.insert(id);
        }
    }

    pub fn toggle_view(&mut self) {
        self.view = match self.view {
            View::Wizard => View::Sniffer,
            View::Sniffer => View::Wizard,
        };
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
