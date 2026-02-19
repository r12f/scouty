//! Syslog loader — receives syslog messages via UDP.
//!
//! Listens on a UDP socket and collects syslog messages. Since the
//! `LogLoader` trait is synchronous, the loader collects messages for
//! a configurable timeout or until max_messages is reached.

#[cfg(test)]
#[path = "syslog_tests.rs"]
mod syslog_tests;

use crate::traits::{LoaderInfo, LoaderType, LogLoader, Result, ScoutyError};
use std::net::UdpSocket;
use std::time::{Duration, Instant};

/// Configuration for the syslog loader.
#[derive(Debug, Clone)]
pub struct SyslogConfig {
    /// Address to bind to (e.g., "0.0.0.0:514" or "127.0.0.1:1514").
    pub bind_addr: String,
    /// Maximum time to collect messages per `load()` call.
    pub timeout: Duration,
    /// Maximum number of messages to collect per `load()` call.
    /// 0 means unlimited (bounded only by timeout).
    pub max_messages: usize,
}

impl Default for SyslogConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:1514".to_string(),
            timeout: Duration::from_secs(5),
            max_messages: 10000,
        }
    }
}

/// Loads syslog messages from a UDP socket.
#[derive(Debug)]
pub struct SyslogLoader {
    config: SyslogConfig,
    info: LoaderInfo,
    socket: Option<UdpSocket>,
}

impl SyslogLoader {
    /// Create a new SyslogLoader. Does not bind the socket until `load()` is called.
    pub fn new(config: SyslogConfig) -> Self {
        let id = format!("syslog:{}", config.bind_addr);
        Self {
            info: LoaderInfo {
                id,
                loader_type: LoaderType::Syslog,
                multiline_enabled: false,
                sample_lines: Vec::new(),
            },
            config,
            socket: None,
        }
    }

    fn ensure_socket(&mut self) -> Result<&UdpSocket> {
        if self.socket.is_none() {
            let sock = UdpSocket::bind(&self.config.bind_addr).map_err(|e| {
                ScoutyError::Io(std::io::Error::new(
                    e.kind(),
                    format!("Failed to bind syslog socket to {}: {}", self.config.bind_addr, e),
                ))
            })?;
            sock.set_nonblocking(true).map_err(ScoutyError::Io)?;
            self.socket = Some(sock);
        }
        Ok(self.socket.as_ref().unwrap())
    }
}

impl LogLoader for SyslogLoader {
    fn info(&self) -> &LoaderInfo {
        &self.info
    }

    fn load(&mut self) -> Result<Vec<String>> {
        self.ensure_socket()?;
        let socket = self.socket.as_ref().unwrap();
        let timeout = self.config.timeout;
        let max_messages = self.config.max_messages;
        let mut messages = Vec::new();
        let mut buf = [0u8; 8192];
        let start = Instant::now();

        loop {
            if start.elapsed() >= timeout {
                break;
            }

            if max_messages > 0 && messages.len() >= max_messages {
                break;
            }

            match socket.recv_from(&mut buf) {
                Ok((len, _addr)) => {
                    if let Ok(msg) = std::str::from_utf8(&buf[..len]) {
                        let trimmed = msg.trim_end_matches(|c| c == '\n' || c == '\r');
                        if !trimmed.is_empty() {
                            messages.push(trimmed.to_string());
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No data available, sleep briefly then retry
                    if messages.is_empty() {
                        std::thread::sleep(Duration::from_millis(10));
                    } else {
                        // We have some messages and no more coming, return what we have
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
