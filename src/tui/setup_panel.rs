use anyhow::Result;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::tui::TUIPanel;

#[derive(Clone, Debug)]
pub struct SetupPanel {
    data: SetupData,
    accepted: bool,
}

#[derive(Clone, Debug)]
struct SetupData {
    url: String,
    token: String,
    focused_field: FieldFocus,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum FieldFocus {
    Url,
    Token,
}

impl SetupPanel {
    pub fn new(initial_url: Option<String>, initial_token: Option<String>) -> Self {
        SetupPanel {
            data: SetupData {
                url: initial_url.unwrap_or_default(),
                token: initial_token.unwrap_or_default(),
                focused_field: FieldFocus::Url,
            },
            accepted: false,
        }
    }

    pub fn final_values(self) -> (Option<String>, Option<String>) {
        if self.accepted {
            (Some(self.data.url), Some(self.data.token))
        } else {
            (None, None)
        }
    }
}

impl TUIPanel for SetupPanel {
    fn draw(&self, frame: &mut Frame) {
        ui_setup(frame, &self.data);
    }

    fn handle_events(&mut self) -> Result<bool> {
        let ev = event::read()?;

        if let Event::Key(KeyEvent { code, kind: KeyEventKind::Press, .. }) = ev {
            match code {
                KeyCode::Esc => {
                    self.accepted = false;
                    return Ok(true);
                }
                KeyCode::Enter => {
                    self.accepted = true;
                    return Ok(true);
                }
                KeyCode::Tab => {
                    self.data.focused_field = match self.data.focused_field {
                        FieldFocus::Url => FieldFocus::Token,
                        FieldFocus::Token => FieldFocus::Url,
                    };
                }
                KeyCode::Backspace => {
                    match self.data.focused_field {
                        FieldFocus::Url => {
                            if !self.data.url.is_empty() {
                                self.data.url.pop();
                            }
                        }
                        FieldFocus::Token => {
                            if !self.data.token.is_empty() {
                                self.data.token.pop();
                            }
                        }
                    }
                }
                KeyCode::Char(c) => {
                    match self.data.focused_field {
                        FieldFocus::Url => self.data.url.push(c),
                        FieldFocus::Token => self.data.token.push(c),
                    }
                }
                _ => {}
            }
        }

        Ok(false)
    }
}

fn ui_setup(frame: &mut Frame, data: &SetupData) {
    let size = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(size);

    // -- 1) The URL Field
    let url_block = Block::default().title("URL").borders(Borders::ALL);

    // If this is the field in focus, highlight it (REVERSED)
    let url_text = if data.focused_field == FieldFocus::Url {
        Text::raw(data.url.clone())
            .patch_style(Style::default().add_modifier(Modifier::REVERSED))
    } else {
        Text::raw(data.url.clone())
    };
    let p1 = Paragraph::new(url_text).block(url_block);
    frame.render_widget(p1, chunks[0]);

    let token_block = Block::default().title("Token").borders(Borders::ALL);

    let token_text = if data.focused_field == FieldFocus::Token {
        Text::raw(data.token.clone())
            .patch_style(Style::default().add_modifier(Modifier::REVERSED))
    } else {
        Text::raw(data.token.clone())
    };
    let p2 = Paragraph::new(token_text).block(token_block);
    frame.render_widget(p2, chunks[1]);

    let instructions = vec![Line::from(vec![
        Span::raw("Press "),
        Span::styled("<Enter>", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to accept changes, "),
        Span::styled("<Esc>", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to quit, "),
        Span::styled("<Tab>", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to switch fields.\nBackspace to delete, type to insert."),
    ])];

    let block = Block::default().title(" Setup TUI ").borders(Borders::ALL);
    let p3 = Paragraph::new(Text::from(instructions)).block(block);
    frame.render_widget(p3, chunks[2]);
}
