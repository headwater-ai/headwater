// SPDX-License-Identifier: Apache-2.0
//! Run the spike over a list of paths and print one TSV row per finding.
//!
//!     harper-spike [--raw] <root> <list-file>
//!
//! The rows go to standard output and the timings to standard error, so the
//! determinism check compares standard output alone.

use std::time::Instant;

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let raw = args.first().is_some_and(|a| a == "--raw");
    if raw {
        args.remove(0);
    }
    let [root, list] = args.as_slice() else {
        eprintln!("usage: harper-spike [--raw] <root> <list-file>");
        std::process::exit(2);
    };
    let paths: Vec<String> = std::fs::read_to_string(list)
        .expect("the list file")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect();
    let sources: Vec<(String, String)> = paths
        .iter()
        .map(|path| {
            let source = std::fs::read_to_string(format!("{root}/{path}"))
                .unwrap_or_else(|e| panic!("{path}: {e}"));
            (path.clone(), source)
        })
        .collect();

    let built = Instant::now();
    let mut linter = harper_spike::linter();
    let build = built.elapsed();

    let mut rows = Vec::new();
    let mut passes = Vec::new();
    // Pass one is cold: the linter's per-chunk cache is empty. Pass two runs
    // the same inputs through the same linter, which is the warm number. Only
    // pass one's rows are printed; the two are compared below.
    for pass in 0..2 {
        let started = Instant::now();
        let mut these = Vec::new();
        for (path, source) in &sources {
            if raw {
                these.extend(harper_spike::raw(path, source, &mut linter));
            } else {
                these.extend(harper_spike::authored(path, source, &mut linter));
            }
        }
        passes.push(started.elapsed());
        if pass == 0 {
            rows = these;
        } else if these != rows {
            eprintln!("harper-spike: the warm pass differs from the cold pass");
            std::process::exit(1);
        }
    }

    println!("{}", harper_spike::HEADER);
    for row in &rows {
        println!("{}", row.tsv());
    }
    let n = sources.len().max(1) as f64;
    eprintln!(
        "harper-spike: mode={} documents={} findings={} linter_build_ms={:.1} cold_ms={:.1} warm_ms={:.1} cold_per_doc_ms={:.2} warm_per_doc_ms={:.2}",
        if raw { "raw" } else { "authored" },
        sources.len(),
        rows.len(),
        build.as_secs_f64() * 1e3,
        passes[0].as_secs_f64() * 1e3,
        passes[1].as_secs_f64() * 1e3,
        passes[0].as_secs_f64() * 1e3 / n,
        passes[1].as_secs_f64() * 1e3 / n,
    );
}
