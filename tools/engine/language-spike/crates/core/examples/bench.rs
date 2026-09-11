// SPDX-License-Identifier: Apache-2.0
//! **Item 4 of the spike.** A warm change-scoped run over a 1,000-document
//! corpus, against the 200 ms hook budget from spec 6.
//!
//! The number that matters is not the total. It is the breakdown, because the
//! breakdown says which part would grow with the corpus. Run with:
//!
//!     cargo run --release --example bench
//!
//! A debug build is roughly twenty times slower and says nothing about the
//! budget.

use headwater_core::corpus::{self, synth};
use headwater_core::default_registry;
use headwater_core::model::Graph;
use headwater_core::runner::{Cache, Runner};
use std::time::{Duration, Instant};

const N: usize = 1000;
const TAXONOMY_HASH: u128 = 0xfeed_face_dead_beef;

fn median(mut xs: Vec<Duration>) -> Duration {
    xs.sort();
    xs[xs.len() / 2]
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

fn main() {
    let files = synth::corpus(N);
    let total_bytes: usize = files.iter().map(|(_, s)| s.len()).sum();
    let registry = default_registry();
    let runner = Runner::new(&registry, TAXONOMY_HASH);

    println!("corpus: {N} documents, {} KiB", total_bytes / 1024);
    println!(
        "checks: {} document, {} edge, {} corpus",
        registry.document.len(),
        registry.edge.len(),
        registry.corpus.len()
    );
    println!();

    // ---- cold: parse everything, build the graph, run every check ----
    let mut cold_parse = Vec::new();
    let mut cold_build = Vec::new();
    let mut cold_check = Vec::new();
    let mut cold_total = Vec::new();

    for _ in 0..5 {
        let t0 = Instant::now();
        let loaded = corpus::load(&files);
        let t1 = Instant::now();
        // `load` parses and builds together, so time a rebuild separately to
        // attribute the cost.
        let docs = loaded.graph.into_documents();
        let graph = Graph::build(docs);
        let t2 = Instant::now();
        let mut cache = Cache::new();
        let report = runner.run_full(&graph, &mut cache);
        let t3 = Instant::now();

        cold_parse.push(t1 - t0);
        cold_build.push(t2 - t1);
        cold_check.push(t3 - t2);
        cold_total.push(t3 - t0);
        std::hint::black_box(report.findings.len());
    }

    println!("cold full run (spec 6 budget: 5 s)");
    println!("  parse + classify + graph : {:7.2} ms", ms(median(cold_parse)));
    println!("  graph rebuild alone      : {:7.2} ms", ms(median(cold_build)));
    println!("  all checks               : {:7.2} ms", ms(median(cold_check)));
    println!("  TOTAL                    : {:7.2} ms", ms(median(cold_total)));
    println!();

    // Report what the run actually found, so the bench is not measuring a
    // pipeline that quietly does nothing.
    let loaded = corpus::load(&files);
    let mut cache = Cache::new();
    let full = runner.run_full(&loaded.graph, &mut cache);
    println!(
        "  instances created {}, evaluated {}, findings {}",
        full.instances_created,
        full.instances_evaluated,
        full.findings.len()
    );
    println!("  first three findings:");
    for f in full.findings.iter().take(3) {
        println!("    {}", f.render());
    }
    println!();

    // ---- warm change-scoped: one document edited ----
    let mut docs = loaded.graph.into_documents();
    let mut reparse = Vec::new();
    let mut rebuild = Vec::new();
    let mut checks = Vec::new();
    let mut totals = Vec::new();
    let mut invalidated = 0usize;

    for iter in 0..30 {
        // Edit a different document each time so the cache is not measured on
        // one lucky entry.
        let idx = (iter * 37) % N;
        let (path, source) = &files[idx];
        // A real edit: change the body, which changes the content hash.
        let edited = source.replace("## Context", "## Context and background");

        let t0 = Instant::now();
        let doc = corpus::parse_document(path, &edited).expect("edited document parses");
        let t1 = Instant::now();
        docs[idx] = doc;
        let graph = Graph::build(std::mem::take(&mut docs));
        let t2 = Instant::now();
        let report = runner.run_changed(&graph, &mut cache, &[idx]);
        let t3 = Instant::now();

        invalidated = report.instances_evaluated;
        reparse.push(t1 - t0);
        rebuild.push(t2 - t1);
        checks.push(t3 - t2);
        totals.push(t3 - t0);

        docs = graph.into_documents();
        // Put the original back so the next iteration starts from a known state.
        docs[idx] = corpus::parse_document(path, source).expect("original parses");
        std::hint::black_box(report.findings.len());
    }

    println!("warm change-scoped run, one document edited (spec 6 budget: 200 ms)");
    println!("  re-parse the one file    : {:7.3} ms", ms(median(reparse)));
    println!("  rebuild graph + index    : {:7.3} ms", ms(median(rebuild)));
    println!("  invalidated checks       : {:7.3} ms", ms(median(checks)));
    let total = ms(median(totals));
    println!("  TOTAL                    : {total:7.3} ms");
    println!("  instances re-evaluated   : {invalidated}");
    println!();

    let verdict = if total < 200.0 { "PASS" } else { "FAIL" };
    println!("verdict: {verdict} — {total:.3} ms against a 200 ms budget");
}
