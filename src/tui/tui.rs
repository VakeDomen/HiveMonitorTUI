use std::io;

use crossterm::{event::{DisableMouseCapture, EnableMouseCapture}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use ratatui::{prelude::CrosstermBackend, Frame, Terminal};
use anyhow::Result;


pub trait TUIPanel {
    fn draw(&self, frame: &mut Frame);
    fn handle_events(&mut self) -> Result<bool>;
}


pub struct TUI {
    exit: bool,
}

impl TUI {
    pub fn new() -> Self {
        TUI { exit: false }
    }

    pub fn run(&mut self, mut panel: impl TUIPanel) -> Result<()> {
        // If `url` or `token` is missing, prompt before the main loo
        self.exit = false;
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        
        while !self.exit {
            terminal.draw(|frame| panel.draw(frame))?;
            self.exit = match panel.handle_events() {
                Ok(b) => b,
                Err(_) => true,
            };
        }


        // Cleanup terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }
}