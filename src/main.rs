// src/main.rs
mod utils;
mod clients;
mod config;
mod errors;
mod models;
mod app;
mod ui;

use std::collections::HashMap;
use std::error::Error;
use chrono::{DateTime, Utc};
use serde_json::Value;
use tokio::sync::mpsc;
use tokio::time::{Duration, Instant};
use crossterm::event::KeyCode;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use anyhow::Result;

use crate::app::{App, Tab};
use crate::clients::infer_client::HiveInferClient;
use crate::clients::manage_client::HiveManageClient;
use crate::config::{load_profiles, save_profiles, Profile};
use crate::errors::ClientError;
use crate::models::{AuthKeys, GenerateRequest, GenerateResponse, QueueMap, WorkerConnections, WorkerPings, WorkerStatuses, WorkerTags, WorkerVersions};
use crate::ui::events::{Event, Events};
use crate::ui::terminal;
use crate::ui::tabs;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    // Load or initialize profiles
    let profiles = load_profiles()?;
    let mut profiles = load_profiles()?;
    // If no profiles exist, create one interactively
    if profiles.is_empty() {
        use std::io::{stdin, stdout, Write};
        println!("No HiveCore profiles found. Let's create one.");
        let mut input = String::new();

        // Profile name
        print!("Profile name: "); stdout().flush().unwrap();
        stdin().read_line(&mut input).unwrap();
        let name = input.trim().to_string();

        // Host
        print!("HiveCore host (e.g. https://hive.example.com): ");
        stdout().flush().unwrap();
        input.clear(); stdin().read_line(&mut input).unwrap();
        let host = input.trim().to_string();

        // Inference port
        print!("Inference port [6666]: "); stdout().flush().unwrap();
        input.clear(); stdin().read_line(&mut input).unwrap();
        let port_infer = input.trim().parse().unwrap_or(6666);

        // Management port
        print!("Management port [6668]: "); stdout().flush().unwrap();
        input.clear(); stdin().read_line(&mut input).unwrap();
        let port_manage = input.trim().parse().unwrap_or(6668);

        // Client token
        print!("Client token: "); stdout().flush().unwrap();
        input.clear(); stdin().read_line(&mut input).unwrap();
        let client_token = input.trim().to_string();

        // Admin token
        print!("Admin token: "); stdout().flush().unwrap();
        input.clear(); stdin().read_line(&mut input).unwrap();
        let admin_token = input.trim().to_string();

        let new_profile = Profile { name, host, port_infer, port_manage, client_token, admin_token };
        // Save to disk
        save_profiles(&[new_profile.clone()])?;
        profiles = vec![new_profile];
    }

    let mut app = App::new(profiles);
    let profile = &app.profiles[app.active_profile];
    let mut manage_client = HiveManageClient::new(
        format!("{}:{}", profile.host, profile.port_manage),
        &profile.admin_token,
    )?;

    if let Err(e) = init_app_data(&mut app).await {
        println!("[Error] Can't init app data: {:#?}", e);
        return Err(e);
    }

    println!("{:#?}", app);

    
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
            Event::Input(key) => match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Left     => app.prev_tab(),
                KeyCode::Right    => app.next_tab(),
                KeyCode::Char('r')=> app.clear_caches(),
                KeyCode::Enter    => {
                    if app.current_tab == Tab::Console {
                        // build the infer client
                        let profile = &app.profiles[app.active_profile];
                        let api = HiveInferClient::new(
                            format!("{}:{}", profile.host, profile.port_infer),
                            &profile.client_token,
                        )?;
                        // pick a model (e.g. first in queue_map) or hardcode
                        let model = app.queue_map
                            .as_ref()
                            .and_then(|qm| qm.keys().next().cloned())
                            .unwrap_or_default();
                        let req = GenerateRequest {
                            model: model.clone(),
                            prompt: app.console_input.clone(),
                            stream: false,
                            node: None,
                        };
                        // run it
                        match api.generate(&req.model, &req.prompt, None, req.stream).await {
                            Ok(raw) => {
                                if let Ok(resp) = serde_json::from_value::<GenerateResponse>(raw) {
                                    app.generate_response = Some(resp.clone());
                                    app.console_output = vec![resp.result];
                                }
                            }
                            Err(e) => app.add_banner(format!("Inference failed: {}", e)),
                        }
                    }
                }
                KeyCode::Char(c)   => {
                    if app.current_tab == Tab::Console {
                        app.console_input.push(c);
                    }
                }
                KeyCode::Backspace => {
                    if app.current_tab == Tab::Console {
                        app.console_input.pop();
                    }
                }
                _ => {}
            },
            Event::Tick => {
                // Instantiate client each tick to pick up profile changes
                let profile = &app.profiles[app.active_profile];
                let mut client = HiveManageClient::new(
                    format!("{}:{}", profile.host, profile.port_manage),
                    &profile.admin_token,
                )?;

                match app.current_tab {
                    Tab::Queues => {
                        if let Ok(resp) = client.get_queue().await {
                            app.queue_map = Some(resp)
                        }
                    }
                    Tab::Nodes => {
                        // fetch node details
                        if let Ok(resp) = client.get_worker_status().await {
                            app.worker_statuses = Some(resp);
                        }
                        if let Ok(resp) = client.get_worker_connections().await {
                            app.worker_connections = Some(resp);
                        }
                        if let Ok(resp) = client.get_worker_pings().await {
                            app.worker_pings = Some(resp);
                        }
                        if let Ok(resp) = client.get_worker_versions().await {
                            app.worker_versions = Some(resp);
                        }
                        if let Ok(resp) = client.get_worker_tags().await {
                            app.worker_tags = Some(resp);
                        }
                    }
                    Tab::Keys => {
                        if let Ok(resp) = client.get_keys().await {
                            app.auth_keys = Some(resp);
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

async fn init_app_data(app: &mut App) -> Result<(), ClientError> {
    let client = HiveManageClient::new(
        format!("{}:{}", 
            app.profiles[app.active_profile].host, 
            app.profiles[app.active_profile].port_manage
        ),
        &app.profiles[app.active_profile].admin_token,
    )?;
    app.worker_statuses = Some(client.get_worker_status().await?);
    app.worker_connections = Some(client.get_worker_connections().await?);
    app.worker_pings = Some(client.get_worker_pings().await?);
    app.worker_versions = Some(client.get_worker_versions().await?);
    app.worker_tags = Some(client.get_worker_tags().await?);
    app.queue_map = Some(client.get_queue().await?);
    app.auth_keys = Some(client.get_keys().await?);
    Ok(())
}
