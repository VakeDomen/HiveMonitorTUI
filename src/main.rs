// src/main.rs
mod utils;
mod clients;
mod config;
mod errors;
mod models;
mod app;
mod ui;

use std::error::Error;
use tokio::sync::mpsc;
use tokio::time::{Duration, Instant};
use crossterm::event::KeyCode;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::app::{App, Tab};
use crate::config::load_profiles;
use crate::ui::events::{Event, Events};
use crate::ui::terminal;
use crate::ui::tabs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load or initialize profiles
    let profiles = load_profiles()?;
    let mut app = App::new(profiles);

    // Terminal setup
    let mut terminal = terminal::setup_terminal()?;

    // Event handling (keyboard + tick)
    let mut events = Events::new(app.intervals.general_secs);

    // Main loop
    loop {
        // Draw UI
        terminal.draw(|f| {
            // Render based on current tab
            match app.current_tab {
                Tab::Dashboard => tabs::dashboard::draw(f, &app),
                Tab::Nodes => tabs::nodes::draw(f, &app),
                Tab::Queues => tabs::queues::draw(f, &app),
                Tab::Keys => tabs::keys::draw(f, &app),
                Tab::Console => tabs::console::draw(f, &app),
                Tab::Logs => tabs::logs::draw(f, &app),
            }
            // Render banners and global components
            ui::terminal::draw_banners(f, &app.banners);
        })?;

        // Handle input or tick
        match events.next().await {
            Event::Input(key) => {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Left => app.prev_tab(),
                    KeyCode::Right => app.next_tab(),
                    KeyCode::Char('r') => {
                        // Trigger manual refresh: clear caches to force reload
                        app.clear_caches();
                    }
                    KeyCode::Char('+') => {
                        app.intervals.general_secs = (app.intervals.general_secs + 1).min(60);
                        events.set_tick_rate(app.intervals.general_secs);
                    }
                    KeyCode::Char('-') => {
                        app.intervals.general_secs = (app.intervals.general_secs.saturating_sub(1)).max(1);
                        events.set_tick_rate(app.intervals.general_secs);
                    }
                    _ => {}
                }
            }
            Event::Tick => {
                // On tick, fetch data for current tab
                // Spawn asynchronous tasks based on active tab
                match app.current_tab {
                    Tab::Queues => {
                        // poll queue data at high frequency
                        // TODO: spawn task to update app.queue_map
                    }
                    _ => {
                        // poll general data (nodes, keys, dashboard)
                        // TODO: spawn tasks to update relevant app fields
                    }
                }
            }
        }
    }

    // Restore terminal
    terminal::restore_terminal()?;
    Ok(())
}
