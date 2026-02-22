//! scouty-tui — Terminal UI for scouty log viewer.

mod app;
mod density;
mod ui;

use app::{App, InputMode};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use std::io::stdout;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: scouty-tui <logfile> [logfile2 ...]");
        std::process::exit(1);
    }

    let files: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();

    // Enter TUI mode first so the user sees a loading screen immediately
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = match Terminal::new(CrosstermBackend::new(stdout())) {
        Ok(t) => t,
        Err(e) => {
            let _ = disable_raw_mode();
            let _ = stdout().execute(LeaveAlternateScreen);
            return Err(e.into());
        }
    };

    // Show loading screen
    if let Err(e) = terminal.draw(|frame| {
        let area = frame.area();
        let msg = if files.len() == 1 {
            format!("Loading {}...", files[0])
        } else {
            format!("Loading {} files...", files.len())
        };
        let text = ratatui::widgets::Paragraph::new(msg);
        let y = area.y + area.height / 2;
        let centered = ratatui::layout::Rect::new(area.x, y, area.width, 1);
        frame.render_widget(text, centered);
    }) {
        let _ = disable_raw_mode();
        let _ = stdout().execute(LeaveAlternateScreen);
        return Err(e.into());
    }

    // Load files (may take several seconds for large files)
    let mut app = match App::load_files(&files) {
        Ok(app) => app,
        Err(e) => {
            let _ = disable_raw_mode();
            let _ = stdout().execute(LeaveAlternateScreen);
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    loop {
        terminal.draw(|frame| {
            let body_area = frame.area();
            let table_height = if app.detail_open {
                let body = body_area.height.saturating_sub(1);
                (body * 60 / 100).saturating_sub(1) as usize
            } else {
                body_area.height.saturating_sub(2) as usize
            };
            app.visible_rows = table_height.max(1);
            ui::render(frame, &app);
        })?;

        if !event::poll(Duration::from_millis(250))? {
            continue;
        }

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            app.status_message = None;
            let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

            match app.input_mode {
                InputMode::Normal => {
                    if ctrl {
                        match key.code {
                            KeyCode::Char('j') | KeyCode::Down => app.page_down(),
                            KeyCode::Char('k') | KeyCode::Up => app.page_up(),
                            KeyCode::Char('g') => {
                                app.input_mode = InputMode::GotoLine;
                                app.goto_input.clear();
                            }
                            KeyCode::Char('f') => {
                                app.input_mode = InputMode::FilterManager;
                                app.filter_manager_cursor = 0;
                            }
                            KeyCode::Char(']') => {
                                app.toggle_follow();
                            }
                            // Ctrl+- = exclude field filter
                            // Some terminals send Char('-'), others Char('\x1f') (ASCII 31)
                            KeyCode::Char('-') | KeyCode::Char('\x1f') => {
                                app.open_field_filter(true);
                            }
                            // Ctrl+= = include field filter
                            KeyCode::Char('=') => {
                                app.open_field_filter(false);
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Esc => {
                                if app.detail_open {
                                    app.detail_open = false;
                                }
                            }
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
                            KeyCode::Char('-') => {
                                app.input_mode = InputMode::QuickExclude;
                                app.quick_filter_input.clear();
                            }
                            KeyCode::Char('=') => {
                                app.input_mode = InputMode::QuickInclude;
                                app.quick_filter_input.clear();
                            }
                            KeyCode::Char('F') => {
                                app.input_mode = InputMode::FilterManager;
                                app.filter_manager_cursor = 0;
                            }
                            KeyCode::Char('n') => app.next_search_match(),
                            KeyCode::Char('N') => app.prev_search_match(),
                            KeyCode::Char('y') => {
                                if let Some(text) = app.copy_raw() {
                                    app::osc52_copy(&text);
                                }
                            }
                            KeyCode::Char('Y') => {
                                app.input_mode = InputMode::CopyFormat;
                            }
                            KeyCode::Char('c') => {
                                app.input_mode = InputMode::ColumnSelector;
                                app.column_config.cursor = 0;
                            }
                            KeyCode::Char('?') => {
                                app.input_mode = InputMode::Help;
                            }
                            _ => {}
                        }
                    }
                }
                InputMode::Filter => match key.code {
                    KeyCode::Enter => {
                        app.apply_filter();
                        if app.filter_error.is_none() {
                            app.input_mode = InputMode::Normal;
                        }
                    }
                    KeyCode::Esc => app.input_mode = InputMode::Normal,
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
                    KeyCode::Esc => app.input_mode = InputMode::Normal,
                    KeyCode::Backspace => {
                        app.search_input.pop();
                    }
                    KeyCode::Char(c) => app.search_input.push(c),
                    _ => {}
                },
                InputMode::TimeJump => match key.code {
                    KeyCode::Enter => {
                        app.jump_to_time();
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Esc => app.input_mode = InputMode::Normal,
                    KeyCode::Backspace => {
                        app.time_input.pop();
                    }
                    KeyCode::Char(c) => app.time_input.push(c),
                    _ => {}
                },
                InputMode::GotoLine => {
                    use ui::windows::goto_line_window::GotoLineWindow;
                    let mut window = GotoLineWindow::new();
                    window.input = app.goto_input.clone();
                    let result = ui::dispatch_key(&mut window, key);
                    app.goto_input = window.input;
                    if result == ui::ComponentResult::Close {
                        if window.confirmed {
                            app.goto_line();
                        }
                        app.input_mode = InputMode::Normal;
                    }
                }
                InputMode::QuickExclude => match key.code {
                    KeyCode::Enter => {
                        app.apply_quick_exclude();
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Esc => app.input_mode = InputMode::Normal,
                    KeyCode::Backspace => {
                        app.quick_filter_input.pop();
                    }
                    KeyCode::Char(c) => app.quick_filter_input.push(c),
                    _ => {}
                },
                InputMode::QuickInclude => match key.code {
                    KeyCode::Enter => {
                        app.apply_quick_include();
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Esc => app.input_mode = InputMode::Normal,
                    KeyCode::Backspace => {
                        app.quick_filter_input.pop();
                    }
                    KeyCode::Char(c) => app.quick_filter_input.push(c),
                    _ => {}
                },
                InputMode::FieldFilter => {
                    use ui::windows::field_filter_window::FieldFilterWindow;
                    if let Some(mut window) = FieldFilterWindow::from_app(&app) {
                        let result = ui::dispatch_key(&mut window, key);
                        match result {
                            ui::ComponentResult::Close => {
                                if window.confirmed {
                                    window.sync_to_app(&mut app);
                                    app.apply_field_filter();
                                } else {
                                    app.field_filter = None;
                                }
                                app.input_mode = InputMode::Normal;
                            }
                            _ => {
                                window.sync_to_app(&mut app);
                            }
                        }
                    } else {
                        app.input_mode = InputMode::Normal;
                    }
                }
                InputMode::FilterManager => {
                    use ui::windows::filter_manager_window::FilterManagerWindow;
                    let mut window = FilterManagerWindow::from_app(&app);
                    let result = ui::dispatch_key(&mut window, key);
                    window.apply_to_app(&mut app);
                    if result == ui::ComponentResult::Close {
                        app.input_mode = InputMode::Normal;
                    }
                }
                InputMode::ColumnSelector => {
                    use ui::windows::column_selector_window::ColumnSelectorWindow;
                    let mut window = ColumnSelectorWindow::from_app(&app);
                    let result = ui::dispatch_key(&mut window, key);
                    window.sync_to_app(&mut app);
                    if result == ui::ComponentResult::Close {
                        app.input_mode = InputMode::Normal;
                    }
                }
                InputMode::CopyFormat => {
                    use ui::windows::copy_format_window::CopyFormatWindow;
                    // Determine format from key before dispatching
                    let format = match key.code {
                        KeyCode::Enter | KeyCode::Char('r') => Some(app::CopyFormat::Raw),
                        KeyCode::Char('j') => Some(app::CopyFormat::Json),
                        KeyCode::Char('y') => Some(app::CopyFormat::Yaml),
                        _ => None,
                    };
                    let mut window = CopyFormatWindow;
                    let result = ui::dispatch_key(&mut window, key);
                    if result == ui::ComponentResult::Close {
                        if let Some(fmt) = format {
                            CopyFormatWindow::select_format(&mut app, fmt);
                        }
                        app.input_mode = InputMode::Normal;
                    }
                }
                InputMode::Help => {
                    use ui::windows::help_window::HelpWindow;
                    let mut window = HelpWindow;
                    let result = ui::dispatch_key(&mut window, key);
                    if result == ui::ComponentResult::Close {
                        app.input_mode = InputMode::Normal;
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
