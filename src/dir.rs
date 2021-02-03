use chrono::{DateTime, Local};
use std::{
    cmp::Ordering,
    ffi::OsString,
    fs::{self, read_dir, DirEntry, Metadata},
    io,
    path::{Path, PathBuf},
};
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Row, Table},
    Frame,
};

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

fn get_entries(path: &Path) -> io::Result<Vec<Entry>> {
    let mut entries = read_dir(path)?
        .filter_map(|entry| match entry {
            Ok(entry) => Some(Entry { raw: entry }),
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
}

impl Dir {
    pub fn new(path: &Path) -> io::Result<Self> {
        let entries = get_entries(path)?;
        Ok(Self {
            path: path.into(),
            entries,
        })
    }

    pub fn on_draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let modified = get_modified(fs::metadata(self.path.as_path()).ok());
        let date_len = modified.len() as u16;

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
            let name_len = area.width - date_len - 3/* for borders */;
            [Constraint::Length(name_len), Constraint::Length(date_len)]
        };
        let table = Table::new(list)
            .widths(&widths)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.path.to_string_lossy().to_string()),
            )
            .highlight_style(Style::default().add_modifier(Modifier::UNDERLINED));
        f.render_widget(table, area);
    }
}
