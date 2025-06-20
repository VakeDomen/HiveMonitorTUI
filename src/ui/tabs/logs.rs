use ratatui::{backend::Backend, Frame, widgets::{Block, Borders, List}, layout::Layout};
use crate::app::App;

/// Draw the Logs tab
pub fn draw(f: &mut Frame, app: &App) {
    let size = f.size();
    let block = Block::default().title("Logs").borders(Borders::ALL);
    f.render_widget(block, size);
    // TODO: tail metrics via polling
}
