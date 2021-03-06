use chrono::{DateTime, Local};
use crossterm::event::{KeyCode, KeyEvent};
use fs_extra::dir;
use std::{
    cmp::{min, Ordering},
    ffi::OsString,
    fs::{self, read_dir, DirEntry, Metadata},
    io,
    path::{Path, PathBuf},
    rc::Rc,
    sync::mpsc::Sender,
    thread,
};
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Row, Table, TableState},
    Frame,
};

use crate::{action::Action, config::Config};

fn get_file_name(name: &OsString, meta: &Metadata) -> String {
    let name = name.to_string_lossy().to_string();
    if meta.is_dir() {
        format!("{}/", name)
    } else {
        name
    }
}

fn get_modified(meta: Option<Metadata>) -> String {
    meta.map(|meta| meta.modified())
        .map(|date| match date {
            Ok(date) => {
                let localtime: DateTime<Local> = date.into();
                Some(localtime.format("%Y-%m-%d %T").to_string())
            }
            _ => None,
        })
        .unwrap_or(None)
        .unwrap_or("-------- --:--:--".to_string())
}

struct Entry {
    raw: DirEntry,
    mark: bool,
}

impl Entry {
    fn new(entry: DirEntry) -> Self {
        Self {
            raw: entry,
            mark: false,
        }
    }

    fn is_dir(&self) -> bool {
        match self.raw.file_type() {
            Ok(file_type) => file_type.is_dir(),
            _ => false,
        }
    }
}

fn get_entries(path: &Path) -> io::Result<Vec<Entry>> {
    let mut entries = read_dir(path)?
        .filter_map(|entry| match entry {
            Ok(entry) => Some(Entry::new(entry)),
            _ => None,
        })
        .collect::<Vec<_>>();
    entries.sort_by(default_sort);
    Ok(entries)
}

fn default_sort(a: &Entry, b: &Entry) -> Ordering {
    let a_is_dir = match a.raw.file_type() {
        Ok(file_type) => file_type.is_dir(),
        _ => false,
    };
    let b_is_dir = match b.raw.file_type() {
        Ok(file_type) => file_type.is_dir(),
        _ => false,
    };
    if a_is_dir == b_is_dir {
        a.raw.path().cmp(&b.raw.path())
    } else if a_is_dir {
        Ordering::Less
    } else {
        Ordering::Greater
    }
}

pub struct Dir {
    config: Rc<Config>,
    path: PathBuf,
    entries: Vec<Entry>,
    state: TableState,
}

impl Dir {
    pub fn new(config: Rc<Config>, path: &Path) -> io::Result<Self> {
        let entries = get_entries(path)?;
        let mut state = TableState::default();
        state.select(Some(0));
        Ok(Self {
            config,
            path: path.into(),
            entries,
            state,
        })
    }
    pub fn new_with_index(config: Rc<Config>, path: &Path, index_path: &Path) -> io::Result<Self> {
        let entries = get_entries(path)?;
        let mut state = TableState::default();
        let index = entries
            .iter()
            .position(|entry| entry.raw.path() == index_path)
            .map(|i| i + 1)
            .unwrap_or(0);
        state.select(Some(index));
        Ok(Self {
            config,
            path: path.into(),
            entries,
            state,
        })
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn refresh(&mut self) {
        let entries = get_entries(self.path.as_path()).unwrap_or_default();
        let index = self.state.selected().unwrap_or_default();
        let mut state = TableState::default();
        state.select(Some(min(index, entries.len())));
        self.entries = entries;
        self.state = state;
    }

    pub fn on_event(&self, key: &KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Char('j') => Some(Action::CursorDown),
            KeyCode::Char('k') => Some(Action::CursorUp),
            KeyCode::Char('h') => Some(Action::ChangeDirToParent(self.path.clone())),
            KeyCode::Char('l') => self.on_change_dir(),
            KeyCode::Char('g') => Some(Action::CursorToFirst),
            KeyCode::Char('G') => Some(Action::CursorToLast),
            KeyCode::Char(' ') => Some(Action::ToggleMark),
            KeyCode::Enter => self.on_enter(),
            KeyCode::Char('e') => self.on_edit(),
            KeyCode::Char('r') => self.on_rename(),
            _ => None,
        }
    }

    fn on_change_dir(&self) -> Option<Action> {
        match self.state.selected() {
            Some(0) => None,
            Some(index) => {
                let entry = &self.entries[index - 1];
                if entry.is_dir() {
                    Some(Action::ChangeDir(entry.raw.path()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    fn on_enter(&self) -> Option<Action> {
        match self.state.selected() {
            Some(0) => Some(Action::ChangeDirToParent(self.path.clone())),
            Some(index) => {
                let entry = &self.entries[index - 1];
                if entry.is_dir() {
                    Some(Action::ChangeDir(entry.raw.path()))
                } else {
                    Some(Action::Execute(entry.raw.path()))
                }
            }
            _ => None,
        }
    }
    fn on_edit(&self) -> Option<Action> {
        match self.state.selected() {
            Some(0) => Some(Action::Edit(self.path().clone())),
            Some(index) => {
                let entry = &self.entries[index - 1];
                let path = entry.raw.path();
                Some(Action::Edit(path))
            }
            _ => None,
        }
    }
    fn on_rename(&self) -> Option<Action> {
        match self.state.selected() {
            Some(0) => None,
            Some(index) => {
                let entry = &self.entries[index - 1];
                let name = entry.raw.file_name();
                let name = name.to_string_lossy().to_string();
                Some(Action::StartRename(name))
            }
            _ => None,
        }
    }

    pub fn on_dispatch(&mut self, action: &Action) {
        match action {
            Action::CursorDown => self.cursor_down(),
            Action::CursorUp => self.cursor_up(),
            Action::CursorToFirst => self.cursor_to_first(),
            Action::CursorToLast => self.cursor_to_last(),
            Action::ToggleMark => self.toggle_mark(),
            Action::Execute(path) => self.config.exec(path.as_path(), self.path.as_path()),
            Action::Edit(path) => self.config.edit(path.as_path(), self.path.as_path()),
            _ => {}
        }
    }

    fn cursor_down(&mut self) {
        if let Some(index) = self.state.selected() {
            let index = min(index + 1, self.entries.len());
            self.state.select(Some(index));
        }
    }
    fn cursor_up(&mut self) {
        if let Some(index) = self.state.selected() {
            let index = index.saturating_sub(1);
            self.state.select(Some(index));
        }
    }
    fn cursor_to_first(&mut self) {
        if self.state.selected().is_some() {
            self.state.select(Some(0));
        }
    }
    fn cursor_to_last(&mut self) {
        if self.state.selected().is_some() {
            self.state.select(Some(self.entries.len()));
        }
    }
    fn toggle_mark(&mut self) {
        match self.state.selected() {
            Some(0) => self.cursor_down(),
            Some(index) => {
                let entry = &mut self.entries[index - 1];
                entry.mark = !entry.mark;
                self.cursor_down();
            }
            _ => {}
        }
    }

    pub fn search(&mut self, pattern: &String) {
        let index = self
            .entries
            .iter()
            .position(|entry| {
                let name = entry.raw.file_name().to_string_lossy().to_lowercase();
                let pattern = pattern.to_lowercase();
                name.starts_with(&pattern)
            })
            .map(|i| i + 1);
        if let Some(index) = index {
            self.state.select(Some(index));
        }
    }
    pub fn copy_marks(&mut self, tx: &Sender<String>, dest_dir: &Path) {
        for entry in self.entries.iter_mut() {
            if entry.mark {
                entry.mark = false;
                let tx = tx.clone();
                let src = entry.raw.path();
                if entry.is_dir() {
                    let dest = PathBuf::from(dest_dir);
                    thread::spawn(move || {
                        let opt = dir::CopyOptions::default();
                        match dir::copy(src, dest, &opt) {
                            Err(e) => {
                                let _ = tx.send(format!("Err: {}", e.to_string()));
                            }
                            _ => {
                                let _ = tx.send(String::new());
                            }
                        }
                    });
                } else {
                    let mut dest = PathBuf::from(dest_dir);
                    dest.push(entry.raw.file_name());
                    thread::spawn(move || match fs::copy(src, dest) {
                        Err(e) => {
                            let _ = tx.send(format!("Err: {}", e.to_string()));
                        }
                        _ => {
                            let _ = tx.send(String::new());
                        }
                    });
                }
            }
        }
    }
    pub fn move_marks(&mut self, tx: &Sender<String>, dest_dir: &Path) {
        for entry in self.entries.iter_mut() {
            if entry.mark {
                entry.mark = false;
                let tx = tx.clone();
                let src = entry.raw.path();
                let mut dest = PathBuf::from(dest_dir);
                dest.push(entry.raw.file_name());
                thread::spawn(move || match fs::rename(src, dest) {
                    Err(e) => {
                        let _ = tx.send(format!("Err: {}", e.to_string()));
                    }
                    _ => {
                        let _ = tx.send(String::new());
                    }
                });
            }
        }
    }
    pub fn delete_marks(&mut self, tx: &Sender<String>) {
        for entry in self.entries.iter_mut() {
            if entry.mark {
                entry.mark = false;
                let tx = tx.clone();
                let src = entry.raw.path();
                if entry.is_dir() {
                    thread::spawn(move || match fs::remove_dir_all(src) {
                        Err(e) => {
                            let _ = tx.send(format!("{}", e.to_string()));
                        }
                        _ => {
                            let _ = tx.send(String::new());
                        }
                    });
                } else {
                    thread::spawn(move || match fs::remove_file(src) {
                        Err(e) => {
                            let _ = tx.send(format!("{}", e.to_string()));
                        }
                        _ => {
                            let _ = tx.send(String::new());
                        }
                    });
                }
            }
        }
    }
    pub fn create_dir(&mut self, name: &String) {
        let mut path = self.path.clone();
        path.push(name);
        match fs::create_dir(path) {
            Err(e) => eprintln!("{}", e.to_string()),
            _ => {}
        }
    }
    pub fn rename(&mut self, name: &String) {
        match self.state.selected() {
            Some(0) => {}
            Some(index) => {
                let mut path = self.path().clone();
                path.push(name);
                let entry = &mut self.entries[index - 1];
                match fs::rename(entry.raw.path(), path) {
                    Err(e) => eprintln!("{}", e.to_string()),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn on_draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, is_src: bool) {
        let modified = get_modified(fs::metadata(self.path.as_path()).ok());
        let date_width = modified.len() as u16;

        let mut list = vec![Row::new(vec!["..".to_string(), modified])];
        list.extend(self.entries.iter().filter_map(|entry| {
            if let Ok(meta) = entry.raw.metadata() {
                let name = get_file_name(&entry.raw.file_name(), &meta);
                let date = get_modified(Some(meta));
                let row = Row::new(vec![name, date]);
                Some(if entry.mark {
                    row.style(Style::default().add_modifier(Modifier::REVERSED))
                } else {
                    row
                })
            } else {
                None
            }
        }));

        let widths = {
            let name_width = area.width - date_width - 3/* for borders */;
            [
                Constraint::Length(name_width),
                Constraint::Length(date_width),
            ]
        };
        let table = Table::new(list).widths(&widths).block(
            Block::default()
                .borders(Borders::ALL)
                .title(self.path.to_string_lossy().to_string()),
        );
        let table = if is_src {
            table.highlight_style(Style::default().add_modifier(Modifier::UNDERLINED))
        } else {
            table
        };
        f.render_stateful_widget(table, area, &mut self.state);
    }
}
