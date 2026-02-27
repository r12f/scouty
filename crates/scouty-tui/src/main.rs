//! scouty-tui — Terminal UI for scouty log viewer.

mod app;
pub mod config;
mod density;
pub mod keybinding;
pub mod panel;
mod pipe;
pub mod text_input;
mod ui;

use app::{App, InputMode};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use std::io::{stdout, IsTerminal};
use std::time::Duration;

/// Check if stdin is a pipe (not a terminal).
fn stdin_is_pipe() -> bool {
    !std::io::stdin().is_terminal()
}

/// Resolve default log file paths when no arguments are provided.
///
/// On Linux, tries `/var/log/syslog` (Debian/Ubuntu) then `/var/log/messages` (RHEL/CentOS).
/// On other platforms (macOS, Windows), prints usage and exits.
fn resolve_default_files(cfg: &config::Config) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Try config default_paths first
    if !cfg.default_paths.is_empty() {
        let expanded = config::expand_default_paths(&cfg.default_paths);
        if !expanded.is_empty() {
            return Ok(expanded);
        }
        // All patterns matched no files — fall through to platform defaults
    }

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

    // Parse CLI flags
    let mut theme_override: Option<String> = None;
    let mut config_override: Option<String> = None;
    let mut file_args: Vec<String> = Vec::new();
    let mut no_tui = false;
    let mut regions_path: Option<String> = None;
    let mut pipe_filters: Vec<String> = Vec::new();
    let mut pipe_level: Option<String> = None;
    let mut pipe_format: Option<String> = None;
    let mut pipe_fields: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--theme" => {
                if i + 1 < args.len() {
                    theme_override = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --theme requires a value");
                    std::process::exit(1);
                }
            }
            arg if arg.starts_with("--theme=") => {
                theme_override = Some(arg.trim_start_matches("--theme=").to_string());
                i += 1;
            }
            "--config" => {
                if i + 1 < args.len() {
                    config_override = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --config requires a value");
                    std::process::exit(1);
                }
            }
            arg if arg.starts_with("--config=") => {
                config_override = Some(arg.trim_start_matches("--config=").to_string());
                i += 1;
            }
            "--help" | "-h" => {
                eprintln!("Usage: scouty-tui [OPTIONS] [FILES...]");
                eprintln!();
                eprintln!("Arguments:");
                eprintln!("  [FILES...]        Local files or ssh://[user@]host[:port]:/path URLs");
                eprintln!();
                eprintln!("Options:");
                eprintln!(
                    "  --theme <name>    Override theme (default, dark, light, solarized, or custom)"
                );
                eprintln!("  --config <path>   Load additional config file (overrides file-based configs)");
                eprintln!("  --regions <path>  Load region definitions (file or directory)");
                eprintln!("  --generate-config          Generate default config to stdout");
                eprintln!(
                    "  --generate-theme <name>    Generate built-in theme to stdout (or 'list')"
                );
                eprintln!();
                eprintln!("Pipe mode (auto when stdout is not a TTY):");
                eprintln!("  --no-tui                   Force pipe mode (no TUI)");
                eprintln!("  --filter <expr>            Filter expression (repeatable, AND logic)");
                eprintln!("  --level <level>            Minimum level (trace/debug/info/notice/warn/error/fatal)");
                eprintln!(
                    "  --format <fmt>             Output format: raw (default), json, yaml, csv"
                );
                eprintln!("  --fields <list>            Comma-separated fields (default: all)");
                eprintln!();
                eprintln!("  -h, --help        Show this help");
                std::process::exit(0);
            }
            "--generate-config" => {
                print!("{}", config::generate_default_config());
                std::process::exit(0);
            }
            "--generate-theme" => {
                if i + 1 < args.len() {
                    let name = &args[i + 1];
                    if name == "list" {
                        for t in config::Theme::builtin_names() {
                            println!("{}", t);
                        }
                        std::process::exit(0);
                    }
                    match config::generate_theme(name) {
                        Some(yaml) => {
                            print!("{}", yaml);
                            std::process::exit(0);
                        }
                        None => {
                            eprintln!("Error: unknown theme '{}'\n\nAvailable themes:", name);
                            for t in config::Theme::builtin_names() {
                                eprintln!("  {}", t);
                            }
                            std::process::exit(1);
                        }
                    }
                } else {
                    eprintln!("Error: --generate-theme requires a theme name (or 'list')");
                    std::process::exit(1);
                }
            }
            arg if arg.starts_with("--generate-theme=") => {
                let name = arg.trim_start_matches("--generate-theme=");
                if name == "list" {
                    for t in config::Theme::builtin_names() {
                        println!("{}", t);
                    }
                    std::process::exit(0);
                }
                match config::generate_theme(name) {
                    Some(yaml) => {
                        print!("{}", yaml);
                        std::process::exit(0);
                    }
                    None => {
                        eprintln!("Error: unknown theme '{}'\n\nAvailable themes:", name);
                        for t in config::Theme::builtin_names() {
                            eprintln!("  {}", t);
                        }
                        std::process::exit(1);
                    }
                }
            }
            "--no-tui" => {
                no_tui = true;
                i += 1;
            }
            "--regions" => {
                if i + 1 < args.len() {
                    regions_path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --regions requires a path");
                    std::process::exit(1);
                }
            }
            arg if arg.starts_with("--regions=") => {
                regions_path = Some(arg.trim_start_matches("--regions=").to_string());
                i += 1;
            }
            "--filter" => {
                if i + 1 < args.len() {
                    pipe_filters.push(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --filter requires a value");
                    std::process::exit(1);
                }
            }
            arg if arg.starts_with("--filter=") => {
                pipe_filters.push(arg.trim_start_matches("--filter=").to_string());
                i += 1;
            }
            "--level" => {
                if i + 1 < args.len() {
                    pipe_level = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --level requires a value");
                    std::process::exit(1);
                }
            }
            arg if arg.starts_with("--level=") => {
                pipe_level = Some(arg.trim_start_matches("--level=").to_string());
                i += 1;
            }
            "--format" => {
                if i + 1 < args.len() {
                    pipe_format = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --format requires a value");
                    std::process::exit(1);
                }
            }
            arg if arg.starts_with("--format=") => {
                pipe_format = Some(arg.trim_start_matches("--format=").to_string());
                i += 1;
            }
            "--fields" => {
                if i + 1 < args.len() {
                    pipe_fields = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --fields requires a value");
                    std::process::exit(1);
                }
            }
            arg if arg.starts_with("--fields=") => {
                pipe_fields = Some(arg.trim_start_matches("--fields=").to_string());
                i += 1;
            }
            _ => {
                file_args.push(args[i].clone());
                i += 1;
            }
        }
    }

    // Determine if we should run in pipe mode
    let stdout_is_tty = std::io::stdout().is_terminal();
    let pipe_mode = no_tui || !stdout_is_tty;

    // pipe + file args are mutually exclusive (in TUI mode, stdin is consumed for terminal)
    if !pipe_mode && piped && !file_args.is_empty() {
        eprintln!("Error: Cannot combine piped stdin with file arguments.");
        eprintln!("Use either: command | scouty-tui  OR  scouty-tui <files>");
        std::process::exit(1);
    }

    // Load config early so default_paths is available for file resolution
    let cfg = config::load_config_layered(config_override.as_deref());

    let files: Vec<String> = if !piped && file_args.is_empty() {
        if pipe_mode {
            // In pipe mode without files or stdin, error
            eprintln!("Error: No input. Provide files or pipe stdin.");
            std::process::exit(1);
        }
        resolve_default_files(&cfg)?
    } else {
        file_args
    };

    // ── Pipe mode: parse/filter/output without TUI ──
    if pipe_mode {
        let format = pipe_format
            .as_deref()
            .map(|f| {
                pipe::OutputFormat::from_str(f).unwrap_or_else(|| {
                    eprintln!("Error: unknown format '{}'. Use: raw, json, yaml, csv", f);
                    std::process::exit(1);
                })
            })
            .unwrap_or(pipe::OutputFormat::Raw);

        let level = pipe_level.as_deref().map(|l| {
            scouty::record::LogLevel::from_str_loose(l).unwrap_or_else(|| {
                eprintln!(
                    "Error: unknown level '{}'. Use: trace, debug, info, notice, warn, error, fatal",
                    l
                );
                std::process::exit(1);
            })
        });

        let fields: Vec<String> = pipe_fields
            .map(|f| f.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        // Read stdin if piped
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

        let pipe_config = pipe::PipeConfig {
            filters: pipe_filters,
            level,
            format,
            fields,
        };

        return pipe::run_pipe_mode(
            files,
            stdin_lines,
            pipe_config,
            cfg.ssh.connect_timeout,
            cfg.ssh.keepalive_interval,
        );
    }

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

    // Load config before entering TUI mode so warnings are visible on stderr
    let keymap = keybinding::Keymap::from_config(&cfg.keybindings);

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
        match App::load_files(&files, cfg.ssh.connect_timeout, cfg.ssh.keepalive_interval) {
            Ok(app) => app,
            Err(e) => {
                let _ = disable_raw_mode();
                let _ = stdout().execute(LeaveAlternateScreen);
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    };

    // Load and process region definitions
    {
        let region_defs = if let Some(ref path) = regions_path {
            let p = std::path::Path::new(path);
            if p.is_dir() {
                scouty::region::config::load_from_dir(p).unwrap_or_else(|e| {
                    eprintln!("Warning: failed to load regions from {}: {}", path, e);
                    Vec::new()
                })
            } else {
                scouty::region::config::load_from_file(p).unwrap_or_else(|e| {
                    eprintln!("Warning: failed to load regions from {}: {}", path, e);
                    Vec::new()
                })
            }
        } else {
            scouty::region::config::load_all()
        };

        if !region_defs.is_empty() {
            let mut processor = scouty::region::processor::RegionProcessor::new(region_defs);
            let records_vec: Vec<scouty::record::LogRecord> =
                app.records.iter().map(|r| (**r).clone()).collect();
            processor.process_records(&records_vec);
            app.regions =
                scouty::region::store::RegionStore::from_regions(processor.regions().to_vec());
        }
    }

    // Apply config settings
    {
        app.theme = config::resolve_theme(&cfg, theme_override.as_deref());

        // Apply general settings
        if piped && cfg.general.follow_on_pipe {
            app.follow_mode = true;
            app.scroll_to_bottom();
        }
        app.detail_panel_ratio = cfg.general.detail_panel_ratio.clamp(0.1, 0.9);
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
                                                         // Reserve space for time-per-column label (e.g. "[500ms/█]" ~10 chars)
                                                         // Reserve space for time-per-column label (e.g. "[500ms/█]" ~10 chars, allow headroom)
            let label_reserve: u16 = 15;
            let chart_width = term_width.saturating_sub(right_width + label_reserve + 2) as usize;
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

                match app.input_mode {
                    InputMode::Normal => {
                        use keybinding::Action;

                        // Detail panel tree navigation (when focused)
                        if app.detail_open && app.detail_tree_focus {
                            use crossterm::event::{KeyCode, KeyModifiers};
                            let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
                            let handled = match key.code {
                                KeyCode::Char('j') | KeyCode::Down if !ctrl => {
                                    app.detail_tree_move_down();
                                    true
                                }
                                KeyCode::Char('k') | KeyCode::Up if !ctrl => {
                                    app.detail_tree_move_up();
                                    true
                                }
                                KeyCode::Char('l') | KeyCode::Enter => {
                                    app.detail_tree_toggle();
                                    true
                                }
                                KeyCode::Char('h') => {
                                    app.detail_tree_collapse_or_parent();
                                    true
                                }
                                KeyCode::Char('H') => {
                                    app.detail_tree_collapse_all();
                                    true
                                }
                                KeyCode::Char('L') => {
                                    app.detail_tree_expand_all();
                                    true
                                }
                                KeyCode::Char('f') => {
                                    app.detail_tree_quick_filter();
                                    true
                                }
                                KeyCode::Tab => {
                                    app.detail_tree_focus = false;
                                    true
                                }
                                KeyCode::Esc => {
                                    app.detail_tree_focus = false;
                                    true
                                }
                                _ => false,
                            };
                            if handled {
                                continue;
                            }
                        }

                        // Tab toggles tree focus when detail panel is open with expanded data
                        if key.code == crossterm::event::KeyCode::Tab && app.detail_open {
                            if let Some(record) = app.selected_record() {
                                if record.expanded.as_ref().is_some_and(|e| !e.is_empty()) {
                                    app.detail_tree_focus = !app.detail_tree_focus;
                                    continue;
                                }
                            }
                        }

                        // Region panel key handling (when focused)
                        if app.panel_state.has_focus()
                            && app.panel_state.active == crate::panel::PanelId::Region
                        {
                            use crate::ui::widgets::region_panel_widget::RegionPanelWidget;
                            use crossterm::event::{KeyCode, KeyModifiers};

                            let entries = RegionPanelWidget::build_entries(&app);
                            let max_cursor = entries.len().saturating_sub(1);
                            let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

                            let handled = match key.code {
                                KeyCode::Char('j') | KeyCode::Down if !ctrl => {
                                    if app.region_manager_cursor < max_cursor {
                                        app.region_manager_cursor += 1;
                                    }
                                    true
                                }
                                KeyCode::Char('k') | KeyCode::Up if !ctrl => {
                                    if app.region_manager_cursor > 0 {
                                        app.region_manager_cursor -= 1;
                                    }
                                    true
                                }
                                KeyCode::Enter => {
                                    // Jump to region start record
                                    if let Some(entry) = entries.get(app.region_manager_cursor) {
                                        app.jump_to_record_index(entry.start_index);
                                        app.panel_state.focus_log_table();
                                    }
                                    true
                                }
                                KeyCode::Char('f') => {
                                    // Filter to selected region's records
                                    if let Some(entry) = entries.get(app.region_manager_cursor) {
                                        let expr = format!(
                                            "_region_type == \"{}\"",
                                            entry.definition_name
                                        );
                                        app.add_filter_expr(&expr);
                                    }
                                    true
                                }
                                KeyCode::Char('t') => {
                                    // Toggle type filter
                                    if let Some(entry) = entries.get(app.region_manager_cursor) {
                                        if app.region_panel_type_filter.as_deref()
                                            == Some(&entry.definition_name)
                                        {
                                            app.region_panel_type_filter = None;
                                        } else {
                                            app.region_panel_type_filter =
                                                Some(entry.definition_name.clone());
                                        }
                                        // Reset cursor when filter changes
                                        app.region_manager_cursor = 0;
                                    }
                                    true
                                }
                                KeyCode::Char('s') => {
                                    // Toggle sort mode
                                    app.region_panel_sort = app.region_panel_sort.toggle();
                                    true
                                }
                                KeyCode::Esc => {
                                    app.panel_state.focus_log_table();
                                    true
                                }
                                _ => false,
                            };
                            if handled {
                                continue;
                            }
                        }

                        // Panel system keybindings (Ctrl+arrows, Tab/BackTab, z)
                        {
                            use crossterm::event::{KeyCode, KeyModifiers};
                            let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
                            let handled = match key.code {
                                KeyCode::Down if ctrl => {
                                    app.panel_state.focus_panel();
                                    // Sync legacy detail_open
                                    app.detail_open = app.panel_state.expanded;
                                    true
                                }
                                KeyCode::Up if ctrl => {
                                    app.panel_state.focus_log_table();
                                    true
                                }
                                KeyCode::Right if ctrl => {
                                    app.panel_state.next_panel();
                                    true
                                }
                                KeyCode::Left if ctrl => {
                                    app.panel_state.prev_panel();
                                    true
                                }
                                // Tab cycles forward: log table → panel content → next panel → ... → log table
                                KeyCode::Tab if key.modifiers.is_empty() && app.panel_state.expanded => {
                                    if app.panel_state.focus == crate::panel::PanelFocus::LogTable {
                                        app.panel_state.focus_panel();
                                    } else {
                                        // On last panel, cycle back to log table; otherwise next panel
                                        let all = crate::panel::PanelId::all();
                                        if app.panel_state.active == *all.last().unwrap() {
                                            app.panel_state.focus_log_table();
                                        } else {
                                            app.panel_state.next_panel();
                                        }
                                    }
                                    true
                                }
                                // Shift+Tab (BackTab) cycles backward
                                KeyCode::BackTab if app.panel_state.expanded => {
                                    if app.panel_state.focus == crate::panel::PanelFocus::LogTable {
                                        // Focus last panel
                                        let all = crate::panel::PanelId::all();
                                        app.panel_state.active = *all.last().unwrap();
                                        app.panel_state.focus_panel();
                                    } else {
                                        let all = crate::panel::PanelId::all();
                                        if app.panel_state.active == all[0] {
                                            app.panel_state.focus_log_table();
                                        } else {
                                            app.panel_state.prev_panel();
                                        }
                                    }
                                    true
                                }
                                KeyCode::Char('z')
                                    if key.modifiers.is_empty() && app.panel_state.expanded =>
                                {
                                    app.panel_state.toggle_maximize();
                                    true
                                }
                                _ => false,
                            };
                            if handled {
                                continue;
                            }
                        }

                        if let Some(action) = keymap.action(&key) {
                            match action {
                                Action::Quit => {
                                    should_break = true;
                                    break;
                                }
                                Action::CloseDetail => {
                                    if app.detail_open {
                                        app.detail_open = false;
                                    }
                                }
                                Action::MoveDown => app.select_down(1),
                                Action::MoveUp => app.select_up(1),
                                Action::PageDown => app.page_down(),
                                Action::PageUp => app.page_up(),
                                Action::ScrollToTop => app.scroll_to_top(),
                                Action::ScrollToBottom => app.scroll_to_bottom(),
                                Action::ToggleDetail => app.toggle_detail(),
                                Action::Filter => {
                                    app.input_mode = InputMode::Filter;
                                }
                                Action::Search => {
                                    app.input_mode = InputMode::Search;
                                }
                                Action::JumpForward => {
                                    app.input_mode = InputMode::JumpForward;
                                    app.time_input.clear();
                                }
                                Action::JumpBackward => {
                                    app.input_mode = InputMode::JumpBackward;
                                    app.time_input.clear();
                                }
                                Action::QuickExclude => {
                                    app.input_mode = InputMode::QuickExclude;
                                    app.quick_filter_input.clear();
                                }
                                Action::QuickInclude => {
                                    app.input_mode = InputMode::QuickInclude;
                                    app.quick_filter_input.clear();
                                }
                                Action::FieldExclude => {
                                    app.open_field_filter(true);
                                }
                                Action::FieldInclude => {
                                    app.open_field_filter(false);
                                }
                                Action::FilterManager => {
                                    app.input_mode = InputMode::FilterManager;
                                    app.filter_manager_cursor = 0;
                                }
                                Action::LevelFilter => {
                                    app.input_mode = InputMode::LevelFilter;
                                    app.level_filter_cursor = app
                                        .level_filter
                                        .map(|l| (l.as_number() - 1) as usize)
                                        .unwrap_or(0);
                                }
                                Action::DensityCycle => {
                                    app.cycle_density_source();
                                }
                                Action::DensitySelector => {
                                    app.density_selector_cursor = app
                                        .density_source_options()
                                        .iter()
                                        .position(|s| *s == app.density_source)
                                        .unwrap_or(0);
                                    app.input_mode = InputMode::DensitySelector;
                                }
                                Action::GotoLine => {
                                    app.input_mode = InputMode::GotoLine;
                                    app.goto_input.clear();
                                }
                                Action::ToggleFollow => {
                                    app.toggle_follow();
                                }
                                Action::NextMatch => app.next_search_match(),
                                Action::PrevMatch => app.prev_search_match(),
                                Action::CopyRaw => {
                                    if let Some(text) = app.copy_raw() {
                                        app::osc52_copy(&text);
                                    }
                                }
                                Action::CopyFormat => {
                                    app.input_mode = InputMode::CopyFormat;
                                    app.copy_format_cursor = 0;
                                }
                                Action::Save => {
                                    app.input_mode = InputMode::SaveDialog;
                                }
                                Action::ColumnSelector => {
                                    app.input_mode = InputMode::ColumnSelector;
                                    app.column_config.cursor = 0;
                                }
                                Action::Help => {
                                    app.input_mode = InputMode::Help;
                                    app.help_scroll = 0;
                                }
                                Action::Command => {
                                    app.command_input.clear();
                                    app.input_mode = InputMode::Command;
                                }
                                Action::AddHighlight => {
                                    app.input_mode = InputMode::Highlight;
                                    app.highlight_input.clear();
                                }
                                Action::HighlightManager => {
                                    app.input_mode = InputMode::HighlightManager;
                                    app.highlight_manager_cursor = 0;
                                }
                                Action::ToggleBookmark => {
                                    app.toggle_bookmark();
                                }
                                Action::NextBookmark => {
                                    app.jump_next_bookmark();
                                }
                                Action::PrevBookmark => {
                                    app.jump_prev_bookmark();
                                }
                                Action::BookmarkManager => {
                                    app.input_mode = InputMode::BookmarkManager;
                                    app.bookmark_manager_cursor = 0;
                                }
                                Action::RegionManager => {
                                    app.panel_state.open(crate::panel::PanelId::Region);
                                }
                                Action::NextRegion => {
                                    // Jump to the next region start after current position
                                    if let Some(record_idx) = app.filtered_indices.get(app.selected)
                                    {
                                        if let Some(region) = app
                                            .regions
                                            .regions()
                                            .iter()
                                            .find(|r| r.start_index > *record_idx)
                                        {
                                            app.jump_to_record_index(region.start_index);
                                        }
                                    }
                                }
                                Action::Stats => {
                                    use ui::windows::stats_window::StatsData;
                                    app.cached_stats = Some(StatsData::compute(&app));
                                    app.input_mode = InputMode::Statistics;
                                }
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
                            match window.action {
                                Some("save_preset") => {
                                    app.preset_name_input.clear();
                                    app.input_mode = InputMode::SavePreset;
                                }
                                Some("load_preset") => {
                                    app.preset_list = crate::config::filter_preset::list_presets();
                                    app.preset_list_cursor = 0;
                                    app.input_mode = InputMode::LoadPreset;
                                }
                                _ => {
                                    app.input_mode = InputMode::Normal;
                                }
                            }
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
                    InputMode::LevelFilter => {
                        use ui::windows::level_filter_window::LevelFilterWindow;
                        let mut window = LevelFilterWindow::from_app(&app);
                        let result = ui::dispatch_key(&mut window, key);
                        if result == ui::ComponentResult::Close {
                            if window.confirmed {
                                if let Some(preset) = window.selected {
                                    app.apply_level_filter(preset);
                                }
                            }
                            app.input_mode = InputMode::Normal;
                        }
                    }
                    InputMode::SavePreset => {
                        use ui::windows::save_preset_window::SavePresetWindow;
                        let mut window = SavePresetWindow::new();
                        window.input = app.preset_name_input.clone();
                        let result = ui::dispatch_key(&mut window, key);
                        app.preset_name_input = window.input;
                        if result == ui::ComponentResult::Close {
                            if window.confirmed {
                                let name = app.preset_name_input.value().to_string();
                                app.save_filter_preset(&name);
                            }
                            app.input_mode = InputMode::Normal;
                        }
                    }
                    InputMode::LoadPreset => {
                        use ui::windows::load_preset_window::LoadPresetWindow;
                        let mut window = LoadPresetWindow::new(app.preset_list.clone());
                        window.cursor = app.preset_list_cursor;
                        let result = ui::dispatch_key(&mut window, key);
                        // Handle deletion
                        if let Some(ref name) = window.delete_name {
                            let _ = crate::config::filter_preset::delete_preset(name);
                        }
                        app.preset_list = window.presets;
                        app.preset_list_cursor = window.cursor;
                        if result == ui::ComponentResult::Close {
                            if window.confirmed {
                                if let Some(ref name) = window.selected {
                                    app.load_filter_preset(name);
                                }
                            }
                            app.input_mode = InputMode::Normal;
                        }
                    }
                    InputMode::DensitySelector => {
                        use ui::windows::density_selector_window::DensitySelectorWindow;
                        let options = app.density_source_options();
                        let mut window =
                            DensitySelectorWindow::new(options, app.density_selector_cursor);
                        let result = ui::dispatch_key(&mut window, key);
                        app.density_selector_cursor = window.cursor;
                        if result == ui::ComponentResult::Close {
                            if window.confirmed {
                                if let Some(source) = window.selected {
                                    app.density_source = source;
                                    app.density_cache = None;
                                    app.set_status(format!(
                                        "Density: {}",
                                        app.density_source_label()
                                    ));
                                }
                            }
                            app.input_mode = InputMode::Normal;
                        }
                    }
                    InputMode::SaveDialog => {
                        use ui::windows::save_dialog_window::SaveDialogWindow;
                        let mut window = SaveDialogWindow::from_app(&app);
                        let result = ui::dispatch_key(&mut window, key);
                        app.save_path_input = window.path_input.clone();
                        app.save_format_cursor = window.format_cursor;
                        app.save_dialog_focus = window.focus;
                        if result == ui::ComponentResult::Close {
                            if window.confirmed {
                                let path = window.expanded_path();
                                let format = window.selected_format();
                                let msg = SaveDialogWindow::execute_save(&app, &path, format);
                                app.set_status(msg);
                            }
                            app.input_mode = InputMode::Normal;
                            app.save_path_input =
                                crate::text_input::TextInput::with_text("./scouty-export.log");
                            app.save_format_cursor = 0;
                            app.save_dialog_focus = ui::windows::save_dialog_window::Focus::Path;
                        }
                    }
                    InputMode::RegionManager => {
                        use ui::windows::region_manager_window::RegionManagerWindow;
                        let mut window = RegionManagerWindow::from_app(&app);
                        let result = ui::dispatch_key(&mut window, key);
                        app.region_manager_cursor = window.cursor;
                        if result == ui::ComponentResult::Close {
                            if let Some(action) = window.action {
                                match action {
                                    ui::windows::region_manager_window::RegionAction::Jump(idx) => {
                                        app.jump_to_record_index(idx);
                                    }
                                    ui::windows::region_manager_window::RegionAction::Filter(
                                        _start,
                                        _end,
                                    ) => {
                                        let def_name =
                                            &app.regions.regions()[window.cursor].definition_name;
                                        let expr = format!("_region_type == \"{}\"", def_name);
                                        app.add_filter_expr(&expr);
                                    }
                                }
                            }
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
