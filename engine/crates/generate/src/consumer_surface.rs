// SPDX-License-Identifier: Apache-2.0
//! The consumer surface page: what an adopter receives, runs and must have
//! installed, rendered from the `surface` block of the taxonomy.
//!
//! # Why a page, and why generated
//!
//! [HW-DR-0077](../../../../docs/decisions/0077-the-consumer-surface-is-what-an-adopter-receives-runs-and-must-have-installed-and-it-is-a-closed-and-declared-list.md)
//! rules that the consumer surface is a closed list and that a declaration
//! holds it. A list that only a check reads is a list an adopter never sees, and
//! a page typed by hand beside the block is a second copy that drifts
//! ([#1051](https://github.com/headwater-ai/headwater/issues/1051)). So the page
//! is a projection of the block and nothing else, and `generate --check`
//! reports it stale the day the block moves.
//!
//! # What it renders, and what it leaves out
//!
//! Every member an adopter meets: `archive`, `integration_points` with what
//! each one depends on, `prerequisites`, `companions` and `commands`.
//! `adopter_documents` and `local_roots` are not rendered as rows. They are the
//! configuration of the rule that holds each listed page, and neither one is a
//! thing an adopter receives or runs. The page says so in one sentence.
//!
//! A rendering is a reader of a member, and it is not a check of one. Nothing
//! here holds the archive to what a release ships.
//!
//! # A taxonomy with no block
//!
//! It writes no page, and the plan reports the declaration as unwritten, as
//! every other emitter reports a source that is not there. A page that said
//! "nothing" would assert a surface of zero members, which is a claim the
//! taxonomy never made.

use crate::{Declaration, Kind, Output, Plan, Unwritten};
use headwater_yaml::value::{Mapping, Value};

/// The `surface` block of a resolved taxonomy, read down to the members this
/// page renders.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Declared {
    pub archive: Vec<String>,
    /// Each integration point, in the order the block declares them, with the
    /// programs it depends on.
    pub integration_points: Vec<(String, Vec<String>)>,
    pub prerequisites: Vec<String>,
    pub companions: Vec<String>,
    pub commands: Vec<String>,
}

impl Declared {
    /// The block at the root of a resolved taxonomy, or `None` when the
    /// taxonomy declares none.
    pub fn read(root: &Mapping) -> Option<Declared> {
        let block = root.get("surface")?.value.as_map()?;
        let points = block
            .get("integration_points")
            .and_then(|node| node.value.as_map())
            .map(|points| {
                points
                    .iter()
                    .map(|entry| {
                        let needs = entry
                            .value
                            .value
                            .as_map()
                            .map(|body| strings(body, "depends_on"))
                            .unwrap_or_default();
                        (entry.key.value.clone(), needs)
                    })
                    .collect()
            })
            .unwrap_or_default();
        Some(Declared {
            archive: strings(block, "archive"),
            integration_points: points,
            prerequisites: strings(block, "prerequisites"),
            companions: strings(block, "companions"),
            commands: strings(block, "commands"),
        })
    }
}

fn strings(map: &Mapping, key: &str) -> Vec<String> {
    match map.get(key).map(|node| &node.value) {
        Some(Value::Seq(items)) => items
            .iter()
            .filter_map(|item| item.value.as_scalar().map(|scalar| scalar.text.clone()))
            .collect(),
        _ => Vec::new(),
    }
}

pub(crate) fn emit(declared: Option<&Declared>, declaration: &Declaration, plan: &mut Plan) {
    let path = declaration.output.clone();
    let Some(declared) = declared else {
        plan.unwritten.push(Unwritten {
            at: path,
            kind: Kind::ConsumerSurface,
            reason: "this taxonomy declares no `surface` block, so there is no consumer surface \
                     to render"
                .to_string(),
        });
        return;
    };
    plan.outputs.push(Output {
        bytes: render(&path, declared),
        path,
        kind: Kind::ConsumerSurface,
    });
}

fn render(output: &str, declared: &Declared) -> String {
    let mut out = String::new();
    let mark = headwater_mark::marker(Kind::ConsumerSurface.name(), output)
        .unwrap_or_else(|| format!("<!-- {} -->", headwater_mark::MARKER));
    out.push_str(&mark);
    out.push_str("\n\n# The consumer surface\n\n");
    out.push_str(
        "This page lists what an adopter receives, what an adopter runs, and what an adopter \
         must have installed. `headwater generate` writes it from the `surface` block of the \
         taxonomy, and `headwater generate --check` reports it stale when the block changes.\n",
    );

    list(
        &mut out,
        "What the archive holds",
        "The files of a release archive, as paths inside it.",
        &declared.archive,
    );

    out.push_str("\n## Integration points\n\n");
    out.push_str("The places where an adopter connects the binary, and what each one needs beyond the binary.\n\n");
    match declared.integration_points.is_empty() {
        true => out.push_str("None declared.\n"),
        false => {
            out.push_str("| Integration point | What it needs beyond the binary |\n|---|---|\n");
            for (name, needs) in &declared.integration_points {
                let needs = match needs.is_empty() {
                    true => "nothing".to_string(),
                    false => needs
                        .iter()
                        .map(|need| format!("`{need}`"))
                        .collect::<Vec<_>>()
                        .join(", "),
                };
                out.push_str(&format!("| `{name}` | {needs} |\n"));
            }
        }
    }

    list(
        &mut out,
        "Prerequisites",
        "Programs that an adopter installs and that this project does not supply.",
        &declared.prerequisites,
    );
    list(
        &mut out,
        "Companions",
        "Tools that an adopter may run and that no conformance level requires.",
        &declared.companions,
    );
    list(
        &mut out,
        "Commands",
        "The first word of every command that a page for an adopter may tell a reader to run.",
        &declared.commands,
    );

    out.push_str(
        "\nThe block also names the pages for an adopter and the roots that only this \
         repository holds. They configure the check that holds each such page, and they are \
         not something an adopter receives or runs, so this page does not list them.\n",
    );
    out
}

fn list(out: &mut String, heading: &str, lead: &str, items: &[String]) {
    out.push_str(&format!("\n## {heading}\n\n{lead}\n\n"));
    match items.is_empty() {
        true => out.push_str("None declared.\n"),
        false => {
            for item in items {
                out.push_str(&format!("- `{item}`\n"));
            }
        }
    }
}
