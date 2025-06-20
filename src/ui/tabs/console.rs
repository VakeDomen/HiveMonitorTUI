use ratatui::{backend::Backend, Frame, widgets::{Block, Borders, Paragraph}, layout::Layout};
use crate::app::App;

/// Draw the Console (Inference) tab
pub fn draw(f: &mut Frame, app: &App) {
    let size = f.size();
    let block = Block::default().title("Console").borders(Borders::ALL);
    f.render_widget(block, size);
    // TODO: form for inference requests and output pane
}