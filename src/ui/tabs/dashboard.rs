use ratatui::{
    backend::Backend, Frame,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Table, Row, Cell, Paragraph},
};
use crate::app::App;

/// Draw the Nodes tab
pub fn draw(f: &mut Frame, app: &App) {
    let area = f.size();
    let block = Block::default().title("Nodes").borders(Borders::ALL);
    f.render_widget(block, area);

    if let (Some(statuses), Some(conns), Some(pings), Some(versions)) = (
        &app.worker_statuses,
        &app.worker_connections,
        &app.worker_pings,
        &app.worker_versions,
    ) {
        let inner = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(area)[0];

        let mut workers: Vec<_> = statuses.keys().collect();
        workers.sort();
        let rows: Vec<Row> = workers.iter().map(|name| {
            let status = statuses.get(*name).map(|s| format!("{:?}", s)).unwrap_or_default();
            let conn = conns.get(*name).unwrap_or(&0).to_string();
            let ping = pings.get(*name)
                .and_then(|times| times.last())
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| "-".into());
            let vers = versions.get(*name)
                .map(|v| format!("h:{} o:{}", v.hive, v.ollama))
                .unwrap_or_else(|| "-".into());
            Row::new(vec![
                Cell::from(name.as_str()),
                Cell::from(status),
                Cell::from(conn),
                Cell::from(ping),
                Cell::from(vers),
            ])
        }).collect();

        let header = Row::new(vec!["Name", "Status", "Conns", "Last Ping", "Versions"]);
        let table = Table::new(
            rows,
            &[Constraint::Percentage(20), Constraint::Percentage(15), Constraint::Percentage(15), Constraint::Percentage(25), Constraint::Percentage(25)]
        )
        .header(header)
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(table, inner);
    } else {
        let loading = Paragraph::new("Loading nodes...")
            .block(Block::default().borders(Borders::ALL).title("Nodes"));
        f.render_widget(loading, area);
    }
}