#[cfg(test)]
mod tests {
    use crate::loader::syslog::{SyslogConfig, SyslogLoader};
    use crate::traits::LogLoader;
    use std::net::UdpSocket;
    use std::time::Duration;

    fn find_free_port() -> u16 {
        let sock = UdpSocket::bind("127.0.0.1:0").unwrap();
        sock.local_addr().unwrap().port()
    }

    #[test]
    fn test_loader_info() {
        let loader = SyslogLoader::new(SyslogConfig {
            bind_addr: "127.0.0.1:0".to_string(),
            ..Default::default()
        });
        let info = loader.info();
        assert_eq!(info.loader_type, crate::traits::LoaderType::Syslog);
        assert!(info.id.starts_with("syslog:"));
    }

    #[test]
    fn test_load_no_messages_timeout() {
        let port = find_free_port();
        let mut loader = SyslogLoader::new(SyslogConfig {
            bind_addr: format!("127.0.0.1:{}", port),
            timeout: Duration::from_millis(100),
            max_messages: 100,
        });
        let messages = loader.load().unwrap();
        assert!(messages.is_empty());
    }

    #[test]
    fn test_load_receives_messages() {
        let port = find_free_port();
        let addr = format!("127.0.0.1:{}", port);

        let mut loader = SyslogLoader::new(SyslogConfig {
            bind_addr: addr.clone(),
            timeout: Duration::from_secs(2),
            max_messages: 10,
        });

        // Send some messages from a separate thread
        let send_addr = addr.clone();
        let handle = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
            for i in 0..3 {
                let msg = format!("<14>Jan 15 10:00:00 host test: message {}", i);
                sender.send_to(msg.as_bytes(), &send_addr).unwrap();
            }
        });

        let messages = loader.load().unwrap();
        handle.join().unwrap();

        assert_eq!(messages.len(), 3);
        assert!(messages[0].contains("message 0"));
        assert!(messages[2].contains("message 2"));
    }

    #[test]
    fn test_max_messages_limit() {
        let port = find_free_port();
        let addr = format!("127.0.0.1:{}", port);

        let mut loader = SyslogLoader::new(SyslogConfig {
            bind_addr: addr.clone(),
            timeout: Duration::from_secs(5),
            max_messages: 2,
        });

        let send_addr = addr.clone();
        let handle = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
            for i in 0..5 {
                let msg = format!("<14>test message {}", i);
                sender.send_to(msg.as_bytes(), &send_addr).unwrap();
                std::thread::sleep(Duration::from_millis(5));
            }
        });

        let messages = loader.load().unwrap();
        handle.join().unwrap();

        assert!(messages.len() <= 2);
    }

    #[test]
    fn test_sample_lines_populated() {
        let port = find_free_port();
        let addr = format!("127.0.0.1:{}", port);

        let mut loader = SyslogLoader::new(SyslogConfig {
            bind_addr: addr.clone(),
            timeout: Duration::from_secs(2),
            max_messages: 100,
        });

        let send_addr = addr.clone();
        let handle = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
            for i in 0..15 {
                let msg = format!("<14>line {}", i);
                sender.send_to(msg.as_bytes(), &send_addr).unwrap();
            }
        });

        loader.load().unwrap();
        handle.join().unwrap();

        assert!(loader.info().sample_lines.len() <= 10);
    }
}
