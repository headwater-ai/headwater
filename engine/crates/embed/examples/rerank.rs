// SPDX-License-Identifier: Apache-2.0
//! The runner of the offline reranking experiment of #1688, and no verb.
//!
//! It reads `prompt_id<TAB>path<TAB>query<TAB>document` lines on standard
//! input and writes `prompt_id<TAB>path<TAB>score` lines on standard output,
//! in input order, with the score the raw logit of the pinned cross-encoder.
//! A tab or a newline inside a query or a document is written as `\t` or `\n`
//! by the caller, and this runner reads it back.
//!
//! Usage: `rerank <pin> <model dir>`. It opens no socket: the files are the
//! ones `tools/embed/fetch-model.sh --pin .headwater/reranker.yml` fetched, and
//! each one is checked against its digest before it is read.

use std::io::{BufRead, Write};
use std::path::Path;

fn unescape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [pin, dir] = args.as_slice() else {
        eprintln!("usage: rerank <pin> <model dir>");
        std::process::exit(2);
    };
    let fail = |message: String| -> ! {
        eprintln!("rerank: {message}");
        std::process::exit(1);
    };
    let pin = headwater_embed::Pin::read_at(Path::new(pin)).unwrap_or_else(|e| fail(e));
    let reranker =
        headwater_embed::Reranker::load(&pin, Path::new(dir)).unwrap_or_else(|e| fail(e));
    eprintln!("rerank: {} at {}, digest {}", pin.model, pin.revision, reranker.digest());
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());
    for (number, line) in std::io::stdin().lock().lines().enumerate() {
        let line = line.unwrap_or_else(|e| fail(format!("line {}: {e}", number + 1)));
        let fields: Vec<&str> = line.splitn(4, '\t').collect();
        let [prompt, path, query, document] = fields.as_slice() else {
            fail(format!("line {} has fewer than four fields", number + 1));
        };
        let score = reranker
            .score(&unescape(query), &unescape(document))
            .unwrap_or_else(|e| fail(format!("line {}: {e}", number + 1)));
        writeln!(out, "{prompt}\t{path}\t{score:.6}")
            .unwrap_or_else(|e| fail(format!("cannot write: {e}")));
    }
    out.flush().unwrap_or_else(|e| fail(format!("cannot write: {e}")));
}
