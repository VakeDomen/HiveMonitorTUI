use ratatui::{buffer::Buffer, layout::Rect, style::Stylize, symbols::border, text::{Line, Text}, widgets::{Block, Paragraph, Widget}};


#[derive(Debug, Clone)]
pub struct Workers {
    selected: usize,
    names: Vec<String>,
}

impl Widget for Workers {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title("Worker threads".bold())
            .border_set(border::THICK);

        let mut content = vec![];
        for (index, line) in self.names.iter().enumerate() {
            let display_line = if index == self.selected {
                line.clone().yellow().bold()
            } else {
                line.clone().bold()
            };
            content.push(Line::from(display_line));
        }

        let cnt = Text::from(content);

        Paragraph::new(cnt).centered().block(block).render(area, buf);
    }
}

impl Workers {
    pub fn up(&mut self) {
        if !self.names.is_empty() {
            self.selected = (self.selected + self.names.len() - 1) % self.names.len();
        }
    }

    pub fn down(&mut self) {
        if !self.names.is_empty() {
            self.selected = (self.selected + self.names.len() + 1) % self.names.len();
        }
    }
}

impl Default for Workers {
    fn default() -> Self {
        Self {
            selected: 0,
            names: vec![
                " Move Up ".to_string(),
                "<Up>".to_string(),
                " Move Down ".to_string(),
                "<Down>".to_string(),
            ],
        }
    }
}
