use crate::hid::device::{ConnectionState, InterfaceInfo};
use crate::tui::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

const IFACE_COLORS: &[Color] = &[
    Color::Cyan,
    Color::Magenta,
    Color::Yellow,
    Color::Green,
    Color::LightBlue,
    Color::LightRed,
    Color::LightMagenta,
    Color::White,
    Color::Blue,
];

fn iface_color(id: u8) -> Color {
    IFACE_COLORS[(id as usize) % IFACE_COLORS.len()]
}

pub fn render(frame: &mut Frame, app: &App) {
    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, vchunks[0], app);

    let hchunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(34), Constraint::Min(0)])
        .split(vchunks[1]);

    render_interfaces(frame, hchunks[0], app);
    render_reports(frame, hchunks[1], app);
    render_footer(frame, vchunks[2], app);
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let (label, color) = match &app.connection {
        ConnectionState::Searching => (
            "● searching for MMO 7+…".to_string(),
            Color::Yellow,
        ),
        ConnectionState::Connected { interfaces } => (
            format!("● connected · {} iface(s)", interfaces.len()),
            Color::Green,
        ),
    };

    let status_chip = if app.paused {
        Span::styled(" PAUSED ", Style::default().bg(Color::Red).fg(Color::White))
    } else {
        Span::styled(" LIVE ", Style::default().bg(Color::Green).fg(Color::Black))
    };

    let follow_chip = if app.follow {
        Span::styled(" follow ", Style::default().fg(Color::Cyan))
    } else {
        Span::styled(" freeze ", Style::default().fg(Color::DarkGray))
    };

    let line = Line::from(vec![
        Span::styled(label, Style::default().fg(color).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        status_chip,
        Span::raw("  "),
        follow_chip,
        Span::raw(format!(
            "  total: {}   shown: {}",
            app.total_received,
            app.reports.len()
        )),
    ]);

    let header = Paragraph::new(line).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" mmo7-mac · hid sniffer ")
            .title_style(Style::default().add_modifier(Modifier::BOLD)),
    );
    frame.render_widget(header, area);
}

fn render_interfaces(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().borders(Borders::ALL).title(" interfaces ");

    let ifaces = app.interfaces();
    if ifaces.is_empty() {
        let p = Paragraph::new("waiting for device…")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        frame.render_widget(p, area);
        return;
    }

    let items: Vec<ListItem> = ifaces.iter().map(|i| iface_item(i, app)).collect();
    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn iface_item<'a>(iface: &InterfaceInfo, app: &App) -> ListItem<'a> {
    let color = iface_color(iface.id);
    let hidden = app.hidden_ifaces.contains(&iface.id);
    let count = app.per_iface_counts.get(&iface.id).copied().unwrap_or(0);

    let marker = match (iface.opened, hidden) {
        (false, _) => "○",
        (true, true) => "·",
        (true, false) => "●",
    };
    let role = iface.role_hint();
    let header = format!(
        "{} [{}] {:<10} {:04X}:{:02X}",
        marker,
        iface.id + 1,
        role,
        iface.usage_page,
        iface.usage,
    );
    let detail = if iface.opened {
        format!("    iface#{}  rcv={}", iface.interface_number, count)
    } else {
        format!("    iface#{}  skipped (OS)", iface.interface_number)
    };

    let header_style = match (iface.opened, hidden) {
        (false, _) => Style::default().fg(Color::DarkGray),
        (true, true) => Style::default().fg(Color::DarkGray),
        (true, false) => Style::default().fg(color).add_modifier(Modifier::BOLD),
    };
    let detail_style = Style::default().fg(Color::DarkGray);

    ListItem::new(vec![
        Line::from(Span::styled(header, header_style)),
        Line::from(Span::styled(detail, detail_style)),
    ])
}

fn render_reports(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .reports
        .iter()
        .map(|r| {
            let ms = r.ts.duration_since(app.started_at).as_millis();
            let tag = format!("[{}]", r.iface.id + 1);
            let prefix = format!(" +{:>8}ms  {:>2}b  ", ms, r.bytes.len());
            let line = Line::from(vec![
                Span::styled(
                    tag,
                    Style::default()
                        .fg(iface_color(r.iface.id))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(prefix, Style::default().fg(Color::DarkGray)),
                Span::styled(r.hex(), Style::default().fg(Color::White)),
                Span::raw("   "),
                Span::styled(r.ascii(), Style::default().fg(Color::Gray)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let block = Block::default().borders(Borders::ALL).title(" reports ");
    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = ListState::default();
    if !app.reports.is_empty() {
        state.select(Some(app.scroll.min(app.reports.len() - 1)));
    }
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_footer(frame: &mut Frame, area: Rect, _app: &App) {
    let help = " q quit · p pause · c clear · ↑↓/jk scroll · g/G top/bot · f follow · 1-9 hide iface ";
    let footer = Paragraph::new(help).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, area);
}
