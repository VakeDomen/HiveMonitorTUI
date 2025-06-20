use ratatui::{
    backend::Backend,
    Frame,
    layout::{Constraint, Layout},
    widgets::{Block, Borders, List, ListItem},
};
use crate::app::App;

/// Draw the Logs / Metrics tab
pub fn draw(f: &mut Frame, app: &App) {
    let area = f.size();
    let block = Block::default().title("Logs / Metrics").borders(Borders::ALL);
    f.render_widget(block, area);

    // Inner area with margin
    let inner = Layout::default()
        .margin(1)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(area)[0];

    // Build list items from pings and connections
    let mut items: Vec<ListItem> = Vec::new();
    if let Some(pings) = &app.worker_pings {
        for (name, times) in pings {
            if let Some(latest) = times.last() {
                let line = format!("Ping [{}]: {}", name, latest.to_rfc3339());
                items.push(ListItem::new(line));
            }
        }
    }
    
    if let Some(conns) = &app.worker_connections {
        for (name, cnt) in conns {
            let line = format!("Conns [{}]: {}", name, cnt);
            items.push(ListItem::new(line));
        }
    }
    if items.is_empty() {
        items.push(ListItem::new("No metrics available"));
    }

    // Render the list
    let list = List::new(items)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(list, inner);
}
