// src/ui/tabs/dashboard.rs
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::app::{App, Focus};

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


/// Draw the Dashboard tab with focusable regions: WorkersList, ActionsList, GlobalView
pub fn draw(f: &mut Frame, app: &App) {
    let size = f.size();
    // Outer block
    let outer = Block::default()
        .title(Span::styled("Hive Monitor", Style::default().fg(COLOR_BORDER)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_BORDER));
    f.render_widget(outer, size);

    // Divide into three columns: workers, actions, global
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // Workers list
            Constraint::Percentage(35), // Actions list + info panel
            Constraint::Percentage(40), // Global view: grid and queues
        ].as_ref())
        .margin(1) // Margin around the three columns
        .split(size);

    // 1) Workers list
    draw_workers_list(f, cols[0], app);
    // 2) Actions list + info panel
    draw_actions_panel(f, cols[1], app);
    // 3) Global view: grid and queues
    draw_global_view(f, cols[2], app);
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

fn draw_global_view(f: &mut Frame, area: Rect, app: &App) {
    // Split the global view area vertically into worker grid and queues
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
        .split(area);

    let worker_grid_area = chunks[0];
    let queues_area = chunks[1];

    // Worker Grid (Block and internal layout)
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
            // Determine grid dimensions
            let cols = (worker_count as f32).sqrt().ceil() as u16;
            let rows = (worker_count as u16 + cols - 1) / cols;

            let row_constraints: Vec<Constraint> = (0..rows)
                .map(|_| Constraint::Min(3)) // Give each row minimum height for worker name + squares (2 for name, 1 for squares, potentially more for wrapped squares)
                .collect();
            let row_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(row_constraints)
                .split(worker_grid_inner_area);

            for (r_idx, row_rect) in row_layout.into_iter().enumerate() {
                let col_constraints: Vec<Constraint> = (0..cols)
                    .map(|_| Constraint::Ratio(1, cols as u32))
                    .collect();
                let col_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(col_constraints)
                    .split(*row_rect);

                for (c_idx, cell_area) in col_layout.into_iter().enumerate() {
                    let idx = r_idx as usize * cols as usize + c_idx as usize;
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

                        // --- Draw individual connection squares with wrapping ---
                        if connection_count > 0 {
                            // Calculate how many squares fit on one line
                            let square_size = 2; // Each square is 2 chars wide (e.g., "[]")
                            let gap_size = 1;    // 1 char for space between squares
                            let effective_square_width = square_size + gap_size; // Total width for a square and its trailing gap

                            // Ensure there's at least 1 char width for the block inner area,
                            // otherwise, `saturating_sub` could lead to large numbers if `width` is 0.
                            let available_width_for_squares = inner_cell_area.width.saturating_sub(1); // Account for border/padding if any

                            let max_squares_per_line = if effective_square_width > 0 {
                                available_width_for_squares / effective_square_width
                            } else {
                                0
                            }.max(1); // At least one square if space allows and width > 0

                            // Calculate required rows for squares
                            let required_square_rows = (connection_count as u16 + max_squares_per_line - 1) / max_squares_per_line;

                            // Create vertical layout for rows of squares
                            // We need enough height for all rows of squares, plus potential margin.
                            // Each square row will be 1 char high.
                            let square_rows_total_height = required_square_rows; // 1 char height per row
                            let square_vertical_margin = 0; // Margin between title and first row of squares
                            let vertical_space_needed = square_rows_total_height + square_vertical_margin;

                            // Ensure there's enough vertical space in the inner cell for squares
                            if inner_cell_area.height >= vertical_space_needed {
                                let square_render_area = Layout::default()
                                    .direction(Direction::Vertical)
                                    .constraints([
                                        Constraint::Length(square_vertical_margin), // Top margin
                                        Constraint::Length(square_rows_total_height), // Space for all square rows
                                        Constraint::Min(0) // Remaining space
                                    ])
                                    .split(inner_cell_area)[1]; // Take the second chunk (where squares will go)

                                let actual_square_row_layout = Layout::default()
                                    .direction(Direction::Vertical)
                                    .constraints((0..required_square_rows).map(|_| Constraint::Length(1)).collect::<Vec<Constraint>>())
                                    .split(square_render_area);


                                for row_num in 0..required_square_rows {
                                    if let Some(row_rect_for_squares) = actual_square_row_layout.get(row_num as usize) {
                                        let start_idx = row_num * max_squares_per_line as u16;
                                        let end_idx = (start_idx + max_squares_per_line).min(connection_count as u16);
                                        let squares_in_this_row = end_idx - start_idx;

                                        if squares_in_this_row > 0 {
                                            // Create horizontal layout for squares in this specific row
                                            let mut h_square_constraints = Vec::new();
                                            for _ in 0..squares_in_this_row {
                                                h_square_constraints.push(Constraint::Length(square_size)); // Square width
                                                h_square_constraints.push(Constraint::Length(gap_size));    // Small gap
                                            }
                                            h_square_constraints.pop(); // Remove last gap
                                            h_square_constraints.push(Constraint::Min(0)); // Remaining space

                                            let h_square_layout = Layout::default()
                                                .direction(Direction::Horizontal)
                                                .constraints(h_square_constraints)
                                                .split(*row_rect_for_squares);

                                            for i in 0..squares_in_this_row {
                                                if let Some(sq_area) = h_square_layout.get(i as usize * 2) { // Get the square area, skipping gaps
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

    // Queues Panel
    let queues_block = Block::default()
        .title(Span::styled("Queues", Style::default().fg(COLOR_BORDER)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_BORDER))
        .style(Style::default().fg(COLOR_DEFAULT_FG).bg(COLOR_DEFAULT_BG));
    f.render_widget(&queues_block, queues_area);
    let queues_inner_area = queues_block.inner(queues_area);

    // --- Split Queues into Model and Worker side-by-side ---
    let queue_sub_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(queues_inner_area);

    let model_queues_area = queue_sub_chunks[0];
    let worker_queues_area = queue_sub_chunks[1];

    let mut model_lines: Vec<Line> = Vec::new();
    let mut worker_lines: Vec<Line> = Vec::new();

    if let Some(queues) = &app.queue_map {
        // Collect and sort keys for consistent display
        let mut sorted_queue_names: Vec<&String> = queues.keys().collect();
        sorted_queue_names.sort_unstable(); // Use unstable sort for performance if order doesn't need to be fully stable

        for name in sorted_queue_names {
            let cnt = queues.get(name).unwrap(); // Should always exist since we got the name from keys()
            if name.starts_with("Model:") {
                model_lines.push(Line::from(format!("{}: {}", name.replace("Model: ", ""), cnt)));
            } else if name.starts_with("Node:") {
                worker_lines.push(Line::from(format!("{}: {}", name.replace("Node: ", ""), cnt)));
            } else {
                // Fallback for any other unexpected queue types
                // You might want to handle these differently or log them
                model_lines.push(Line::from(format!("{}: {}", name, cnt)));
            }
        }
    } else {
        model_lines.push(Line::from("No model queues"));
        worker_lines.push(Line::from("No worker queues"));
    }

    // Render Model Queues
    let model_paragraph = Paragraph::new(vec![
        Line::from(Span::styled("MODEL", Style::default().add_modifier(Modifier::BOLD).fg(COLOR_CATEGORY_TITLE))),
        Line::from(""), // Add a blank line for spacing
    ].into_iter().chain(model_lines.into_iter()).collect::<Vec<Line>>())
    .style(Style::default().fg(COLOR_DEFAULT_FG).bg(COLOR_DEFAULT_BG));
    f.render_widget(model_paragraph, model_queues_area);

    // Render Worker Queues
    let worker_paragraph = Paragraph::new(vec![
        Line::from(Span::styled("WORKER", Style::default().add_modifier(Modifier::BOLD).fg(COLOR_CATEGORY_TITLE))),
        Line::from(""), // Add a blank line for spacing
    ].into_iter().chain(worker_lines.into_iter()).collect::<Vec<Line>>())
    .style(Style::default().fg(COLOR_DEFAULT_FG).bg(COLOR_DEFAULT_BG));
    f.render_widget(worker_paragraph, worker_queues_area);
}