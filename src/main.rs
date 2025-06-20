// src/main.rs
mod utils;
mod clients;
mod config;
mod errors;
mod models;
mod app;
mod ui;
mod events;

use std::collections::HashMap;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use tokio::sync::Mutex;
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
use crate::events::handler::handle_events;
use crate::events::spawner::{Event, EventSpawner};
use crate::models::{AuthKeys, GenerateRequest, GenerateResponse, QueueMap, WorkerConnections, WorkerPings, WorkerStatuses, WorkerTags, WorkerVersions};
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

    if let Err(e) = init_app_data(&mut app).await {
        println!("[Error] Can't init app data: {:#?}", e);
        return Err(e);
    }


    let app_arc = Arc::new(Mutex::new(app));
    let should_stop = Arc::new(AtomicBool::new(false));

    
    // Terminal setup
    let mut terminal = terminal::setup_terminal()?;

    // Event handling (keyboard + tick)
    let mut event_spawner = EventSpawner::new(30);
    event_spawner.add_spawn_interval(Event::Tick, Duration::from_secs(1));


    event_spawner.push_generate_event(Event::Stop, Duration::from_secs(5));

    let app_arc_clone = app_arc.clone();
    let should_stop_clone = should_stop.clone();
    tokio::spawn(async move {
        let _ = handle_events(event_spawner, app_arc_clone).await;
        should_stop_clone.store(true, Ordering::Relaxed);
    });

    // Main loop
    loop {
        // Draw UI
        let app = app_arc.lock().await;
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

        sleep(Duration::from_millis(16));
        if should_stop.fetch_and(true, Ordering::Relaxed) {
            break;
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
