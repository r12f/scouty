use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use scouty::parser::unified_syslog_parser::UnifiedSyslogParser;
use std::sync::Arc;

const BSD_LINE: &str =
    "Feb 19 14:23:45 myhost myapp[12345]: This is a log message with some content here";
const EXTENDED_LINE: &str =
    "2025 Nov 24 17:56:03.073872 BSL-0101 NOTICE restapi#root: message repeated 47 times with extra content";
const ISO_LINE: &str =
    "2026-02-15T00:00:08.954827-08:00 r12f-ms01 systemd[1]: rsyslog.service: Sent signal SIGHUP to main process 1181";

fn generate_bsd_lines(count: usize) -> Vec<String> {
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let procs = ["kernel", "sshd", "systemd", "cron", "nginx", "postfix"];
    let msgs = [
        "Connection accepted from 192.168.1.100",
        "pam_unix(sshd:session): session opened for user admin",
        "Out of memory: Kill process 12345 (java) score 900",
        "TCP: request_sock_TCP: Possible SYN flooding on port 80",
    ];
    (0..count)
        .map(|i| {
            format!(
                "{} {:2} {:02}:{:02}:{:02} myhost {}[{}]: {}",
                months[i % 12],
                (i % 28) + 1,
                i % 24,
                i % 60,
                i % 60,
                procs[i % procs.len()],
                1000 + (i % 50000),
                msgs[i % msgs.len()]
            )
        })
        .collect()
}

fn generate_extended_lines(count: usize) -> Vec<String> {
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let levels = ["INFO", "NOTICE", "WARNING", "ERR"];
    let procs = [
        "restapi#root",
        "pmon#stormond[37]",
        "dockerd[871]",
        "memory_checker",
    ];
    (0..count)
        .map(|i| {
            format!(
                "2025 {} {:2} {:02}:{:02}:{:02}.{:06} BSL-0101 {} {}: sample message number {}",
                months[i % 12],
                (i % 28) + 1,
                i % 24,
                i % 60,
                i % 60,
                i % 999999,
                levels[i % levels.len()],
                procs[i % procs.len()],
                i
            )
        })
        .collect()
}

fn generate_iso_lines(count: usize) -> Vec<String> {
    let procs = [
        "systemd[1]",
        "rsyslogd",
        "sshd[1234]",
        "cron[999]",
        "nginx[80]",
    ];
    (0..count)
        .map(|i| {
            format!(
                "2026-{:02}-{:02}T{:02}:{:02}:{:02}.{:06}-08:00 r12f-ms01 {}: log entry number {}",
                (i % 12) + 1,
                (i % 28) + 1,
                i % 24,
                i % 60,
                i % 60,
                i % 999999,
                procs[i % procs.len()],
                i
            )
        })
        .collect()
}

fn bench_unified_single(c: &mut Criterion) {
    let parser = UnifiedSyslogParser::new("bench");
    let source: Arc<str> = Arc::from("bench");
    let loader_id: Arc<str> = Arc::from("bench");

    let mut group = c.benchmark_group("unified_single");

    group.bench_function("bsd", |b| {
        b.iter(|| black_box(parser.parse_shared(black_box(BSD_LINE), &source, &loader_id, 0)))
    });

    group.bench_function("extended", |b| {
        b.iter(|| black_box(parser.parse_shared(black_box(EXTENDED_LINE), &source, &loader_id, 0)))
    });

    group.bench_function("iso", |b| {
        b.iter(|| black_box(parser.parse_shared(black_box(ISO_LINE), &source, &loader_id, 0)))
    });

    group.finish();
}

fn bench_unified_100k(c: &mut Criterion) {
    let parser = UnifiedSyslogParser::new("bench");
    let source: Arc<str> = Arc::from("bench");
    let loader_id: Arc<str> = Arc::from("bench");

    let bsd_lines = generate_bsd_lines(100_000);
    let ext_lines = generate_extended_lines(100_000);
    let iso_lines = generate_iso_lines(100_000);

    let mut group = c.benchmark_group("unified_100k");
    group.throughput(Throughput::Elements(100_000));
    group.sample_size(10);

    group.bench_function("bsd", |b| {
        b.iter(|| {
            for (i, line) in bsd_lines.iter().enumerate() {
                black_box(parser.parse_shared(line, &source, &loader_id, i as u64));
            }
        })
    });

    group.bench_function("extended", |b| {
        b.iter(|| {
            for (i, line) in ext_lines.iter().enumerate() {
                black_box(parser.parse_shared(line, &source, &loader_id, i as u64));
            }
        })
    });

    group.bench_function("iso", |b| {
        b.iter(|| {
            for (i, line) in iso_lines.iter().enumerate() {
                black_box(parser.parse_shared(line, &source, &loader_id, i as u64));
            }
        })
    });

    group.finish();
}

criterion_group!(benches, bench_unified_single, bench_unified_100k);
criterion_main!(benches);
