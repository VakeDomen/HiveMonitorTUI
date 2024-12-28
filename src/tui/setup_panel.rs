use std::io;

use crossterm::{event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use anyhow::Result;
use ratatui::{layout::{Constraint, Direction, Layout}, prelude::CrosstermBackend, style::{Modifier, Style}, text::{Line, Span, Text}, widgets::{Block, Borders, Paragraph}, Frame, Terminal};

/// Minimal struct to hold the text fields we want to edit in the setup TUI
#[derive(Clone, Debug)]
struct SetupData {
    url: String,
    token: String,
    focused_field: FieldFocus,  // which field is currently active
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum FieldFocus {
    Url,
    Token,
}

/// Runs a small TUI that allows the user to fill in `url` and `token`.
/// If provided partial arguments from CLI, we start with them; otherwise they default to empty.
pub fn setup_tui(initial_url: Option<String>, initial_token: Option<String>) -> Result<(String, String)> {
    let mut data = SetupData {
        url: initial_url.unwrap_or_default(),
        token: initial_token.unwrap_or_default(),
        focused_field: FieldFocus::Url,
    };

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main loop
    loop {
        terminal.draw(|f| ui_setup(f, &data))?;

        // Read user events
        match event::read()? {
            Event::Key(KeyEvent {
                code, kind: KeyEventKind::Press, ..
            }) => match code {
                KeyCode::Esc => {
                    // user wants to abort - you could handle differently
                    break;
                }
                KeyCode::Enter => {
                    // Done editing, let's accept
                    break;
                }
                KeyCode::Tab => {
                    // Switch fields
                    data.focused_field = match data.focused_field {
                        FieldFocus::Url => FieldFocus::Token,
                        FieldFocus::Token => FieldFocus::Url,
                    };
                }
                KeyCode::Backspace => {
                    match data.focused_field {
                        FieldFocus::Url => {
                            if !data.url.is_empty() {
                                data.url.pop();
                            }
                        }
                        FieldFocus::Token => {
                            if !data.token.is_empty() {
                                data.token.pop();
                            }
                        }
                    }
                }
                KeyCode::Left => {
                    // optional: handle cursor left?
                }
                KeyCode::Right => {
                    // optional: handle cursor right?
                }
                KeyCode::Char(c) => {
                    // Insert typed character
                    match data.focused_field {
                        FieldFocus::Url => data.url.push(c),
                        FieldFocus::Token => data.token.push(c),
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Return the final strings
    Ok((data.url, data.token))
}

/// The UI for the setup TUI.  
/// Renders 2 "lines" the user can edit: URL, Token
fn ui_setup(frame: &mut Frame, data: &SetupData) {
    let size = frame.size();

    // Layout
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

    // Draw the URL line
    let url_block = Block::default().title("URL").borders(Borders::ALL);
    let url_text = if data.focused_field == FieldFocus::Url {
        // highlight if focused
        Text::raw(data.url.clone()).patch_style(Style::default().add_modifier(Modifier::REVERSED))
    } else {
        Text::raw(data.url.clone())
    };
    let p1 = Paragraph::new(url_text).block(url_block);
    frame.render_widget(p1, chunks[0]);

    // Draw the Token line
    let token_block = Block::default().title("Token").borders(Borders::ALL);
    let token_text = if data.focused_field == FieldFocus::Token {
        Text::raw(data.token.clone()).patch_style(Style::default().add_modifier(Modifier::REVERSED))
    } else {
        Text::raw(data.token.clone())
    };
    let p2 = Paragraph::new(token_text).block(token_block);
    frame.render_widget(p2, chunks[1]);

    // Some instructions
    let instructions = vec![
        Line::from(vec![
            Span::raw("Press "),
            Span::styled("<Enter>", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to accept changes, "),
            Span::styled("<Esc>", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to quit, "),
            Span::styled("<Tab>", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to switch fields.\nBackspace to delete, type to insert."),
        ]),
    ];
    let block = Block::default().title(" Setup TUI ").borders(Borders::ALL);
    let p3 = Paragraph::new(Text::from(instructions)).block(block);
    frame.render_widget(p3, chunks[2]);
}

