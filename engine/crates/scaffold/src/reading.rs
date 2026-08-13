// SPDX-License-Identifier: Apache-2.0
//! The capture-cost store: one reading per run of `headwater new`.
//!
//! [Spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric)
//! makes capture cost a tracked metric and says the fraction trends. A verb
//! that prints one number trends nothing, and the run is the only moment the
//! number is knowable: Q4 keeps `created_by` on the relation type, so no later
//! reader of a committed corpus can tell a scaffolded edge from a hand-typed
//! one. So the reading is written where the work happens or it is lost.
//!
//! # What a reading holds, and why each field is in it
//!
//! Six members, and the last four are [`crate::Assisted`] verbatim.
//!
//! - `lock` is the digest of the taxonomy that set the denominator. The
//!   denominator is a count of declarations, so a required facet the scaffolder
//!   can fill raises the fraction with no change in what an author typed. Two
//!   readings taken under two digests are two different measurements, and the
//!   store says which is which rather than averaging them.
//! - `date` is the injected clock, so a reading is reproducible under `--now`.
//! - `kind` is what the reading is about, and the term the per-kind report
//!   groups by.
//! - `document` is the path at birth. `id` is the identifier, which is the join
//!   key that survives a rename; a kind that mints none carries no `id`, and
//!   such a reading joins on the path alone.
//!
//! # What a reading deliberately does not hold
//!
//! **No person and no agent.** No user name, no host, no `drafted_by`. Spec 3
//! aims the remedy for a falling fraction at the taxonomy — derive more, or
//! require less — and never at the author. A per-author capture-cost number is
//! a performance measure, and a performance measure changes the behavior it
//! measures.
//!
//! **No wall-clock time and no duration.** Neither is reproducible, and both
//! would put a value in the store that differs on every run for reasons that
//! have nothing to do with the reading.
//!
//! **No prose, and no title.** The store holds counts. The title is in the
//! document that the reading names.
//!
//! **No run that refused.** A refusal wrote no document, so there is nothing to
//! attribute a reading to. A count of refusals would be a measure of the
//! author's fumbling, which is the field above under another name.
//!
//! **Nothing a hook or a skill did.**
//! [Spec 5](../../../../docs/spec/05-ai-integration.md#the-hook-contract-and-what-a-hook-cannot-bind)
//! says a hook binds nothing and that a session in which no hook spoke is no
//! evidence. A "the hook fired" counter has a denominator that `disableAllHooks`
//! empties with no record anywhere, so the number would fall when a harness
//! changed and read as a fall in authoring. The store refuses the event rather
//! than holding one that nobody can interpret.
//!
//! # Where it lands
//!
//! [`STORE`], which is under `.headwater/` and therefore outside the corpus
//! root. No census row covers it, no language regime binds it, and no rule
//! reads it — the same boundary a taxonomy source sits on. It is committed
//! plain text, so every reader of the repository recomputes each aggregate from
//! it. A capture-cost number that a reader cannot recompute is a number nobody
//! should cite.
//!
//! # What makes a reading hard to fake
//!
//! An entry and the document it names land in one working tree, so the diff
//! that adds the entry is the diff that adds the document. An entry naming a
//! document that the corpus does not classify is reported as one and never
//! counted, which is what stops a store from absorbing a document that arrived
//! by a route this verb does not watch: such a document raises the reach
//! denominator and nothing else.

use crate::{Assisted, Plan};
use headwater_check::Date;
use headwater_yaml::json::Json;
use std::path::Path;

/// Where the store lives, relative to the repository root.
///
/// One line per reading. A line is appended and no line is ever rewritten, so
/// two branches that each scaffolded a document both keep their reading under
/// the `merge=union` attribute this repository declares for the path.
pub const STORE: &str = ".headwater/capture-cost.jsonl";

/// One run of `headwater new`, as the store holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reading {
    pub lock: String,
    pub date: Date,
    pub kind: String,
    pub document: String,
    pub id: Option<String>,
    pub assisted: Assisted,
}

impl Reading {
    /// The reading a plan produces, derived from the plan rather than tracked
    /// beside it, in the way [`Plan::assisted`] is.
    pub fn of(plan: &Plan, lock: &str, date: Date) -> Reading {
        Reading {
            lock: lock.to_string(),
            date,
            kind: plan.kind.clone(),
            document: plan.path.clone(),
            id: plan.minting.as_ref().map(|minting| minting.id.clone()),
            assisted: plan.assisted(),
        }
    }

    /// One line of the store, with no trailing newline.
    ///
    /// The member order is fixed and the writer never sorts, so a reading
    /// written by a later engine diffs against an earlier one line for line.
    pub fn render(&self) -> String {
        let pair = |value: (usize, usize)| {
            Json::Array(vec![
                Json::Raw(value.0.to_string()),
                Json::Raw(value.1.to_string()),
            ])
        };
        let mut members = vec![
            ("lock", Json::string(&self.lock)),
            ("date", Json::string(self.date.render())),
            ("kind", Json::string(&self.kind)),
            ("document", Json::string(&self.document)),
        ];
        if let Some(id) = &self.id {
            members.push(("id", Json::string(id)));
        }
        members.extend([
            ("fields", pair(self.assisted.fields)),
            ("sections", pair(self.assisted.sections)),
            ("identifier", pair(self.assisted.identifier)),
            ("edge_halves", pair(self.assisted.edge_halves)),
        ]);
        Json::object(members).render()
    }

    /// One line of the store, read back.
    ///
    /// JSON is a subset of the YAML 1.2 core schema, so the loader that reads
    /// every taxonomy source reads this too, and the store needs no second
    /// parser. A member this cannot read is an error naming the member, because
    /// a reading that silently lost a term would move an aggregate and report
    /// nothing.
    pub fn parse(line: &str) -> Result<Reading, String> {
        let root = headwater_yaml::load(line)
            .map_err(|errors| format!("it is not a JSON object ({} parse errors)", errors.len()))?;
        let map = root
            .value
            .as_map()
            .ok_or_else(|| "it is not a JSON object".to_string())?;
        let text = |key: &str| -> Result<String, String> {
            map.get(key)
                .and_then(|node| node.value.as_scalar())
                .map(|scalar| scalar.text.clone())
                .ok_or_else(|| format!("it names no `{key}`"))
        };
        let pair = |key: &str| -> Result<(usize, usize), String> {
            let items = map
                .get(key)
                .and_then(|node| node.value.as_seq())
                .ok_or_else(|| format!("it names no `{key}`"))?;
            let read = |at: usize| -> Result<usize, String> {
                items
                    .get(at)
                    .and_then(|node| node.value.as_scalar())
                    .and_then(|scalar| scalar.text.parse::<usize>().ok())
                    .ok_or_else(|| format!("`{key}` is not a pair of whole numbers"))
            };
            match items.len() {
                2 => Ok((read(0)?, read(1)?)),
                other => Err(format!("`{key}` holds {other} values rather than two")),
            }
        };
        let date = text("date")?;
        Ok(Reading {
            lock: text("lock")?,
            date: Date::parse(&date)
                .ok_or_else(|| format!("`date` is `{date}` rather than a date"))?,
            kind: text("kind")?,
            document: text("document")?,
            id: map
                .get("id")
                .and_then(|node| node.value.as_scalar())
                .map(|scalar| scalar.text.clone()),
            assisted: Assisted {
                fields: pair("fields")?,
                sections: pair("sections")?,
                identifier: pair("identifier")?,
                edge_halves: pair("edge_halves")?,
            },
        })
    }
}

/// Put one reading on the end of the store, creating the file if it is absent.
///
/// The caller writes the document first. A reading whose document did not land
/// would name a file nobody can read, and that is the one error this order
/// cannot make.
pub fn append(root: &Path, reading: &Reading) -> Result<(), String> {
    use std::io::Write;
    let path = root.join(STORE);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|error| error.to_string())?;
    writeln!(file, "{}", reading.render()).map_err(|error| error.to_string())
}

/// A line the store holds that no reading could be read from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unreadable {
    /// One-based, so it names the line a person opens the file to.
    pub line: usize,
    pub why: String,
}

/// Every reading the store holds, and every line it holds that is not one.
///
/// An absent store is an empty store rather than an error. A corpus that has
/// never run the verb is the ordinary first state, and it is the state the
/// reach number reports honestly.
pub fn load(root: &Path) -> Result<(Vec<Reading>, Vec<Unreadable>), String> {
    let path = root.join(STORE);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok((vec![], vec![])),
        Err(error) => return Err(error.to_string()),
    };
    let mut readings = Vec::new();
    let mut unreadable = Vec::new();
    for (at, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        match Reading::parse(line) {
            Ok(reading) => readings.push(reading),
            Err(why) => unreadable.push(Unreadable { line: at + 1, why }),
        }
    }
    Ok((readings, unreadable))
}

/// The four terms of every reading, summed.
pub fn total(readings: &[Reading]) -> Assisted {
    let mut total = Assisted::default();
    for reading in readings {
        let add = |into: &mut (usize, usize), from: (usize, usize)| {
            into.0 += from.0;
            into.1 += from.1;
        };
        add(&mut total.fields, reading.assisted.fields);
        add(&mut total.sections, reading.assisted.sections);
        add(&mut total.identifier, reading.assisted.identifier);
        add(&mut total.edge_halves, reading.assisted.edge_halves);
    }
    total
}

/// `(kind, readings, the four terms summed)`, in the order the kinds first
/// appear in the store, so the report is a function of the file.
pub fn by_kind(readings: &[Reading]) -> Vec<(String, usize, Assisted)> {
    let mut out: Vec<(String, Vec<Reading>)> = Vec::new();
    for reading in readings {
        match out.iter_mut().find(|(kind, _)| kind == &reading.kind) {
            Some((_, held)) => held.push(reading.clone()),
            None => out.push((reading.kind.clone(), vec![reading.clone()])),
        }
    }
    out.into_iter()
        .map(|(kind, held)| (kind, held.len(), total(&held)))
        .collect()
}

/// Every distinct lock digest the store holds, in first-seen order.
///
/// More than one means the readings were taken against more than one
/// denominator, and the report says so rather than averaging across them.
pub fn locks(readings: &[Reading]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for reading in readings {
        if !out.contains(&reading.lock) {
            out.push(reading.lock.clone());
        }
    }
    out
}

/// Every classified document of a corpus, as much of it as the reach join
/// needs.
///
/// The caller builds it from a census and an identifier index, so this crate
/// joins without walking a tree and a test states a corpus in four lines.
#[derive(Clone, Debug, Default)]
pub struct Classified {
    /// Every classified document, by path, in census order.
    pub paths: Vec<String>,
    /// `(identifier, path)` for each classified document that declares one.
    pub identified: Vec<(String, String)>,
}

/// How far the authoring verb reaches into a corpus.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Reach {
    /// Distinct classified documents that a reading names, in the order the
    /// readings name them. Two readings over one document count once.
    pub reached: Vec<String>,
    /// `(reading, where the document is now)` for a reading whose document
    /// moved after it was taken, joined by identifier.
    pub moved: Vec<(usize, String)>,
    /// Readings naming a document this corpus does not classify, by index.
    /// None of them is counted in `reached`.
    pub lost: Vec<usize>,
}

/// Join the store against the corpus.
///
/// The identifier is tried first, because it survives a rename and a path does
/// not. The path is the fallback for a kind that mints no identifier.
///
/// **A document that arrived by a route this verb does not watch raises
/// `Classified::paths` and nothing else**, so the reach fraction falls. That is
/// the direction that makes the number worth reading: a store that absorbed
/// such a document would report a reach it never had.
pub fn reach(readings: &[Reading], corpus: &Classified) -> Reach {
    let mut out = Reach::default();
    for (at, reading) in readings.iter().enumerate() {
        let by_id = reading.id.as_deref().and_then(|id| {
            corpus
                .identified
                .iter()
                .find(|(known, _)| known == id)
                .map(|(_, path)| path.clone())
        });
        let by_path = corpus
            .paths
            .iter()
            .find(|path| *path == &reading.document)
            .cloned();
        match by_id.or(by_path) {
            None => out.lost.push(at),
            Some(path) => {
                if path != reading.document {
                    out.moved.push((at, path.clone()));
                }
                if !out.reached.contains(&path) {
                    out.reached.push(path);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a_reading() -> Reading {
        Reading {
            lock: "sha256:abc".to_string(),
            date: Date::parse("2026-08-14").expect("a date"),
            kind: "obligation_record".to_string(),
            document: "docs/obligations/0109-a-record.md".to_string(),
            id: Some("OBL-repo-0109".to_string()),
            assisted: Assisted {
                fields: (4, 5),
                sections: (3, 3),
                identifier: (1, 1),
                edge_halves: (0, 0),
            },
        }
    }

    #[test]
    fn a_reading_renders_as_one_line_in_a_fixed_member_order() {
        assert_eq!(
            a_reading().render(),
            "{\"lock\":\"sha256:abc\",\"date\":\"2026-08-14\",\"kind\":\"obligation_record\",\
             \"document\":\"docs/obligations/0109-a-record.md\",\"id\":\"OBL-repo-0109\",\
             \"fields\":[4,5],\"sections\":[3,3],\"identifier\":[1,1],\"edge_halves\":[0,0]}"
        );
    }

    #[test]
    fn what_the_writer_emits_is_what_the_reader_reads() {
        let reading = a_reading();
        assert_eq!(Reading::parse(&reading.render()), Ok(reading));
    }

    /// A kind that mints no identifier joins on its path alone, so the member is
    /// absent rather than empty. An empty string would be an identifier that
    /// resolves to nothing.
    #[test]
    fn a_reading_without_an_identifier_carries_no_member_for_one() {
        let mut reading = a_reading();
        reading.id = None;
        assert!(!reading.render().contains("\"id\""));
        assert_eq!(Reading::parse(&reading.render()), Ok(reading));
    }

    #[test]
    fn a_member_that_cannot_be_read_is_an_error_naming_the_member() {
        let line = a_reading().render().replace("\"kind\"", "\"knid\"");
        assert_eq!(Reading::parse(&line), Err("it names no `kind`".to_string()));
        let line = a_reading().render().replace("[4,5]", "[4]");
        assert_eq!(
            Reading::parse(&line),
            Err("`fields` holds 1 values rather than two".to_string())
        );
        let line = a_reading().render().replace("2026-08-14", "the fourteenth");
        assert_eq!(
            Reading::parse(&line),
            Err("`date` is `the fourteenth` rather than a date".to_string())
        );
    }

    #[test]
    fn an_absent_store_is_an_empty_store() {
        let scratch = std::env::temp_dir().join("headwater-reading-absent");
        let _ = std::fs::remove_dir_all(&scratch);
        std::fs::create_dir_all(&scratch).expect("a scratch directory");
        assert_eq!(load(&scratch), Ok((vec![], vec![])));
        let _ = std::fs::remove_dir_all(&scratch);
    }

    #[test]
    fn a_line_the_store_cannot_read_is_named_by_its_line_number_and_never_counted() {
        let scratch = std::env::temp_dir().join("headwater-reading-unreadable");
        let _ = std::fs::remove_dir_all(&scratch);
        std::fs::create_dir_all(&scratch).expect("a scratch directory");
        append(&scratch, &a_reading()).expect("it appends");
        std::fs::write(
            scratch.join(STORE),
            format!("{}\nnot a reading at all\n", a_reading().render()),
        )
        .expect("it writes");
        let (readings, unreadable) = load(&scratch).expect("it loads");
        assert_eq!(readings, vec![a_reading()]);
        assert_eq!(unreadable.len(), 1);
        assert_eq!(unreadable[0].line, 2);
        let _ = std::fs::remove_dir_all(&scratch);
    }

    #[test]
    fn appending_twice_holds_both_readings_in_the_order_they_were_taken() {
        let scratch = std::env::temp_dir().join("headwater-reading-append");
        let _ = std::fs::remove_dir_all(&scratch);
        std::fs::create_dir_all(&scratch).expect("a scratch directory");
        let mut second = a_reading();
        second.kind = "decision".to_string();
        second.document = "docs/decisions/0023-a-decision.md".to_string();
        second.assisted.fields = (4, 5);
        append(&scratch, &a_reading()).expect("it appends");
        append(&scratch, &second).expect("it appends");
        let (readings, unreadable) = load(&scratch).expect("it loads");
        assert_eq!(unreadable, vec![]);
        assert_eq!(readings, vec![a_reading(), second]);
        assert_eq!(total(&readings).supplied(), 16);
        assert_eq!(total(&readings).total(), 18);
        assert_eq!(
            by_kind(&readings)
                .into_iter()
                .map(|(kind, count, _)| (kind, count))
                .collect::<Vec<_>>(),
            vec![
                ("obligation_record".to_string(), 1),
                ("decision".to_string(), 1)
            ]
        );
        assert_eq!(locks(&readings), vec!["sha256:abc".to_string()]);
        let _ = std::fs::remove_dir_all(&scratch);
    }

    /// The denominator is a count of declarations, so a reading taken under a
    /// second taxonomy is a second measurement. The store keeps both digests,
    /// which is what lets a report refuse to average them.
    #[test]
    fn two_taxonomies_are_two_digests_and_the_store_keeps_both() {
        let mut later = a_reading();
        later.lock = "sha256:def".to_string();
        later.assisted.fields = (5, 6);
        let readings = vec![a_reading(), later];
        assert_eq!(
            locks(&readings),
            vec!["sha256:abc".to_string(), "sha256:def".to_string()]
        );
        assert_eq!(total(&readings).total(), 19);
    }
}
