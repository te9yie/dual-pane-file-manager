use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    layout::Rect,
    widgets::{Block, Borders},
    Frame,
};

use crate::action::Action;

pub struct App;

impl App {
    pub fn new() -> Self {
        Self {}
    }

    pub fn on_event(&self, key: &KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Char('q') => Some(Action::Quit),
            _ => None,
        }
    }

    pub fn on_dispatch(&mut self, _action: &Action) {}

    pub fn on_draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let block = Block::default()
            .title("dual-pane-file-manager")
            .borders(Borders::ALL);
        f.render_widget(block, area);
    }
}
