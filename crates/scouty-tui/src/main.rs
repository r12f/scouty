//! scouty-tui — Terminal UI for scouty log viewer.

mod app;
mod ui;

use app::{App, InputMode};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use std::io::stdout;
use std::time::Duration;

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
            let log_area_height = if app.detail_open {
                // 60% of body area (body = total - header - footer)
                let body = frame.area().height.saturating_sub(3);
                (body * 60 / 100) as usize
            } else {
                frame.area().height.saturating_sub(2) as usize
            };
            app.visible_rows = log_area_height;
            ui::render(frame, &app);
        })?;

        // Poll with timeout for potential live refresh support
        if !event::poll(Duration::from_millis(250))? {
            continue;
        }

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            // Clear status message on any key
            app.status_message = None;

            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Down | KeyCode::Char('j') => app.select_down(1),
                    KeyCode::Up | KeyCode::Char('k') => app.select_up(1),
                    KeyCode::PageDown => app.page_down(),
                    KeyCode::PageUp => app.page_up(),
                    KeyCode::Home | KeyCode::Char('g') => app.scroll_to_top(),
                    KeyCode::End | KeyCode::Char('G') => app.scroll_to_bottom(),
                    KeyCode::Enter => app.toggle_detail(),
                    KeyCode::Char('f') => {
                        app.input_mode = InputMode::Filter;
                    }
                    KeyCode::Char('/') => {
                        app.input_mode = InputMode::Search;
                    }
                    KeyCode::Char('t') => {
                        app.input_mode = InputMode::TimeJump;
                        app.time_input.clear();
                    }
                    KeyCode::Char('n') => app.next_search_match(),
                    KeyCode::Char('N') => app.prev_search_match(),
                    KeyCode::Char('?') | KeyCode::Char('h') => {
                        app.input_mode = InputMode::Help;
                    }
                    _ => {}
                },
                InputMode::Filter => match key.code {
                    KeyCode::Enter => {
                        app.apply_filter();
                        if app.filter_error.is_none() {
                            app.input_mode = InputMode::Normal;
                        }
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Backspace => {
                        app.filter_input.pop();
                        app.filter_error = None;
                    }
                    KeyCode::Char(c) => {
                        app.filter_input.push(c);
                        app.filter_error = None;
                    }
                    _ => {}
                },
                InputMode::Search => match key.code {
                    KeyCode::Enter => {
                        app.execute_search();
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Backspace => {
                        app.search_input.pop();
                    }
                    KeyCode::Char(c) => {
                        app.search_input.push(c);
                    }
                    _ => {}
                },
                InputMode::TimeJump => match key.code {
                    KeyCode::Enter => {
                        app.jump_to_time();
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Backspace => {
                        app.time_input.pop();
                    }
                    KeyCode::Char(c) => {
                        app.time_input.push(c);
                    }
                    _ => {}
                },
                InputMode::Help => {
                    // Any key closes help
                    app.input_mode = InputMode::Normal;
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
