mod hid;
mod tui;
mod wizard;

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

    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        return Ok(());
    }
    let seize_mouse = args.iter().any(|a| a == "--seize-mouse");

    let mut terminal = ratatui::init();
    let result = run(&mut terminal, seize_mouse).await;
    ratatui::restore();
    result
}

fn print_help() {
    println!("mmo7-mac — Mad Catz M.M.O. 7+ HID sniffer and mapping wizard");
    println!();
    println!("USAGE:");
    println!("  mmo7-mac [FLAGS]");
    println!();
    println!("FLAGS:");
    println!("  --seize-mouse    Also open the Generic Desktop / Mouse top-level");
    println!("                   collection. Required to capture standard mouse");
    println!("                   buttons (sniper, side, thumb-pad). While enabled,");
    println!("                   the cursor will be frozen — operate the wizard");
    println!("                   via keyboard only.");
    println!("  -h, --help       Show this help and exit.");
}

fn init_logging() {
    use tracing_subscriber::{EnvFilter, fmt};
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));
    let _ = fmt().with_env_filter(filter).with_writer(std::io::stderr).try_init();
}

async fn run(
    terminal: &mut ratatui::DefaultTerminal,
    seize_mouse: bool,
) -> color_eyre::Result<()> {
    let DeviceHandles { mut reports, mut state } = spawn_reader(seize_mouse);
    let mut app = App::new(seize_mouse);
    let mut events = EventStream::new();
    let mut tick = interval(Duration::from_millis(50));

    while app.running {
        terminal.draw(|f| render(f, &app))?;

        tokio::select! {
            _ = tick.tick() => {
                app.wizard.tick();
            }
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
