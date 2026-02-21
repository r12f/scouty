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

    // Full pipeline as App::load_file now does it (single load)
    let t0 = Instant::now();

    // Stage 1: File I/O
    let t_io_start = Instant::now();
    let mut loader = FileLoader::new(&path, false);
    let lines = loader.load().expect("load failed");
    let t_io = t_io_start.elapsed();

    // Stage 2: Parser creation
    let t_pc_start = Instant::now();
    let info = loader.info().clone();
    let group = ParserFactory::create_parser_group(&info);
    let t_pc = t_pc_start.elapsed();

    // Stage 3: Parse + Store insert
    let t_parse_start = Instant::now();
    let mut store = LogStore::new();
    for (i, line) in lines.iter().enumerate() {
        if let Some(record) = group.parse(line, &info.id, &info.id, i as u64) {
            store.insert(record);
        }
    }
    let t_parse_store = t_parse_start.elapsed();

    // Stage 4: Flush OOO buffer and Arc clone out
    let t_clone_start = Instant::now();
    store.compact_ooo();
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
        "Parser create:   {:>8.1}ms ({:>5.1}%)",
        t_pc.as_secs_f64() * 1000.0,
        t_pc.as_secs_f64() / total.as_secs_f64() * 100.0
    );
    eprintln!(
        "Parse+Store:     {:>8.1}ms ({:>5.1}%)",
        t_parse_store.as_secs_f64() * 1000.0,
        t_parse_store.as_secs_f64() / total.as_secs_f64() * 100.0
    );
    eprintln!(
        "Arc clone out:   {:>8.1}ms ({:>5.1}%)",
        t_clone.as_secs_f64() * 1000.0,
        t_clone.as_secs_f64() / total.as_secs_f64() * 100.0
    );
    eprintln!("Total:           {:>8.1}ms", total.as_secs_f64() * 1000.0);
    eprintln!(
        "\nRecords: {} | Throughput: {:.1}M rec/sec",
        n,
        n as f64 / total.as_secs_f64() / 1_000_000.0
    );
}
