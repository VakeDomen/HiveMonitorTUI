use ratatui::{
    backend::Backend, Frame,
    layout::{Constraint, Layout},
    widgets::{Block, Borders, Table, Row, Cell, Paragraph},
};
use crate::app::App;
use chrono::Local;

/// Draw the Keys tab
pub fn draw(f: &mut Frame, app: &App) {
    let area = f.size();
    let block = Block::default().title("Keys").borders(Borders::ALL);
    f.render_widget(block, area);

    if let Some(ref keys) = app.auth_keys {
        if keys.is_empty() {
            let empty = Paragraph::new("No authentication keys found.")
                .block(Block::default().borders(Borders::ALL).title("Keys"));
            f.render_widget(empty, area);
            return;
        }

        // split out margins
        let inner = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(area)[0];

        // build rows
        let rows: Vec<Row> = keys.iter().map(|k| {
            let created = k.created_at.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S").to_string();
            Row::new(vec![
                Cell::from(k.id.clone()),
                Cell::from(k.name.clone()),
                Cell::from(k.role.clone()),
                Cell::from(created),
            ])
        }).collect();

        let header = Row::new(vec!["ID", "Name", "Role", "Created At"]);
        let table = Table::new(
            rows,
            &[Constraint::Percentage(30), Constraint::Percentage(30), Constraint::Percentage(20), Constraint::Percentage(20)]
        )
        .header(header)
        .block(Block::default().borders(Borders::ALL))
        .widths(&[Constraint::Percentage(30), Constraint::Percentage(30), Constraint::Percentage(20), Constraint::Percentage(20)]);
        f.render_widget(table, inner);
    } else {
        let loading = Paragraph::new("Loading keys...")
            .block(Block::default().borders(Borders::ALL).title("Keys"));
        f.render_widget(loading, area);
    }
}
