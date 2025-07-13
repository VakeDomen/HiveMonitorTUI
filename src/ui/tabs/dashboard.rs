// src/ui/tabs/dashboard.rs
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::app::{ActionPanelState, ActionType, App, Focus};

// --- Color Scheme Definitions ---
const COLOR_DEFAULT_FG: Color = Color::White;
const COLOR_DEFAULT_BG: Color = Color::Black;
const COLOR_BORDER: Color = Color::Cyan;
const COLOR_HIGHLIGHT_BG: Color = Color::LightCyan;
const COLOR_HIGHLIGHT_FG: Color = Color::Black;
const COLOR_SELECTED_INACTIVE_FOCUS_BG: Color = Color::DarkGray; // For selected worker when focus is on actions
const COLOR_SELECTED_INACTIVE_FOCUS_FG: Color = Color::LightCyan;
const COLOR_STATUS_GOOD: Color = Color::Green; // For "Polling" or active connections (Green in diagram)
const COLOR_STATUS_BAD: Color = Color::Red;    // For "Working" or problematic (Red in diagram)
const COLOR_CATEGORY_TITLE: Color = Color::Yellow;


pub fn draw(f: &mut Frame, app: &App) {
    let size = f.size();
    let outer = Block::default()
        .title(Span::styled("Hive Monitor", Style::default().fg(COLOR_BORDER)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_BORDER));
    f.render_widget(outer, size);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(35),
            Constraint::Percentage(40),
        ].as_ref())
        .margin(1)
        .split(size);

    draw_workers_list(f, cols[0], app);
    draw_actions_panel(f, cols[1], app);

    // Conditional rendering for the third column based on action_panel_state and focus
    match &app.action_panel_state {
        ActionPanelState::None => {
            draw_global_view(f, cols[2], app);
        },
        ActionPanelState::PullModel => { // This is now the input state
            draw_action_input_panel(f, cols[2], app, ActionType::Pull);
        },
        ActionPanelState::DeleteModel => { // This is now the input state
            draw_action_input_panel(f, cols[2], app, ActionType::Delete);
        },
        ActionPanelState::Confirmation(model_name, action_type) => {
            draw_model_confirmation_panel(f, cols[2], app, model_name, *action_type);
        },
        ActionPanelState::Response(model_name, action_type, result, is_success) => {
            draw_model_response_panel(f, cols[2], app, model_name, *action_type, result, is_success);
        }
    }
}

fn draw_workers_list(f: &mut Frame, area: Rect, app: &App) {
    let mut items = Vec::new();

    if let Some(statuses) = &app.worker_statuses {
        let mut names: Vec<_> = statuses.keys().filter(|&n| n != "Unauthenticated").cloned().collect();
        names.sort();

        for (i, name) in names.iter().enumerate() {
            let conns = app.worker_connections
                .as_ref()
                .and_then(|m| m.get(name))
                .copied()
                .unwrap_or(0);
            let label = format!("{} ({})", name, conns);
            let style = if app.focus == Focus::WorkersList && i == app.selected_worker {
                Style::default().fg(COLOR_HIGHLIGHT_FG).bg(COLOR_HIGHLIGHT_BG)
            } else if (app.focus == Focus::ActionsList || app.focus == Focus::GlobalView) && i == app.selected_worker {
                Style::default().fg(COLOR_SELECTED_INACTIVE_FOCUS_FG).bg(COLOR_SELECTED_INACTIVE_FOCUS_BG)
            }
            else {
                Style::default().fg(COLOR_DEFAULT_FG).bg(COLOR_DEFAULT_BG)
            };
            items.push(ListItem::new(label).style(style));
        }
    }

    if let Some(cnt) = app.worker_connections.as_ref().and_then(|m| m.get("Unauthenticated")).copied() {
        items.push(ListItem::new(format!("Unauthenticated ({})", cnt)).style(Style::default().fg(Color::DarkGray)));
    }

    let block_title = Span::styled("Workers", Style::default().fg(COLOR_BORDER));
    let block_style = Style::default().fg(COLOR_DEFAULT_FG).bg(COLOR_DEFAULT_BG);
    let block = Block::default()
        .title(block_title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_BORDER))
        .style(block_style);

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn draw_actions_panel(f: &mut Frame, area: Rect, app: &App) {
    let parts = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(app.worker_actions.len() as u16 + 2), Constraint::Min(0)].as_ref())
        .split(area);

    // Actions list
    let mut items = Vec::new();
    for (i, act) in app.worker_actions.iter().enumerate() {
        let style = if app.focus == Focus::ActionsList && i == app.selected_action {
            Style::default().fg(COLOR_HIGHLIGHT_FG).bg(COLOR_HIGHLIGHT_BG)
        } else {
            Style::default().fg(COLOR_DEFAULT_FG).bg(COLOR_DEFAULT_BG)
        };
        items.push(ListItem::new(Span::raw(*act)).style(style));
    }
    let actions_block = Block::default()
        .title(Span::styled("Worker Actions", Style::default().fg(COLOR_BORDER)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_BORDER))
        .style(Style::default().fg(COLOR_DEFAULT_FG).bg(COLOR_DEFAULT_BG));
    let actions = List::new(items).block(actions_block);
    f.render_widget(actions, parts[0]);

    // Info panel for selected worker
    let mut info_block_title = "Info".to_string(); // Default title if no worker selected or loading

    let info = if let Some(statuses) = &app.worker_statuses {
        let mut names: Vec<_> = statuses.keys().filter(|&n| n != "Unauthenticated").cloned().collect();
        names.sort();

        if let Some(name) = names.get(app.selected_worker) {
            info_block_title = format!("Info: {}", name);
            let mut lines: Vec<Line> = Vec::new();

            lines.push(Line::from(Span::styled("Versions:", Style::default().add_modifier(Modifier::BOLD))));
            if let Some(vs) = &app.worker_versions {
                if let Some(v) = vs.get(name) {
                    lines.push(Line::from(format!("  Hive: {}", v.hive)));
                    lines.push(Line::from(format!("  Ollama: {}", v.ollama)));
                } else {
                    lines.push(Line::from("  No version info"));
                }
            } else {
                lines.push(Line::from("  Loading versions..."));
            }

            lines.push(Line::from(Span::styled("Last Ping:", Style::default().add_modifier(Modifier::BOLD))));
            if let Some(pings) = &app.worker_pings {
                if let Some(times) = pings.get(name) {
                    if let Some(latest) = times.last() {
                        lines.push(Line::from(format!("  {}", latest.to_rfc3339())));
                    } else {
                        lines.push(Line::from("  No ping data"));
                    }
                } else {
                    lines.push(Line::from("  No ping data for worker"));
                }
            } else {
                lines.push(Line::from("  Loading pings..."));
            }

            lines.push(Line::from(Span::styled("Models:", Style::default().add_modifier(Modifier::BOLD))));
            if let Some(tags) = &app.worker_tags {
                if let Some(worker_models) = tags.get(name) {
                    if worker_models.is_empty() {
                         lines.push(Line::from("  No models loaded"));
                    } else {
                        for model_name in worker_models {
                            lines.push(Line::from(format!("  - {}", model_name)));
                        }
                    }
                } else {
                    lines.push(Line::from("  No model tags for worker"));
                }
            } else {
                lines.push(Line::from("  Loading models..."));
            }

            Paragraph::new(lines).block(Block::default()
                .title(Span::styled(&info_block_title, Style::default().fg(COLOR_BORDER)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_BORDER))
                .style(Style::default().fg(COLOR_DEFAULT_FG).bg(COLOR_DEFAULT_BG)))
        } else {
            Paragraph::new("No worker selected (or data loading)").block(Block::default()
                .title(Span::styled(&info_block_title, Style::default().fg(COLOR_BORDER)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_BORDER))
                .style(Style::default().fg(COLOR_DEFAULT_FG).bg(COLOR_DEFAULT_BG)))
        }
    } else {
        Paragraph::new("Loading worker statuses...").block(Block::default()
            .title(Span::styled(&info_block_title, Style::default().fg(COLOR_BORDER)))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(COLOR_BORDER))
            .style(Style::default().fg(COLOR_DEFAULT_FG).bg(COLOR_DEFAULT_BG)))
    };
    f.render_widget(info, parts[1]);
}




// Helper to draw the main global view (Workers Busy grid and Queues) - unchanged
fn draw_global_view(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
        .split(area);

    let worker_grid_area = chunks[0];
    let queues_area = chunks[1];

    let worker_grid_block = Block::default()
        .title(Span::styled("Workers Busy", Style::default().fg(COLOR_BORDER)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_BORDER))
        .style(Style::default().fg(COLOR_DEFAULT_FG).bg(COLOR_DEFAULT_BG));
    f.render_widget(&worker_grid_block, worker_grid_area);
    let worker_grid_inner_area = worker_grid_block.inner(worker_grid_area);

    if let Some(statuses) = &app.worker_statuses {
        let mut names: Vec<_> = statuses.keys()
            .filter(|&n| n != "Unauthenticated")
            .cloned()
            .collect();
        names.sort();

        let worker_count = names.len();
        if worker_count == 0 {
            let p = Paragraph::new("No workers online.")
                .block(Block::default().title(Span::styled("Workers Busy", Style::default().fg(COLOR_BORDER))).borders(Borders::ALL).border_style(Style::default().fg(COLOR_BORDER)));
            f.render_widget(p, worker_grid_inner_area);
        } else {
            let cols = (worker_count as f32).sqrt().ceil() as u16;
            let rows = (worker_count as u16).div_ceil(cols);

            let row_constraints: Vec<Constraint> = (0..rows)
                .map(|_| Constraint::Min(3))
                .collect();
            let row_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(row_constraints)
                .split(worker_grid_inner_area);

            for (r_idx, row_rect) in row_layout.iter().enumerate() {
                let col_constraints: Vec<Constraint> = (0..cols)
                    .map(|_| Constraint::Ratio(1, cols as u32))
                    .collect();
                let col_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(col_constraints)
                    .split(*row_rect);

                for (c_idx, cell_area) in col_layout.iter().enumerate() {
                    let idx = r_idx * cols as usize + c_idx;
                    if let Some(name) = names.get(idx) {
                        let worker_block = Block::default()
                            .title(name.as_str())
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(COLOR_BORDER));
                        f.render_widget(&worker_block, *cell_area);

                        let inner_cell_area = worker_block.inner(*cell_area);

                        let connection_count = app.worker_connections
                            .as_ref()
                            .and_then(|m| m.get(name))
                            .copied()
                            .unwrap_or(0);

                        let worker_status = statuses.get(name)
                            .and_then(|v| v.last())
                            .map(String::as_str)
                            .unwrap_or("Unknown");

                        let connection_color = if worker_status == "Working" {
                            COLOR_STATUS_BAD
                        } else {
                            COLOR_STATUS_GOOD
                        };

                        if connection_count > 0 {
                            let square_size = 2;
                            let gap_size = 1;
                            let effective_square_width = square_size + gap_size;
                            let available_width_for_squares = inner_cell_area.width.saturating_sub(1);

                            let max_squares_per_line = if effective_square_width > 0 {
                                available_width_for_squares / effective_square_width
                            } else {
                                0
                            }.max(1);

                            let required_square_rows = (connection_count as u16).div_ceil(max_squares_per_line);

                            let square_rows_total_height = required_square_rows;
                            let square_vertical_margin = 0;
                            let vertical_space_needed = square_rows_total_height + square_vertical_margin;

                            if inner_cell_area.height >= vertical_space_needed {
                                let square_render_area = Layout::default()
                                    .direction(Direction::Vertical)
                                    .constraints([
                                        Constraint::Length(square_vertical_margin),
                                        Constraint::Length(square_rows_total_height),
                                        Constraint::Min(0)
                                    ])
                                    .split(inner_cell_area)[1];

                                let actual_square_row_layout = Layout::default()
                                    .direction(Direction::Vertical)
                                    .constraints((0..required_square_rows).map(|_| Constraint::Length(1)).collect::<Vec<Constraint>>())
                                    .split(square_render_area);


                                for row_num in 0..required_square_rows {
                                    if let Some(row_rect_for_squares) = actual_square_row_layout.get(row_num as usize) {
                                        let start_idx = row_num * max_squares_per_line;
                                        let end_idx = (start_idx + max_squares_per_line).min(connection_count as u16);
                                        let squares_in_this_row = end_idx - start_idx;

                                        if squares_in_this_row > 0 {
                                            let mut h_square_constraints = Vec::new();
                                            for _ in 0..squares_in_this_row {
                                                h_square_constraints.push(Constraint::Length(square_size));
                                                h_square_constraints.push(Constraint::Length(gap_size));
                                            }
                                            h_square_constraints.pop();
                                            h_square_constraints.push(Constraint::Min(0));

                                            let h_square_layout = Layout::default()
                                                .direction(Direction::Horizontal)
                                                .constraints(h_square_constraints)
                                                .split(*row_rect_for_squares);

                                            for i in 0..squares_in_this_row {
                                                if let Some(sq_area) = h_square_layout.get(i as usize * 2) {
                                                    let square = Block::default().style(Style::default().bg(connection_color));
                                                    f.render_widget(square, *sq_area);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        let loading = Paragraph::new("Loading worker statuses...")
            .block(Block::default().title(Span::styled("Workers Busy", Style::default().fg(COLOR_BORDER))).borders(Borders::ALL).border_style(Style::default().fg(COLOR_BORDER)));
        f.render_widget(loading, worker_grid_inner_area);
    }

    let queues_block = Block::default()
        .title(Span::styled("Queues", Style::default().fg(COLOR_BORDER)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_BORDER))
        .style(Style::default().fg(COLOR_DEFAULT_FG).bg(COLOR_DEFAULT_BG));
    f.render_widget(&queues_block, queues_area);
    let queues_inner_area = queues_block.inner(queues_area);

    let queue_sub_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(queues_inner_area);

    let model_queues_area = queue_sub_chunks[0];
    let worker_queues_area = queue_sub_chunks[1];

    let mut model_lines: Vec<Line> = Vec::new();
    let mut worker_lines: Vec<Line> = Vec::new();

    if let Some(queues) = &app.queue_map {
        let mut sorted_queue_names: Vec<&String> = queues.keys().collect();
        sorted_queue_names.sort_unstable();

        for name in sorted_queue_names {
            let cnt = queues.get(name).unwrap();
            if name.starts_with("Model:") {
                model_lines.push(Line::from(name.replace("Model: ", "").to_string()));
            } else if name.starts_with("Node:") {
                worker_lines.push(Line::from(format!("{}: {}", name.replace("Node: ", ""), cnt)));
            } else {
                model_lines.push(Line::from(format!("{}: {}", name, cnt)));
            }
        }
    } else {
        model_lines.push(Line::from("No model queues"));
        worker_lines.push(Line::from("No worker queues"));
    }

    let model_paragraph = Paragraph::new(vec![
        Line::from(Span::styled("MODEL", Style::default().add_modifier(Modifier::BOLD).fg(COLOR_CATEGORY_TITLE))),
        Line::from(""),
    ].into_iter().chain(model_lines).collect::<Vec<Line>>())
    .style(Style::default().fg(COLOR_DEFAULT_FG).bg(COLOR_DEFAULT_BG));
    f.render_widget(model_paragraph, model_queues_area);

    let worker_paragraph = Paragraph::new(vec![
        Line::from(Span::styled("WORKER", Style::default().add_modifier(Modifier::BOLD).fg(COLOR_CATEGORY_TITLE))),
        Line::from(""),
    ].into_iter().chain(worker_lines).collect::<Vec<Line>>())
    .style(Style::default().fg(COLOR_DEFAULT_FG).bg(COLOR_DEFAULT_BG));
    f.render_widget(worker_paragraph, worker_queues_area);
}


// --- New drawing functions for Action Panel ---

fn draw_action_input_panel(f: &mut Frame, area: Rect, app: &App, action_type: ActionType) {
    let action_verb = match action_type {
        ActionType::Pull => "Pull",
        ActionType::Delete => "Delete",
    };
    let title = format!("{} Model", action_verb);

    let block = Block::default()
        .title(Span::styled(&title, Style::default().fg(COLOR_BORDER)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_BORDER))
        .style(Style::default().fg(COLOR_DEFAULT_FG).bg(COLOR_DEFAULT_BG));
    f.render_widget(&block, area);

    let inner_area = block.inner(area);

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Prompt "Model Name:"
            Constraint::Length(3), // Input box (with borders)
            Constraint::Min(0),    // Spacer
            Constraint::Length(1), // Instructions
            Constraint::Length(1), // ESC instruction
        ].as_ref())
        .margin(1) // Inner margin
        .split(inner_area);

    let prompt_area = main_layout[0];
    let input_area = main_layout[1];
    let instructions_area = main_layout[3];
    let esc_instruction_area = main_layout[4];

    // Prompt text
    f.render_widget(Paragraph::new(format!("{} Model Name:", action_verb))
        .style(Style::default().fg(COLOR_DEFAULT_FG)), prompt_area);

    // Input box
    let input_block = Block::default().borders(Borders::ALL)
        .border_style(Style::default().fg(if app.focus == Focus::ActionPanelInput { COLOR_HIGHLIGHT_BG } else { COLOR_BORDER }));
    let input_paragraph = Paragraph::new(app.action_input_model_name.as_str())
        .style(Style::default().fg(COLOR_DEFAULT_FG))
        .block(input_block);

    f.render_widget(input_paragraph, input_area);

    // Set cursor position if focused
    if app.focus == Focus::ActionPanelInput {
        f.set_cursor(
            input_area.x + 1 + app.action_input_cursor_position as u16,
            input_area.y + 1,
        );
    }

    // Instructions
    f.render_widget(Paragraph::new(Line::from("Type model name, press ENTER to confirm."))
        .style(Style::default().fg(Color::DarkGray)), instructions_area);
    f.render_widget(Paragraph::new(Line::from("Press ESC to cancel."))
        .style(Style::default().fg(Color::DarkGray)), esc_instruction_area);
}

fn draw_model_confirmation_panel(f: &mut Frame, area: Rect, app: &App, model_name: &str, action_type: ActionType) {
    let action_verb = match action_type {
        ActionType::Pull => "pull",
        ActionType::Delete => "delete",
    };
    let title = format!("Confirm {} Model", action_verb);

    let block = Block::default()
        .title(Span::styled(&title, Style::default().fg(COLOR_BORDER)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_BORDER))
        .style(Style::default().fg(COLOR_DEFAULT_FG).bg(COLOR_DEFAULT_BG));
    f.render_widget(&block, area);

    let inner_area = block.inner(area);

    let text = vec![
        Line::from(format!("Are you sure you want to {} model:", action_verb)),
        Line::from(Span::styled(format!("  {}", model_name), Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))),
        Line::from(""),
    ];

    let confirmation_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(text.len() as u16), // Height for the text
            Constraint::Min(0), // Spacer
            Constraint::Length(3), // Height for Yes/No buttons (including borders)
            Constraint::Length(1), // Height for instruction
        ].as_ref())
        .margin(1) // Margin around the whole confirmation content
        .split(inner_area);

    let text_area = confirmation_layout[0];
    let buttons_area = confirmation_layout[2];
    let instruction_area = confirmation_layout[3];

    f.render_widget(Paragraph::new(text).alignment(ratatui::layout::Alignment::Center), text_area);

    // Render Yes/No buttons side-by-side
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Yes
            Constraint::Percentage(50), // No
        ].as_ref())
        .split(buttons_area);

    let yes_style = if app.focus == Focus::ActionPanelConfirm && app.confirmation_selection == 0 {
        Style::default().fg(COLOR_HIGHLIGHT_FG).bg(COLOR_HIGHLIGHT_BG)
    } else {
        Style::default().fg(COLOR_DEFAULT_FG).bg(COLOR_DEFAULT_BG)
    };
    let no_style = if app.focus == Focus::ActionPanelConfirm && app.confirmation_selection == 1 {
        Style::default().fg(COLOR_HIGHLIGHT_FG).bg(COLOR_HIGHLIGHT_BG)
    } else {
        Style::default().fg(COLOR_DEFAULT_FG).bg(COLOR_DEFAULT_BG)
    };

    let yes_button = Paragraph::new(Span::styled("  [ Yes ]  ", yes_style))
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(yes_style));
    let no_button = Paragraph::new(Span::styled("  [ No ]   ", no_style))
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(no_style));

    f.render_widget(yes_button, button_chunks[0]);
    f.render_widget(no_button, button_chunks[1]);

    f.render_widget(Paragraph::new(Line::from("Use LEFT/RIGHT to select, ENTER to confirm")).alignment(ratatui::layout::Alignment::Center).style(Style::default().fg(Color::DarkGray)), instruction_area);
}


fn draw_model_response_panel(f: &mut Frame, area: Rect, app: &App, model_name: &str, action_type: ActionType, output_lines: &Vec<String>, is_overall_success: &bool) {
    let action_verb = match action_type {
        ActionType::Pull => "Pull",
        ActionType::Delete => "Delete",
    };
    let title = format!("{} Model Result", action_verb);

    let block_style = if *is_overall_success {
        Style::default().fg(COLOR_STATUS_GOOD).bg(COLOR_DEFAULT_BG)
    } else {
        Style::default().fg(COLOR_STATUS_BAD).bg(COLOR_DEFAULT_BG)
    };

    let block = Block::default()
        .title(Span::styled(&title, Style::default().fg(COLOR_BORDER)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_BORDER))
        .style(block_style);
    f.render_widget(&block, area);

    let inner_area = block.inner(area);

    let mut lines_to_display: Vec<Line> = vec![
        Line::from(format!("{}: {}", action_verb, model_name)),
        Line::from(""),
    ];

    // Convert output_lines to Line objects
    let content_lines: Vec<Line> = output_lines.iter().map(|s| Line::from(s.clone())).collect();
    lines_to_display.extend(content_lines);

    lines_to_display.push(Line::from("")); // Spacer before instructions
    lines_to_display.push(Line::from(Span::styled("Use UP/DOWN to scroll, any key to dismiss", Style::default().fg(Color::DarkGray))));


    let paragraph = Paragraph::new(lines_to_display)
        .scroll((app.action_panel_scroll, 0)) // Apply scrolling here
        .alignment(ratatui::layout::Alignment::Left) // Left align for log-like output
        .style(Style::default().fg(COLOR_DEFAULT_FG).bg(COLOR_DEFAULT_BG));

    f.render_widget(paragraph, inner_area);
}
// --- New drawing functions for Action Panel ---

fn draw_model_action_panel(f: &mut Frame, area: Rect, app: &App, model_name: &str, action_type: ActionType) {
    let action_title = match action_type {
        ActionType::Pull => "Pull Model",
        ActionType::Delete => "Delete Model",
    };

    let block = Block::default()
        .title(Span::styled(action_title, Style::default().fg(COLOR_BORDER)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_BORDER))
        .style(Style::default().fg(COLOR_DEFAULT_FG).bg(COLOR_DEFAULT_BG));
    f.render_widget(&block, area);

    let inner_area = block.inner(area);

    let lines = vec![
        Line::from(format!("Action: {}", action_title)),
        Line::from(format!("Model: {}", model_name)),
        Line::from(""),
        Line::from("Press ENTER to confirm."),
        Line::from("Press ESC to cancel."), // Add ESC for cancellation
    ];

    let paragraph = Paragraph::new(lines)
        .alignment(ratatui::layout::Alignment::Center)
        .style(Style::default().fg(COLOR_DEFAULT_FG).bg(COLOR_DEFAULT_BG));
    f.render_widget(paragraph, inner_area);
}

