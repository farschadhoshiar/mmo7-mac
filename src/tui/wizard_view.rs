use crate::tui::app::App;
use crate::wizard::{BASELINE_DURATION, ButtonMapping, PROBES, RECORD_DURATION, WizardStep};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
};
use std::time::Instant;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" mapping wizard ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    match &app.wizard.step {
        WizardStep::Intro => render_intro(frame, inner, app),
        WizardStep::Baseline { started, .. } => render_baseline(frame, inner, *started),
        WizardStep::Ready { probe_idx, .. } => render_ready(frame, inner, *probe_idx, app),
        WizardStep::Recording { probe_idx, started, captured, .. } => {
            render_recording(frame, inner, *probe_idx, *started, captured.len())
        }
        WizardStep::Result { probe_idx, mappings, .. } => {
            render_result(frame, inner, *probe_idx, mappings)
        }
        WizardStep::Done { save_path, save_error } => {
            render_done(frame, inner, app, save_path.as_ref().map(|p| p.display().to_string()), save_error.as_deref())
        }
    }
}

fn split_two(area: Rect, top: u16) -> [Rect; 2] {
    let l = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(top), Constraint::Min(0)])
        .split(area);
    [l[0], l[1]]
}

fn render_intro(frame: &mut Frame, area: Rect, app: &App) {
    let opened = app.interfaces().iter().filter(|i| i.opened).count();
    let total = app.interfaces().len();
    let lines = vec![
        Line::from(Span::styled(
            "Interactive button mapping wizard",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!(
            "{} of {} HID interfaces opened (mouse + keyboard skipped)",
            opened, total
        )),
        Line::from(""),
        Line::from("How it works:"),
        Line::from("  1. Don't touch the mouse — we record a 1.5s idle baseline"),
        Line::from("  2. We prompt you to press one input at a time"),
        Line::from("  3. We diff baseline ↔ pressed bytes and store the mapping"),
        Line::from("  4. At the end, mappings are saved to mmo7-mapping.toml"),
        Line::from(""),
        Line::from(Span::styled(
            "Press SPACE to begin · Tab to switch to sniffer · q to quit",
            Style::default().fg(Color::Cyan),
        )),
    ];
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
}

fn render_baseline(frame: &mut Frame, area: Rect, started: Instant) {
    let elapsed = started.elapsed();
    let progress = (elapsed.as_millis() as f64 / BASELINE_DURATION.as_millis() as f64).min(1.0);
    let chunks = split_two(area, 4);

    let header = vec![
        Line::from(Span::styled(
            "Recording idle baseline",
            Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from("Don't touch the mouse for ~1.5s"),
    ];
    frame.render_widget(Paragraph::new(header), chunks[0]);

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Yellow))
        .ratio(progress)
        .label(format!("{:>3}%", (progress * 100.0) as u32));
    let g_area = Rect {
        x: chunks[1].x,
        y: chunks[1].y,
        width: chunks[1].width,
        height: 1.min(chunks[1].height),
    };
    frame.render_widget(gauge, g_area);
}

fn render_ready(frame: &mut Frame, area: Rect, probe_idx: usize, app: &App) {
    let probe = PROBES[probe_idx];
    let mapped = app.wizard.results.iter().filter(|r| r.mapping.is_some()).count();
    let lines = vec![
        Line::from(vec![
            Span::styled(
                format!("Step {} / {} · ", probe_idx + 1, PROBES.len()),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                probe.name,
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            probe.hint,
            Style::default().add_modifier(Modifier::ITALIC),
        )),
        Line::from(""),
        Line::from(format!(
            "Mapped so far: {} · Skipped: {}",
            mapped,
            app.wizard.results.len() - mapped
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("SPACE", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" begin recording · "),
            Span::styled("n", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" skip · "),
            Span::styled("b", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" re-baseline · "),
            Span::styled("q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" quit"),
        ]),
    ];
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
}

fn render_recording(
    frame: &mut Frame,
    area: Rect,
    probe_idx: usize,
    started: Instant,
    captured_count: usize,
) {
    let probe = PROBES[probe_idx];
    let elapsed = started.elapsed();
    let progress = (elapsed.as_millis() as f64 / RECORD_DURATION.as_millis() as f64).min(1.0);
    let chunks = split_two(area, 6);

    let header = vec![
        Line::from(Span::styled(
            format!("RECORDING · {}", probe.name),
            Style::default().add_modifier(Modifier::BOLD).fg(Color::Red),
        )),
        Line::from(""),
        Line::from(Span::styled(
            probe.hint,
            Style::default().add_modifier(Modifier::ITALIC),
        )),
        Line::from(""),
        Line::from(format!("captured: {} reports", captured_count)),
    ];
    frame.render_widget(Paragraph::new(header), chunks[0]);

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Red))
        .ratio(progress)
        .label(format!("{:>3}%", (progress * 100.0) as u32));
    let g_area = Rect {
        x: chunks[1].x,
        y: chunks[1].y,
        width: chunks[1].width,
        height: 1.min(chunks[1].height),
    };
    frame.render_widget(gauge, g_area);
}

fn render_result(frame: &mut Frame, area: Rect, probe_idx: usize, mappings: &[ButtonMapping]) {
    let probe = PROBES[probe_idx];

    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                format!("{} · ", probe.name),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "result",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(""),
    ];

    if mappings.is_empty() {
        lines.push(Line::from(Span::styled(
            "No changes detected. Possible reasons:",
            Style::default().fg(Color::Yellow),
        )));
        lines.push(Line::from("  • the input is routed via the standard mouse interface (skipped)"));
        lines.push(Line::from("  • you didn't press during the recording window"));
        lines.push(Line::from("  • the button isn't physically present on your variant"));
    } else {
        lines.push(Line::from(Span::styled(
            "Detected changes (top candidate first):",
            Style::default().fg(Color::Green),
        )));
        lines.push(Line::from(""));
        for (i, m) in mappings.iter().take(6).enumerate() {
            let chip = if i == 0 { "▸" } else { " " };
            let line = format!(
                "  {} iface[{}] byte {:>2}  mask 0x{:02X}  ({:02X} → {:02X})  ×{}",
                chip, m.iface_id + 1, m.byte_index, m.mask, m.baseline_value, m.pressed_value, m.occurrences
            );
            let style = if i == 0 {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            lines.push(Line::from(Span::styled(line, style)));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(" accept · "),
        Span::styled("r", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" retry · "),
        Span::styled("n", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" skip · "),
        Span::styled("b", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" re-baseline"),
    ]));

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
}

fn render_done(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    save_path: Option<String>,
    save_error: Option<&str>,
) {
    let total = app.wizard.results.len();
    let mapped = app.wizard.results.iter().filter(|r| r.mapping.is_some()).count();

    let chunks = split_two(area, 6);

    let mut header = vec![
        Line::from(Span::styled(
            "Wizard complete",
            Style::default().add_modifier(Modifier::BOLD).fg(Color::Green),
        )),
        Line::from(""),
        Line::from(format!("{} mapped · {} skipped", mapped, total - mapped)),
    ];
    if let Some(path) = save_path {
        header.push(Line::from(Span::styled(
            format!("Saved to {}", path),
            Style::default().fg(Color::Green),
        )));
    }
    if let Some(err) = save_error {
        header.push(Line::from(Span::styled(
            format!("Save failed: {}", err),
            Style::default().fg(Color::Red),
        )));
    }
    header.push(Line::from(""));
    header.push(Line::from(Span::styled(
        "q to quit · Tab for sniffer view",
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(Paragraph::new(header), chunks[0]);

    let items: Vec<ListItem> = app
        .wizard
        .results
        .iter()
        .map(|r| {
            let style = if r.mapping.is_some() {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let summary = match &r.mapping {
                Some(m) => format!(
                    "  ✓ {:<28} iface[{}] byte {:>2} mask 0x{:02X}",
                    r.probe.name,
                    m.iface_id + 1,
                    m.byte_index,
                    m.mask
                ),
                None => format!("  · {:<28} skipped", r.probe.name),
            };
            ListItem::new(Line::from(Span::styled(summary, style)))
        })
        .collect();
    frame.render_widget(
        List::new(items).block(Block::default().borders(Borders::TOP).title(" summary ")),
        chunks[1],
    );
}
