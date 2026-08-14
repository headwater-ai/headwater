// SPDX-License-Identifier: Apache-2.0
//! Inbound integration: edges from an external system of record, against a
//! committed pin.
//!
//! [Q19](../../../../docs/decisions/0019-inbound-integration-an-external-system-of-record.md)
//! rules that "imported content is `transcribed` against a committed pin, and an
//! imported edge carries full weight from the first release". This crate is the
//! edge half of that ruling. The content half is a projection kind and a
//! resolver that reads text, and neither is here.
//!
//! # Why this is a correctness root
//!
//! [Spec 13](../../../../docs/spec/13-open-obligations.md#the-correctness-roots)
//! puts an importer beside the scaffolder and gives the same reason: "Edges
//! marked `created_by: import` arrive in bulk from a system that this corpus
//! does not govern, and nobody reads them one at a time. A wrong imported edge
//! produces a *correct* check result over a *wrong* graph, so no check finds it
//! and no advisory posture helps."
//!
//! Read that in the other direction and it says what this crate owes. Every
//! defect a check could have caught is a defect that never reaches a check,
//! because the check runs over the graph the import produced and agrees with it.
//! So the refusals below are the whole instrument, and each one has a fixture
//! that provokes it. Nothing here is advisory and nothing here is a finding: an
//! import either lands whole or lands not at all.
//!
//! # What stops a wrong import
//!
//! Six refusals, in the order they fire, and the order is what makes each one
//! say the right thing.
//!
//! 1. **Nothing pins the snapshot.** A digest the engine took out of the
//!    directory in front of it is a pin against itself. `taxonomy vendor`
//!    refuses to self-certify for this reason and so does this.
//! 2. **Nothing names the channel.** The paragraph below is the whole argument.
//! 3. **The artifact is not the pinned one.** This is
//!    [`headwater_resolve::release::verify`] unchanged, so a changed byte, a
//!    file the record does not name, a file the artifact lost and a whole
//!    consistent re-publication are all refused with the file named.
//! 4. **The relation is not one an importer may write.** A relation this
//!    taxonomy does not declare, and a relation whose `created_by` is anything
//!    other than `import`.
//! 5. **An endpoint does not exist.** A `from` that names no document of this
//!    corpus, a `from` whose kind the relation does not admit, a relation whose
//!    far end admits no anchor at all, and a `to` the snapshot does not name.
//! 6. **The link is a repeat.** Q4 identifies an edge by its source, relation
//!    and target, so one snapshot naming a triple twice is a snapshot the
//!    importer refuses rather than a document with the line in it twice.
//!
//! # What an imported edge inherits its weight from
//!
//! Q19 says an imported edge "carries full weight from the first release", and
//! that ruling is about *this crate* rather than about the snapshot. Its
//! argument is that an importer is a producer of graph facts rather than a rule,
//! so evidence about a false-positive rate is the wrong instrument and a fixture
//! set is the right one. The fixture set is what the refusals above are.
//!
//! **It says nothing about whether the snapshot is what the upstream system
//! holds, and no fixture here can.** A digest over a snapshot proves that these
//! bytes are the bytes somebody pinned. It proves nothing about who produced
//! them ([OBL-repo-0115](../../../../docs/obligations/0115-a-pinned-digest-authenticates-the-pin-and-never-the-publisher.md)),
//! and a record that travels inside the artifact it describes can only ever
//! verify internal consistency. So an imported edge does not inherit the
//! digest's authority. It inherits the authority of whoever handed a person the
//! digest, and of the person who wrote it down.
//!
//! That is why a declaration carries `channel` beside `digest`, and why an
//! import with no channel is refused rather than defaulted. Both fields are
//! authored in `.headwater/taxonomy.yml`, committed, and read in a diff, and
//! neither is ever written by a verb. The channel is a sentence a person wrote
//! next to a number, which is exactly as much as this engine can honestly
//! record, and putting it in the snapshot instead would be the self-certifying
//! move that the pin exists to refuse.
//!
//! # The engine never fetches
//!
//! [Spec 0](../../../../docs/spec/00-vision-and-scope.md#non-negotiables)
//! forbids a network dependency at check time and no crate of this engine
//! depends on the network. This crate takes the path of a directory that the
//! caller already fetched and committed, on the shape `taxonomy vendor` set. A
//! verb that takes a path opens no socket, and there is no code path here that
//! could.

pub mod snapshot;
pub mod write;

use headwater_check::shape::Shape;
use headwater_graph::declarations::Declarations;
use headwater_graph::index::Index;
use headwater_resolve::release;
use std::path::Path;

/// The creator value a relation declares when an importer is what pays for its
/// edges. [Spec 2](../../../../docs/spec/02-taxonomy-model.md#who-creates-each-edge)
/// closes the set this comes from.
pub const IMPORT: &str = "import";

/// One import, as `.headwater/taxonomy.yml` declares it.
///
/// Authored and never written by a verb, on the same terms as the taxonomy pin
/// beside it. The two fields that are not the path are the two an import rests
/// on, and the module documentation argues why the second is not optional.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declaration {
    /// The name the declaration is keyed by, which is what a caller names on
    /// the command line.
    pub name: String,
    /// Where the committed snapshot sits, relative to the repository root.
    pub at: String,
    /// The digest of the snapshot artifact, copied from wherever the channel
    /// below carried it.
    pub digest: Option<String>,
    /// How the digest reached this repository, in the words of the person who
    /// wrote it down.
    pub channel: Option<String>,
}

/// What one import would put into the corpus, composed and not yet written.
#[derive(Clone, Debug)]
pub struct Plan {
    pub declaration: Declaration,
    pub release: release::Release,
    pub snapshot: snapshot::Snapshot,
    pub edges: Vec<Proposed>,
}

/// One edge half this import would write.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proposed {
    /// The document that will declare it, by identifier and by path.
    pub from: String,
    pub path: String,
    pub relation: String,
    pub to: String,
    /// The upstream revision this edge was checked against, which the edge
    /// carries as an instance attribute. Spec 2 declares the attribute on the
    /// relation type and this is the value that fills it.
    pub verified_revision: String,
    /// Whether the document already declares this edge at this revision, in
    /// which case the import writes nothing for it. An import that ran twice
    /// over one snapshot writes once.
    pub present: bool,
}

/// Why an import did not happen. A closed set, so that a test is written
/// against a name rather than against a rendered string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// No declaration of that name, and the names there are.
    Undeclared { name: String, declared: Vec<String> },
    /// Nothing pins the snapshot.
    Unpinned { name: String },
    /// Nothing says how the digest arrived.
    NoChannel { name: String },
    /// The artifact is not the pinned one. The message is the release layer's,
    /// verbatim, because it is the one that names what moved.
    Artifact(String),
    /// The payload did not read.
    Payload(String),
    /// The taxonomy declares no such relation.
    UnknownRelation {
        relation: String,
        from: String,
        to: String,
    },
    /// The relation exists and an importer may not write it.
    NotAnImportRelation {
        relation: String,
        created_by: Option<String>,
    },
    /// The near end names no document of this corpus.
    NoSuchSource {
        from: String,
        relation: String,
        to: String,
    },
    /// The near end exists and the relation does not admit its kind.
    SourceKindRefused {
        from: String,
        kind: String,
        relation: String,
        permitted: Vec<String>,
    },
    /// The relation's far end admits no anchor kind, so no item can sit at it.
    NoAnchorEnd {
        relation: String,
        permitted: Vec<String>,
    },
    /// The far end names an item the snapshot does not hold.
    NoSuchItem {
        to: String,
        from: String,
        relation: String,
    },
    /// One snapshot names one triple twice.
    RepeatedLink {
        from: String,
        relation: String,
        to: String,
    },
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Refusal::Undeclared { name, declared } => {
                write!(f, "`{name}` is not a declared import")?;
                match declared.is_empty() {
                    true => write!(
                        f,
                        ". `.headwater/taxonomy.yml` declares an `imports` block and this \
                         repository has none"
                    ),
                    false => write!(f, ". This repository declares {}", declared.join(", ")),
                }
            }
            Refusal::Unpinned { name } => write!(
                f,
                "nothing pins the snapshot of `{name}`. A digest the engine took from the \
                 directory in front of it is a pin against itself, so this refuses rather than \
                 records what it received. Write the digest the publisher gave you as \
                 `imports.{name}.digest` in `.headwater/taxonomy.yml`, or pass it with `--expect`"
            ),
            Refusal::NoChannel { name } => write!(
                f,
                "`imports.{name}` names no channel. A digest authenticates the pin and never the \
                 publisher, so an edge imported here carries the weight of whatever carried the \
                 digest to this repository rather than the weight of the digest. Write that in \
                 `imports.{name}.channel` in `.headwater/taxonomy.yml`, in the words of the \
                 person who wrote the digest down"
            ),
            Refusal::Artifact(why) | Refusal::Payload(why) => write!(f, "{why}"),
            Refusal::UnknownRelation { relation, from, to } => write!(
                f,
                "the snapshot links {from} to {to} by `{relation}`, and this taxonomy declares no \
                 such relation"
            ),
            Refusal::NotAnImportRelation {
                relation,
                created_by,
            } => match created_by {
                Some(value) => write!(
                    f,
                    "`{relation}` declares `created_by: {value}`, and this verb writes an edge \
                     only where a taxonomy expects an importer to pay for one. An edge written \
                     here would be indistinguishable from the {value}'s own"
                ),
                None => write!(
                    f,
                    "`{relation}` declares no `created_by`, so nothing says an importer may write \
                     it. `taxonomy validate` requires the member and this refuses without it"
                ),
            },
            Refusal::NoSuchSource { from, relation, to } => write!(
                f,
                "the snapshot links {from} to {to} by `{relation}`, and no document of this \
                 corpus carries the identifier {from}"
            ),
            Refusal::SourceKindRefused {
                from,
                kind,
                relation,
                permitted,
            } => write!(
                f,
                "{from} is a `{kind}` and `{relation}` runs from {}",
                permitted.join(" or ")
            ),
            Refusal::NoAnchorEnd {
                relation,
                permitted,
            } => write!(
                f,
                "`{relation}` ends on {}, and none of those is an anchor kind. An imported edge \
                 ends on the external item it came from, so this relation is not one an import \
                 can write",
                permitted.join(" or ")
            ),
            Refusal::NoSuchItem { to, from, relation } => write!(
                f,
                "the snapshot links {from} to {to} by `{relation}`, and the snapshot names no \
                 item {to}. An edge into an item nothing pinned records a revision that no later \
                 fetch can compare against"
            ),
            Refusal::RepeatedLink { from, relation, to } => write!(
                f,
                "the snapshot names {from} `{relation}` {to} more than once, and Q4 identifies an \
                 edge by exactly those three"
            ),
        }
    }
}

/// Render a refusal set the way a caller prints it.
pub fn render(refusals: &[Refusal]) -> String {
    let mut out = String::new();
    for refusal in refusals {
        out.push_str(&refusal.to_string());
        out.push('\n');
    }
    out
}

/// Every import this repository declares, in declaration order.
///
/// Read out of `.headwater/taxonomy.yml`, which is also where the taxonomy pin
/// is. One file and two blocks, each read by the crate that owns it: an
/// `imports` member on [`headwater_resolve::package::Consumer`] would put a
/// concept of inbound integration into the crate that resolves a taxonomy.
pub fn declared(root: &Path) -> Result<Vec<Declaration>, String> {
    let path = root.join(headwater_resolve::package::CONSUMER);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("{}: {error}", path.display())),
    };
    let root = headwater_yaml::load(&text)
        .map_err(|errors| headwater_yaml::error::render(&errors))
        .map_err(|why| format!("{}: {why}", path.display()))?;
    let Some(map) = root.value.as_map() else {
        return Err(format!("{} is not a mapping", path.display()));
    };
    let Some(imports) = map.get("imports").and_then(|node| node.value.as_map()) else {
        return Ok(Vec::new());
    };

    let mut out = Vec::new();
    for entry in imports {
        let Some(block) = entry.value.value.as_map() else {
            return Err(format!(
                "`imports.{}` is {}, and an import is a mapping",
                entry.key.value,
                entry.value.value.kind_name()
            ));
        };
        let text_of = |key: &str| {
            block
                .get(key)
                .and_then(|node| node.value.as_scalar())
                .map(|scalar| scalar.text.clone())
        };
        let at = text_of("at").ok_or_else(|| {
            format!(
                "`imports.{}` names no `at`, so nothing says where the committed snapshot is",
                entry.key.value
            )
        })?;
        out.push(Declaration {
            name: entry.key.value.clone(),
            at,
            digest: text_of("digest"),
            channel: text_of("channel"),
        });
    }
    Ok(out)
}

/// What the corpus declares, as the importer reads it.
pub struct Corpus<'a> {
    pub index: &'a Index,
    pub relations: &'a Declarations,
    pub shape: &'a Shape,
}

/// Compose one import, and write nothing.
///
/// The caller writes what this returns, so a dry run and a real one differ by
/// one loop rather than by a second composer. `headwater_scaffold::write` sets
/// that rule and it holds here for the reason it holds there: the thing under
/// test is what the run decided, and a second composer is where a report and a
/// write start to disagree.
///
/// Every refusal is collected rather than the first one returned. A snapshot
/// with forty wrong links is one fetch to redo, and reporting one link at a
/// time turns it into forty.
pub fn plan(
    root: &Path,
    declaration: &Declaration,
    expect: Option<&str>,
    corpus: &Corpus<'_>,
) -> Result<Plan, Vec<Refusal>> {
    // The pin first, and the channel with it. Both are the caller's own
    // declaration, so neither says anything about the artifact, and refusing
    // here means no byte of an unpinned directory has been read.
    let Some(pinned) = expect
        .map(str::to_string)
        .or_else(|| declaration.digest.clone())
    else {
        return Err(vec![Refusal::Unpinned {
            name: declaration.name.clone(),
        }]);
    };
    if declaration
        .channel
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        return Err(vec![Refusal::NoChannel {
            name: declaration.name.clone(),
        }]);
    }

    let dir = root.join(&declaration.at);
    let release = release::verify(&dir, &pinned)
        .map_err(|error| vec![Refusal::Artifact(error.to_string())])?;
    let snapshot = snapshot::at(&dir).map_err(|error| vec![Refusal::Payload(error.to_string())])?;

    let mut refusals = Vec::new();
    let mut edges: Vec<Proposed> = Vec::new();
    let mut seen: Vec<(String, String, String)> = Vec::new();

    for link in &snapshot.links {
        let key = (link.from.clone(), link.relation.clone(), link.to.clone());
        if seen.contains(&key) {
            refusals.push(Refusal::RepeatedLink {
                from: link.from.clone(),
                relation: link.relation.clone(),
                to: link.to.clone(),
            });
            continue;
        }
        seen.push(key);

        // The relation, by the name it is declared under. An inverse name is
        // not accepted: an import writes the half it is told to write, and
        // resolving a name to the other end of a relation would silently move
        // which document the edge lands in.
        let Some(relation) = corpus
            .relations
            .relations
            .iter()
            .find(|known| known.name == link.relation)
        else {
            refusals.push(Refusal::UnknownRelation {
                relation: link.relation.clone(),
                from: link.from.clone(),
                to: link.to.clone(),
            });
            continue;
        };

        // The marker that makes an imported edge readable as one. Q4 puts
        // `created_by` on the relation type, so this is the only grain at which
        // the graph separates an import from an author. Writing an edge on any
        // other relation would spend that distinction, which is why it is a
        // refusal rather than a warning.
        if relation.created_by.as_deref() != Some(IMPORT) {
            let refusal = Refusal::NotAnImportRelation {
                relation: relation.name.clone(),
                created_by: relation.created_by.clone(),
            };
            if !refusals.contains(&refusal) {
                refusals.push(refusal);
            }
            continue;
        }

        // The far end. An imported edge ends on the external item it came from,
        // so the relation has to admit an anchor kind there. A relation whose
        // `to` names only document kinds is refused against the declaration
        // rather than against the link, because every link of it is wrong for
        // one reason.
        if !relation
            .to
            .iter()
            .any(|name| corpus.relations.anchor(name).is_some())
        {
            let refusal = Refusal::NoAnchorEnd {
                relation: relation.name.clone(),
                permitted: relation.to.clone(),
            };
            if !refusals.contains(&refusal) {
                refusals.push(refusal);
            }
            continue;
        }

        // The near end: a document of this corpus that resolved a kind and
        // carries the identifier. `Index::node` is the node set, so a document
        // the census did not type is not found here, which is the right answer:
        // an edge out of an untyped document is an edge out of a non-node.
        let Some(node) = corpus.index.node(&link.from) else {
            refusals.push(Refusal::NoSuchSource {
                from: link.from.clone(),
                relation: link.relation.clone(),
                to: link.to.clone(),
            });
            continue;
        };
        let kind = node.kind.clone().unwrap_or_default();
        if !relation
            .from
            .iter()
            .any(|allowed| corpus.shape.descends_from(&kind, allowed))
        {
            refusals.push(Refusal::SourceKindRefused {
                from: link.from.clone(),
                kind,
                relation: relation.name.clone(),
                permitted: relation.from.clone(),
            });
            continue;
        }

        // The item, which has to be one the snapshot pinned. An edge into an
        // item the snapshot does not name would record no revision, and Q19's
        // drift report is per edge.
        let Some(item) = snapshot.item(&link.to) else {
            refusals.push(Refusal::NoSuchItem {
                to: link.to.clone(),
                from: link.from.clone(),
                relation: link.relation.clone(),
            });
            continue;
        };

        let present = write::declares(root, &node.path, &link.relation, &link.to, &item.revision);
        edges.push(Proposed {
            from: link.from.clone(),
            path: node.path.clone(),
            relation: link.relation.clone(),
            to: link.to.clone(),
            verified_revision: item.revision.clone(),
            present,
        });
    }

    match refusals.is_empty() {
        true => Ok(Plan {
            declaration: declaration.clone(),
            release,
            snapshot,
            edges,
        }),
        false => Err(refusals),
    }
}

impl Plan {
    /// The edges this plan would write, which excludes the ones already there.
    pub fn to_write(&self) -> Vec<&Proposed> {
        self.edges.iter().filter(|edge| !edge.present).collect()
    }

    /// The plan, rendered for a reader.
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "import {} from {}\n",
            self.declaration.name, self.snapshot.source
        ));
        out.push_str(&format!("  fetched {}\n", self.snapshot.fetched));
        out.push_str(&format!("  digest {}\n", self.release.digest));
        out.push_str(&format!(
            "  channel {}\n",
            self.declaration.channel.as_deref().unwrap_or_default()
        ));
        out.push_str(&format!(
            "  {} items, {} links\n",
            self.snapshot.items.len(),
            self.snapshot.links.len()
        ));
        for edge in &self.edges {
            out.push_str(&format!(
                "  {} {} {} at revision {}{}\n",
                edge.from,
                edge.relation,
                edge.to,
                edge.verified_revision,
                match edge.present {
                    true => " — already declared",
                    false => "",
                }
            ));
        }
        out
    }
}
