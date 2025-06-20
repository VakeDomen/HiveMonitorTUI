use ratatui::{
    backend::Backend,
    Frame,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph, List, ListItem}
};
use crate::app::App;

/// Draw the Console (Inference) tab
pub fn draw(f: &mut Frame, app: &App) {
    let area = f.size();
    let block = Block::default().title("Console").borders(Borders::ALL);
    f.render_widget(block, area);

    // Split into prompt input (3 lines) and output
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .margin(1)
        .split(area);

    // Prompt area (show current prompt or placeholder)
    let prompt_text = if let Some(resp) = &app.generate_response {
        resp.result.clone()
    } else {
        "<Enter prompt and press Enter>".into()
    };
    let prompt = Paragraph::new(prompt_text)
        .block(Block::default().title("Prompt").borders(Borders::ALL));
    f.render_widget(prompt, chunks[0]);

    // Output area: list of lines from console_output
    let items: Vec<ListItem> = if app.console_output.is_empty() {
        vec![ListItem::new("<No output>")]
    } else {
        app.console_output.iter().map(|l| ListItem::new(l.clone())).collect()
    };
    let output = List::new(items)
        .block(Block::default().title("Output").borders(Borders::ALL));
    f.render_widget(output, chunks[1]);
}