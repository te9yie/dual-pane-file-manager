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
    src_index: usize,
}

impl App {
    pub fn new(path: &Path) -> io::Result<Self> {
        let dirs = [Dir::new(path)?, Dir::new(path)?];
        let src_index = 0usize;
        Ok(Self { dirs, src_index })
    }

    pub fn on_event(&self, key: &KeyEvent) -> Option<Action> {
        let action = self.src_dir().on_event(key);
        if action.is_none() {
            match key.code {
                KeyCode::Char('q') => Some(Action::Quit),
                KeyCode::Tab => Some(Action::SwitchSrc),
                _ => None,
            }
        } else {
            action
        }
    }

    pub fn on_dispatch(&mut self, action: &Action) {
        self.src_dir_mut().on_dispatch(action);
        match action {
            Action::SwitchSrc => self.src_index = 1 - self.src_index,
            _ => {}
        }
    }

    pub fn on_draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        for (i, chunk) in chunks.iter().enumerate() {
            let is_src = i == self.src_index;
            self.dirs[i].on_draw(f, *chunk, is_src);
        }
    }

    fn src_dir(&self) -> &Dir {
        &self.dirs[self.src_index]
    }
    fn src_dir_mut(&mut self) -> &mut Dir {
        &mut self.dirs[self.src_index]
    }
}
