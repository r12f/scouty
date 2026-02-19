#[cfg(test)]
mod tests {
    use crate::loader::otlp::{OtlpConfig, OtlpLoader};
    use crate::traits::LogLoader;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::time::Duration;

    fn find_free_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    }

    #[test]
    fn test_loader_info() {
        let loader = OtlpLoader::new(OtlpConfig::default());
        let info = loader.info();
        assert_eq!(info.loader_type, crate::traits::LoaderType::Otlp);
        assert!(info.id.starts_with("otlp:"));
    }

    #[test]
    fn test_parse_otlp_json() {
        let json = r#"{
            "resourceLogs": [{
                "scopeLogs": [{
                    "logRecords": [
                        {
                            "timeUnixNano": "1705312200000000000",
                            "severityText": "ERROR",
                            "body": { "stringValue": "Something failed" }
                        },
                        {
                            "severityText": "INFO",
                            "body": { "stringValue": "All good" }
                        }
                    ]
                }]
            }]
        }"#;

        let lines = OtlpLoader::parse_otlp_json(json);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("ERROR"));
        assert!(lines[0].contains("Something failed"));
        assert!(lines[1].contains("INFO"));
        assert!(lines[1].contains("All good"));
    }

    #[test]
    fn test_parse_otlp_json_empty() {
        let json = r#"{"resourceLogs": []}"#;
        let lines = OtlpLoader::parse_otlp_json(json);
        assert!(lines.is_empty());
    }

    #[test]
    fn test_parse_otlp_json_invalid() {
        let lines = OtlpLoader::parse_otlp_json("not json");
        assert!(lines.is_empty());
    }

    #[test]
    fn test_load_no_connections_timeout() {
        let port = find_free_port();
        let mut loader = OtlpLoader::new(OtlpConfig {
            bind_addr: format!("127.0.0.1:{}", port),
            timeout: Duration::from_millis(100),
            max_messages: 100,
        });
        let messages = loader.load().unwrap();
        assert!(messages.is_empty());
    }

    #[test]
    fn test_load_receives_http_post() {
        let port = find_free_port();
        let addr = format!("127.0.0.1:{}", port);

        let mut loader = OtlpLoader::new(OtlpConfig {
            bind_addr: addr.clone(),
            timeout: Duration::from_secs(3),
            max_messages: 100,
        });

        let send_addr = addr.clone();
        let handle = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(100));
            let body = r#"{"resourceLogs":[{"scopeLogs":[{"logRecords":[{"severityText":"WARN","body":{"stringValue":"test warning"}}]}]}]}"#;
            let request = format!(
                "POST /v1/logs HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            let mut stream = TcpStream::connect(&send_addr).unwrap();
            stream.write_all(request.as_bytes()).unwrap();
            // Read response
            let mut response = vec![0u8; 1024];
            let _ = stream.read(&mut response);
        });

        let messages = loader.load().unwrap();
        handle.join().unwrap();

        assert_eq!(messages.len(), 1);
        assert!(messages[0].contains("WARN"));
        assert!(messages[0].contains("test warning"));
    }
}
