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
        eprintln!("Usage: scouty-tui <logfile>");
        std::process::exit(1);
    }

    let mut app = App::load_file(&args[1])?;

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

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
                            // Ctrl++ = include field filter
                            // '+' requires Shift on most keyboards, so also accept Ctrl+=
                            KeyCode::Char('+') | KeyCode::Char('=') => {
                                app.open_field_filter(false);
                            }
                            // Ctrl+c = copy raw
                            KeyCode::Char('c') => {
                                if let Some(text) = app.copy_raw() {
                                    app::osc52_copy(&text);
                                }
                            }
                            // Ctrl+Shift+C = copy format dialog
                            KeyCode::Char('C') => {
                                app.input_mode = InputMode::CopyFormat;
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
                            KeyCode::Char('+') => {
                                app.input_mode = InputMode::QuickInclude;
                                app.quick_filter_input.clear();
                            }
                            KeyCode::Char('F') => {
                                app.input_mode = InputMode::FilterManager;
                                app.filter_manager_cursor = 0;
                            }
                            KeyCode::Char('n') => app.next_search_match(),
                            KeyCode::Char('N') => app.prev_search_match(),
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
                InputMode::GotoLine => match key.code {
                    KeyCode::Enter => {
                        app.goto_line();
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Esc => app.input_mode = InputMode::Normal,
                    KeyCode::Backspace => {
                        app.goto_input.pop();
                    }
                    KeyCode::Char(c) if c.is_ascii_digit() => app.goto_input.push(c),
                    _ => {}
                },
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
                    if let Some(ref mut ff) = app.field_filter {
                        match key.code {
                            KeyCode::Up | KeyCode::Char('k') => {
                                ff.cursor = ff.cursor.saturating_sub(1);
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                if ff.cursor + 1 < ff.fields.len() {
                                    ff.cursor += 1;
                                }
                            }
                            KeyCode::PageUp => {
                                ff.cursor = ff.cursor.saturating_sub(10);
                            }
                            KeyCode::PageDown => {
                                ff.cursor = (ff.cursor + 10).min(ff.fields.len().saturating_sub(1));
                            }
                            KeyCode::Char(' ') => {
                                let cur = ff.cursor;
                                ff.fields[cur].2 = !ff.fields[cur].2;
                            }
                            KeyCode::Tab => {
                                ff.exclude = !ff.exclude;
                            }
                            KeyCode::Char('o') => {
                                ff.logic_or = !ff.logic_or;
                            }
                            KeyCode::Enter => {
                                app.apply_field_filter();
                            }
                            KeyCode::Esc => {
                                app.field_filter = None;
                                app.input_mode = InputMode::Normal;
                            }
                            _ => {}
                        }
                    } else {
                        app.input_mode = InputMode::Normal;
                    }
                }
                InputMode::FilterManager => match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.filter_manager_cursor = app.filter_manager_cursor.saturating_sub(1);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if !app.filters.is_empty()
                            && app.filter_manager_cursor + 1 < app.filters.len()
                        {
                            app.filter_manager_cursor += 1;
                        }
                    }
                    KeyCode::PageUp => {
                        app.filter_manager_cursor = app.filter_manager_cursor.saturating_sub(10);
                    }
                    KeyCode::PageDown => {
                        if !app.filters.is_empty() {
                            app.filter_manager_cursor = (app.filter_manager_cursor + 10)
                                .min(app.filters.len().saturating_sub(1));
                        }
                    }
                    KeyCode::Char('d') | KeyCode::Delete => {
                        if !app.filters.is_empty() {
                            let idx = app.filter_manager_cursor;
                            app.remove_filter(idx);
                            if app.filter_manager_cursor > 0
                                && app.filter_manager_cursor >= app.filters.len()
                            {
                                app.filter_manager_cursor = app.filters.len().saturating_sub(1);
                            }
                        }
                    }
                    KeyCode::Char('c') => {
                        app.clear_filters();
                        app.filter_manager_cursor = 0;
                    }
                    KeyCode::Esc | KeyCode::Enter => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
                InputMode::ColumnSelector => match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.column_config.cursor = app.column_config.cursor.saturating_sub(1);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if app.column_config.cursor + 1 < app.column_config.columns.len() {
                            app.column_config.cursor += 1;
                        }
                    }
                    KeyCode::Char(' ') | KeyCode::Enter => {
                        let cur = app.column_config.cursor;
                        app.column_config.toggle(cur);
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
                InputMode::CopyFormat => match key.code {
                    KeyCode::Char('r') | KeyCode::Char('1') => {
                        if let Some(text) = app.copy_as_format(app::CopyFormat::Raw) {
                            app::osc52_copy(&text);
                        }
                    }
                    KeyCode::Char('j') | KeyCode::Char('2') => {
                        if let Some(text) = app.copy_as_format(app::CopyFormat::Json) {
                            app::osc52_copy(&text);
                        }
                    }
                    KeyCode::Char('y') | KeyCode::Char('3') => {
                        if let Some(text) = app.copy_as_format(app::CopyFormat::Yaml) {
                            app::osc52_copy(&text);
                        }
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
                InputMode::Help => {
                    app.input_mode = InputMode::Normal;
                }
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
