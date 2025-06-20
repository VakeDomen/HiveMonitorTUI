use ratatui::{backend::Backend, Frame, widgets::{Block, Borders, Table}, layout::Layout};
use crate::app::App;

/// Draw the Nodes tab
pub fn draw(f: &mut Frame, app: &App) {
    let size = f.size();
    let block = Block::default().title("Nodes").borders(Borders::ALL);
    f.render_widget(block, size);
    // TODO: table of worker nodes
}