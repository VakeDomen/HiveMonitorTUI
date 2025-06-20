use std::sync::Arc;

use tokio::sync::Mutex;

use crossterm::event::KeyCode;

use crate::{app::{App, Tab}, clients::{infer_client::HiveInferClient, manage_client::HiveManageClient}, events::spawner::{Event, EventSpawner}, models::{GenerateRequest, GenerateResponse}};

pub async fn handle_events(mut event_spawner: EventSpawner, app_arc: Arc<Mutex<App>>) {
    let client = {
        let mut app = app_arc.lock().await;
        // Instantiate client each tick to pick up profile changes
        let profile = &app.profiles[app.active_profile];
        let client = match HiveManageClient::new(
            format!("{}:{}", profile.host, profile.port_manage),
            &profile.admin_token,
        ) {
            Ok(c) => c,
            Err(e) => {
                app.add_banner(format!("Can't contact HiveCore: {}", e));
                return;
            },
        };

        client
    };

    loop {

        let tab = {
            let app = app_arc.lock().await;
            app.current_tab.clone()
        };

        match event_spawner.next().await {
            Event::Input(key) => {
                let mut app = app_arc.lock().await;
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Left | KeyCode::Char('a')    => on_key_left(&mut app),
                    KeyCode::Right | KeyCode::Char('d')    => on_key_right(&mut app),
                    KeyCode::Up | KeyCode::Char('w') => on_key_up(&mut app),
                    KeyCode::Down | KeyCode::Char('s') => on_key_down(&mut app),
                    KeyCode::Char('r')=> on_key_r(&mut app),
                    KeyCode::Enter    => on_enter(&mut app).await,
                    KeyCode::Backspace => on_backspace(app),
                    KeyCode::Char(c) => on_unhandled_character(&mut app, c),
                    _ => {}
                }
            },
            Event::Tick => {
                match tab {
                    Tab::Queues => {
                        if let Ok(resp) = client.get_queue().await {
                            {
                                let mut app = app_arc.lock().await;
                                app.queue_map = Some(resp)
                            }
                        }
                    }
                    Tab::Nodes | Tab::Dashboard => {
                        if let Ok(resp) = client.get_worker_status().await {
                            {
                                let mut app = app_arc.lock().await;
                                app.worker_statuses = Some(resp);
                            }
                        }
                        if let Ok(resp) = client.get_worker_connections().await {
                            {
                                let mut app = app_arc.lock().await;
                                app.worker_connections = Some(resp);
                            }
                        }
                        if let Ok(resp) = client.get_worker_pings().await {
                            {
                                let mut app = app_arc.lock().await;
                                app.worker_pings = Some(resp);
                            }
                        }
                        if let Ok(resp) = client.get_worker_versions().await {
                            {
                                let mut app = app_arc.lock().await;
                                app.worker_versions = Some(resp);
                            }
                        }
                        if let Ok(resp) = client.get_worker_tags().await {
                            {
                                let mut app = app_arc.lock().await;
                                app.worker_tags = Some(resp);
                            }
                        }
                    }
                    Tab::Keys => {
                        if let Ok(resp) = client.get_keys().await {
                            {
                                let mut app = app_arc.lock().await;
                                app.auth_keys = Some(resp);
                            }
                        }
                    }
                    _ => {}
                }
            }
            Event::Stop => break,
        }
    }
}

fn on_key_down(app: &mut tokio::sync::MutexGuard<'_, App>) {
    app.focus_down();
}

fn on_key_up(app: &mut tokio::sync::MutexGuard<'_, App>) {
    app.focus_up();
}

fn on_backspace(mut app: tokio::sync::MutexGuard<'_, App>) {
    if app.current_tab == Tab::Console {
        app.console_input.pop();
    }
}

fn on_unhandled_character(app: &mut tokio::sync::MutexGuard<'_, App>, c: char) {
    if app.current_tab == Tab::Console {
        app.console_input.push(c);
    }
}

fn on_key_r(app: &mut tokio::sync::MutexGuard<'_, App>) {
    app.clear_caches()
}

fn on_key_right(app: &mut tokio::sync::MutexGuard<'_, App>) {
    app.focus_right();
}

fn on_key_left(app: &mut tokio::sync::MutexGuard<'_, App>) {
    app.focus_left();
}

async fn on_enter(app: &mut tokio::sync::MutexGuard<'_, App>) {
    if app.current_tab == Tab::Console {
        // build the infer client
        let profile = &app.profiles[app.active_profile];
        let api = match HiveInferClient::new(
            format!("{}:{}", profile.host, profile.port_infer),
            &profile.client_token,
        ) {
            Ok(c) => c,
            Err(e) => {
                app.add_banner(format!("Can't contact HiveCore: {}", e));
                return;
            },
        };
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