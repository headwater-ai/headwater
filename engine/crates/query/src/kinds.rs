// SPDX-License-Identifier: Apache-2.0
//! The kinds a resolved taxonomy keeps, as an agent reads them before it
//! drafts a document (#1580).
//!
//! One renderer, and two callers: `headwater taxonomy kinds` and the MCP
//! `kinds` tool. Both hand it the [`Shape`] and the census [`Taxonomy`] read
//! out of one lock, so the verb and the tool answer the same bytes. A second
//! renderer in the CLI would be the drift
//! [principle 2](../../../../docs/spec/00-vision-and-scope.md#design-principles)
//! rules against.
//!
//! Every word of a report is the lock's, with one exception. No taxonomy
//! declares when to write a kind: the meta-schema's `kind` block has no member
//! for it. So each kind carries [`UNDECLARED_WHEN`], one fixed sentence that
//! says so and sends the reader to the `answers` of the kind's purpose. A
//! sentence written here for each kind would describe this repository's
//! taxonomy to an adopter whose taxonomy is a different one.
//!
//! An abstract kind is a kind no document ever is
//! ([spec 2](../../../../docs/spec/02-taxonomy-model.md#abstract-kinds)), so it
//! gets no entry of its own. It appears as the `is_a` parent its children
//! name, and what it requires appears in their inherited facets and
//! sections. The header counts the abstract kinds it left out, so a reader
//! sees that the list is not every kind the lock declares.

use headwater_census::shelves::{ShelfBody, Taxonomy};
use headwater_check::Shape;
use headwater_yaml::json::Json;

/// The "when to write one" line of every kind, while no taxonomy declares one.
///
/// It is a fact about the meta-schema rather than about a kind, so it is the
/// same sentence for every kind of every taxonomy.
pub const UNDECLARED_WHEN: &str = "the taxonomy declares no trigger for this kind; \
                                   the answers of its purpose are what the lock states instead";

/// One shelf that carries a kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Carrier {
    pub shelf: String,
    /// The shelf's path pattern, as the lock writes it.
    pub path: String,
    /// The facet that chooses the kind on a heterogeneous shelf, and `None` on
    /// a homogeneous one.
    pub discriminator: Option<String>,
}

/// One concrete kind, with what a document of it owes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub is_a: Option<String>,
    /// The purpose, inherited through `is_a` where the kind names none.
    pub purpose: Option<String>,
    pub intent: Option<String>,
    pub answers: Vec<String>,
    pub carriers: Vec<Carrier>,
    /// Required facets after inheritance, in the order a check reports them.
    pub facets: Vec<String>,
    /// Required sections after inheritance.
    pub sections: Vec<String>,
}

impl Entry {
    /// The command that drafts a document of this kind, where a shelf can
    /// place one. A kind no shelf carries has no place for `headwater new`
    /// to write, so it has no command.
    pub fn draft(&self) -> Option<String> {
        match self.carriers.is_empty() {
            true => None,
            false => Some(format!("headwater new {} \"<title>\"", self.name)),
        }
    }
}

/// The concrete kinds of one taxonomy, in the order the lock declares them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Kinds {
    pub package: String,
    pub version: String,
    pub kinds: Vec<Entry>,
    /// The abstract kinds the list leaves out, in lock order.
    pub abstracts: Vec<String>,
}

impl Kinds {
    /// Read the kinds out of the two structures one lock produced.
    pub fn of(package: &str, version: &str, shape: &Shape, taxonomy: &Taxonomy) -> Self {
        let is_abstract = |name: &str| {
            taxonomy
                .kinds
                .iter()
                .any(|kind| kind.name == name && kind.is_abstract)
        };
        let mut kinds = Vec::new();
        let mut abstracts = Vec::new();
        for kind in &shape.kinds {
            if is_abstract(&kind.name) {
                abstracts.push(kind.name.clone());
                continue;
            }
            let purpose = shape.purpose_of(&kind.name);
            let carriers = taxonomy
                .shelves
                .iter()
                .filter_map(|shelf| match &shelf.body {
                    ShelfBody::Homogeneous { kind: carried } if *carried == kind.name => {
                        Some(Carrier {
                            shelf: shelf.name.clone(),
                            path: shelf.pattern.source().to_string(),
                            discriminator: None,
                        })
                    }
                    ShelfBody::Heterogeneous {
                        discriminator,
                        kinds,
                    } if kinds.contains(&kind.name) => Some(Carrier {
                        shelf: shelf.name.clone(),
                        path: shelf.pattern.source().to_string(),
                        discriminator: Some(discriminator.clone()),
                    }),
                    _ => None,
                })
                .collect();
            kinds.push(Entry {
                name: kind.name.clone(),
                is_a: kind.is_a.clone(),
                purpose: purpose.map(|purpose| purpose.name.clone()),
                intent: purpose.and_then(|purpose| purpose.intent.clone()),
                answers: purpose
                    .map(|purpose| purpose.answers.clone())
                    .unwrap_or_default(),
                carriers,
                facets: shape.required_facets(&kind.name),
                sections: shape.required_sections(&kind.name),
            });
        }
        Kinds {
            package: package.to_string(),
            version: version.to_string(),
            kinds,
            abstracts,
        }
    }

    /// The report for a reader, which is the text the MCP tool returns.
    pub fn text(&self) -> String {
        let mut out = format!(
            "{} {}: {} kind(s) a document can be",
            self.package,
            self.version,
            self.kinds.len()
        );
        match self.abstracts.is_empty() {
            true => out.push('\n'),
            false => out.push_str(&format!(
                ", and {} abstract kind(s) no document is: {}\n",
                self.abstracts.len(),
                self.abstracts.join(", ")
            )),
        }
        for entry in &self.kinds {
            out.push('\n');
            out.push_str(&entry.name);
            out.push('\n');
            let mut line = |label: &str, value: &str| {
                out.push_str(&format!("  {label:<9} {value}\n"));
            };
            line("is a", entry.is_a.as_deref().unwrap_or("(no parent)"));
            match (&entry.purpose, &entry.intent) {
                (Some(purpose), Some(intent)) => line("purpose", &format!("{purpose}: {intent}")),
                (Some(purpose), None) => line("purpose", purpose),
                (None, _) => line("purpose", "(none declared)"),
            }
            for answer in &entry.answers {
                line("answers", answer);
            }
            match entry.carriers.is_empty() {
                true => line("shelf", "(no shelf carries this kind)"),
                false => {
                    for carrier in &entry.carriers {
                        let chosen = match &carrier.discriminator {
                            Some(facet) => format!(", where `{facet}` names it"),
                            None => String::new(),
                        };
                        line(
                            "shelf",
                            &format!("{} ({}{chosen})", carrier.shelf, carrier.path),
                        );
                    }
                }
            }
            line("facets", &listed(&entry.facets));
            line("sections", &listed(&entry.sections));
            match entry.draft() {
                Some(draft) => line("draft", &draft),
                None => line("draft", "(no shelf can place one)"),
            }
            line("when", UNDECLARED_WHEN);
        }
        out
    }

    /// The same report as one JSON value, which the MCP tool returns as its
    /// `structuredContent` and `--json` prints.
    pub fn json(&self) -> Json {
        let strings =
            |values: &[String]| Json::Array(values.iter().cloned().map(Json::String).collect());
        let optional = |value: &Option<String>| match value {
            Some(text) => Json::string(text.clone()),
            None => Json::Raw("null".to_string()),
        };
        Json::object([
            ("version", Json::string(crate::json::VERSION)),
            ("package", Json::string(self.package.clone())),
            ("package_version", Json::string(self.version.clone())),
            ("abstract", strings(&self.abstracts)),
            (
                "kinds",
                Json::Array(
                    self.kinds
                        .iter()
                        .map(|entry| {
                            Json::object([
                                ("name", Json::string(entry.name.clone())),
                                ("is_a", optional(&entry.is_a)),
                                (
                                    "purpose",
                                    match &entry.purpose {
                                        Some(name) => Json::object([
                                            ("name", Json::string(name.clone())),
                                            ("intent", optional(&entry.intent)),
                                            ("answers", strings(&entry.answers)),
                                        ]),
                                        None => Json::Raw("null".to_string()),
                                    },
                                ),
                                (
                                    "shelves",
                                    Json::Array(
                                        entry
                                            .carriers
                                            .iter()
                                            .map(|carrier| {
                                                Json::object([
                                                    ("name", Json::string(carrier.shelf.clone())),
                                                    ("path", Json::string(carrier.path.clone())),
                                                    (
                                                        "discriminator",
                                                        optional(&carrier.discriminator),
                                                    ),
                                                ])
                                            })
                                            .collect(),
                                    ),
                                ),
                                ("required_facets", strings(&entry.facets)),
                                ("required_sections", strings(&entry.sections)),
                                ("draft", optional(&entry.draft())),
                                (
                                    "when",
                                    Json::object([
                                        ("declared", Json::Raw("null".to_string())),
                                        ("note", Json::string(UNDECLARED_WHEN)),
                                    ]),
                                ),
                            ])
                        })
                        .collect(),
                ),
            ),
        ])
    }
}

/// A list on one line, or a word that says it is empty.
fn listed(values: &[String]) -> String {
    match values.is_empty() {
        true => "(none)".to_string(),
        false => values.join(", "),
    }
}
