use crossterm::event::{KeyCode, KeyEvent};
use std::{
    io,
    path::{Path, PathBuf},
    rc::Rc,
    sync::mpsc::Sender,
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Span, Spans},
    widgets::Paragraph,
    Frame,
};

use crate::{
    action::Action, bookmark::Bookmarks, config::Config, dir::Dir, input::InputBox,
    search::SearchLine,
};

enum InputMode {
    CreateDir(InputBox),
    Rename(InputBox),
}

pub struct App {
    config: Rc<Config>,
    tx: Sender<String>,
    dirs: [Dir; 2],
    src_index: usize,
    search_line: Option<SearchLine>,
    input_mode: Option<InputMode>,
    bookmarks: Option<Bookmarks>,
    message: String,
}

impl App {
    pub fn new(config: Rc<Config>, tx: Sender<String>, path: &Path) -> io::Result<Self> {
        let dirs = [
            Dir::new(Rc::clone(&config), path)?,
            Dir::new(Rc::clone(&config), path)?,
        ];
        let src_index = 0usize;
        Ok(Self {
            config,
            tx,
            dirs,
            src_index,
            search_line: None,
            input_mode: None,
            bookmarks: None,
            message: String::from("Welcome."),
        })
    }

    pub fn on_event(&mut self, key: &KeyEvent) -> Option<Action> {
        if let Some(ref mut input_mode) = self.input_mode {
            match input_mode {
                InputMode::CreateDir(input) => input.on_event(key),
                InputMode::Rename(input) => input.on_event(key),
            }
        } else if let Some(ref mut search_line) = self.search_line {
            search_line.on_event(key)
        } else if let Some(ref mut bookmarks) = self.bookmarks {
            bookmarks.on_event(key)
        } else {
            let action = self.src_dir().on_event(key);
            if action.is_none() {
                match key.code {
                    KeyCode::Char('q') => Some(Action::Quit),
                    KeyCode::Tab => Some(Action::SwitchSrc),
                    KeyCode::Char('o') => Some(Action::DuplicateDir),
                    KeyCode::Char('/') => Some(Action::StartSearch),
                    KeyCode::Char('c') => Some(Action::Copy),
                    KeyCode::Char('m') => Some(Action::Move),
                    KeyCode::Char('d') => Some(Action::Delete),
                    KeyCode::Char('i') => Some(Action::StartCreateDir),
                    KeyCode::Char('b') => Some(Action::OpenBookmarks),
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
            Action::Refresh => {
                for dir in self.dirs.iter_mut() {
                    dir.refresh();
                }
            }
            Action::SwitchSrc => self.src_index = 1 - self.src_index,
            Action::DuplicateDir => self.duplicate_dir(),
            Action::ChangeDir(path) => self.change_dir(path.as_path()),
            Action::ChangeDirToParent(path) => self.change_dir_to_parent(path.as_path()),
            Action::StartSearch => self.search_line = Some(SearchLine::new()),
            Action::EndSearch => self.search_line = None,
            Action::Search(pattern) => self.src_dir_mut().search(pattern),
            Action::Copy => self.copy_marks(),
            Action::Move => self.move_marks(),
            Action::Delete => self.delete_marks(),
            Action::StartCreateDir => {
                let mode = InputMode::CreateDir(InputBox::new("Dir: ".to_string()));
                self.input_mode = Some(mode);
            }
            Action::StartRename(name) => {
                let mode = InputMode::Rename(InputBox::new_with_default(
                    "Rename: ".to_string(),
                    name.to_owned(),
                ));
                self.input_mode = Some(mode);
            }
            Action::EndInputText(value) => {
                if let Some(value) = value {
                    match self.input_mode {
                        Some(InputMode::CreateDir(_)) => self.create_dir(value),
                        Some(InputMode::Rename(_)) => self.rename(value),
                        _ => {}
                    }
                }
                self.input_mode = None;
            }
            Action::OpenBookmarks => self.open_bookmarks(),
            Action::CloseBookmarks(path) => self.close_bookmarks(path),
            _ => {}
        }
    }
    pub fn push_message(&mut self, message: String) {
        self.message = message;
    }

    fn duplicate_dir(&mut self) {
        let path = self.dest_dir().path();
        if let Ok(dir) = Dir::new(Rc::clone(&self.config), path.as_path()) {
            self.dirs[self.src_index] = dir;
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
        let tx = self.tx.clone();
        self.src_dir_mut().copy_marks(&tx, path.as_path());
        self.src_dir_mut().refresh();
    }
    fn move_marks(&mut self) {
        let path = self.dest_dir().path();
        let tx = self.tx.clone();
        self.src_dir_mut().move_marks(&tx, path.as_path());
        self.src_dir_mut().refresh();
        self.dest_dir_mut().refresh();
    }
    fn delete_marks(&mut self) {
        let tx = self.tx.clone();
        self.src_dir_mut().delete_marks(&tx);
        self.src_dir_mut().refresh();
    }
    fn create_dir(&mut self, name: &String) {
        if !name.is_empty() {
            self.src_dir_mut().create_dir(name);
            self.src_dir_mut().refresh();
        }
    }
    fn rename(&mut self, name: &String) {
        if !name.is_empty() {
            self.src_dir_mut().rename(name);
            self.src_dir_mut().refresh();
        }
    }
    fn open_bookmarks(&mut self) {
        self.bookmarks = Some(Bookmarks::new(Rc::clone(&self.config)));
    }
    fn close_bookmarks(&mut self, path: &Option<PathBuf>) {
        if let Some(path) = path {
            if let Ok(dir) = Dir::new(Rc::clone(&self.config), path.as_path()) {
                self.dirs[self.src_index] = dir;
            }
        }
        self.bookmarks = None;
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
        if !self.message.is_empty() {
            let text = vec![Spans::from(vec![Span::raw(self.message.clone())])];
            let paragraph = Paragraph::new(text);
            f.render_widget(paragraph, v_chunks[1]);
        }
        for (i, chunk) in chunks.iter().enumerate() {
            let is_src = i == self.src_index;
            self.dirs[i].on_draw(f, *chunk, is_src);
        }
        if let Some(ref mut line) = self.search_line {
            line.on_draw(f, v_chunks[1]);
        }
        if let Some(ref mut input_mode) = self.input_mode {
            match input_mode {
                InputMode::CreateDir(input) => input.on_draw(f, v_chunks[1]),
                InputMode::Rename(input) => input.on_draw(f, v_chunks[1]),
            }
        }
        if let Some(ref mut bookmarks) = self.bookmarks {
            bookmarks.on_draw(f, chunks[self.src_index]);
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
