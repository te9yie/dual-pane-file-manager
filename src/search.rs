use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    layout::Rect,
    text::{Span, Spans},
    widgets::{Clear, Paragraph},
    Frame,
};

use crate::action::Action;

pub struct SearchLine {
    pattern: String,
}

impl SearchLine {
    pub fn new() -> Self {
        Self {
            pattern: String::new(),
        }
    }

    pub fn on_event(&mut self, key: &KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Char(c) => {
                self.pattern.push(c);
                Some(Action::Search(self.pattern.clone()))
            }
            KeyCode::Backspace => {
                self.pattern.pop();
                Some(Action::Search(self.pattern.clone()))
            }
            _ => Some(Action::EndSearch),
        }
    }

    pub fn on_draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let text = vec![Spans::from(vec![
            Span::raw("/"),
            Span::raw(self.pattern.clone()),
        ])];
        let paragraph = Paragraph::new(text);
        f.render_widget(Clear, area);
        f.render_widget(paragraph, area);
    }
}
