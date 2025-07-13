use std::io;
use crossterm::{execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode}};
use ratatui::{backend::CrosstermBackend, Terminal, Frame};
use ratatui::layout::{Rect};
use ratatui::widgets::{Block, Borders, Paragraph};

/// Set up the terminal in raw mode and enter the alternate screen
pub fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    Ok(terminal)
}

/// Restore the terminal to its previous state
pub fn restore_terminal() -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

/// Draw banner messages at the top of the frame
pub fn draw_banners(f: &mut Frame, banners: &[String]) {
    if banners.is_empty() {
        return;
    }
    // Reserve the first line for banners
    let area = Rect { x: 0, y: 0, width: f.area().width, height: banners.len() as u16 };
    let text = banners.join(" | ");
    let para = Paragraph::new(text)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(para, area);
}

