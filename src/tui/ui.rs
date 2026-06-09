use crate::hid::device::ConnectionState;
use crate::tui::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, chunks[0], app);
    render_reports(frame, chunks[1], app);
    render_footer(frame, chunks[2], app);
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let (label, color) = match app.connection {
        ConnectionState::Searching => (
            "● searching for MMO 7+…".to_string(),
            Color::Yellow,
        ),
        ConnectionState::Connected { vid, pid } => (
            format!("● connected ({:04X}:{:04X})", vid, pid),
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
            "  reports: {}   buffered: {}",
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

fn render_reports(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .reports
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let ms = r.ts.duration_since(app.started_at).as_millis();
            let prefix = format!("{:>5}  +{:>8}ms  [{:>2}b]  ", i, ms, r.bytes.len());
            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::DarkGray)),
                Span::styled(r.hex(), Style::default().fg(Color::Cyan)),
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
    let help = " q/Esc quit · p/space pause · c clear · ↑↓/jk scroll · PgUp/PgDn ±10 · g/G top/bot · f follow ";
    let footer = Paragraph::new(help).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, area);
}
