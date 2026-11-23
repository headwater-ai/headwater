// SPDX-License-Identifier: Apache-2.0
//! The writer of `generate` writes nothing outside the repository root,
//! whatever the plan says (#1967).
//!
//! `taxonomy resolve` refuses a projection output that leaves the repository,
//! and the lock reader refuses it again, so no plan the CLI builds carries one.
//! This holds the writer's own guard behind both, with a plan built by hand:
//! one output climbs out and one stays inside. The climbing one is refused,
//! and the inside one is withheld, because a run that refuses writes nothing
//! (#1466).
//!
//! # Watched failing
//!
//! With the guard in `run` removed, the case failed: `../escaped.md` was
//! written beside the root.

use headwater_generate::{write, Kind, Output, Plan, Verdict};

fn marked(path: &str) -> String {
    format!(
        "{}\n\nbody\n",
        headwater_mark::marker(Kind::ShelfIndex.name(), path).expect("Markdown carries a marker")
    )
}

#[test]
fn an_output_that_climbs_out_is_refused_and_nothing_is_written() {
    let scratch = std::env::temp_dir().join(format!(
        "headwater-generate-containment-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&scratch);
    let root = scratch.join("root");
    std::fs::create_dir_all(&root).expect("the root is made");

    let output = |path: &str| Output {
        path: path.to_string(),
        kind: Kind::ShelfIndex,
        committed: true,
        bytes: marked(path),
    };
    let plan = Plan {
        outputs: vec![output("../escaped.md"), output("inside.md")],
        ..Plan::default()
    };
    let report = write(&root, &plan);

    let escaped = scratch.join("escaped.md");
    let inside = root.join("inside.md");
    let wrote_escaped = escaped.exists();
    let wrote_inside = inside.exists();
    let _ = std::fs::remove_dir_all(&scratch);

    assert!(!wrote_escaped, "a file was written above the root");
    assert!(report.withheld, "the run refused before it wrote");
    assert!(!wrote_inside, "the output inside the root was withheld");
    let verdicts: Vec<(&str, &Verdict)> = report
        .wrote
        .iter()
        .map(|wrote| (wrote.path.as_str(), &wrote.verdict))
        .collect();
    assert_eq!(
        verdicts,
        [
            ("../escaped.md", &Verdict::Escapes),
            ("inside.md", &Verdict::Withheld)
        ]
    );
    assert!(report.has_errors());
}
