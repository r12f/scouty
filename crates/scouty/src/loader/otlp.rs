//! OTLP (OpenTelemetry Protocol) log loader.
//!
//! Receives OTLP log export requests via HTTP (JSON format).
//! Implements a simple HTTP server that accepts POST requests at
//! `/v1/logs` and collects log records into a buffer.
//!
//! For gRPC support, a `tonic`-based implementation can be added later;
//! this module covers the HTTP/JSON protocol which is widely supported.

#[cfg(test)]
#[path = "otlp_tests.rs"]
mod otlp_tests;

use crate::traits::{LoaderInfo, LoaderType, LogLoader, Result, ScoutyError};
use serde::Deserialize;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::time::{Duration, Instant};

/// Configuration for the OTLP loader.
#[derive(Debug, Clone)]
pub struct OtlpConfig {
    /// Address to bind the HTTP server to (e.g., "127.0.0.1:4318").
    pub bind_addr: String,
    /// Maximum time to collect messages per `load()` call.
    pub timeout: Duration,
    /// Maximum number of log lines to collect per `load()` call.
    pub max_messages: usize,
}

impl Default for OtlpConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:4318".to_string(),
            timeout: Duration::from_secs(5),
            max_messages: 10000,
        }
    }
}

// === OTLP JSON Schema (simplified) ===

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportLogsServiceRequest {
    #[serde(default)]
    resource_logs: Vec<ResourceLogs>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResourceLogs {
    #[serde(default)]
    scope_logs: Vec<ScopeLogs>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ScopeLogs {
    #[serde(default)]
    log_records: Vec<OtlpLogRecord>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OtlpLogRecord {
    #[serde(default)]
    time_unix_nano: Option<String>,
    #[serde(default)]
    severity_text: Option<String>,
    #[serde(default)]
    body: Option<AnyValue>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AnyValue {
    #[serde(default)]
    string_value: Option<String>,
}

/// OTLP HTTP log loader.
#[derive(Debug)]
pub struct OtlpLoader {
    config: OtlpConfig,
    info: LoaderInfo,
    listener: Option<TcpListener>,
}

impl OtlpLoader {
    pub fn new(config: OtlpConfig) -> Self {
        let id = format!("otlp:{}", config.bind_addr);
        Self {
            info: LoaderInfo {
                id,
                loader_type: LoaderType::Otlp,
                multiline_enabled: false,
                sample_lines: Vec::new(),
            },
            config,
            listener: None,
        }
    }

    fn ensure_listener(&mut self) -> Result<()> {
        if self.listener.is_none() {
            let listener = TcpListener::bind(&self.config.bind_addr).map_err(|e| {
                ScoutyError::Io(std::io::Error::new(
                    e.kind(),
                    format!(
                        "Failed to bind OTLP HTTP server to {}: {}",
                        self.config.bind_addr, e
                    ),
                ))
            })?;
            listener.set_nonblocking(true).map_err(ScoutyError::Io)?;
            self.listener = Some(listener);
        }
        Ok(())
    }

    /// Parse an OTLP JSON export request and extract log body strings.
    pub fn parse_otlp_json(json: &str) -> Vec<String> {
        let request: std::result::Result<ExportLogsServiceRequest, _> = serde_json::from_str(json);
        match request {
            Ok(req) => {
                let mut lines = Vec::new();
                for rl in &req.resource_logs {
                    for sl in &rl.scope_logs {
                        for lr in &sl.log_records {
                            let mut parts = Vec::new();

                            if let Some(ts) = &lr.time_unix_nano {
                                parts.push(ts.clone());
                            }
                            if let Some(sev) = &lr.severity_text {
                                parts.push(sev.clone());
                            }
                            if let Some(body) = &lr.body {
                                if let Some(s) = &body.string_value {
                                    parts.push(s.clone());
                                }
                            }

                            if !parts.is_empty() {
                                lines.push(parts.join(" "));
                            }
                        }
                    }
                }
                lines
            }
            Err(_) => Vec::new(),
        }
    }

    fn handle_request(stream: &mut std::net::TcpStream) -> Option<String> {
        stream
            .set_read_timeout(Some(Duration::from_millis(500)))
            .ok()?;

        let mut reader = BufReader::new(stream.try_clone().ok()?);
        let mut request_line = String::new();
        reader.read_line(&mut request_line).ok()?;

        // Read headers
        let mut content_length: usize = 0;
        loop {
            let mut header = String::new();
            reader.read_line(&mut header).ok()?;
            if header.trim().is_empty() {
                break;
            }
            if let Some(val) = header
                .strip_prefix("Content-Length:")
                .or_else(|| header.strip_prefix("content-length:"))
            {
                content_length = val.trim().parse().unwrap_or(0);
            }
        }

        // Only handle POST /v1/logs
        if !request_line.contains("POST") || !request_line.contains("/v1/logs") {
            let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
            let _ = stream.write_all(response.as_bytes());
            return None;
        }

        // Read body
        if content_length == 0 {
            let response = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n";
            let _ = stream.write_all(response.as_bytes());
            return None;
        }

        let mut body = vec![0u8; content_length];
        reader.read_exact(&mut body).ok()?;

        // Send success response
        let response = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}";
        let _ = stream.write_all(response.as_bytes());

        String::from_utf8(body).ok()
    }
}

impl LogLoader for OtlpLoader {
    fn info(&self) -> &LoaderInfo {
        &self.info
    }

    fn load(&mut self) -> Result<Vec<String>> {
        self.ensure_listener()?;
        let listener = self.listener.as_ref().unwrap();
        let timeout = self.config.timeout;
        let max_messages = self.config.max_messages;
        let mut messages = Vec::new();
        let start = Instant::now();

        loop {
            if start.elapsed() >= timeout {
                break;
            }
            if max_messages > 0 && messages.len() >= max_messages {
                break;
            }

            match listener.accept() {
                Ok((mut stream, _addr)) => {
                    if let Some(body) = Self::handle_request(&mut stream) {
                        let parsed = Self::parse_otlp_json(&body);
                        messages.extend(parsed);
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if messages.is_empty() {
                        std::thread::sleep(Duration::from_millis(10));
                    } else {
                        break;
                    }
                }
                Err(e) => return Err(ScoutyError::Io(e)),
            }
        }

        self.info.sample_lines = messages.iter().take(10).cloned().collect();
        Ok(messages)
    }
}
