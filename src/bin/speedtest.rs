use simbiota_tlsh::TLSH;
use std::time::Instant;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let path = &args[1];

    let mut tlshs = Vec::new();
    for line in std::fs::read_to_string(path).unwrap().lines().take(10) {
        let hash = TLSH::from_digest(line);
        tlshs.push(hash);
    }

    println!("using {} for diffing", simbiota_tlsh::tlsh_diff_mode());
    let comparisions = tlshs.len().pow(2);

    let thread_count: usize = std::env::var("SPEEDTEST_THREADS")
        .unwrap_or("1".into())
        .parse()
        .unwrap();
    println!("performing {} comparisions with {} threads", comparisions, thread_count);
    let start = Instant::now();
    std::thread::scope(|scope| {
        for _ in 0..thread_count {
            scope.spawn(|| {
                for h1 in &tlshs {
                    for h2 in &tlshs {
                        let _diff = TLSH::diff(h1, h2);
                        println!("{_diff}");
                    }
                }
            });
        }
    });
    let comparisions = comparisions * thread_count;
    let end = start.elapsed();
    println!("{} comparisions took {:?}", comparisions, end);
    let secs = end.as_secs_f64();
    let comparisions = comparisions as f64;
    let cps = comparisions / secs;
    println!("speed: {:.2} cmp/s", cps);
}
