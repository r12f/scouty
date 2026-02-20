use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use scouty::parser::regex_parser::RegexParser;
use scouty::parser::syslog_parser::SyslogParser;
use scouty::traits::LogParser;
use std::sync::Arc;

/// Generate realistic syslog lines.
fn generate_syslog_lines(count: usize) -> Vec<String> {
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let facilities = ["kernel", "sshd", "systemd", "cron", "nginx", "postfix"];
    let messages = [
        "Connection accepted from 192.168.1.100",
        "Starting daily cleanup of temporary directories",
        "pam_unix(sshd:session): session opened for user admin",
        "New USB device found, idVendor=0781, idProduct=5567",
        "Out of memory: Kill process 12345 (java) score 900",
        "segfault at 0000000000000000 ip 00007f3c2a1b3c40",
        "TCP: request_sock_TCP: Possible SYN flooding on port 80",
        "audit: type=1400 audit(1234567890.123:456): apparmor=DENIED",
    ];

    (0..count)
        .map(|i| {
            let month = months[i % 12];
            let day = (i % 28) + 1;
            let hour = i % 24;
            let min = i % 60;
            let sec = i % 60;
            let facility = facilities[i % facilities.len()];
            let pid = 1000 + (i % 50000);
            let msg = messages[i % messages.len()];
            format!(
                "{} {:2} {:02}:{:02}:{:02} myhost {}[{}]: {}",
                month, day, hour, min, sec, facility, pid, msg
            )
        })
        .collect()
}

fn create_syslog_parser() -> RegexParser {
    RegexParser::new(
        "syslog",
        r"^(?P<timestamp>[A-Z][a-z]{2}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2})\s+\S+\s+(?P<process>[^\[]+)\[(?P<pid>\d+)\]:\s+(?P<message>.+)$",
        Some("%b %e %H:%M:%S".to_string()),
    )
    .unwrap()
}

fn bench_parse_syslog_single(c: &mut Criterion) {
    let parser = create_syslog_parser();
    let line = "Feb 19 14:23:45 myhost myapp[12345]: This is a log message";

    let mut group = c.benchmark_group("parse_syslog_single");

    // Using LogParser trait (allocates Arc each call)
    group.bench_function("trait_api", |b| {
        b.iter(|| {
            black_box(parser.parse(
                black_box(line),
                black_box("test"),
                black_box("loader"),
                black_box(0),
            ))
        })
    });

    // Using parse_shared (pre-allocated Arc)
    let source: Arc<str> = Arc::from("test");
    let loader_id: Arc<str> = Arc::from("loader");
    group.bench_function("shared_api", |b| {
        b.iter(|| {
            black_box(parser.parse_shared(
                black_box(line),
                black_box(&source),
                black_box(&loader_id),
                black_box(0),
            ))
        })
    });

    group.finish();
}

fn bench_parse_syslog_batch_1k(c: &mut Criterion) {
    let parser = create_syslog_parser();
    let lines = generate_syslog_lines(1_000);
    let source: Arc<str> = Arc::from("test");
    let loader_id: Arc<str> = Arc::from("loader");

    let mut group = c.benchmark_group("parse_syslog_batch");
    group.throughput(Throughput::Elements(1_000));

    // Manual loop with parse_shared
    group.bench_function("1k_shared", |b| {
        b.iter(|| {
            for (i, line) in lines.iter().enumerate() {
                black_box(parser.parse_shared(line, &source, &loader_id, i as u64));
            }
        })
    });

    // Batch API
    let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
    group.bench_function("1k_batch_api", |b| {
        b.iter(|| {
            black_box(parser.parse_batch(&line_refs, &source, &loader_id, 0));
        })
    });

    // Batch owned API
    group.bench_function("1k_batch_owned", |b| {
        b.iter(|| {
            let owned: Vec<String> = lines.clone();
            black_box(parser.parse_batch_owned(owned, &source, &loader_id, 0));
        })
    });

    group.finish();
}

fn bench_parse_syslog_batch_100k(c: &mut Criterion) {
    let parser = create_syslog_parser();
    let lines = generate_syslog_lines(100_000);
    let source: Arc<str> = Arc::from("test");
    let loader_id: Arc<str> = Arc::from("loader");

    let mut group = c.benchmark_group("parse_syslog_100k");
    group.throughput(Throughput::Elements(100_000));
    group.sample_size(10);

    // Manual loop
    group.bench_function("shared", |b| {
        b.iter(|| {
            for (i, line) in lines.iter().enumerate() {
                black_box(parser.parse_shared(line, &source, &loader_id, i as u64));
            }
        })
    });

    // Batch API
    let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
    group.bench_function("batch_api", |b| {
        b.iter(|| {
            black_box(parser.parse_batch(&line_refs, &source, &loader_id, 0));
        })
    });

    // Batch owned
    group.bench_function("batch_owned", |b| {
        b.iter(|| {
            let owned: Vec<String> = lines.clone();
            black_box(parser.parse_batch_owned(owned, &source, &loader_id, 0));
        })
    });

    group.finish();
}

fn bench_syslog_parser_single(c: &mut Criterion) {
    let regex_parser = create_syslog_parser();
    let syslog_parser = SyslogParser::new("syslog");
    let line = "Feb 19 14:23:45 myhost myapp[12345]: This is a log message";
    let source: Arc<str> = Arc::from("test");
    let loader_id: Arc<str> = Arc::from("loader");

    let mut group = c.benchmark_group("syslog_vs_regex_single");

    group.bench_function("regex_shared", |b| {
        b.iter(|| {
            black_box(regex_parser.parse_shared(
                black_box(line),
                black_box(&source),
                black_box(&loader_id),
                black_box(0),
            ))
        })
    });

    group.bench_function("syslog_shared", |b| {
        b.iter(|| {
            black_box(syslog_parser.parse_shared(
                black_box(line),
                black_box(&source),
                black_box(&loader_id),
                black_box(0),
            ))
        })
    });

    group.finish();
}

fn bench_syslog_parser_100k(c: &mut Criterion) {
    let regex_parser = create_syslog_parser();
    let syslog_parser = SyslogParser::new("syslog");
    let lines = generate_syslog_lines(100_000);
    let source: Arc<str> = Arc::from("test");
    let loader_id: Arc<str> = Arc::from("loader");

    let mut group = c.benchmark_group("syslog_vs_regex_100k");
    group.throughput(Throughput::Elements(100_000));
    group.sample_size(10);

    group.bench_function("regex_shared", |b| {
        b.iter(|| {
            for (i, line) in lines.iter().enumerate() {
                black_box(regex_parser.parse_shared(line, &source, &loader_id, i as u64));
            }
        })
    });

    group.bench_function("syslog_shared", |b| {
        b.iter(|| {
            for (i, line) in lines.iter().enumerate() {
                black_box(syslog_parser.parse_shared(line, &source, &loader_id, i as u64));
            }
        })
    });

    let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
    group.bench_function("syslog_batch", |b| {
        b.iter(|| {
            black_box(syslog_parser.parse_batch(&line_refs, &source, &loader_id, 0));
        })
    });

    group.bench_function("syslog_batch_owned", |b| {
        b.iter(|| {
            let owned: Vec<String> = lines.clone();
            black_box(syslog_parser.parse_batch_owned(owned, &source, &loader_id, 0));
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_parse_syslog_single,
    bench_parse_syslog_batch_1k,
    bench_parse_syslog_batch_100k,
    bench_syslog_parser_single,
    bench_syslog_parser_100k,
);
criterion_main!(benches);
