use crossterm::event::{KeyCode, KeyEvent};
use std::{path::PathBuf, rc::Rc};
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    widgets::{Block, Borders, Clear, Row, Table},
    Frame,
};

use crate::{action::Action, config::Config};

fn index_to_alphabet(i: usize) -> char {
    let alphabet = "abcdefghijklmnopqrstuvwxyz".chars().collect::<Vec<_>>();
    alphabet[i]
}
fn alphabet_to_index(c: char) -> Option<usize> {
    let alphabet = "abcdefghijklmnopqrstuvwxyz".chars().collect::<Vec<_>>();
    alphabet.iter().position(|&letter| letter == c)
}

pub struct Bookmarks {
    config: Rc<Config>,
}

impl Bookmarks {
    pub fn new(config: Rc<Config>) -> Self {
        Self { config }
    }

    pub fn on_event(&mut self, key: &KeyEvent) -> Option<Action> {
        match self.config.bookmarks() {
            Some(bookmarks) => match key.code {
                KeyCode::Char(c) => match alphabet_to_index(c) {
                    Some(i) if i < bookmarks.len() => {
                        let path = PathBuf::from(&bookmarks[i]);
                        Some(Action::CloseBookmarks(Some(path)))
                    }
                    _ => Some(Action::CloseBookmarks(None)),
                },
                _ => Some(Action::CloseBookmarks(None)),
            },
            _ => Some(Action::CloseBookmarks(None)),
        }
    }

    pub fn on_draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        if let Some(bookmarks) = self.config.bookmarks() {
            let list = bookmarks
                .iter()
                .enumerate()
                .map(|(i, path)| {
                    let letter = index_to_alphabet(i);
                    Row::new(vec![letter.to_string(), path.clone()])
                })
                .collect::<Vec<_>>();
            let widths = {
                let letter_width = 2 as u16;
                let path_width = area.width - letter_width - 3/* for borders */;
                [
                    Constraint::Length(letter_width),
                    Constraint::Length(path_width),
                ]
            };
            let table = Table::new(list).widths(&widths).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Bookmarks".to_string()),
            );
            f.render_widget(Clear, area);
            f.render_widget(table, area);
        }
    }
}
