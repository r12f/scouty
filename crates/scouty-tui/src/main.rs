//! scouty-tui — Terminal UI for scouty log viewer.

mod app;
pub mod config;
mod density;
pub mod text_input;
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

/// Check if stdin is a pipe (not a terminal).
fn stdin_is_pipe() -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        let fd = std::io::stdin().as_raw_fd();
        unsafe { libc::isatty(fd) == 0 }
    }
    #[cfg(not(unix))]
    {
        false
    }
}

/// Resolve default log file paths when no arguments are provided.
///
/// On Linux, tries `/var/log/syslog` (Debian/Ubuntu) then `/var/log/messages` (RHEL/CentOS).
/// On other platforms (macOS, Windows), prints usage and exits.
fn resolve_default_files() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    if cfg!(target_os = "linux") {
        let candidates = ["/var/log/syslog", "/var/log/messages"];
        for path in &candidates {
            if std::path::Path::new(path).exists() {
                eprintln!("No file specified, defaulting to {}", path);
                return Ok(vec![path.to_string()]);
            }
        }
        eprintln!(
            "Error: No log file specified and no default syslog found.\n\
             Tried: /var/log/syslog, /var/log/messages\n\n\
             Usage: scouty-tui <logfile> [logfile2 ...]"
        );
        std::process::exit(1);
    } else {
        eprintln!("Usage: scouty-tui <logfile> [logfile2 ...]");
        std::process::exit(1);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let piped = stdin_is_pipe();
    let file_args: Vec<String> = if args.len() >= 2 {
        args[1..].to_vec()
    } else {
        Vec::new()
    };

    // pipe + file args are mutually exclusive
    if piped && !file_args.is_empty() {
        eprintln!("Error: Cannot combine piped stdin with file arguments.");
        eprintln!("Use either: command | scouty-tui  OR  scouty-tui <files>");
        std::process::exit(1);
    }

    let files: Vec<String> = if !piped && file_args.is_empty() {
        resolve_default_files()?
    } else {
        file_args
    };

    // If piped, read all stdin lines before entering TUI (stdin will be consumed).
    //
    // NOTE: This reads the entire stdin into memory before launching the UI.
    // Streaming sources (e.g. `journalctl -f | scouty-tui`) will block here
    // until the upstream command exits or the pipe is closed (e.g. Ctrl+C on
    // the producer).  Streaming / incremental ingestion is planned as a future
    // enhancement — see: https://github.com/r12f/scouty/issues/180
    let stdin_lines: Option<Vec<String>> = if piped {
        use std::io::BufRead;
        let stdin = std::io::stdin();
        let lines: Vec<String> = stdin
            .lock()
            .lines()
            .collect::<std::result::Result<_, _>>()?;
        Some(lines)
    } else {
        None
    };

    let files: Vec<&str> = files.iter().map(|s| s.as_str()).collect();

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
    let loading_msg = if piped {
        "Loading from stdin...".to_string()
    } else if files.len() == 1 {
        format!("Loading {}...", files[0])
    } else {
        format!("Loading {} files...", files.len())
    };
    if let Err(e) = terminal.draw(|frame| {
        let area = frame.area();
        let text = ratatui::widgets::Paragraph::new(loading_msg.as_str());
        let y = area.y + area.height / 2;
        let centered = ratatui::layout::Rect::new(area.x, y, area.width, 1);
        frame.render_widget(text, centered);
    }) {
        let _ = disable_raw_mode();
        let _ = stdout().execute(LeaveAlternateScreen);
        return Err(e.into());
    }

    // Load data
    let mut app = if let Some(lines) = stdin_lines {
        match App::load_stdin(lines) {
            Ok(mut app) => {
                app.set_status("stdin closed \u{2014} all input loaded".to_string());
                app
            }
            Err(e) => {
                let _ = disable_raw_mode();
                let _ = stdout().execute(LeaveAlternateScreen);
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        match App::load_files(&files) {
            Ok(app) => app,
            Err(e) => {
                let _ = disable_raw_mode();
                let _ = stdout().execute(LeaveAlternateScreen);
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    };

    // Load config and resolve theme
    {
        let cfg = config::load_config();
        app.theme = config::resolve_theme(&cfg, None);
    }

    loop {
        // Pre-compute density cache using exact position text width
        if let Ok(size) = crossterm::terminal::size() {
            let term_width = size.0;
            // Compute exact right_text width (mirrors StatusBarWidget::render_line1)
            let position = if app.total() == 0 {
                format!("0/0 (Total: {})", app.total_records)
            } else {
                let current = app.selected + 1;
                let filtered = app.total();
                let total = app.total_records;
                if filtered == total {
                    format!("{}/{}", current, total)
                } else {
                    format!("{}/{} (Total: {})", current, filtered, total)
                }
            };
            let right_width = position.len() as u16 + 2; // " {} " padding
            let chart_width = term_width.saturating_sub(right_width + 2) as usize;
            if chart_width >= 4 && app.total() > 0 {
                app.get_density_cache(chart_width);
            }
        }

        terminal.draw(|frame| {
            ui::render(frame, &mut app);
        })?;

        if !event::poll(Duration::from_millis(250))? {
            app.tick_status_clear();
            continue;
        }

        // Drain all pending key events before next draw (input coalescing)
        let mut should_break = false;
        loop {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    if !event::poll(Duration::from_millis(0))? {
                        break;
                    }
                    continue;
                }

                app.clear_status();
                let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

                match app.input_mode {
                    InputMode::Normal => {
                        if ctrl {
                            match key.code {
                                KeyCode::Down => app.page_down(),
                                KeyCode::Up => app.page_up(),
                                KeyCode::Char('j') => {
                                    app.page_down();
                                }
                                KeyCode::Char('k') => {
                                    app.page_up();
                                }
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
                                _ => {}
                            }
                        } else {
                            match key.code {
                                KeyCode::Char('q') => {
                                    should_break = true;
                                    break;
                                }
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
                                KeyCode::Char(']') => {
                                    app.input_mode = InputMode::JumpForward;
                                    app.time_input.clear();
                                }
                                KeyCode::Char('[') => {
                                    app.input_mode = InputMode::JumpBackward;
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
                                KeyCode::Char('_') => {
                                    app.open_field_filter(true);
                                }
                                KeyCode::Char('+') => {
                                    app.open_field_filter(false);
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
                                    app.copy_format_cursor = 0;
                                }
                                KeyCode::Char('c') => {
                                    app.input_mode = InputMode::ColumnSelector;
                                    app.column_config.cursor = 0;
                                }
                                KeyCode::Char('?') => {
                                    app.input_mode = InputMode::Help;
                                    app.help_scroll = 0;
                                }
                                KeyCode::Char(':') => {
                                    app.command_input.clear();
                                    app.input_mode = InputMode::Command;
                                }
                                KeyCode::Char('h') => {
                                    app.input_mode = InputMode::Highlight;
                                    app.highlight_input.clear();
                                }
                                KeyCode::Char('H') => {
                                    app.input_mode = InputMode::HighlightManager;
                                    app.highlight_manager_cursor = 0;
                                }
                                KeyCode::Char('m') => {
                                    app.toggle_bookmark();
                                }
                                KeyCode::Char('\'') => {
                                    app.jump_next_bookmark();
                                }
                                KeyCode::Char('"') => {
                                    app.jump_prev_bookmark();
                                }
                                KeyCode::Char('M') => {
                                    app.input_mode = InputMode::BookmarkManager;
                                    app.bookmark_manager_cursor = 0;
                                }
                                KeyCode::Char('S') => {
                                    use ui::windows::stats_window::StatsData;
                                    app.cached_stats = Some(StatsData::compute(&app));
                                    app.input_mode = InputMode::Statistics;
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
                        _ => {
                            if app.filter_input.handle_key(key) {
                                app.filter_error = None;
                            }
                        }
                    },
                    InputMode::Search => match key.code {
                        KeyCode::Enter => {
                            app.execute_search();
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Esc => app.input_mode = InputMode::Normal,
                        _ => {
                            app.search_input.handle_key(key);
                        }
                    },
                    InputMode::JumpForward | InputMode::JumpBackward => match key.code {
                        KeyCode::Enter => {
                            let forward = app.input_mode == InputMode::JumpForward;
                            if app.jump_relative(forward) {
                                app.input_mode = InputMode::Normal;
                            }
                        }
                        KeyCode::Esc => app.input_mode = InputMode::Normal,
                        _ => {
                            app.time_input.handle_key(key);
                        }
                    },
                    InputMode::GotoLine => {
                        use ui::windows::goto_line_window::GotoLineWindow;
                        let mut window = GotoLineWindow::new();
                        window.input = app.goto_input.value().to_string();
                        let result = ui::dispatch_key(&mut window, key);
                        app.goto_input.set(&window.input);
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
                        _ => {
                            app.quick_filter_input.handle_key(key);
                        }
                    },
                    InputMode::QuickInclude => match key.code {
                        KeyCode::Enter => {
                            app.apply_quick_include();
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Esc => app.input_mode = InputMode::Normal,
                        _ => {
                            app.quick_filter_input.handle_key(key);
                        }
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
                        let mut window = CopyFormatWindow::from_app(&app);
                        let result = ui::dispatch_key(&mut window, key);
                        app.copy_format_cursor = window.cursor;
                        if result == ui::ComponentResult::Close {
                            if window.confirmed {
                                CopyFormatWindow::select_format(&mut app, window.selected_format());
                            }
                            app.input_mode = InputMode::Normal;
                            app.copy_format_cursor = 0;
                        }
                    }
                    InputMode::Help => {
                        use ui::windows::help_window::HelpWindow;
                        let mut window = HelpWindow::new(&app.theme);
                        window.scroll = app.help_scroll;
                        let result = ui::dispatch_key(&mut window, key);
                        app.help_scroll = window.scroll;
                        if result == ui::ComponentResult::Close {
                            app.input_mode = InputMode::Normal;
                        }
                    }
                    InputMode::Statistics => {
                        use ui::windows::stats_window::StatsWindow;
                        // Stats are pre-computed on mode entry; reuse cached data.
                        if let Some(ref stats) = app.cached_stats {
                            let mut window = StatsWindow {
                                stats,
                                theme: &app.theme,
                            };
                            let result = ui::dispatch_key(&mut window, key);
                            if result == ui::ComponentResult::Close {
                                app.cached_stats = None;
                                app.input_mode = InputMode::Normal;
                            }
                        } else {
                            app.input_mode = InputMode::Normal;
                        }
                    }
                    InputMode::Command => match key.code {
                        KeyCode::Enter => {
                            app.execute_command();
                            app.input_mode = InputMode::Normal;
                            if app.should_quit {
                                should_break = true;
                                break;
                            }
                        }
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                        }
                        _ => {
                            app.command_input.handle_key(key);
                        }
                    },
                    InputMode::Highlight => match key.code {
                        KeyCode::Enter => {
                            let pattern = app.highlight_input.value().to_string();
                            if let Err(e) = app.add_highlight_rule(&pattern) {
                                app.set_status(e);
                            }
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                        }
                        _ => {
                            app.highlight_input.handle_key(key);
                        }
                    },
                    InputMode::HighlightManager => {
                        use ui::windows::highlight_manager_window::HighlightManagerWindow;
                        let mut window = HighlightManagerWindow::from_app(&app);
                        let result = ui::dispatch_key(&mut window, key);
                        window.apply_to_app(&mut app);
                        if result == ui::ComponentResult::Close {
                            app.input_mode = InputMode::Normal;
                        }
                    }
                    InputMode::BookmarkManager => {
                        use ui::windows::bookmark_manager_window::BookmarkManagerWindow;
                        let mut window = BookmarkManagerWindow::from_app(&app);
                        let result = ui::dispatch_key(&mut window, key);
                        window.apply_to_app(&mut app);
                        if result == ui::ComponentResult::Close {
                            app.input_mode = InputMode::Normal;
                        }
                    }
                }
            } // if let Event::Key

            // For input coalescing: in Normal mode, coalesce consecutive key presses
            // while events are available. For other modes (input, dialogs), process
            // one key at a time.
            if app.input_mode != InputMode::Normal || !event::poll(Duration::from_millis(0))? {
                break;
            }
        } // drain loop

        if should_break {
            break;
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
