use ratatui::{backend::Backend, Frame, widgets::{Block, Borders, List}, layout::Layout};
use crate::app::App;

/// Draw the Keys tab
pub fn draw(f: &mut Frame, app: &App) {
    let size = f.size();
    let block = Block::default().title("Keys").borders(Borders::ALL);
    f.render_widget(block, size);
    // TODO: list and create auth keys
}