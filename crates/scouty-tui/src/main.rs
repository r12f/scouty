//! scouty-tui — Terminal UI for scouty log viewer.

mod app;
mod ui;

use app::App;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use std::io::stdout;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: scouty-tui <logfile>");
        std::process::exit(1);
    }

    let mut app = App::load_file(&args[1])?;

    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Main loop
    loop {
        terminal.draw(|frame| {
            app.visible_rows = frame.area().height.saturating_sub(2) as usize;
            ui::render(frame, &app);
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Down | KeyCode::Char('j') => app.scroll_down(1),
                KeyCode::Up | KeyCode::Char('k') => app.scroll_up(1),
                KeyCode::PageDown => app.page_down(),
                KeyCode::PageUp => app.page_up(),
                KeyCode::Home | KeyCode::Char('g') => app.scroll_to_top(),
                KeyCode::End | KeyCode::Char('G') => app.scroll_to_bottom(),
                _ => {}
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
