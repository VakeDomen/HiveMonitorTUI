use ratatui::{backend::Backend, Frame, widgets::{Block, Borders, Paragraph}, layout::{Constraint, Direction, Layout}};
use crate::app::App;

/// Draw the Dashboard tab
pub fn draw(f: &mut Frame, app: &App) {
    let size = f.size();
    let block = Block::default().title("Dashboard").borders(Borders::ALL);
    f.render_widget(block, size);
    // TODO: render summary metrics and sparklines
}