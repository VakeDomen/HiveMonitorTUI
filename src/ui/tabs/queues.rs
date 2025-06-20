use ratatui::{
    backend::Backend, Frame,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Table, Row, Cell, Paragraph},
};
use crate::app::App;

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.size();
    let block = Block::default().title("Queues").borders(Borders::ALL);
    f.render_widget(block, area);

    if let Some(ref queue_map) = app.queue_map {
        // split into two halves (for model vs. node, later)
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(area);

        // for now we just show a unified list on the left
        let mut items: Vec<(&String, &usize)> = queue_map.iter().collect();
        items.sort_by_key(|(k, _)| *k);

        let rows: Vec<Row> = items.iter()
            .map(|(name, cnt)| Row::new(vec![Cell::from(name.as_str()), Cell::from(cnt.to_string())]))
            .collect();

        let table = Table::new(
                rows, 
                &[Constraint::Percentage(70), Constraint::Percentage(30)]
            )
            .header(Row::new(vec![Cell::from("Queue"), Cell::from("Count")]))
            .block(Block::default().borders(Borders::ALL))
            .widths(&[Constraint::Percentage(70), Constraint::Percentage(30)]);
        f.render_widget(table, cols[0]);

        // TODO: render node-specific queues in cols[1]
    } else {
        let loading = Paragraph::new("Loading queues…")
            .block(Block::default().title("Queues").borders(Borders::ALL));
        f.render_widget(loading, area);
    }
}
