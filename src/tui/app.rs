use crate::hid::device::ConnectionState;
use crate::hid::report::RawReport;
use std::collections::VecDeque;
use std::time::Instant;

const MAX_REPORTS: usize = 2048;

pub struct App {
    pub running: bool,
    pub paused: bool,
    pub follow: bool,
    pub scroll: usize,
    pub connection: ConnectionState,
    pub reports: VecDeque<RawReport>,
    pub started_at: Instant,
    pub total_received: u64,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            paused: false,
            follow: true,
            scroll: 0,
            connection: ConnectionState::Searching,
            reports: VecDeque::with_capacity(MAX_REPORTS),
            started_at: Instant::now(),
            total_received: 0,
        }
    }

    pub fn push_report(&mut self, report: RawReport) {
        self.total_received = self.total_received.wrapping_add(1);
        if self.paused {
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

    pub fn quit(&mut self) {
        self.running = false;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
