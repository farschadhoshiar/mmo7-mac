use crate::hid::report::RawReport;
use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub const BASELINE_DURATION: Duration = Duration::from_millis(1500);
pub const RECORD_DURATION: Duration = Duration::from_millis(1500);

#[derive(Clone, Copy, Debug)]
pub struct Probe {
    pub key: &'static str,
    pub name: &'static str,
    pub hint: &'static str,
}

pub const PROBES: &[Probe] = &[
    Probe { key: "wheel_up",   name: "Scroll Up",                    hint: "Roll the wheel UP a few notches" },
    Probe { key: "wheel_down", name: "Scroll Down",                  hint: "Roll the wheel DOWN a few notches" },
    Probe { key: "wheel_click",name: "Wheel Click",                  hint: "Press and HOLD the scroll wheel" },
    Probe { key: "side_back",  name: "Back Side Button",             hint: "Hold the BACK thumb-side button (rearmost)" },
    Probe { key: "side_fwd",   name: "Forward Side Button",          hint: "Hold the FORWARD thumb-side button" },
    Probe { key: "sniper",     name: "Sniper / Precision",           hint: "Hold the sniper button (left of LMB)" },
    Probe { key: "shift",      name: "Shift Modifier",               hint: "Hold the SHIFT modifier button" },
    Probe { key: "mode",       name: "Mode Cycle",                   hint: "Tap the MODE button ONCE" },
    Probe { key: "5d_up",      name: "5D — Up",                      hint: "Push the 5D button UP" },
    Probe { key: "5d_down",    name: "5D — Down",                    hint: "Push the 5D button DOWN" },
    Probe { key: "5d_left",    name: "5D — Left",                    hint: "Push the 5D button LEFT" },
    Probe { key: "5d_right",   name: "5D — Right",                   hint: "Push the 5D button RIGHT" },
    Probe { key: "5d_press",   name: "5D — Click",                   hint: "Press the 5D button STRAIGHT IN" },
    Probe { key: "thumb_1",    name: "Thumb 1",                      hint: "Hold thumb button 1 (top-row left)" },
    Probe { key: "thumb_2",    name: "Thumb 2",                      hint: "Hold thumb button 2" },
    Probe { key: "thumb_3",    name: "Thumb 3",                      hint: "Hold thumb button 3" },
    Probe { key: "thumb_4",    name: "Thumb 4",                      hint: "Hold thumb button 4" },
    Probe { key: "thumb_5",    name: "Thumb 5",                      hint: "Hold thumb button 5" },
    Probe { key: "thumb_6",    name: "Thumb 6 (top-row right)",      hint: "Hold thumb button 6" },
    Probe { key: "thumb_7",    name: "Thumb 7 (bottom-row left)",    hint: "Hold thumb button 7" },
    Probe { key: "thumb_8",    name: "Thumb 8",                      hint: "Hold thumb button 8" },
    Probe { key: "thumb_9",    name: "Thumb 9",                      hint: "Hold thumb button 9" },
    Probe { key: "thumb_10",   name: "Thumb 10",                     hint: "Hold thumb button 10" },
    Probe { key: "thumb_11",   name: "Thumb 11",                     hint: "Hold thumb button 11" },
    Probe { key: "thumb_12",   name: "Thumb 12 (bottom-row right)",  hint: "Hold thumb button 12" },
];

#[derive(Clone, Default, Debug)]
pub struct Baseline {
    pub per_iface: HashMap<u8, Vec<Vec<u8>>>,
}

impl Baseline {
    pub fn record(&mut self, report: &RawReport) {
        self.per_iface
            .entry(report.iface.id)
            .or_default()
            .push(report.bytes.clone());
    }

    pub fn known_iface(&self, id: u8) -> bool {
        self.per_iface.contains_key(&id)
    }

    pub fn values_at(&self, iface: u8, byte_index: usize) -> HashSet<u8> {
        self.per_iface
            .get(&iface)
            .map(|history| history.iter().filter_map(|h| h.get(byte_index).copied()).collect())
            .unwrap_or_default()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ButtonMapping {
    pub iface_id: u8,
    pub byte_index: usize,
    pub mask: u8,
    pub baseline_value: u8,
    pub pressed_value: u8,
    pub occurrences: u32,
}

pub fn diff(baseline: &Baseline, pressed: &[RawReport]) -> Vec<ButtonMapping> {
    let mut findings: HashMap<(u8, usize, u8, u8), ButtonMapping> = HashMap::new();

    for r in pressed {
        let known = baseline.known_iface(r.iface.id);
        for (i, &p) in r.bytes.iter().enumerate() {
            let baseline_vals = if known {
                baseline.values_at(r.iface.id, i)
            } else {
                let mut s = HashSet::new();
                s.insert(0u8);
                s
            };
            if baseline_vals.contains(&p) {
                continue;
            }
            let base = *baseline_vals.iter().next().unwrap_or(&0);
            let mask = base ^ p;
            let entry = findings
                .entry((r.iface.id, i, base, p))
                .or_insert(ButtonMapping {
                    iface_id: r.iface.id,
                    byte_index: i,
                    mask,
                    baseline_value: base,
                    pressed_value: p,
                    occurrences: 0,
                });
            entry.occurrences += 1;
        }
    }

    let mut v: Vec<_> = findings.into_values().collect();
    v.sort_by(|a, b| {
        b.occurrences
            .cmp(&a.occurrences)
            .then_with(|| a.iface_id.cmp(&b.iface_id))
            .then_with(|| a.byte_index.cmp(&b.byte_index))
    });
    v
}

#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub probe: Probe,
    pub mapping: Option<ButtonMapping>,
}

#[derive(Debug)]
pub enum WizardStep {
    Intro,
    Baseline {
        started: Instant,
        baseline: Baseline,
    },
    Ready {
        probe_idx: usize,
        baseline: Baseline,
    },
    Recording {
        probe_idx: usize,
        baseline: Baseline,
        started: Instant,
        captured: Vec<RawReport>,
    },
    Result {
        probe_idx: usize,
        baseline: Baseline,
        mappings: Vec<ButtonMapping>,
    },
    Done {
        save_path: Option<PathBuf>,
        save_error: Option<String>,
    },
}

pub struct Wizard {
    pub step: WizardStep,
    pub results: Vec<ProbeResult>,
}

impl Wizard {
    pub fn new() -> Self {
        Self {
            step: WizardStep::Intro,
            results: Vec::new(),
        }
    }

    pub fn on_report(&mut self, r: &RawReport) {
        match &mut self.step {
            WizardStep::Baseline { baseline, .. } => baseline.record(r),
            WizardStep::Recording { captured, .. } => captured.push(r.clone()),
            _ => {}
        }
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        let next = match &mut self.step {
            WizardStep::Baseline { started, baseline } => {
                if now.duration_since(*started) >= BASELINE_DURATION {
                    Some(WizardStep::Ready {
                        probe_idx: 0,
                        baseline: baseline.clone(),
                    })
                } else {
                    None
                }
            }
            WizardStep::Recording { probe_idx, baseline, started, captured } => {
                if now.duration_since(*started) >= RECORD_DURATION {
                    let mappings = diff(baseline, captured);
                    Some(WizardStep::Result {
                        probe_idx: *probe_idx,
                        baseline: baseline.clone(),
                        mappings,
                    })
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(s) = next {
            self.step = s;
        }
    }

    pub fn on_space(&mut self) {
        let next = match &self.step {
            WizardStep::Intro => Some(WizardStep::Baseline {
                started: Instant::now(),
                baseline: Baseline::default(),
            }),
            WizardStep::Ready { probe_idx, baseline } => Some(WizardStep::Recording {
                probe_idx: *probe_idx,
                baseline: baseline.clone(),
                started: Instant::now(),
                captured: Vec::new(),
            }),
            _ => None,
        };
        if let Some(s) = next {
            self.step = s;
        }
    }

    pub fn on_accept(&mut self) {
        if let WizardStep::Result { probe_idx, baseline, mappings } = &self.step {
            let probe = PROBES[*probe_idx];
            let chosen = mappings.first().cloned();
            self.results.push(ProbeResult { probe, mapping: chosen });
            self.advance(*probe_idx, baseline.clone());
        }
    }

    pub fn on_skip(&mut self) {
        let (idx, baseline) = match &self.step {
            WizardStep::Ready { probe_idx, baseline } => (*probe_idx, baseline.clone()),
            WizardStep::Recording { probe_idx, baseline, .. } => (*probe_idx, baseline.clone()),
            WizardStep::Result { probe_idx, baseline, .. } => (*probe_idx, baseline.clone()),
            _ => return,
        };
        let probe = PROBES[idx];
        self.results.push(ProbeResult { probe, mapping: None });
        self.advance(idx, baseline);
    }

    pub fn on_retry(&mut self) {
        match &self.step {
            WizardStep::Result { probe_idx, baseline, .. } => {
                self.step = WizardStep::Ready {
                    probe_idx: *probe_idx,
                    baseline: baseline.clone(),
                };
            }
            WizardStep::Recording { probe_idx, baseline, .. } => {
                self.step = WizardStep::Ready {
                    probe_idx: *probe_idx,
                    baseline: baseline.clone(),
                };
            }
            _ => {}
        }
    }

    pub fn on_rebaseline(&mut self) {
        self.step = WizardStep::Baseline {
            started: Instant::now(),
            baseline: Baseline::default(),
        };
    }

    fn advance(&mut self, current_idx: usize, baseline: Baseline) {
        let next = current_idx + 1;
        if next >= PROBES.len() {
            let (path, err) = match save_results(&self.results) {
                Ok(p) => (Some(p), None),
                Err(e) => (None, Some(e.to_string())),
            };
            self.step = WizardStep::Done {
                save_path: path,
                save_error: err,
            };
        } else {
            self.step = WizardStep::Ready {
                probe_idx: next,
                baseline,
            };
        }
    }
}

fn save_results(results: &[ProbeResult]) -> std::io::Result<PathBuf> {
    let mut out = String::new();
    let _ = writeln!(out, "# mmo7-mac discovered mapping");
    let _ = writeln!(out, "# generated by the interactive wizard\n");
    let _ = writeln!(out, "[device]");
    let _ = writeln!(out, "vid = 0x0738");
    let _ = writeln!(out, "pid = 0x1C02\n");

    for r in results {
        let mapped = r.mapping.is_some();
        let _ = writeln!(out, "[[buttons]]");
        let _ = writeln!(out, "key = \"{}\"", r.probe.key);
        let _ = writeln!(out, "name = \"{}\"", r.probe.name);
        if let Some(m) = &r.mapping {
            let _ = writeln!(out, "iface = {}", m.iface_id);
            let _ = writeln!(out, "byte = {}", m.byte_index);
            let _ = writeln!(out, "mask = 0x{:02X}", m.mask);
            let _ = writeln!(out, "baseline = 0x{:02X}", m.baseline_value);
            let _ = writeln!(out, "pressed = 0x{:02X}", m.pressed_value);
            let _ = writeln!(out, "observations = {}", m.occurrences);
        } else {
            let _ = writeln!(out, "mapped = false");
        }
        let _ = writeln!(out);
        let _ = mapped;
    }

    let path = PathBuf::from("mmo7-mapping.toml");
    std::fs::write(&path, out)?;
    Ok(path)
}
