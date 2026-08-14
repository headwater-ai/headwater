// SPDX-License-Identifier: Apache-2.0
//! External-anchor resolution: normalize a target string, then bind it.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-correctness-roots)
//! names this a correctness root and says exactly how it fails: "a resolver
//! that mis-normalizes makes `governs` edges silently miss". Nothing raises an
//! error when it happens. Write-time impact detection fires on anchor identity
//! ([spec 5](../../../../docs/spec/05-ai-integration.md)), so an anchor that
//! normalizes to a second spelling of one target is a second node, and every
//! document that governs the first one stops reaching the code that changed.
//!
//! # Three outcomes, and the third is never reported as the second
//!
//! [Spec 1](../../../../docs/spec/01-conceptual-model.md#external-anchor) fixes
//! the set: an anchor resolves, or it fails to resolve, or its target sits
//! behind a declared withholding. The third is [`Binding::Withheld`], and
//! nothing produces it yet, because an export filter is
//! [M6](https://github.com/headwater-ai/headwater/milestone/6) and no profile
//! exists to withhold anything. The variant is declared now rather than added
//! later, so that the day a filter arrives the compiler lists every site that
//! has to tell the two apart, which is
//! [Q1](../../../../docs/spec/09-decisions.md#q1--implementation-language)'s
//! argument for a closed set.
//!
//! # A resolver never reaches the network
//!
//! Spec 2 states the rule and gives both reasons: check time stays offline, and
//! a resolution result stays reproducible. [`SourceTree`] reads the working
//! tree under the corpus base. The second resolver this specification names, a
//! committed snapshot of an external system of record, reads committed files by
//! the same rule and lives in `headwater_import::anchors`. The third, a pinned
//! corpus export at the federation tier, does not exist yet.
//!
//! # Why the snapshot resolver is not in this file
//!
//! A snapshot resolver reads a snapshot, and the crate that reads one is
//! `headwater-import`, which already depends on this crate for
//! [`crate::declarations::Declarations`] and [`crate::index::Index`]. So this
//! crate cannot name it, and [`Resolvers::with`] is how a caller that has both
//! puts them together. That is a property of the dependency order rather than a
//! preference: the alternative moves the snapshot format into the graph crate,
//! and Q19 rules that the shape of a snapshot is a property of its resolver.

use headwater_census::walk::{Corpus, Exclusion};
use std::path::PathBuf;

/// What a resolver made of one anchor string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Binding {
    Resolved {
        /// The normalized string. Two spellings of one target normalize to one
        /// value, and that value is the node's identity.
        normalized: String,
        /// Set when the target lies under a path the corpus declared is not
        /// corpus content. The anchor still resolves: the file is there, and a
        /// `governs` edge over it is the thing write-time impact detection
        /// reads. What the note carries is the exclusion that claims it, so
        /// that a reader is not told a document governs something the corpus
        /// has said it does not hold.
        excluded_by: Option<String>,
    },
    /// A defect, and the message says which one.
    Unresolved(String),
    /// The source that owns the target withheld it under a declared export
    /// filter. Somebody's declared decision, and never a defect.
    Withheld { profile: String },
}

/// The single component that owns identity for one anchor type.
pub trait Resolver {
    /// The name an anchor kind names it by.
    fn name(&self) -> &str;

    /// Normalize the string, then bind it.
    fn resolve(&self, raw: &str) -> Binding;
}

/// Every resolver a run has, by name.
#[derive(Default)]
pub struct Resolvers {
    entries: Vec<Box<dyn Resolver>>,
}

impl Resolvers {
    pub fn new(entries: Vec<Box<dyn Resolver>>) -> Self {
        Self { entries }
    }

    /// The resolvers a run over a repository has today: the source tree, and
    /// nothing else. Every other anchor kind the taxonomy declares reports that
    /// no resolver claims it, which is the finding spec 2 asks for.
    pub fn over(corpus: &Corpus) -> Self {
        Self::new(vec![Box::new(SourceTree::over(corpus))])
    }

    /// Add one resolver that this crate cannot build, and refuse a second of
    /// one name.
    ///
    /// Spec 2 rules that "every anchor kind names exactly one resolver, and no
    /// two anchor kinds claim the same resolver namespace". Two entries of one
    /// name would leave [`Resolvers::get`] answering with whichever was pushed
    /// first, so one of the two would own an identity in silence. That is the
    /// same failure the module comment opens with, one level up, so it is a
    /// refusal the caller has to handle rather than a value this returns.
    pub fn with(mut self, resolver: Box<dyn Resolver>) -> Result<Self, String> {
        if let Some(held) = self.get(resolver.name()) {
            return Err(format!(
                "two resolvers are both named `{}`, and exactly one component owns each anchor \
                 identity",
                held.name()
            ));
        }
        self.entries.push(resolver);
        Ok(self)
    }

    pub fn get(&self, name: &str) -> Option<&dyn Resolver> {
        self.entries
            .iter()
            .find(|resolver| resolver.name() == name)
            .map(|resolver| resolver.as_ref())
    }
}

impl std::fmt::Debug for Resolvers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let names: Vec<&str> = self.entries.iter().map(|entry| entry.name()).collect();
        f.debug_struct("Resolvers")
            .field("entries", &names)
            .finish()
    }
}

/// The `source-tree` resolver: a path in the repository that holds the corpus.
#[derive(Clone, Debug)]
pub struct SourceTree {
    base: PathBuf,
    /// The corpus exclusions, so that a hit inside one says so.
    exclusions: Vec<Exclusion>,
}

impl SourceTree {
    pub fn over(corpus: &Corpus) -> Self {
        Self {
            base: corpus.base.clone(),
            exclusions: corpus.exclusions.clone(),
        }
    }
}

impl Resolver for SourceTree {
    fn name(&self) -> &str {
        "source-tree"
    }

    fn resolve(&self, raw: &str) -> Binding {
        let normalized = match normalize(raw) {
            Ok(normalized) => normalized,
            Err(why) => return Binding::Unresolved(why),
        };

        if !self.base.join(&normalized).exists() {
            return Binding::Unresolved(format!("no `{normalized}` in the source tree"));
        }

        let excluded_by = self
            .exclusions
            .iter()
            .find(|exclusion| exclusion.pattern.matches(&normalized))
            .map(|exclusion| exclusion.pattern.source().to_string());

        Binding::Resolved {
            normalized,
            excluded_by,
        }
    }
}

/// Normalize a repository path.
///
/// The rules are written out rather than delegated to a path library, and the
/// reason is the failure mode: a library that resolves `..` against the real
/// filesystem follows a symlink out of the repository, and two different
/// strings would then be one node for a reason nothing in the corpus states.
/// Everything below is lexical.
///
/// - `\` becomes `/`, because a path written on Windows names the same file.
/// - A leading `./` goes, and so does every interior `.` segment.
/// - A `..` segment cancels the segment before it.
/// - A trailing `/` goes, so that a directory has one spelling.
/// - An absolute path, an empty path, and a path that climbs above the base are
///   refused rather than clamped. Each one is a defect in the anchor, and
///   clamping would turn it into a hit on a file the author did not name.
pub fn normalize(raw: &str) -> Result<String, String> {
    let text = raw.trim().replace('\\', "/");
    if text.is_empty() {
        return Err("an empty anchor names nothing".to_string());
    }
    if text.starts_with('/') {
        return Err(format!(
            "`{text}` is absolute, and an anchor is relative to the repository"
        ));
    }

    let mut segments: Vec<&str> = Vec::new();
    for segment in text.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                if segments.pop().is_none() {
                    return Err(format!("`{text}` climbs above the repository"));
                }
            }
            other => segments.push(other),
        }
    }
    if segments.is_empty() {
        return Err(format!("`{text}` normalizes to nothing"));
    }
    Ok(segments.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_spellings_of_one_target_normalize_to_one_node() {
        // The whole correctness-root argument in one assertion. Each of these
        // is the same file, and a resolver that made two nodes of them would
        // make every `governs` edge over the second one miss in silence.
        for spelling in [
            "engine/crates/graph/src/anchors.rs",
            "./engine/crates/graph/src/anchors.rs",
            "engine/crates/doc/../graph/src/anchors.rs",
            "engine//crates/./graph/src/anchors.rs",
            "engine\\crates\\graph\\src\\anchors.rs",
            "  engine/crates/graph/src/anchors.rs  ",
        ] {
            assert_eq!(
                normalize(spelling).expect("normalizes"),
                "engine/crates/graph/src/anchors.rs",
                "{spelling}"
            );
        }
    }

    #[test]
    fn a_directory_has_one_spelling() {
        assert_eq!(normalize("docs/spec/").unwrap(), "docs/spec");
    }

    #[test]
    fn a_path_that_leaves_the_repository_is_refused_and_never_clamped() {
        assert!(normalize("/etc/passwd").is_err());
        assert!(normalize("../outside").is_err());
        assert!(normalize("docs/../../outside").is_err());
        assert!(normalize("   ").is_err());
        assert!(normalize("./").is_err());
    }

    #[test]
    fn a_path_that_is_not_in_the_tree_is_unresolved_and_says_which_path() {
        let resolver = SourceTree {
            base: PathBuf::from(env!("CARGO_MANIFEST_DIR")),
            exclusions: Vec::new(),
        };
        assert!(matches!(
            resolver.resolve("Cargo.toml"),
            Binding::Resolved { .. }
        ));
        let Binding::Unresolved(why) = resolver.resolve("src/invented.rs") else {
            panic!("a path that is not there resolved");
        };
        assert!(why.contains("src/invented.rs"), "{why}");
    }

    /// A resolver that answers whatever it was built to answer, so that the set
    /// can be tested without a corpus and without a snapshot.
    struct Fixed(&'static str);

    impl Resolver for Fixed {
        fn name(&self) -> &str {
            self.0
        }

        fn resolve(&self, _raw: &str) -> Binding {
            Binding::Unresolved("a fixture resolver binds nothing".to_string())
        }
    }

    #[test]
    fn a_resolver_the_caller_supplies_joins_the_set_and_is_found_by_name() {
        let resolvers = Resolvers::default()
            .with(Box::new(Fixed("ado-snapshot")))
            .expect("the set had no resolver of that name");
        assert!(resolvers.get("ado-snapshot").is_some());
        assert!(resolvers.get("source-tree").is_none());
    }

    /// Spec 2: exactly one component owns each anchor identity. A second entry
    /// of one name is refused rather than shadowed, because `get` would answer
    /// with the first one and nothing would say the second was ignored.
    #[test]
    fn a_second_resolver_of_one_name_is_refused_rather_than_shadowed() {
        let why = Resolvers::default()
            .with(Box::new(Fixed("ado-snapshot")))
            .expect("the first one lands")
            .with(Box::new(Fixed("ado-snapshot")))
            .expect_err("the second one is refused");
        assert!(why.contains("ado-snapshot"), "{why}");
    }

    #[test]
    fn a_hit_inside_a_declared_exclusion_resolves_and_names_the_rule() {
        let resolver = SourceTree {
            base: PathBuf::from(env!("CARGO_MANIFEST_DIR")),
            exclusions: vec![Exclusion::new("src/**", "a fixture exclusion")],
        };
        let Binding::Resolved { excluded_by, .. } = resolver.resolve("src/anchors.rs") else {
            panic!("the file is there, so the anchor resolves");
        };
        assert_eq!(excluded_by.as_deref(), Some("src/**"));
    }
}
