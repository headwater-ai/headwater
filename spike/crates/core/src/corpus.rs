//! Loading documents, and generating a synthetic corpus for the bench.

use crate::frontmatter;
use crate::hash;
use crate::model::{Body, Document, Graph, Kind};
use crate::span::{Position, Span, Spanned};

/// Phase A outcome for one file. Every file under the root gets one of these,
/// which is the census that fixes the coverage denominator.
#[derive(Debug, Clone)]
pub enum Census {
    Classified { path: String, kind: Kind },
    Unparseable { path: String, reason: String },
}

pub struct Loaded {
    pub graph: Graph,
    pub census: Vec<Census>,
}

/// Parse one file. Failures become census entries rather than silent drops.
pub fn parse_document(path: &str, source: &str) -> Result<Document, String> {
    let front = frontmatter::parse(source).map_err(|e| e.to_string())?;
    let split = frontmatter::split(source).map_err(|e| e.to_string())?;
    let body = parse_body(split.body, split.body_line_offset, split.yaml_byte_offset);

    let id = front.scalar("id").unwrap_or("").to_string();
    let kind = front.scalar("kind").map(Kind::parse).unwrap_or(Kind::Unclassified);

    Ok(Document {
        path: path.to_string(),
        id,
        kind,
        front,
        body,
        content_hash: hash::content(source.as_bytes()),
    })
}

/// Minimal Markdown scan: ATX headings and inline links, with spans. Enough to
/// prove that body positions land in file coordinates alongside front-matter
/// positions, which is the second half of item 1.
fn parse_body(body: &str, line_offset: usize, byte_base: usize) -> Body {
    let mut headings = Vec::new();
    let mut links = Vec::new();
    let mut word_count = 0usize;
    let mut offset = 0usize;

    for (i, line) in body.lines().enumerate() {
        let line_no = line_offset + i + 1;
        word_count += line.split_whitespace().count();

        if let Some(rest) = line.strip_prefix("## ") {
            let col = 4;
            headings.push(Spanned::new(
                rest.trim().to_string(),
                Span::new(
                    Position::new(line_no, col, byte_base + offset + 3),
                    Position::new(line_no, col + rest.len(), byte_base + offset + line.len()),
                ),
            ));
        } else if let Some(rest) = line.strip_prefix("# ") {
            headings.push(Spanned::new(
                rest.trim().to_string(),
                Span::new(
                    Position::new(line_no, 3, byte_base + offset + 2),
                    Position::new(line_no, 3 + rest.len(), byte_base + offset + line.len()),
                ),
            ));
        }

        let bytes = line.as_bytes();
        let mut j = 0;
        while j < bytes.len() {
            if bytes[j] == b'[' {
                if let Some(close) = line[j..].find("](") {
                    let after = j + close + 2;
                    if let Some(end) = line[after..].find(')') {
                        let target = &line[after..after + end];
                        links.push(Spanned::new(
                            target.to_string(),
                            Span::new(
                                Position::new(line_no, after + 1, byte_base + offset + after),
                                Position::new(
                                    line_no,
                                    after + end + 1,
                                    byte_base + offset + after + end,
                                ),
                            ),
                        ));
                        j = after + end;
                    }
                }
            }
            j += 1;
        }
        offset += line.len() + 1;
    }

    Body {
        headings,
        links,
        word_count,
    }
}

pub fn load(files: &[(String, String)]) -> Loaded {
    let mut docs = Vec::new();
    let mut census = Vec::new();
    for (path, source) in files {
        match parse_document(path, source) {
            Ok(d) => {
                census.push(Census::Classified {
                    path: path.clone(),
                    kind: d.kind,
                });
                docs.push(d);
            }
            Err(reason) => census.push(Census::Unparseable {
                path: path.clone(),
                reason,
            }),
        }
    }
    Loaded {
        graph: Graph::build(docs),
        census,
    }
}

/// A synthetic corpus for item 4. Shaped to be representative rather than
/// convenient: every document carries relations, one in twenty has a defect,
/// and the reciprocal links are real so that most edges pass.
pub mod synth {
    /// `n` documents, each ~40 lines, with a chain of supersession and a
    /// scattering of governs edges.
    pub fn corpus(n: usize) -> Vec<(String, String)> {
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            out.push((format!("docs/decisions/dr-{i:04}.md"), document(i, n)));
        }
        out
    }

    fn document(i: usize, n: usize) -> String {
        let id = format!("DR-{i:04}");
        let mut fm = String::new();
        fm.push_str("---\n");
        fm.push_str(&format!("id: {id}\n"));
        fm.push_str("kind: decision\n");
        // One in twenty has a bad status, so the bench measures findings being
        // produced rather than only checks returning empty.
        if i % 20 == 7 {
            fm.push_str("status: currrent\n");
        } else {
            fm.push_str("status: current\n");
        }
        // One in twenty is missing a required facet.
        if i % 20 != 3 {
            fm.push_str(&format!("owner: team-{}\n", i % 9));
        }
        fm.push_str("confidentiality: internal\n");
        fm.push_str(&format!("last_verified: 2026-0{}-15\n", (i % 8) + 1));

        // A supersession chain: doc i supersedes i-1, and declares the
        // reciprocal so most edges pass reciprocity.
        if i > 0 {
            fm.push_str(&format!("supersedes:\n  - DR-{:04}\n", i - 1));
        }
        if i + 1 < n {
            // One in fifty omits the reciprocal, producing a real finding.
            if i % 50 != 11 {
                fm.push_str(&format!("superseded_by:\n  - DR-{:04}\n", i + 1));
            }
        }
        fm.push_str("governs:\n");
        fm.push_str(&format!("  - DR-{:04}\n", (i * 7 + 3) % n));
        fm.push_str("---\n\n");

        let mut body = String::new();
        body.push_str(&format!("# Decision {id}\n\n"));
        body.push_str("## Context\n\n");
        for k in 0..6 {
            body.push_str(&format!(
                "This paragraph {k} describes the situation that led to decision {id} and the constraints that applied at the time.\n\n"
            ));
        }
        // One in twenty lacks the required Consequences section.
        if i % 20 != 5 {
            body.push_str("## Consequences\n\n");
            body.push_str("The consequences follow from the context above.\n\n");
        }
        body.push_str("## References\n\n");
        body.push_str(&format!("See [the previous decision](dr-{:04}.md).\n", i.saturating_sub(1)));

        format!("{fm}{body}")
    }
}
