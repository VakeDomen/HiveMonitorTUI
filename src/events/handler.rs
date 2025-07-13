use std::sync::Arc;

use tokio::sync::Mutex;

use futures::{StreamExt, TryStreamExt};
use crossterm::event::KeyCode;

use crate::{app::{ActionPanelState, ActionType, App, Focus, Tab}, clients::{infer_client::HiveInferClient, manage_client::HiveManageClient}, events::spawner::{Event, EventSpawner}, models::{GenerateRequest, GenerateResponse}};

pub async fn handle_events(mut event_spawner: EventSpawner, app_arc: Arc<Mutex<App>>) {
    let manage_client = {
        let app = app_arc.lock().await;
        let profile = &app.profiles[app.active_profile];
        match HiveManageClient::new(
            format!("{}:{}", profile.host, profile.port_manage),
            &profile.admin_token,
        ) {
            Ok(c) => c,
            Err(e) => {
                app_arc.clone().lock().await.add_banner(format!("Can't contact HiveCore Manage API: {}", e));
                return;
            },
        }
    };

    loop {
        // Capture necessary state values from app_arc at the start of each loop iteration
        let current_tab;
        let current_focus;
        // current_action_panel_state is now less directly used outside locks
        // let current_action_panel_state;
        { // Scope to release lock quickly
            let app = app_arc.lock().await;
            current_tab = app.current_tab;
            current_focus = app.focus;
            // current_action_panel_state = app.action_panel_state.clone();
        } // `app` MutexGuard is dropped here.

        match event_spawner.next().await {
            Event::Input(key) => {
                // If an action is in progress and we are NOT in the response view,
                // we should prevent most input. The `focus_left/right` already
                // handles this with banners, but for general chars/enter, we can
                // ignore them.
                if app_arc.lock().await.is_action_in_progress && app_arc.lock().await.focus != Focus::ActionPanelResponse {
                    // Allow Q to quit even during action.
                    if key.code == KeyCode::Char('q') { break; }
                    // Allow ESC to cancel confirmation/input.
                    if key.code == KeyCode::Esc {
                        let mut app = app_arc.lock().await;
                        if app.focus == Focus::ActionPanelInput || app.focus == Focus::ActionPanelConfirm {
                            // This part duplicates logic, consider moving ESC handler to a single place
                            // or ensuring it cascades. For now, it's safer to allow this critical escape.
                            if let Some(handle) = app.action_task_handle.take() { handle.abort(); }
                            app.is_action_in_progress = false;
                            app.action_panel_state = ActionPanelState::None;
                            app.focus = Focus::ActionsList;
                            app.action_input_model_name.clear();
                            app.action_input_cursor_position = 0;
                            app.action_panel_scroll = 0;
                            app.add_banner("Action cancelled by user.");
                            continue; // Skip further processing of this key
                        }
                    }
                    // For other keys, just ignore and add banner
                    app_arc.lock().await.add_banner("Action in progress. Input ignored.");
                    continue; // Skip processing other keys if action is in progress
                }


                let mut app = app_arc.lock().await; // Lock once for input handling

                match app.focus {
                    Focus::WorkersList | Focus::ActionsList | Focus::GlobalView => {
                        match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Left | KeyCode::Char('a') => on_key_left(&mut app),
                            KeyCode::Right | KeyCode::Char('d') => on_key_right(&mut app),
                            KeyCode::Up | KeyCode::Char('w') => on_key_up(&mut app),
                            KeyCode::Down | KeyCode::Char('s') => on_key_down(&mut app),
                            KeyCode::Char('r') => on_key_r(&mut app),
                            KeyCode::Enter => on_enter_main_view(&mut app).await,
                            KeyCode::Backspace => on_backspace(app),
                            KeyCode::Char(c) => on_unhandled_character(&mut app, c),
                            _ => {}
                        }
                    },
                    Focus::ActionPanelInput => {
                        match key.code {
                            KeyCode::Enter => {
                                if !app.action_input_model_name.is_empty() {
                                    let model_name = app.action_input_model_name.clone();
                                    let action_type = match app.action_panel_state {
                                        ActionPanelState::PullModel => ActionType::Pull,
                                        ActionPanelState::DeleteModel => ActionType::Delete,
                                        _ => {
                                            app.add_banner("Unexpected action state. Please restart action.");
                                            app.focus = Focus::ActionsList;
                                            app.action_panel_state = ActionPanelState::None;
                                            return;
                                        }
                                    };
                                    app.action_panel_state = ActionPanelState::Confirmation(model_name, action_type);
                                    app.focus = Focus::ActionPanelConfirm;
                                    app.confirmation_selection = 0;
                                } else {
                                    app.add_banner("Model name cannot be empty.");
                                }
                            },
                            KeyCode::Esc => {
                                // Cancel from input view
                                app.action_panel_state = ActionPanelState::None;
                                app.focus = Focus::ActionsList;
                                app.action_input_model_name.clear();
                                app.action_input_cursor_position = 0;
                                app.action_panel_scroll = 0;
                            },
                            KeyCode::Backspace => {
                                app.backspace_action_field();
                            },
                            KeyCode::Left => {
                                app.focus_left();
                            },
                            KeyCode::Right => {
                                app.focus_right();
                            },
                            KeyCode::Char(c) => {
                                if key.modifiers.is_empty() {
                                    app.input_char_into_action_field(c);
                                }
                            },
                            _ => {}
                        }
                    },
                    Focus::ActionPanelConfirm => {
                        match key.code {
                            KeyCode::Left | KeyCode::Right => {
                                app.confirmation_selection = 1 - app.confirmation_selection;
                            },

                            KeyCode::Enter => {
                                let action_state = app.action_panel_state.clone();
                                let model_name_for_action = if let ActionPanelState::Confirmation(ref m_name, _) = action_state {
                                    m_name.clone()
                                } else {
                                    "".to_string()
                                };
                                let selected_worker_name = app.get_selected_worker_name();
                                let profile = app.profiles[app.active_profile].clone();
                                let client_token = profile.client_token.clone();

                                // Set initial processing state for UI, then drop lock
                                app.action_panel_state = ActionPanelState::Response(
                                    model_name_for_action.clone(),
                                    match action_state { ActionPanelState::Confirmation(_, action_type) => action_type, _ => ActionType::Pull },
                                    vec!["Initiating action...".to_string()],
                                    true,
                                );
                                app.focus = Focus::ActionPanelResponse;
                                app.is_action_in_progress = true; // Set flag
                                drop(app); // Release lock

                                let app_arc_for_spawn = app_arc.clone(); // Clone for spawned task

                                let task_handle = tokio::spawn(async move {
                                    let infer_client = match HiveInferClient::new(
                                        format!("{}:{}", profile.host, profile.port_infer),
                                        &client_token,
                                    ) {
                                        Ok(c) => c,
                                        Err(e) => {
                                            let mut app = app_arc_for_spawn.lock().await;
                                            app.add_banner(format!("Failed to create InferClient: {}", e));
                                            app.add_action_output_line(format!("Client error: {}", e), false);
                                            app.is_action_in_progress = false; // Reset flag on failure
                                            return;
                                        }
                                    };

                                    let mut api_overall_result_message: Result<(), String> = Ok(()); // Initialize as Ok(())

                                    let should_proceed = {
                                        let app_guard = app_arc_for_spawn.lock().await;
                                        app_guard.confirmation_selection == 0
                                    };

                                    if should_proceed {
                                        let node = selected_worker_name.as_deref();
                                        match action_state {
                                            ActionPanelState::Confirmation(model_name, action_type) => {
                                                match action_type {
                                                    ActionType::Pull => {
                                                        match infer_client.pull_model(&model_name, node, app_arc_for_spawn.clone()).await {
                                                            Ok(_) => {},
                                                            Err(e) => api_overall_result_message = Err(format!("Pull failed: {}", e)),
                                                        }
                                                    },
                                                    ActionType::Delete => {
                                                        match infer_client.delete_model(&model_name, node, app_arc_for_spawn.clone()).await {
                                                            Ok(_) => {},
                                                            Err(e) => api_overall_result_message = Err(format!("Delete failed: {}", e)),
                                                        }
                                                    },
                                                }
                                            },
                                            _ => api_overall_result_message = Err("Unexpected action panel state.".to_string()),
                                        }
                                    } else {
                                        app_arc_for_spawn.lock().await.add_action_output_line("Action cancelled by user.".to_string(), true);
                                    }

                                    // After the operation (streaming or single call) is done,
                                    // ensure the overall status is reflected and clean up.
                                    let mut app = app_arc_for_spawn.lock().await;
                                    if let ActionPanelState::Response(ref _m_name, ref _act_type, ref mut lines, ref mut overall_success) = app.action_panel_state {
                                        if api_overall_result_message.is_err() {
                                            *overall_success = false;
                                            lines.push(api_overall_result_message.unwrap_err()); // Push the error message
                                        }
                                        // Auto-scroll to bottom of logs on completion/final update
                                        // This is a common UX for streaming logs
                                        app.action_panel_scroll = lines.len().saturating_sub(
                                            // Subtract estimated visible height to set scroll to end
                                            // You'd need to accurately get the height of the response panel
                                            // For now, let's just make it a large number to ensure it's at the end.
                                            // A better way is to pass current `inner_area.height` to App during draw
                                            1 // Minimum height for 1 line to be visible at end
                                        ) as u16;
                                    }
                                    app.is_action_in_progress = false; // Action is now complete, reset flag
                                }).abort_handle(); // Get the AbortHandle

                                // Store the AbortHandle so the main loop can cancel it if needed
                                let mut app = app_arc.lock().await; // Re-acquire lock to store handle
                                app.action_task_handle = Some(task_handle);
                            },
                            KeyCode::Esc => { // Cancel confirmation
                                app.action_panel_state = ActionPanelState::None;
                                app.focus = Focus::ActionsList;
                                app.action_input_model_name.clear();
                                app.action_input_cursor_position = 0;
                                app.action_panel_scroll = 0;
                            }
                            _ => {}
                        }
                    },
                    Focus::ActionPanelResponse => {
                        match key.code {
                            KeyCode::Up | KeyCode::Char('w') => on_key_up(&mut app),
                            KeyCode::Down | KeyCode::Char('s') => on_key_down(&mut app),
                            KeyCode::Esc => { // ESC can always dismiss a response panel
                                if let Some(handle) = app.action_task_handle.take() {
                                    handle.abort(); // Abort if task still running
                                }
                                app.is_action_in_progress = false;
                                app.action_panel_state = ActionPanelState::None;
                                app.focus = Focus::ActionsList;
                                app.action_input_model_name.clear();
                                app.action_input_cursor_position = 0;
                                app.action_panel_scroll = 0;
                                app.add_banner("Response dismissed, action aborted if running.");
                            },
                            _ => { // Any other key also dismisses
                                if let Some(handle) = app.action_task_handle.take() {
                                    handle.abort(); // Abort if task still running
                                }
                                app.is_action_in_progress = false;
                                app.action_panel_state = ActionPanelState::None;
                                app.focus = Focus::ActionsList;
                                app.action_input_model_name.clear();
                                app.action_input_cursor_position = 0;
                                app.action_panel_scroll = 0;
                                app.add_banner("Response dismissed, action aborted if running.");
                            }
                        }
                    }
                    _ => {}
                }
            },
            Event::Tick => {
                // Polling logic remains the same
                match current_tab {
                   Tab::Dashboard => {
                        if let Ok(resp) = manage_client.get_worker_status().await {
                            let mut app = app_arc.lock().await;
                            app.worker_statuses = Some(resp);
                        }
                        if let Ok(resp) = manage_client.get_worker_connections().await {
                            let mut app = app_arc.lock().await;
                            app.worker_connections = Some(resp);
                        }
                        if let Ok(resp) = manage_client.get_worker_pings().await {
                            let mut app = app_arc.lock().await;
                            app.worker_pings = Some(resp);
                        }
                        if let Ok(resp) = manage_client.get_worker_versions().await {
                            let mut app = app_arc.lock().await;
                            app.worker_versions = Some(resp);
                        }
                        if let Ok(resp) = manage_client.get_worker_tags().await {
                            let mut app = app_arc.lock().await;
                            app.worker_tags = Some(resp);
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

// Renamed and modified `on_enter` to `on_enter_main_view`
async fn on_enter_main_view(app: &mut tokio::sync::MutexGuard<'_, App>) { // No app_arc here needed
    if app.current_tab == Tab::Dashboard {
        if app.focus == Focus::ActionsList {
            let selected_action_name = app.worker_actions.get(app.selected_action).copied();
            // This is the source of the problem: `selected_model_name` is from the old flow
            // and should not be used here to directly jump to Confirmation.
            // The input field handles the model selection now.
            // let selected_model_name = app.get_selected_info_panel_model();

            match selected_action_name {
                Some("Pull model") => {
                    // Corrected: Transition to input state
                    app.action_panel_state = ActionPanelState::PullModel;
                    app.focus = Focus::ActionPanelInput;
                    app.action_input_model_name.clear(); // Clear input field
                    app.action_input_cursor_position = 0;
                },
                Some("Delete model") => {
                    // Corrected: Transition to input state
                    app.action_panel_state = ActionPanelState::DeleteModel;
                    app.focus = Focus::ActionPanelInput;
                    app.action_input_model_name.clear(); // Clear input field
                    app.action_input_cursor_position = 0;
                },
                _ => {}
            }
        }
    } else if app.current_tab == Tab::Console {
        // Original console logic for generate
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