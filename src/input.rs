use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    layout::Rect,
    text::{Span, Spans},
    widgets::Paragraph,
    Frame,
};

use crate::action::Action;

pub struct InputBox {
    prefix: String,
    value: String,
}

impl InputBox {
    pub fn new(prefix: String) -> Self {
        Self {
            prefix,
            value: String::new(),
        }
    }

    pub fn on_event(&mut self, key: &KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Char(c) => {
                self.value.push(c);
                Some(Action::Refresh)
            }
            KeyCode::Backspace => {
                self.value.pop();
                Some(Action::Refresh)
            }
            KeyCode::Enter => Some(Action::EndInputText(Some(self.value.clone()))),
            _ => Some(Action::EndInputText(None)),
        }
    }

    pub fn on_draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let text = vec![Spans::from(vec![
            Span::raw(self.prefix.clone()),
            Span::raw(self.value.clone()),
        ])];
        let paragraph = Paragraph::new(text);
        f.render_widget(paragraph, area);
    }
}
