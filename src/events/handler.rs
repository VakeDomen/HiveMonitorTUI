use std::sync::Arc;

use tokio::sync::Mutex;

use crossterm::event::KeyCode;

use crate::{app::{App, Tab}, clients::{infer_client::HiveInferClient, manage_client::HiveManageClient}, events::spawner::{Event, EventSpawner}, models::{GenerateRequest, GenerateResponse}};

pub async fn handle_events(mut event_spawner: EventSpawner, app_arc: Arc<Mutex<App>>) {
    
    loop {
        match event_spawner.next().await {
            Event::Input(key) => {
                        let mut app = app_arc.lock().await;
                        match key.code {

                            KeyCode::Char('q') => break,
                            KeyCode::Left     => app.prev_tab(),
                            KeyCode::Right    => app.next_tab(),
                            KeyCode::Char('r')=> app.clear_caches(),
                            KeyCode::Enter    => {
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
                                            continue;
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
                            KeyCode::Char(c) => {
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
                        }

                    },
            Event::Tick => {
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
                                break;
                            },
                        };

                        match app.current_tab {
                            Tab::Queues => {
                                if let Ok(resp) = client.get_queue().await {
                                    app.queue_map = Some(resp)
                                }
                            }
                            Tab::Nodes | Tab::Dashboard => {
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
            Event::Stop => break,
        }
    }
}