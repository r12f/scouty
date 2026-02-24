//! SSH-based remote log loader — loads log lines from a remote host via SSH.

#[cfg(test)]
#[path = "ssh_tests.rs"]
mod ssh_tests;

use crate::traits::{LoaderInfo, LoaderType, LogLoader, Result, ScoutyError};
use std::process::{Command, Stdio};

/// Parsed SSH URL components.
#[derive(Debug, Clone, PartialEq)]
pub struct SshUrl {
    /// Username (optional, defaults to current user).
    pub user: Option<String>,
    /// Hostname or IP.
    pub host: String,
    /// Port (optional, defaults to 22).
    pub port: Option<u16>,
    /// Absolute path on the remote host.
    pub path: String,
}

impl SshUrl {
    /// Parse an SSH URL: `ssh://[user@]host[:port]:/absolute/path`
    pub fn parse(url: &str) -> std::result::Result<Self, String> {
        let rest = url
            .strip_prefix("ssh://")
            .ok_or_else(|| "URL must start with ssh://".to_string())?;

        if rest.is_empty() {
            return Err("Empty SSH URL".to_string());
        }

        // Split user@host[:port]:/path
        // Find the colon-slash that separates host part from path
        let colon_slash_pos = rest
            .find(":/")
            .ok_or_else(|| "Missing ':/' separator between host and path".to_string())?;

        let host_part = &rest[..colon_slash_pos];
        let path = &rest[colon_slash_pos + 1..]; // includes leading /

        if path.is_empty() || !path.starts_with('/') {
            return Err("Path must be absolute (start with /)".to_string());
        }

        // Parse user@host[:port]
        let (user, host_and_port) = if let Some(at_pos) = host_part.find('@') {
            let user = &host_part[..at_pos];
            if user.is_empty() {
                return Err("Empty username in SSH URL".to_string());
            }
            (Some(user.to_string()), &host_part[at_pos + 1..])
        } else {
            (None, host_part)
        };

        // Parse host[:port]
        let (host, port) = if let Some(colon_pos) = host_and_port.rfind(':') {
            let port_str = &host_and_port[colon_pos + 1..];
            match port_str.parse::<u16>() {
                Ok(p) => (host_and_port[..colon_pos].to_string(), Some(p)),
                Err(_) => {
                    // Not a valid port number, treat whole thing as host
                    (host_and_port.to_string(), None)
                }
            }
        } else {
            (host_and_port.to_string(), None)
        };

        if host.is_empty() {
            return Err("Empty hostname in SSH URL".to_string());
        }

        Ok(SshUrl {
            user,
            host,
            port,
            path: path.to_string(),
        })
    }

    /// Format as display string: `ssh://[user@]host[:port]:/path`
    pub fn to_url_string(&self) -> String {
        let mut s = "ssh://".to_string();
        if let Some(ref user) = self.user {
            s.push_str(user);
            s.push('@');
        }
        s.push_str(&self.host);
        if let Some(port) = self.port {
            s.push(':');
            s.push_str(&port.to_string());
        }
        s.push(':');
        s.push_str(&self.path);
        s
    }

    /// Build SSH destination string for the ssh command (user@host or host).
    fn ssh_destination(&self) -> String {
        match &self.user {
            Some(user) => format!("{}@{}", user, self.host),
            None => self.host.clone(),
        }
    }
}

/// Check if a string looks like an SSH URL.
pub fn is_ssh_url(s: &str) -> bool {
    s.starts_with("ssh://")
}

/// Loads log lines from a remote host via SSH.
#[derive(Debug)]
pub struct SshLoader {
    url: SshUrl,
    info: LoaderInfo,
    connect_timeout: u32,
}

impl SshLoader {
    /// Create a new SSH loader.
    ///
    /// `connect_timeout` is in seconds (default: 10).
    pub fn new(url: SshUrl, connect_timeout: u32) -> Self {
        let id = url.to_url_string();
        Self {
            info: LoaderInfo {
                id,
                loader_type: LoaderType::TextFile,
                multiline_enabled: false,
                sample_lines: Vec::new(),
                file_mod_year: None,
            },
            url,
            connect_timeout,
        }
    }
}

impl LogLoader for SshLoader {
    fn info(&self) -> &LoaderInfo {
        &self.info
    }

    fn load(&mut self) -> Result<Vec<String>> {
        let mut cmd = Command::new("ssh");

        // Connection timeout
        cmd.arg("-o")
            .arg(format!("ConnectTimeout={}", self.connect_timeout));

        // Batch mode — no interactive prompts
        cmd.arg("-o").arg("BatchMode=yes");

        // Port
        if let Some(port) = self.url.port {
            cmd.arg("-p").arg(port.to_string());
        }

        // Destination
        cmd.arg(self.url.ssh_destination());

        // Remote command: cat the file
        cmd.arg(format!("cat {}", shell_escape(&self.url.path)));

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let child = cmd.spawn().map_err(|e| {
            ScoutyError::Io(std::io::Error::new(
                e.kind(),
                format!(
                    "SSH: Failed to spawn ssh command for {}: {}",
                    self.url.to_url_string(),
                    e
                ),
            ))
        })?;

        let output = child.wait_with_output().map_err(|e| {
            ScoutyError::Io(std::io::Error::new(
                e.kind(),
                format!(
                    "SSH: Failed to read output from {}: {}",
                    self.url.to_url_string(),
                    e
                ),
            ))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let msg = if stderr.contains("Connection refused") {
                format!(
                    "SSH: Connection refused to {}:{}",
                    self.url.host,
                    self.url.port.unwrap_or(22)
                )
            } else if stderr.contains("Permission denied") {
                format!(
                    "SSH: Permission denied for {} (key-based auth required)",
                    self.url.ssh_destination()
                )
            } else if stderr.contains("Could not resolve hostname") {
                format!("SSH: Could not resolve hostname '{}'", self.url.host)
            } else if stderr.contains("No such file") {
                format!("SSH: Remote file not found: {}", self.url.path)
            } else {
                format!(
                    "SSH: Command failed for {}: {}",
                    self.url.to_url_string(),
                    stderr.trim()
                )
            };
            return Err(ScoutyError::Io(std::io::Error::other(msg)));
        }

        let content = String::from_utf8(output.stdout).map_err(|e| {
            ScoutyError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "SSH: Remote file '{}' contains invalid UTF-8: {}",
                    self.url.path, e
                ),
            ))
        })?;

        let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        self.info.sample_lines = lines.iter().take(10).cloned().collect();

        Ok(lines)
    }
}

/// Simple shell escaping for the remote path (single-quote wrapping).
fn shell_escape(s: &str) -> String {
    // Wrap in single quotes, escaping any single quotes in the path
    format!("'{}'", s.replace('\'', "'\\''"))
}
