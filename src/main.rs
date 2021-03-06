use action::Action;
use config::Config;
use crossterm::{
    cursor::{Hide, Show},
    event::{poll, read, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    env::current_dir,
    io::{stdout, Stdout},
    rc::Rc,
    sync::mpsc::channel,
    time::Duration,
};
use tui::{backend::CrosstermBackend, Terminal};

mod action;
mod app;
mod bookmark;
mod config;
mod dir;
mod input;
mod search;

struct Main {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Main {
    fn new() -> crossterm::Result<Self> {
        let stdout = stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        execute!(terminal.backend_mut(), EnterAlternateScreen, Hide)?;
        enable_raw_mode()?;
        Ok(Self { terminal })
    }
}

impl Drop for Main {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), Show, LeaveAlternateScreen);
    }
}

fn main() {
    let config = Rc::new(Config::default().unwrap());
    let path = current_dir().unwrap();
    let mut main = Main::new().unwrap();
    let (tx, rx) = channel::<String>();
    let mut app = app::App::new(config, tx, path.as_path()).unwrap();

    main.terminal
        .draw(|f| {
            app.on_draw(f, f.size());
        })
        .unwrap();

    loop {
        let action: Option<Action> = if let Ok(message) = rx.try_recv() {
            app.push_message(message);
            Some(Action::Refresh)
        } else if poll(Duration::from_millis(100)).unwrap_or(false) {
            match read() {
                Ok(event) => match event {
                    Event::Key(key) => app.on_event(&key),
                    _ => None,
                },
                _ => continue,
            }
        } else {
            continue;
        };

        if let Some(action) = action {
            app.on_dispatch(&action);
            match action {
                Action::Quit => break,
                _ => {}
            }
        }

        main.terminal
            .draw(|f| {
                app.on_draw(f, f.size());
            })
            .unwrap();
    }
}
