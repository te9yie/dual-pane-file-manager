use chrono::{DateTime, Local};
use crossterm::event::{KeyCode, KeyEvent};
use std::{
    cmp::{min, Ordering},
    ffi::OsString,
    fs::{self, read_dir, DirEntry, Metadata},
    io,
    path::{Path, PathBuf},
};
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Row, Table, TableState},
    Frame,
};

use crate::action::Action;

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
}

impl Entry {
    fn new(entry: DirEntry) -> Self {
        Self { raw: entry }
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
    path: PathBuf,
    entries: Vec<Entry>,
    state: TableState,
}

impl Dir {
    pub fn new(path: &Path) -> io::Result<Self> {
        let entries = get_entries(path)?;
        let mut state = TableState::default();
        state.select(Some(0));
        Ok(Self {
            path: path.into(),
            entries,
            state,
        })
    }
    pub fn new_with_index(path: &Path, index_path: &Path) -> io::Result<Self> {
        let entries = get_entries(path)?;
        let mut state = TableState::default();
        let index = entries
            .iter()
            .position(|entry| entry.raw.path() == index_path)
            .map(|i| i + 1)
            .unwrap_or(0);
        state.select(Some(index));
        Ok(Self {
            path: path.into(),
            entries,
            state,
        })
    }

    pub fn on_event(&self, key: &KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Char('j') => Some(Action::CursorDown),
            KeyCode::Char('k') => Some(Action::CursorUp),
            KeyCode::Char('h') => Some(Action::ChangeDirToParent(self.path.clone())),
            KeyCode::Char('l') => self.on_change_dir(),
            KeyCode::Char('g') => Some(Action::CursorToFirst),
            KeyCode::Char('G') => Some(Action::CursorToLast),
            KeyCode::Enter => self.on_enter(),
            _ => None,
        }
    }

    fn on_change_dir(&self) -> Option<Action> {
        match self.state.selected() {
            Some(0) => None,
            Some(index) => {
                let entry = &self.entries[index - 1];
                if entry.is_dir() {
                    Some(Action::ChangeDir(entry.raw.path().clone()))
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
                    Some(Action::ChangeDir(entry.raw.path().clone()))
                } else {
                    None
                }
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

    pub fn on_draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, is_src: bool) {
        let modified = get_modified(fs::metadata(self.path.as_path()).ok());
        let date_width = modified.len() as u16;

        let mut list = vec![Row::new(vec!["..".to_string(), modified])];
        list.extend(self.entries.iter().filter_map(|entry| {
            if let Ok(meta) = entry.raw.metadata() {
                let name = get_file_name(&entry.raw.file_name(), &meta);
                let date = get_modified(Some(meta));
                Some(Row::new(vec![name, date]))
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
