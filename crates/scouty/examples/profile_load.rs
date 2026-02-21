use scouty::loader::file::FileLoader;
use scouty::parser::factory::ParserFactory;
use scouty::record::LogRecord;
use scouty::store::LogStore;
use scouty::traits::LogLoader;
use std::sync::Arc;
use std::time::Instant;

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/data/syslog".to_string());

    let t0 = Instant::now();

    // Stage 1: File I/O
    let t_io_start = Instant::now();
    let mut loader = FileLoader::new(&path, false);
    let lines = loader.load().expect("load failed");
    let t_io = t_io_start.elapsed();

    // Stage 2: Parser creation
    let info = loader.info().clone();
    let group = ParserFactory::create_parser_group(&info);

    // Stage 3: Parse + Store (with raw line reuse)
    let t_ps_start = Instant::now();
    let mut store = LogStore::new();
    for (i, line) in lines.into_iter().enumerate() {
        if let Some(mut record) = group.parse(&line, &info.id, &info.id, i as u64) {
            record.raw = line;
            store.insert(record);
        }
    }
    store.compact_ooo();
    let t_ps = t_ps_start.elapsed();

    // Stage 4: Arc clone out
    let t_clone_start = Instant::now();
    let records: Vec<Arc<LogRecord>> = store.iter_arc().cloned().collect();
    let t_clone = t_clone_start.elapsed();

    let total = t0.elapsed();
    let n = records.len();

    eprintln!("=== E2E Performance ===");
    eprintln!(
        "File I/O:        {:>8.1}ms ({:>5.1}%)",
        t_io.as_secs_f64() * 1000.0,
        t_io.as_secs_f64() / total.as_secs_f64() * 100.0
    );
    eprintln!(
        "Parse+Store:     {:>8.1}ms ({:>5.1}%)",
        t_ps.as_secs_f64() * 1000.0,
        t_ps.as_secs_f64() / total.as_secs_f64() * 100.0
    );
    eprintln!(
        "Arc clone:       {:>8.1}ms ({:>5.1}%)",
        t_clone.as_secs_f64() * 1000.0,
        t_clone.as_secs_f64() / total.as_secs_f64() * 100.0
    );
    eprintln!("Total:           {:>8.1}ms", total.as_secs_f64() * 1000.0);
    eprintln!(
        "Records: {} | Throughput: {:.1}M rec/sec",
        n,
        n as f64 / total.as_secs_f64() / 1e6
    );
}
