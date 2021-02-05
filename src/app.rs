use std::{io, path::Path, rc::Rc};

use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::{action::Action, config::Config, dir::Dir, search::SearchLine};

pub struct App {
    config: Rc<Config>,
    dirs: [Dir; 2],
    src_index: usize,
    search_line: Option<SearchLine>,
}

impl App {
    pub fn new(config: Rc<Config>, path: &Path) -> io::Result<Self> {
        let dirs = [
            Dir::new(Rc::clone(&config), path)?,
            Dir::new(Rc::clone(&config), path)?,
        ];
        let src_index = 0usize;
        Ok(Self {
            config,
            dirs,
            src_index,
            search_line: None,
        })
    }

    pub fn on_event(&mut self, key: &KeyEvent) -> Option<Action> {
        if let Some(ref mut search_line) = self.search_line {
            search_line.on_event(key)
        } else {
            let action = self.src_dir().on_event(key);
            if action.is_none() {
                match key.code {
                    KeyCode::Char('q') => Some(Action::Quit),
                    KeyCode::Char('/') => Some(Action::StartSearch),
                    KeyCode::Char('c') => Some(Action::Copy),
                    KeyCode::Char('m') => Some(Action::Move),
                    KeyCode::Tab => Some(Action::SwitchSrc),
                    _ => None,
                }
            } else {
                action
            }
        }
    }

    pub fn on_dispatch(&mut self, action: &Action) {
        self.src_dir_mut().on_dispatch(action);
        match action {
            Action::SwitchSrc => self.src_index = 1 - self.src_index,
            Action::ChangeDir(path) => self.change_dir(path.as_path()),
            Action::ChangeDirToParent(path) => self.change_dir_to_parent(path.as_path()),
            Action::StartSearch => self.search_line = Some(SearchLine::new()),
            Action::EndSearch => self.search_line = None,
            Action::Search(pattern) => self.src_dir_mut().search(pattern),
            Action::Copy => self.copy_marks(),
            Action::Move => self.move_marks(),
            _ => {}
        }
    }

    fn change_dir(&mut self, path: &Path) {
        if let Ok(dir) = Dir::new(Rc::clone(&self.config), path) {
            self.dirs[self.src_index] = dir;
        }
    }
    fn change_dir_to_parent(&mut self, path: &Path) {
        if let Some(parent_path) = path.parent() {
            if let Ok(dir) = Dir::new_with_index(Rc::clone(&self.config), parent_path, path) {
                self.dirs[self.src_index] = dir;
            }
        }
    }
    fn copy_marks(&mut self) {
        let path = self.dest_dir().path();
        self.src_dir_mut().copy_marks(path.as_path());
        self.src_dir_mut().refresh();
        self.dest_dir_mut().refresh();
    }
    fn move_marks(&mut self) {
        let path = self.dest_dir().path();
        self.src_dir_mut().move_marks(path.as_path());
        self.src_dir_mut().refresh();
        self.dest_dir_mut().refresh();
    }

    pub fn on_draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let main_height = area.height - 1;
        let v_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(main_height), Constraint::Length(1)])
            .split(area);
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(v_chunks[0]);
        for (i, chunk) in chunks.iter().enumerate() {
            let is_src = i == self.src_index;
            self.dirs[i].on_draw(f, *chunk, is_src);
        }
        if let Some(ref mut line) = self.search_line {
            line.on_draw(f, v_chunks[1]);
        }
    }

    fn src_dir(&self) -> &Dir {
        &self.dirs[self.src_index]
    }
    fn src_dir_mut(&mut self) -> &mut Dir {
        &mut self.dirs[self.src_index]
    }
    fn dest_dir(&self) -> &Dir {
        &self.dirs[1 - self.src_index]
    }
    fn dest_dir_mut(&mut self) -> &mut Dir {
        &mut self.dirs[1 - self.src_index]
    }
}
