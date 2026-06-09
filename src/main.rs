mod hid;
mod tui;

use crate::hid::device::{DeviceHandles, spawn_reader};
use crate::tui::app::App;
use crate::tui::events::handle_key;
use crate::tui::ui::render;
use crossterm::event::{Event, EventStream, KeyEventKind};
use futures::StreamExt;
use std::time::Duration;
use tokio::time::interval;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    init_logging();

    let mut terminal = ratatui::init();
    let result = run(&mut terminal).await;
    ratatui::restore();
    result
}

fn init_logging() {
    use tracing_subscriber::{EnvFilter, fmt};
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));
    let _ = fmt().with_env_filter(filter).with_writer(std::io::stderr).try_init();
}

async fn run(terminal: &mut ratatui::DefaultTerminal) -> color_eyre::Result<()> {
    let DeviceHandles { mut reports, mut state } = spawn_reader();
    let mut app = App::new();
    let mut events = EventStream::new();
    let mut tick = interval(Duration::from_millis(50));

    while app.running {
        terminal.draw(|f| render(f, &app))?;

        tokio::select! {
            _ = tick.tick() => {}
            maybe_evt = events.next() => {
                if let Some(evt) = maybe_evt {
                    match evt? {
                        Event::Key(k) if k.kind == KeyEventKind::Press => handle_key(&mut app, k),
                        _ => {}
                    }
                }
            }
            Some(report) = reports.recv() => {
                app.push_report(report);
                while let Ok(more) = reports.try_recv() {
                    app.push_report(more);
                }
            }
            Ok(()) = state.changed() => {
                app.connection = state.borrow_and_update().clone();
                app.on_connection_changed();
            }
        }
    }
    Ok(())
}
