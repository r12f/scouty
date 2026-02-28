//! scouty-tui — Terminal UI for scouty log viewer.

mod app;
pub mod config;
mod density;
pub mod keybinding;
pub mod panel;
mod pipe;
pub mod text_input;
mod ui;

use crate::ui::framework::Window;
use app::{App, InputMode};
use crossterm::{
    event::{self, Event, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use std::io::{stdout, IsTerminal};
use std::time::Duration;
use tracing_appender::rolling;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

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
    let mut log_level: Option<String> = None;
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
                eprintln!("  --log [level]     Enable logging to ~/.scouty/log/ (default: info)");
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
            "--log" => {
                // --log [level]  (level is optional, default "info")
                if i + 1 < args.len() && !args[i + 1].starts_with("-") {
                    log_level = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    log_level = Some("info".to_string());
                    i += 1;
                }
            }
            arg if arg.starts_with("--log=") => {
                log_level = Some(arg.trim_start_matches("--log=").to_string());
                i += 1;
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

    // ── Initialize tracing (only when --log is passed) ──
    let _tracing_guard = if let Some(ref level) = log_level {
        let log_dir = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".scouty")
            .join("log");
        let _ = std::fs::create_dir_all(&log_dir);

        let file_appender = rolling::daily(&log_dir, "scouty");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        let env_filter =
            EnvFilter::try_from_env("SCOUTY_LOG").unwrap_or_else(|_| EnvFilter::new(level));

        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                fmt::layer()
                    .with_writer(non_blocking)
                    .with_ansi(false)
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_file(true)
                    .with_line_number(true),
            )
            .init();

        tracing::info!("scouty starting up, version {}", env!("CARGO_PKG_VERSION"));
        Some(guard)
    } else {
        None
    };

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

        // Load and process categories
        let (cat_defs, cat_warnings) = scouty::category::load_categories();
        for w in &cat_warnings {
            tracing::warn!("{}", w);
        }
        if !cat_defs.is_empty() {
            let bucket_count = 100; // default density buckets
            let mut processor = scouty::category::CategoryProcessor::new(cat_defs, bucket_count);
            processor.process_records(&app.records);
            tracing::info!(
                categories = processor.store.categories.len(),
                "Category processing complete"
            );
            app.category_processor = Some(processor);
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

    // Wrap app + keymap in MainWindow for the new architecture
    let mut main_window = ui::windows::main_window::MainWindow::new(app, keymap);

    loop {
        // Pre-compute density cache using exact position text width
        if let Ok(size) = crossterm::terminal::size() {
            let term_width = size.0;
            let position = if main_window.app.total() == 0 {
                format!("0/0 (Total: {})", main_window.app.total_records)
            } else {
                let current = main_window.app.selected + 1;
                let filtered = main_window.app.total();
                let total = main_window.app.total_records;
                if filtered == total {
                    format!("{}/{}", current, total)
                } else {
                    format!("{}/{} (Total: {})", current, filtered, total)
                }
            };
            let right_width = position.len() as u16 + 2;
            let label_reserve: u16 = 15;
            let chart_width = term_width.saturating_sub(right_width + label_reserve + 2) as usize;
            if chart_width >= 4 && main_window.app.total() > 0 {
                main_window.app.get_density_cache(chart_width);
            }
        }

        // Update shortcut hints cache from MainWindow
        let hints = main_window.shortcut_hints();
        main_window.app.shortcut_hints_cache = hints
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        terminal.draw(|frame| {
            ui::render(frame, &mut main_window.app);
        })?;

        if !event::poll(Duration::from_millis(250))? {
            main_window.app.tick_status_clear();
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

                use crate::ui::framework::{Window, WindowAction};
                let action = main_window.handle_key(key);
                if action == WindowAction::Close {
                    should_break = true;
                    break;
                }
                if main_window.app.should_quit {
                    should_break = true;
                    break;
                }
            } // if let Event::Key

            // For input coalescing
            if main_window.app.input_mode != InputMode::Normal
                || !event::poll(Duration::from_millis(0))?
            {
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
