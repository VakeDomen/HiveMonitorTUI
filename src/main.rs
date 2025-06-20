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
use crate::clients::manage_client::HiveManageClient;
use crate::config::load_profiles;
use crate::models::{AuthKeys, QueueMap, WorkerConnections, WorkerPings, WorkerStatuses, WorkerTags, WorkerVersions};
use crate::ui::events::{Event, Events};
use crate::ui::terminal;
use crate::ui::tabs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load or initialize profiles
    let profiles = load_profiles()?;
    let mut app = App::new(profiles);
    let profile = &app.profiles[app.active_profile];
    let mut manage_client = HiveManageClient::new(
        format!("{}:{}", profile.host, profile.port_manage),
        &profile.admin_token,
    )?;

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
                // Instantiate client each tick to pick up profile changes
                let profile = &app.profiles[app.active_profile];
                let mut client = HiveManageClient::new(
                    format!("{}:{}", profile.host, profile.port_manage),
                    &profile.admin_token,
                )?;

                match app.current_tab {
                    Tab::Queues => {
                        if let Ok(raw) = client.get_queue().await {
                            match serde_json::from_value::<QueueMap>(raw) {
                                Ok(map)    => app.queue_map = Some(map),
                                Err(e)     => app.add_banner(format!("Parse queue failed: {}", e)),
                            }
                        }
                    }
                    Tab::Nodes => {
                        // fetch node details
                        if let Ok(raw) = client.get_worker_status().await {
                            if let Ok(parsed) = serde_json::from_value::<WorkerStatuses>(raw) {
                                app.worker_statuses = Some(parsed);
                            }
                        }
                        if let Ok(raw) = client.get_worker_connections().await {
                            if let Ok(parsed) = serde_json::from_value::<WorkerConnections>(raw) {
                                app.worker_connections = Some(parsed);
                            }
                        }
                        if let Ok(raw) = client.get_worker_pings().await {
                            if let Ok(parsed) = serde_json::from_value::<WorkerPings>(raw) {
                                app.worker_pings = Some(parsed);
                            }
                        }
                        if let Ok(raw) = client.get_worker_versions().await {
                            if let Ok(parsed) = serde_json::from_value::<WorkerVersions>(raw) {
                                app.worker_versions = Some(parsed);
                            }
                        }
                        if let Ok(raw) = client.get_worker_tags().await {
                            if let Ok(parsed) = serde_json::from_value::<WorkerTags>(raw) {
                                app.worker_tags = Some(parsed);
                            }
                        }
                    }
                    Tab::Keys => {
                        if let Ok(raw) = client.get_keys().await {
                            if let Ok(parsed) = serde_json::from_value::<AuthKeys>(raw) {
                                app.auth_keys = Some(parsed);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    terminal::restore_terminal()?;
    Ok(())
}
