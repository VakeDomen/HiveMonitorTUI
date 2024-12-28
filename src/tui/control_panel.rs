use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer, 
    layout::{Constraint, Direction, Layout, Rect}, 
    style::Stylize, 
    symbols::border, 
    text::{Line, Text}, 
    widgets::{Block, Paragraph, Widget}, 
    Frame
};

use crate::widgets::workers::Workers;
use anyhow::Result;

use super::tui::TUIPanel;

/// Our application state
#[derive(Debug, Default)]
pub struct App {
    url: String,
    token: String,
    counter: u8,
    workers: Workers,
}

impl From<(String, String)> for App {
    fn from(value: (String, String)) -> Self {
        Self { url: value.0, token: value.1, counter: 0, workers: Workers::default()}
    }
}

impl App {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> bool {
        match key_event.code {
            KeyCode::Char('q') => return true,
            KeyCode::Left => self.decrement_counter(),
            KeyCode::Right => self.increment_counter(),
            KeyCode::Up => self.workers.up(),
            KeyCode::Down => self.workers.down(),
            _ => {}
        };
        false
    }

    fn increment_counter(&mut self) {
        self.counter = self.counter.wrapping_add(1);
    }

    fn decrement_counter(&mut self) {
        self.counter = self.counter.wrapping_sub(1);
    }
}

impl TUIPanel for App {
       
    fn draw(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(frame.area());

        frame.render_widget(self, chunks[0]);

        let chunks2 = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        frame.render_widget(self.workers.clone(), chunks2[0]);
        frame.render_widget(self.workers.clone(), chunks2[1]);
    }

    fn handle_events(&mut self) -> Result<bool> {
        Ok(match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => self.handle_key_event(key_event),
            _ => false
        })
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let line1 = self.url.clone();
        let line2 = self.token.clone();

        let title = Line::from(" Counter App Tutorial ".bold());
        let instructions = Line::from(vec![
            " Decrement ".bold().into(),
            "<Left>".blue().bold().into(),
            " Increment ".bold(),
            "<Right>".blue().bold(),
            " Quit ".bold(),
            "<Q> ".blue().bold(),
        ]);

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let lines = vec![
            Line::from(format!("Value: {}", self.counter).yellow()),
            Line::from(line1),
            Line::from(line2),
        ];
        let paragraph = Paragraph::new(Text::from(lines))
            .centered()
            .block(block);

        paragraph.render(area, buf);
    }
}