use action::Action;
use crossterm::{
    event::{read, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{stdout, Stdout};
use tui::{backend::CrosstermBackend, Terminal};

mod action;
mod app;

struct Main {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Main {
    fn new() -> crossterm::Result<Self> {
        let stdout = stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        execute!(terminal.backend_mut(), EnterAlternateScreen)?;
        Ok(Self { terminal })
    }
}

impl Drop for Main {
    fn drop(&mut self) {
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen).unwrap();
    }
}

fn main() {
    let mut main = Main::new().unwrap();
    let mut app = app::App::new();

    main.terminal
        .draw(|f| {
            app.on_draw(f, f.size());
        })
        .unwrap();

    while let Ok(event) = read() {
        match event {
            Event::Key(key) => {
                if let Some(action) = app.on_event(&key) {
                    app.on_dispatch(&action);
                    match action {
                        Action::Quit => break,
                        //_ => continue,
                    }
                }
            }
            _ => {}
        }

        main.terminal
            .draw(|f| {
                app.on_draw(f, f.size());
            })
            .unwrap();
    }
}
