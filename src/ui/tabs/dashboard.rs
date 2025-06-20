use ratatui::{
    backend::Backend, layout::{Alignment, Constraint, Direction, Layout}, style::{Color, Modifier, Style}, widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Sparkline, Table}, Frame
};
use crate::app::App;

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.size();
    let outer = Block::default().title("Dashboard").borders(Borders::ALL);
    f.render_widget(outer, size);

    // Reserve top 6 lines for gauges & info, rest for worker grid
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // gauges
            Constraint::Length(3), // info
            Constraint::Min(0),    // grid
        ].as_ref())
        .margin(1)
        .split(size);

    // --- Row 1: Gauges ---
    let gauge_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(sections[0]);

    let total_nodes = app.worker_statuses.as_ref().map(|m| m.len()).unwrap_or(0) as u16;
    let g1 = Gauge::default()
        .block(Block::default().title("Nodes").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Green))
        .ratio((total_nodes.min(100)) as f64 / 100.0)
        .label(format!("{}", total_nodes));
    f.render_widget(g1, gauge_cols[0]);

    let total_queued = app.queue_map.as_ref().map(|q| q.values().sum::<usize>() as u16).unwrap_or(0);
    let g2 = Gauge::default()
        .block(Block::default().title("Queued").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Yellow))
        .ratio((total_queued.min(100)) as f64 / 100.0)
        .label(format!("{}", total_queued));
    f.render_widget(g2, gauge_cols[1]);

    // --- Row 2: Info panels ---
    let info_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(sections[1]);

    let total_keys = app.auth_keys.as_ref().map(|k| k.len()).unwrap_or(0);
    let p3 = Paragraph::new(format!("{} keys", total_keys))
        .block(Block::default().title("Auth Keys").borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan))
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(p3, info_cols[0]);

    let avg_ping = app.worker_pings.as_ref().map(|map| {
        let sum: i64 = map.values()
            .filter_map(|times| times.last().map(|dt| dt.timestamp_millis()))
            .sum();
        let cnt = map.len() as i64;
        if cnt>0 { sum/cnt } else { 0 }
    }).unwrap_or(0);
    let p4 = Paragraph::new(format!("{} ms", avg_ping))
        .block(Block::default().title("Avg Ping").borders(Borders::ALL))
        .style(Style::default().fg(Color::Magenta))
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(p4, info_cols[1]);

    // --- Row 3: Worker grid ---
    let binding = std::collections::HashMap::new();
    let statuses = app.worker_statuses.as_ref().unwrap_or(&binding);
    let count = statuses.len().max(1);
    let cols = (count as f32).sqrt().ceil() as usize;
    let rows = ((count + cols - 1) / cols) as usize;

    // build a Vec<Constraint> for the row heights
    let mut row_constraints = Vec::with_capacity(rows);
    for _ in 0..rows {
        row_constraints.push(Constraint::Ratio(1, rows as u32));
    }

    let grid_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(sections[2]);

    let mut names: Vec<_> = statuses.keys().cloned().collect();
    names.sort();

    for (r, row_area) in grid_rows.into_iter().enumerate() {
        // build a Vec<Constraint> for the column widths
        let mut col_constraints = Vec::with_capacity(cols);
        for _ in 0..cols {
            col_constraints.push(Constraint::Ratio(1, cols as u32));
        }

        let grid_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(col_constraints)
            .split(*row_area);

        // for each cell in this row
        for c in 0..cols {
            if let Some(worker) = names.get(r * cols + c) {
                let last = statuses.get(worker)
                                .and_then(|v| v.last())
                                .map(String::as_str)
                                .unwrap_or("");
                let color = if last == "Polling" { Color::Green } else { Color::Red };
                let cell = Block::default()
                    .title(worker.as_str())
                    .borders(Borders::ALL)
                    .style(Style::default().bg(color));
                f.render_widget(cell, grid_cols[c]);
            }
        }
    }
}
