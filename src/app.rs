use std::{io, path::Path};

use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::{action::Action, dir::Dir};

pub struct App {
    dirs: [Dir; 2],
}

impl App {
    pub fn new(path: &Path) -> io::Result<Self> {
        let dirs = [Dir::new(path)?, Dir::new(path)?];
        Ok(Self { dirs })
    }

    pub fn on_event(&self, key: &KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Char('q') => Some(Action::Quit),
            _ => None,
        }
    }

    pub fn on_dispatch(&mut self, _action: &Action) {}

    pub fn on_draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        for (i, chunk) in chunks.iter().enumerate() {
            self.dirs[i].on_draw(f, *chunk);
        }
    }
}
