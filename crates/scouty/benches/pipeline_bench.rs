use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use scouty::loader::file::FileLoader;
use scouty::parser::factory::ParserFactory;
use scouty::parser::group::ParserGroup;
use scouty::parser::unified_syslog_parser::UnifiedSyslogParser;
use scouty::store::LogStore;
use scouty::traits::LogLoader;
use std::io::Write;
use std::sync::Arc;
use tempfile::NamedTempFile;

/// Generate realistic ISO 8601 syslog lines (modern Ubuntu/rsyslog format).
fn generate_iso_syslog_lines(count: usize) -> Vec<String> {
    let hostnames = ["webserver01", "dbhost", "appnode-3"];
    let processes = [
        ("systemd", Some(1)),
        ("sshd", Some(2345)),
        ("nginx", Some(8901)),
        ("cron", Some(567)),
        ("kernel", None),
    ];
    let messages = [
        "Connection accepted from 192.168.1.100 port 52341",
        "Starting daily cleanup of temporary directories",
        "pam_unix(sshd:session): session opened for user admin by (uid=0)",
        "GET /api/v1/status HTTP/1.1 200 OK (12ms)",
        "Out of memory: Kill process 12345 (java) score 900 or sacrifice child",
        "segfault at 0000000000000000 ip 00007f3c2a1b3c40 sp 00007ffd3a2b1c30",
        "TCP: request_sock_TCP: Possible SYN flooding on port 80. Sending cookies.",
        "audit: type=1400 audit(1234567890.123:456): apparmor=\"DENIED\" operation=\"open\"",
        "Started Session 42 of User admin.",
        "Reloading configuration files.",
    ];

    (0..count)
        .map(|i| {
            let sec = (i / 1000) % 60;
            let min = (i / 60000) % 60;
            let hour = (i / 3600000) % 24;
            let micro = (i * 137) % 1_000_000;
            let host = hostnames[i % hostnames.len()];
            let (proc, pid) = &processes[i % processes.len()];
            let msg = messages[i % messages.len()];
            match pid {
                Some(p) => format!(
                    "2026-01-15T{:02}:{:02}:{:02}.{:06}-08:00 {} {}[{}]: {}",
                    hour, min, sec, micro, host, proc, p, msg
                ),
                None => format!(
                    "2026-01-15T{:02}:{:02}:{:02}.{:06}-08:00 {} {}: {}",
                    hour, min, sec, micro, host, proc, msg
                ),
            }
        })
        .collect()
}

/// Generate realistic BSD syslog lines.
fn generate_bsd_syslog_lines(count: usize) -> Vec<String> {
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let processes = ["sshd", "systemd", "cron", "nginx", "kernel"];
    let messages = [
        "Connection from 10.0.0.1 port 22",
        "Started cleanup timer",
        "pam_unix: session opened for user root",
        "GET /health 200",
        "Out of memory: Kill process 999",
    ];

    (0..count)
        .map(|i| {
            let month = months[i % 12];
            let day = (i % 28) + 1;
            let sec = (i / 1000) % 60;
            let min = (i / 60000) % 60;
            let hour = (i / 3600000) % 24;
            let proc = processes[i % processes.len()];
            let msg = messages[i % messages.len()];
            format!(
                "{} {:2} {:02}:{:02}:{:02} webserver01 {}[{}]: {}",
                month,
                day,
                hour,
                min,
                sec,
                proc,
                1000 + (i % 9000),
                msg
            )
        })
        .collect()
}

/// Write lines to a temp file and return it.
fn lines_to_tempfile(lines: &[String]) -> NamedTempFile {
    let mut f = NamedTempFile::new().expect("create tempfile");
    for line in lines {
        writeln!(f, "{}", line).expect("write line");
    }
    f.flush().expect("flush");
    f
}

/// Benchmark: full pipeline (file read → auto-detect → parse → store insert).
fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("e2e_pipeline");

    for &count in &[10_000, 100_000] {
        // ISO syslog
        let iso_lines = generate_iso_syslog_lines(count);
        let iso_file = lines_to_tempfile(&iso_lines);

        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            BenchmarkId::new("iso_syslog", count),
            &iso_file,
            |b, file| {
                b.iter(|| {
                    let mut loader = FileLoader::new(file.path(), false);
                    let lines = loader.load().expect("load");
                    let info = loader.info().clone();
                    let parser = ParserFactory::create_parser_group(&info);
                    let mut store = LogStore::new();
                    for (i, line) in lines.into_iter().enumerate() {
                        if let Some(mut record) = parser.parse(&line, &info.id, &info.id, i as u64)
                        {
                            record.raw = line;
                            store.insert(record);
                        }
                    }
                    store.compact_ooo();
                    black_box(store.len())
                });
            },
        );

        // BSD syslog
        let bsd_lines = generate_bsd_syslog_lines(count);
        let bsd_file = lines_to_tempfile(&bsd_lines);

        group.bench_with_input(
            BenchmarkId::new("bsd_syslog", count),
            &bsd_file,
            |b, file| {
                b.iter(|| {
                    let mut loader = FileLoader::new(file.path(), false);
                    let lines = loader.load().expect("load");
                    let info = loader.info().clone();
                    let parser = ParserFactory::create_parser_group(&info);
                    let mut store = LogStore::new();
                    for (i, line) in lines.into_iter().enumerate() {
                        if let Some(mut record) = parser.parse(&line, &info.id, &info.id, i as u64)
                        {
                            record.raw = line;
                            store.insert(record);
                        }
                    }
                    store.compact_ooo();
                    black_box(store.len())
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: parse-only (no file I/O, no store — measures pure parser throughput).
fn bench_parse_only(c: &mut Criterion) {
    let mut group = c.benchmark_group("e2e_parse_only");
    let count = 100_000;

    let iso_lines = generate_iso_syslog_lines(count);
    let source: Arc<str> = Arc::from("bench");
    let loader_id: Arc<str> = Arc::from("bench");

    let parser = UnifiedSyslogParser::new("bench");

    group.throughput(Throughput::Elements(count as u64));
    group.bench_function("iso_syslog", |b| {
        b.iter(|| {
            let mut parsed = 0usize;
            for (i, line) in iso_lines.iter().enumerate() {
                if parser
                    .parse_shared(line, &source, &loader_id, i as u64)
                    .is_some()
                {
                    parsed += 1;
                }
            }
            black_box(parsed)
        });
    });

    let bsd_lines = generate_bsd_syslog_lines(count);
    group.bench_function("bsd_syslog", |b| {
        b.iter(|| {
            let mut parsed = 0usize;
            for (i, line) in bsd_lines.iter().enumerate() {
                if parser
                    .parse_shared(line, &source, &loader_id, i as u64)
                    .is_some()
                {
                    parsed += 1;
                }
            }
            black_box(parsed)
        });
    });

    group.finish();
}

/// Benchmark: store insert only (measures LogStore insertion throughput).
fn bench_store_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("e2e_store_insert");
    let count = 100_000;

    let iso_lines = generate_iso_syslog_lines(count);
    let parser = UnifiedSyslogParser::new("bench");
    let source: Arc<str> = Arc::from("bench");
    let loader_id: Arc<str> = Arc::from("bench");

    // Pre-parse records
    let records: Vec<_> = iso_lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| parser.parse_shared(line, &source, &loader_id, i as u64))
        .collect();

    group.throughput(Throughput::Elements(records.len() as u64));
    group.bench_function("monotonic_insert", |b| {
        b.iter(|| {
            let mut store = LogStore::new();
            for record in &records {
                store.insert(record.clone());
            }
            black_box(store.len())
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_full_pipeline,
    bench_parse_only,
    bench_store_insert
);
criterion_main!(benches);
