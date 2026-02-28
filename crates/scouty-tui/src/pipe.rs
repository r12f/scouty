//! Pipe output mode — non-interactive parse/filter/output to stdout.

#[cfg(test)]
#[path = "pipe_tests.rs"]
mod pipe_tests;

use scouty::loader::file::FileLoader;
use scouty::loader::ssh::{is_ssh_url, SshLoader, SshUrl};
use scouty::parser::factory::ParserFactory;
use scouty::record::{LogLevel, LogRecord};
use scouty::traits::{LoaderInfo, LogLoader};
use std::io::{self, BufWriter, Write};
use tracing::instrument;

/// Output format for pipe mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Raw,
    Json,
    Yaml,
    Csv,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "raw" => Some(Self::Raw),
            "json" => Some(Self::Json),
            "yaml" => Some(Self::Yaml),
            "csv" => Some(Self::Csv),
            _ => None,
        }
    }
}

/// Pipe mode configuration.
pub struct PipeConfig {
    pub filters: Vec<String>,
    pub level: Option<LogLevel>,
    pub format: OutputFormat,
    pub fields: Vec<String>,
}

/// Run pipe mode: parse input, apply filters, output to stdout.
#[instrument(skip(files, stdin_lines, config))]
pub fn run_pipe_mode(
    files: Vec<String>,
    stdin_lines: Option<Vec<String>>,
    config: PipeConfig,
    ssh_connect_timeout: u32,
    ssh_keepalive_interval: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    // Compile filter expressions into a single FilterEngine
    let mut filter_engine = scouty::filter::engine::FilterEngine::new();
    for expr in &config.filters {
        if let Err(e) =
            filter_engine.add_expr_filter(scouty::filter::engine::FilterAction::Include, expr)
        {
            eprintln!("Warning: invalid filter '{}': {}", expr, e);
        }
    }

    let use_all_fields =
        config.fields.is_empty() || (config.fields.len() == 1 && config.fields[0] == "all");

    // CSV header
    if config.format == OutputFormat::Csv {
        let fields = if use_all_fields {
            default_fields()
        } else {
            config.fields.clone()
        };
        writeln!(writer, "{}", fields.join(","))?;
    }

    let mut record_id: u64 = 0;

    // Process stdin lines
    if let Some(lines) = stdin_lines {
        let loader = scouty::loader::stdin::StdinLoader::new();
        let mut info = loader.info().clone();
        info.sample_lines = lines.iter().take(10).cloned().collect();
        process_lines(
            &mut writer,
            lines,
            &info,
            &mut record_id,
            &config,
            &filter_engine,
            use_all_fields,
        )?;
    }

    // Process files
    for path in &files {
        if is_ssh_url(path) {
            let url = SshUrl::parse(path).map_err(|e| {
                Box::<dyn std::error::Error>::from(format!("Invalid SSH URL '{}': {}", path, e))
            })?;
            let mut loader = SshLoader::new(url, ssh_connect_timeout, ssh_keepalive_interval);
            let lines = loader.load()?;
            let info = loader.info().clone();
            process_lines(
                &mut writer,
                lines,
                &info,
                &mut record_id,
                &config,
                &filter_engine,
                use_all_fields,
            )?;
        } else {
            let mut loader = FileLoader::new(path, false);
            let lines = loader.load()?;
            let info = loader.info().clone();
            process_lines(
                &mut writer,
                lines,
                &info,
                &mut record_id,
                &config,
                &filter_engine,
                use_all_fields,
            )?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn process_lines(
    writer: &mut impl Write,
    lines: Vec<String>,
    info: &LoaderInfo,
    record_id: &mut u64,
    config: &PipeConfig,
    filter_engine: &scouty::filter::engine::FilterEngine,
    use_all_fields: bool,
) -> io::Result<()> {
    let group = ParserFactory::create_parser_group(info);

    for line in lines {
        if let Some(record) = group.parse(&line, &info.id, &info.id, *record_id) {
            *record_id += 1;

            // Level filter
            if let Some(min_level) = &config.level {
                if let Some(rec_level) = &record.level {
                    if !level_passes(*rec_level, *min_level) {
                        continue;
                    }
                }
            }

            // Expression filters (AND)
            if !filter_engine.matches(&record) {
                continue;
            }

            // Output
            match config.format {
                OutputFormat::Raw => {
                    if record.raw.is_empty() {
                        writeln!(writer, "{}", line)?;
                    } else {
                        writeln!(writer, "{}", record.raw)?;
                    }
                }
                OutputFormat::Json => {
                    write_json(writer, &record, &config.fields, use_all_fields)?;
                }
                OutputFormat::Yaml => {
                    write_yaml(writer, &record, &config.fields, use_all_fields)?;
                }
                OutputFormat::Csv => {
                    write_csv(writer, &record, &config.fields, use_all_fields)?;
                }
            }
        }
    }
    Ok(())
}

/// Check if a record level passes the minimum level filter.
fn level_passes(record_level: LogLevel, min_level: LogLevel) -> bool {
    level_rank(record_level) >= level_rank(min_level)
}

fn level_rank(level: LogLevel) -> u8 {
    match level {
        LogLevel::Trace => 0,
        LogLevel::Debug => 1,
        LogLevel::Info => 2,
        LogLevel::Notice => 3,
        LogLevel::Warn => 4,
        LogLevel::Error => 5,
        LogLevel::Fatal => 6,
    }
}

fn default_fields() -> Vec<String> {
    vec![
        "timestamp".to_string(),
        "level".to_string(),
        "hostname".to_string(),
        "component".to_string(),
        "pid".to_string(),
        "message".to_string(),
    ]
}

fn record_field(record: &LogRecord, field: &str) -> String {
    match field.to_lowercase().as_str() {
        "timestamp" | "time" | "ts" => record.timestamp.to_rfc3339(),
        "level" | "severity" => record.level.map(|l| format!("{:?}", l)).unwrap_or_default(),
        "message" | "msg" => record.message.clone(),
        "hostname" | "host" => record.hostname.clone().unwrap_or_default(),
        "component" | "service" | "logger" => record.component_name.clone().unwrap_or_default(),
        "process" | "process_name" => record.process_name.clone().unwrap_or_default(),
        "pid" => record.pid.map(|p| p.to_string()).unwrap_or_default(),
        "tid" | "thread" => record.tid.map(|t| t.to_string()).unwrap_or_default(),
        "container" => record.container.clone().unwrap_or_default(),
        "context" => record.context.clone().unwrap_or_default(),
        "function" => record.function.clone().unwrap_or_default(),
        "source" => record.source.to_string(),
        "raw" => record.raw.clone(),
        _ => {
            // Check metadata
            record
                .metadata
                .as_ref()
                .and_then(|m| m.get(field).cloned())
                .unwrap_or_default()
        }
    }
}

fn write_json(
    writer: &mut impl Write,
    record: &LogRecord,
    fields: &[String],
    use_all: bool,
) -> io::Result<()> {
    let fields_list = if use_all {
        default_fields()
    } else {
        fields.to_vec()
    };

    let mut map = serde_json::Map::new();
    for f in &fields_list {
        let val = record_field(record, f);
        map.insert(f.clone(), serde_json::Value::String(val));
    }
    let json = serde_json::Value::Object(map);
    writeln!(writer, "{}", json)
}

fn write_yaml(
    writer: &mut impl Write,
    record: &LogRecord,
    fields: &[String],
    use_all: bool,
) -> io::Result<()> {
    let fields_list = if use_all {
        default_fields()
    } else {
        fields.to_vec()
    };

    writeln!(writer, "---")?;
    for f in &fields_list {
        let val = record_field(record, f);
        writeln!(writer, "{}: \"{}\"", f, val.replace('"', "\\\""))?;
    }
    Ok(())
}

fn write_csv(
    writer: &mut impl Write,
    record: &LogRecord,
    fields: &[String],
    use_all: bool,
) -> io::Result<()> {
    let fields_list = if use_all {
        default_fields()
    } else {
        fields.to_vec()
    };

    let values: Vec<String> = fields_list
        .iter()
        .map(|f| {
            let val = record_field(record, f);
            if val.contains(',') || val.contains('"') || val.contains('\n') {
                format!("\"{}\"", val.replace('"', "\"\""))
            } else {
                val
            }
        })
        .collect();
    writeln!(writer, "{}", values.join(","))
}
