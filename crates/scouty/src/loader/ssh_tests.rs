#[cfg(test)]
mod tests {
    use super::super::{is_ssh_url, SshUrl};

    #[test]
    fn parse_basic_url() {
        let url = SshUrl::parse("ssh://host:/var/log/syslog").unwrap();
        assert_eq!(url.user, None);
        assert_eq!(url.host, "host");
        assert_eq!(url.port, None);
        assert_eq!(url.path, "/var/log/syslog");
    }

    #[test]
    fn parse_url_with_user() {
        let url = SshUrl::parse("ssh://admin@prod:/var/log/app.log").unwrap();
        assert_eq!(url.user, Some("admin".to_string()));
        assert_eq!(url.host, "prod");
        assert_eq!(url.port, None);
        assert_eq!(url.path, "/var/log/app.log");
    }

    #[test]
    fn parse_url_with_port() {
        let url = SshUrl::parse("ssh://user@host:2222:/var/log/syslog").unwrap();
        assert_eq!(url.user, Some("user".to_string()));
        assert_eq!(url.host, "host");
        assert_eq!(url.port, Some(2222));
        assert_eq!(url.path, "/var/log/syslog");
    }

    #[test]
    fn parse_url_host_only_with_port() {
        let url = SshUrl::parse("ssh://myserver:22:/tmp/test.log").unwrap();
        assert_eq!(url.user, None);
        assert_eq!(url.host, "myserver");
        assert_eq!(url.port, Some(22));
        assert_eq!(url.path, "/tmp/test.log");
    }

    #[test]
    fn parse_url_ip_address() {
        let url = SshUrl::parse("ssh://root@192.168.1.100:/var/log/messages").unwrap();
        assert_eq!(url.user, Some("root".to_string()));
        assert_eq!(url.host, "192.168.1.100");
        assert_eq!(url.port, None);
        assert_eq!(url.path, "/var/log/messages");
    }

    #[test]
    fn parse_url_missing_scheme() {
        assert!(SshUrl::parse("host:/var/log/syslog").is_err());
    }

    #[test]
    fn parse_url_missing_path() {
        assert!(SshUrl::parse("ssh://host").is_err());
    }

    #[test]
    fn parse_url_relative_path() {
        // Path must be absolute
        assert!(SshUrl::parse("ssh://host:relative/path").is_err());
    }

    #[test]
    fn parse_url_empty_host() {
        assert!(SshUrl::parse("ssh://:/var/log/syslog").is_err());
    }

    #[test]
    fn parse_url_empty_user() {
        assert!(SshUrl::parse("ssh://@host:/var/log/syslog").is_err());
    }

    #[test]
    fn to_url_string_basic() {
        let url = SshUrl {
            user: None,
            host: "prod".to_string(),
            port: None,
            path: "/var/log/syslog".to_string(),
        };
        assert_eq!(url.to_url_string(), "ssh://prod:/var/log/syslog");
    }

    #[test]
    fn to_url_string_full() {
        let url = SshUrl {
            user: Some("admin".to_string()),
            host: "prod".to_string(),
            port: Some(2222),
            path: "/var/log/app.log".to_string(),
        };
        assert_eq!(
            url.to_url_string(),
            "ssh://admin@prod:2222:/var/log/app.log"
        );
    }

    #[test]
    fn roundtrip_parse() {
        let original = "ssh://user@host:2222:/var/log/syslog";
        let url = SshUrl::parse(original).unwrap();
        assert_eq!(url.to_url_string(), original);
    }

    #[test]
    fn is_ssh_url_positive() {
        assert!(is_ssh_url("ssh://host:/path"));
        assert!(is_ssh_url("ssh://user@host:22:/path"));
    }

    #[test]
    fn is_ssh_url_negative() {
        assert!(!is_ssh_url("/var/log/syslog"));
        assert!(!is_ssh_url("http://host/path"));
        assert!(!is_ssh_url(""));
    }

    #[test]
    fn shell_escape_basic() {
        assert_eq!(
            super::super::shell_escape("/var/log/syslog"),
            "'/var/log/syslog'"
        );
    }

    #[test]
    fn shell_escape_with_quotes() {
        assert_eq!(
            super::super::shell_escape("/var/log/it's a file"),
            "'/var/log/it'\\''s a file'"
        );
    }
}
