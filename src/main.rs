use crossterm::event::{read, Event, KeyCode};
use std::io::stdout;
use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders},
    Terminal,
};

fn main() {
    let stdout = stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.clear().unwrap();
    terminal
        .draw(|f| {
            let area = f.size();
            let block = Block::default()
                .title("dual-pane-file-manager")
                .borders(Borders::ALL);
            f.render_widget(block, area);
        })
        .unwrap();

    while let Ok(event) = read() {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => break,
                _ => {}
            },
            _ => {}
        }
    }
}
