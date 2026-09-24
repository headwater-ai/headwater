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

use headwater_census::walk::{Corpus, Entry, EntryKind, Exclusion};
use headwater_meta::pattern::Pattern;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// What a resolver made of one anchor string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Binding {
    Resolved {
        /// The normalized string. Two spellings of one target normalize to one
        /// value, and that value is the node's identity.
        normalized: String,
        /// The tree entries this one anchor string matched, sorted and with no
        /// duplicate, and never empty for a binding this variant carries.
        ///
        /// [HW-DR-0074](../../../../docs/decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md):
        /// a value with no wildcard names one entry, itself, so this is
        /// `vec![normalized.clone()]` there. A pattern with a wildcard names
        /// however many entries the tree holds under it, excluding an entry a
        /// corpus exclusion claims. `headwater explain` reads this for the
        /// count a reader is owed, and [`crate::edges::Target::resolution`]
        /// renders it into a cache key, so a file added or removed under a
        /// governed subtree is a different key
        /// ([HW-OBL-0117](../../../../docs/obligations/0117-a-cached-verdict-about-an-anchor-survives-the-change-that-falsifies-it.md)).
        matched: Vec<String>,
        /// Set when the target lies under a path the corpus declared is not
        /// corpus content. The anchor still resolves: the file is there, and a
        /// `governs` edge over it is the thing write-time impact detection
        /// reads. What the note carries is the exclusion that claims it, so
        /// that a reader is not told a document governs something the corpus
        /// has said it does not hold.
        excluded_by: Option<String>,
        /// What the resolver's source says the target is at now, and `None`
        /// where the resolver has no such notion.
        ///
        /// A committed snapshot is the one resolver that has one: it pins an
        /// identity and a revision for every item in it, so the answer carries
        /// both. [`SourceTree`] answers `None`, because a path in a working
        /// tree is at no revision that this engine can name, and inventing a
        /// commit for it would put a value in a cache key that no source
        /// stated.
        ///
        /// Two components read it. `headwater_check::suspect` compares it
        /// against the `verified_revision` an import wrote onto the edge, which
        /// is the drift report [spec 7](../../../../docs/spec/07-distribution-and-federation.md#upstream-awareness)
        /// asks for per edge. And `Target::resolution` renders it into the
        /// cache key of every edge instance, so a verdict about an edge cannot
        /// outlive the advance that falsifies it
        /// ([#160](https://github.com/headwater-ai/headwater/issues/160)).
        revision: Option<String>,
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

    /// Bind the string for an edge that `asserter`, a document identifier,
    /// declares. Every production binding goes through this method. A
    /// resolver whose verdict does not depend on who asserts the edge keeps
    /// the default, which is [`Resolver::resolve`]. [`CommentScan`] does not:
    /// its verdict is whether the target cites the asserter (#967).
    fn resolve_for(&self, raw: &str, asserter: &str) -> Binding {
        let _ = asserter;
        self.resolve(raw)
    }
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
    /// Every subtree this resolver has already walked in this run, keyed by
    /// the literal prefix a wildcard pattern rooted the walk at.
    ///
    /// [HW-DR-0074](../../../../docs/decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md):
    /// "the resolver reads the tree once, not once per edge." Two edges whose
    /// patterns share a prefix, or one prefix a list anchor names twice, walk
    /// the filesystem once between them. The cache is scoped to this
    /// resolver's own lifetime — one `headwater check` process builds one
    /// [`Resolvers`] and so one `SourceTree` — and it never crosses a run.
    /// That is the cache `.headwater/cache/checks` answers for
    /// ([HW-OBL-0117](../../../../docs/obligations/0117-a-cached-verdict-about-an-anchor-survives-the-change-that-falsifies-it.md)),
    /// and this one does not touch it: a fresh process still reads a fresh
    /// tree, because it builds a fresh `SourceTree`.
    walked: RefCell<HashMap<String, Rc<[Entry]>>>,
}

impl SourceTree {
    pub fn over(corpus: &Corpus) -> Self {
        Self {
            base: corpus.base.clone(),
            exclusions: corpus.exclusions.clone(),
            walked: RefCell::new(HashMap::new()),
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

        let pattern = Pattern::new(&normalized);
        if pattern.is_literal() {
            // Unchanged from before a pattern language reached this resolver:
            // one `exists` call, and the one entry this value names is itself.
            //
            // This is also the one case where an excluded target still
            // resolves rather than counting as no match. The two branches
            // answer two different questions. This one already confirmed the
            // target *exists* — `exists()` just returned true — so it reports
            // what the corpus says about that real file, which is metadata
            // ("declared not to be content") and never a reason to call the
            // edge broken: `excluded_by` carries the note, and a document
            // that governs an excluded file keeps governing it, which is the
            // behavior every edge onto one already had before this ruling and
            // HW-DR-0074's Decision section keeps ("every edge this corpus
            // declares keeps the meaning it has"). The wildcard branch below
            // is answering a coverage question instead — does *some* real,
            // included entry back this claim at all — and there "an entry the
            // corpus excludes is not a match" is exactly the rule HW-DR-0074
            // states, because a set with only excluded entries in it is the
            // stale, over-broad claim `relation.target.unresolved` exists to
            // raise. Changing this branch to match that rule would break a
            // value with no wildcard resolving "exactly as it does on
            // `origin/main` today", which the issue's Done-when requires.
            if !self.base.join(&normalized).exists() {
                return Binding::Unresolved(format!("no `{normalized}` in the source tree"));
            }

            let excluded_by = self
                .exclusions
                .iter()
                .find(|exclusion| exclusion.pattern.matches(&normalized))
                .map(|exclusion| exclusion.pattern.source().to_string());

            return Binding::Resolved {
                matched: vec![normalized.clone()],
                normalized,
                excluded_by,
                // A path in a working tree is at no revision this resolver can
                // name. See the field.
                revision: None,
            };
        }

        // A pattern with a wildcard: search the one subtree no path it admits
        // can lie outside of, and keep every non-excluded entry it matches.
        let prefix = pattern.literal_prefix();
        if prefix.is_empty() {
            // A pattern that opens with `**` or `*` fixes no leading segment,
            // so the one subtree "no path it admits can lie outside of" is
            // the whole tree this resolver was given — every repository this
            // engine has ever measured opens every one of its own patterns on
            // a literal directory (see `Pattern::literal_prefix`'s doc
            // comment), and a walk with no root also has no way to write a
            // relative path for what it finds: `headwater_census::walk`'s
            // `descend` starts one level of recursion in with an empty
            // `prefix` and unconditionally writes `{prefix}/{name}`, which is
            // `/name` rather than `name` for every entry at the top of that
            // walk. Refused rather than fixed with a leading-separator strip,
            // because the cost this shape pays even fixed — a walk of
            // everything under the repository root, `.git` and every
            // `target/` included — is the one this repository's own patterns
            // never pay and a caller almost certainly did not mean to.
            return Binding::Unresolved(format!(
                "`{normalized}` opens with a wildcard, and no literal segment bounds the search; \
                 write a literal directory before the first `*` or `**`"
            ));
        }
        // HW-DR-0074: "the resolver reads the tree once, not once per edge."
        // Two patterns that share a prefix — two edges, or two members of one
        // list — walk the filesystem once between them rather than once each.
        let entries = {
            let mut walked = self.walked.borrow_mut();
            match walked.get(&prefix) {
                Some(entries) => Rc::clone(entries),
                None => {
                    let scoped =
                        Corpus::new(self.base.clone(), &prefix).excluding(self.exclusions.clone());
                    let entries: Rc<[Entry]> = headwater_census::walk::walk(&scoped).into();
                    walked.insert(prefix.clone(), Rc::clone(&entries));
                    entries
                }
            }
        };
        let mut matched: Vec<String> = entries
            .iter()
            .filter(|entry: &&Entry| entry.excluded_by.is_none())
            .filter(|entry| matches!(entry.kind, EntryKind::File | EntryKind::Symlink { .. }))
            .map(|entry| entry.path.clone())
            .filter(|path| pattern.matches(path))
            .collect();
        matched.sort();
        matched.dedup();

        if matched.is_empty() {
            return Binding::Unresolved(format!(
                "no entry in the source tree matches `{normalized}`"
            ));
        }

        Binding::Resolved {
            normalized,
            excluded_by: None,
            revision: None,
            matched,
        }
    }
}

/// The `comment-scan` resolver: a path binds only where a comment inside it
/// cites the identifier of the document that asserts the edge, and that
/// identifier is minted.
///
/// [HW-DR-0073](../../../../docs/decisions/0073-a-verification-is-a-kind-and-its-identity-is-minted-rather-than-found-in-the-code-that-cites-it.md)
/// ruling 4: "A `comment-scan` resolver ships in the binary. The default
/// reads a pattern declared in the overlay, so the convention stays in
/// configuration and out of the engine." [`AnchorKind::pattern`](crate::declarations::AnchorKind::pattern)
/// is that declaration, and `engine/crates/cli/src/main.rs` is where a
/// corpus that declares no pattern for this resolver ends up with none
/// registered, because there is nothing there to build one from.
///
/// # It reads bytes, and that is the whole argument the amendment rests on
///
/// [`SourceTree::resolve`] calls `Path::exists` and never opens the file.
/// This resolver opens it, and calls [`crate::comments::rust_comment_text`]
/// on what it reads, which is the one primitive this file and
/// `headwater_check::fragment::comment_links` both call rather than each
/// hand-rolling a Rust comment tokenizer. So a `governs` edge onto a
/// `test_site` anchor is the first anchor binding in this engine whose
/// verdict is a function of the bytes at the target, and never only of
/// whether the target exists.
///
/// # The pattern is a prefix, and the reason is the namespace rule
///
/// A resolved identifier scheme in this corpus is disjoint from every other
/// scheme's literal segment (`.headwater/overlay.yml`'s own words: "no two
/// schemes admit one string"). So a candidate token that starts with the
/// declared prefix cannot be mistaken for an identifier of a different
/// scheme, and a prefix is enough: this resolver does not have to parse the
/// rest of an identifier scheme's `{namespace}-VER-{seq:04d}` template to
/// tell a citation from ordinary prose.
///
/// # Whose identifier, and why order does not matter
///
/// A citation proves something about the document it names and about no
/// other. So the only citation that binds an edge is the identifier of the
/// document that asserts it, which [`Resolver::resolve_for`] passes in, and
/// a file that cites some other minted identifier binds nothing (#967:
/// before this, the first minted token in the file bound any edge onto it).
/// Every candidate in the file is read, so the asserter's citation binds
/// wherever it sits, and a file that serves two verifications can cite
/// both. [`Resolver::resolve`], which names no asserter, binds nothing.
///
/// # What "minted" means here, and why this reads no document
///
/// [HW-DR-0054](../../../../docs/decisions/0054-the-upper-bound-of-a-reconcile-first-allocator-is-the-corpus-and-a-claim-store.md)
/// gives a minted identifier a file of its own at
/// `.headwater/ids/<scheme>/<identifier>`, written once, before any document
/// graph exists. [`Resolvers::over`] and every call to [`Resolvers::with`] run
/// before this crate builds an [`crate::index::Index`] over the census, so a
/// resolver built at this point cannot ask the graph whether an identifier is
/// real. The claim store answers the same question from disk, at the same
/// time every other resolver is built, which is the property
/// [`CommentScan::claimed`] uses.
#[derive(Clone, Debug)]
pub struct CommentScan {
    base: PathBuf,
    /// The literal segment a citation must open with. Read from
    /// [`AnchorKind::pattern`](crate::declarations::AnchorKind::pattern), and
    /// never compiled in.
    prefix: String,
    /// Every identifier this corpus has a claim file for, of any scheme. The
    /// prefix above is what tells one scheme's citations from another's, so
    /// this set does not need to be scoped to one scheme itself.
    minted: std::collections::BTreeSet<String>,
}

impl CommentScan {
    pub fn new(
        base: impl Into<PathBuf>,
        prefix: impl Into<String>,
        minted: std::collections::BTreeSet<String>,
    ) -> Self {
        Self {
            base: base.into(),
            prefix: prefix.into(),
            minted,
        }
    }

    /// Every identifier this repository's claim store holds, read fresh off
    /// disk. [`crate::declarations::AnchorKind`]'s own doc comment states why
    /// this cannot instead be a lookup into the document graph.
    ///
    /// A repository with no store yet, or no `.headwater/ids` directory at
    /// all, answers the empty set rather than an error: an empty corpus has
    /// minted nothing, and that is a fact rather than a defect this resolver
    /// exists to report.
    pub fn claimed(base: &Path) -> std::collections::BTreeSet<String> {
        let mut out = std::collections::BTreeSet::new();
        let Ok(schemes) = std::fs::read_dir(base.join(".headwater/ids")) else {
            return out;
        };
        for scheme in schemes.flatten() {
            let Ok(entries) = std::fs::read_dir(scheme.path()) else {
                continue;
            };
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    out.insert(name.to_string());
                }
            }
        }
        out
    }

    /// The identifier-shaped tokens of `text` that open with `prefix`: a
    /// maximal run of ASCII letters, digits and hyphens, with a trailing
    /// hyphen trimmed off so that a sentence's punctuation is not read as
    /// part of the token that precedes it.
    fn candidates<'a>(text: &'a str, prefix: &str) -> Vec<&'a str> {
        let bytes = text.as_bytes();
        let mut out = Vec::new();
        let mut i = 0;
        while i < bytes.len() {
            if (bytes[i] as char).is_ascii_alphanumeric() {
                let start = i;
                while i < bytes.len()
                    && ((bytes[i] as char).is_ascii_alphanumeric() || bytes[i] == b'-')
                {
                    i += 1;
                }
                let mut end = i;
                while end > start && bytes[end - 1] == b'-' {
                    end -= 1;
                }
                let token = &text[start..end];
                if token.starts_with(prefix) {
                    out.push(token);
                }
            } else {
                i += 1;
            }
        }
        out
    }
}

impl Resolver for CommentScan {
    fn name(&self) -> &str {
        "comment-scan"
    }

    /// Refuses. A citation binds an edge only for the document that asserts
    /// it, and this call names none. See [`CommentScan`].
    fn resolve(&self, raw: &str) -> Binding {
        Binding::Unresolved(format!(
            "`comment-scan` binds `{raw}` only for the document that asserts the edge, \
             and none was named"
        ))
    }

    fn resolve_for(&self, raw: &str, asserter: &str) -> Binding {
        let normalized = match normalize(raw) {
            Ok(normalized) => normalized,
            Err(why) => return Binding::Unresolved(why),
        };

        let Ok(source) = std::fs::read_to_string(self.base.join(&normalized)) else {
            return Binding::Unresolved(format!("no `{normalized}` in the source tree"));
        };

        // An asserter outside the declared prefix can never be cited, because
        // `candidates` returns only tokens that open with it. So the refusal
        // names the identifier and the prefix, and never a citation to add.
        if !asserter.starts_with(self.prefix.as_str()) {
            return Binding::Unresolved(format!(
                "`{asserter}`, the document that asserts this edge, does not start `{}`, so no \
                 comment in `{normalized}` can cite it",
                self.prefix
            ));
        }

        let text = crate::comments::rust_comment_text(&source);
        let candidates = Self::candidates(&text, &self.prefix);
        let Some(first) = candidates.first() else {
            return Binding::Unresolved(format!(
                "no comment in `{normalized}` cites an identifier starting `{}`",
                self.prefix
            ));
        };

        if !candidates.contains(&asserter) {
            return Binding::Unresolved(format!(
                "`{normalized}` cites `{first}`, not `{asserter}`, the document that asserts \
                 this edge"
            ));
        }

        if self.minted.contains(asserter) {
            Binding::Resolved {
                matched: vec![normalized.clone()],
                normalized,
                excluded_by: None,
                // A claim file names no revision, and neither does the
                // source tree it sits beside. See `SourceTree::resolve`.
                revision: None,
            }
        } else {
            Binding::Unresolved(format!(
                "`{asserter}`, cited in `{normalized}`, is shaped like an identifier this \
                 corpus mints, and no document mints it"
            ))
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
            walked: RefCell::new(HashMap::new()),
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

    /// A directory this test builds and tears down, so the pattern fixtures
    /// below do not depend on the shape of this crate's own `src/` staying
    /// still. Mirrors `scratch` below, which the `CommentScan` tests already
    /// use for the same reason.
    fn pattern_fixture(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("a clock later than the epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "headwater-source-tree-pattern-{name}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(dir.join(".claude/hooks")).expect("a fixture tree");
        std::fs::create_dir_all(dir.join(".claude/hooks-disabled")).expect("a fixture tree");
        std::fs::write(dir.join(".claude/hooks/write.sh"), "#!/bin/sh\n").expect("a fixture file");
        std::fs::write(dir.join(".claude/hooks-disabled/write.sh"), "#!/bin/sh\n")
            .expect("a fixture file");
        dir
    }

    /// The decisive fixture HW-DR-0074 names: a naive prefix test, or a bare
    /// `starts_with`, passes `.claude/hooks-disabled/write.sh` for an anchor
    /// written `.claude/hooks/**`, and this is the one case that catches it in
    /// either direction.
    #[test]
    fn a_wildcard_anchor_reaches_its_own_subtree_and_never_a_sibling_that_shares_a_prefix() {
        let dir = pattern_fixture("boundary");
        let resolver = SourceTree {
            base: dir.clone(),
            exclusions: Vec::new(),
            walked: RefCell::new(HashMap::new()),
        };

        let Binding::Resolved { matched, .. } = resolver.resolve(".claude/hooks/**") else {
            panic!("the pattern matches a real file under it");
        };
        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(matched, vec![".claude/hooks/write.sh".to_string()]);
        assert!(
            !matched.contains(&".claude/hooks-disabled/write.sh".to_string()),
            "{matched:?}"
        );
    }

    /// A directory named with no wildcard matches the directory entry alone,
    /// which is the existing, unchanged behavior a value with no wildcard
    /// keeps under HW-DR-0074.
    #[test]
    fn a_bare_directory_matches_the_directory_and_nothing_under_it() {
        let dir = pattern_fixture("bare-directory");
        let resolver = SourceTree {
            base: dir.clone(),
            exclusions: Vec::new(),
            walked: RefCell::new(HashMap::new()),
        };

        let Binding::Resolved { matched, .. } = resolver.resolve(".claude/hooks") else {
            panic!("the directory is there, so the anchor resolves");
        };
        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(matched, vec![".claude/hooks".to_string()]);
    }

    /// A pattern that matches no entry is unresolved and names itself, exactly
    /// as a path that is not in the tree does today.
    #[test]
    fn a_pattern_that_matches_no_entry_is_unresolved_and_names_the_pattern() {
        let dir = pattern_fixture("no-match");
        let resolver = SourceTree {
            base: dir.clone(),
            exclusions: Vec::new(),
            walked: RefCell::new(HashMap::new()),
        };

        let Binding::Unresolved(why) = resolver.resolve(".claude/nothing-here/*.sh") else {
            panic!("a pattern with no match resolved");
        };
        std::fs::remove_dir_all(&dir).ok();

        assert!(why.contains(".claude/nothing-here/*.sh"), "{why}");
    }

    /// A pattern with no literal segment before its first wildcard is
    /// refused rather than walked. `literal_prefix()` answers `""` there, and
    /// a walk rooted at that empty prefix is the one case
    /// `headwater_census::walk::descend` writes a leading `/` for every entry
    /// at its top level: it starts one recursion in with `prefix = ""` and
    /// unconditionally writes `{prefix}/{name}`.
    #[test]
    fn a_pattern_with_no_literal_prefix_is_refused_rather_than_walked_from_the_root() {
        let dir = pattern_fixture("no-prefix");
        let resolver = SourceTree {
            base: dir.clone(),
            exclusions: Vec::new(),
            walked: RefCell::new(HashMap::new()),
        };

        let Binding::Unresolved(why) = resolver.resolve("**/write.sh") else {
            panic!("a pattern with no literal prefix resolved");
        };
        std::fs::remove_dir_all(&dir).ok();

        assert!(why.contains("**/write.sh"), "{why}");
        assert!(why.contains("wildcard"), "{why}");
    }

    /// An entry the corpus excludes does not count as a match, so a pattern
    /// whose every hit is excluded is unresolved rather than resolved on the
    /// strength of a file the corpus has said is not its content.
    #[test]
    fn an_excluded_entry_is_not_a_match() {
        let dir = pattern_fixture("excluded");
        let resolver = SourceTree {
            base: dir.clone(),
            exclusions: vec![Exclusion::new(".claude/hooks/**", "a fixture exclusion")],
            walked: RefCell::new(HashMap::new()),
        };

        let Binding::Unresolved(why) = resolver.resolve(".claude/hooks/**") else {
            panic!("every match is excluded, so the pattern should not resolve");
        };
        std::fs::remove_dir_all(&dir).ok();

        assert!(why.contains(".claude/hooks/**"), "{why}");
    }

    /// [HW-OBL-0117](../../../../docs/obligations/0117-a-cached-verdict-about-an-anchor-survives-the-change-that-falsifies-it.md):
    /// a file added under a governed `**` anchor is a different matched set,
    /// and [`crate::edges::Target::resolution`] is the derived `Debug` of the
    /// whole binding, so a cache keyed on it goes cold rather than serving the
    /// verdict of the tree before the file arrived.
    #[test]
    fn a_file_added_under_a_wildcard_anchor_changes_the_matched_set_and_so_the_cache_key() {
        let dir = pattern_fixture("cache-key");
        let build = || SourceTree {
            base: dir.clone(),
            exclusions: Vec::new(),
            walked: RefCell::new(HashMap::new()),
        };

        // Two resolvers, not one asked twice: `SourceTree::walked` caches a
        // walk for its own lifetime, and one `headwater check` process builds
        // one `SourceTree` (see the field's doc comment). What this proves —
        // a stale in-process cache never masking the file that arrived
        // between two runs — needs the tree read by a resolver each run
        // actually builds fresh, exactly as two runs would.
        let before = build().resolve(".claude/hooks/**");
        std::fs::write(dir.join(".claude/hooks/second.sh"), "#!/bin/sh\n")
            .expect("a second fixture file");
        let after = build().resolve(".claude/hooks/**");
        std::fs::remove_dir_all(&dir).ok();

        let (
            Binding::Resolved {
                matched: before, ..
            },
            Binding::Resolved { matched: after, .. },
        ) = (&before, &after)
        else {
            panic!("both resolve: {before:?} {after:?}");
        };
        assert_ne!(
            before, after,
            "a file added under the pattern left the matched set unchanged"
        );
        assert_eq!(after.len(), before.len() + 1, "{after:?}");

        // The property a cache key rests on: two `Target::Anchor` bindings
        // that hold two different matched sets render two different
        // resolutions, which is what makes a warm cache miss rather than
        // serve the answer the tree gave before the file arrived.
        let member = |matched: &[String]| crate::edges::PatternMember {
            pattern: ".claude/hooks/**".to_string(),
            matched: matched.to_vec(),
        };
        let anchor = |matched: &[String]| crate::edges::Target::Anchor {
            anchor_kind: "code_path".to_string(),
            resolver: "source-tree".to_string(),
            normalized: ".claude/hooks/**".to_string(),
            excluded_by: None,
            revision: None,
            patterns: vec![member(matched)],
        };
        assert_ne!(
            anchor(before).resolution(),
            anchor(after).resolution(),
            "the cache key would serve a stale verdict across the new file"
        );
    }

    /// The positive half of the claim above: within one resolver's own
    /// lifetime, a second pattern over a prefix already walked reads the
    /// cached entries rather than the tree. Proven by mutating the tree
    /// *without* building a second `SourceTree` and showing the answer does
    /// not move — the one case that tells "cached" apart from "read fresh and
    /// happened to agree".
    #[test]
    fn two_patterns_that_share_a_prefix_walk_the_tree_once() {
        let dir = pattern_fixture("shared-prefix");
        let resolver = SourceTree {
            base: dir.clone(),
            exclusions: Vec::new(),
            walked: RefCell::new(HashMap::new()),
        };

        // Same prefix (`.claude/hooks`) as the pattern below, asked first so
        // the walk lands in the cache under that key.
        let first = resolver.resolve(".claude/hooks/**");
        let Binding::Resolved { matched: first, .. } = first else {
            panic!("the pattern matches a real file under it");
        };
        assert_eq!(first, vec![".claude/hooks/write.sh".to_string()]);

        // A file arrives, but on the same resolver instance: a fresh walk
        // would see it, and the cached one must not.
        std::fs::write(dir.join(".claude/hooks/second.sh"), "#!/bin/sh\n")
            .expect("a second fixture file");
        let second = resolver.resolve(".claude/hooks/*.sh");
        std::fs::remove_dir_all(&dir).ok();
        let Binding::Resolved {
            matched: second, ..
        } = second
        else {
            panic!("the pattern matches a real file under it");
        };
        assert_eq!(
            second,
            vec![".claude/hooks/write.sh".to_string()],
            "a second pattern over the same prefix re-walked the tree instead of reading the \
             cache, and so saw the file the first call could not have"
        );
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
            walked: RefCell::new(HashMap::new()),
        };
        let Binding::Resolved { excluded_by, .. } = resolver.resolve("src/anchors.rs") else {
            panic!("the file is there, so the anchor resolves");
        };
        assert_eq!(excluded_by.as_deref(), Some("src/**"));
    }

    /// A directory nothing else in this process writes into, on the same
    /// terms `engine/crates/check/src/fragment.rs`'s `comment_links` tests
    /// take: cargo runs a target's cases as threads of one process, so the
    /// pid alone is not a key.
    fn scratch(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("a clock later than the epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "headwater-comment-scan-{name}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("a scratch directory");
        dir
    }

    /// The decisive fixture: one file, two comments, and the resolver's
    /// verdict flips on the identifier the second line cites rather than on
    /// anything about the file itself.
    #[test]
    fn a_citation_shaped_like_an_identifier_this_corpus_mints_and_unminted_is_refused_and_named() {
        let dir = scratch("unminted");
        std::fs::write(dir.join("sample.rs"), "//! proves HW-VER-9999\nfn f() {}\n")
            .expect("a fixture file");

        let resolver = CommentScan::new(
            &dir,
            "HW-VER-",
            std::collections::BTreeSet::from(["HW-VER-0001".to_string()]),
        );
        let Binding::Unresolved(why) = resolver.resolve_for("sample.rs", "HW-VER-9999") else {
            panic!("an unminted citation resolved");
        };
        std::fs::remove_dir_all(&dir).ok();

        assert!(why.contains("HW-VER-9999"), "{why}");
        assert!(why.contains("no document mints it"), "{why}");
    }

    #[test]
    fn a_citation_of_a_minted_identifier_resolves() {
        let dir = scratch("minted");
        std::fs::write(dir.join("sample.rs"), "//! proves HW-VER-0001\nfn f() {}\n")
            .expect("a fixture file");

        let resolver = CommentScan::new(
            &dir,
            "HW-VER-",
            std::collections::BTreeSet::from(["HW-VER-0001".to_string()]),
        );
        let outcome = resolver.resolve_for("sample.rs", "HW-VER-0001");
        std::fs::remove_dir_all(&dir).ok();

        assert!(matches!(outcome, Binding::Resolved { .. }), "{outcome:?}");
    }

    #[test]
    fn a_file_with_no_citation_is_refused_and_says_so() {
        let dir = scratch("no-citation");
        std::fs::write(dir.join("sample.rs"), "//! nothing here cites anything\n")
            .expect("a fixture file");

        let resolver = CommentScan::new(&dir, "HW-VER-", std::collections::BTreeSet::new());
        let Binding::Unresolved(why) = resolver.resolve_for("sample.rs", "HW-VER-0001") else {
            panic!("a file with no citation resolved");
        };
        std::fs::remove_dir_all(&dir).ok();

        assert!(why.contains("no comment"), "{why}");
    }

    /// A citation inside a string literal is not a comment, exactly as
    /// `headwater_check::fragment::comment_links` already holds for a link.
    /// This is the one behavior [`CommentScan`] shares with that module by
    /// calling the same primitive rather than by agreeing with it twice.
    #[test]
    fn a_citation_inside_a_string_literal_is_not_a_comment() {
        let dir = scratch("string-literal");
        std::fs::write(
            dir.join("sample.rs"),
            "let s = \"HW-VER-0001\"; // no citation here\n",
        )
        .expect("a fixture file");

        let resolver = CommentScan::new(
            &dir,
            "HW-VER-",
            std::collections::BTreeSet::from(["HW-VER-0001".to_string()]),
        );
        let Binding::Unresolved(why) = resolver.resolve_for("sample.rs", "HW-VER-0001") else {
            panic!("a citation inside a string literal resolved");
        };
        std::fs::remove_dir_all(&dir).ok();

        assert!(why.contains("no comment"), "{why}");
    }

    /// #967: a file that cites a minted identifier of a different document
    /// proves nothing about the one that asserts the edge.
    #[test]
    fn a_citation_of_a_different_minted_identifier_is_refused_and_names_both() {
        let dir = scratch("other-minted");
        std::fs::write(dir.join("sample.rs"), "//! proves HW-VER-0002\nfn f() {}\n")
            .expect("a fixture file");

        let resolver = CommentScan::new(
            &dir,
            "HW-VER-",
            ["HW-VER-0001", "HW-VER-0002"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        let Binding::Unresolved(why) = resolver.resolve_for("sample.rs", "HW-VER-0001") else {
            panic!("a citation of another document's identifier resolved");
        };
        std::fs::remove_dir_all(&dir).ok();

        assert!(why.contains("HW-VER-0002"), "{why}");
        assert!(why.contains("HW-VER-0001"), "{why}");
    }

    /// The asserter's citation binds wherever it sits among the candidates.
    #[test]
    fn the_asserter_cited_second_still_binds() {
        let dir = scratch("two-citations");
        std::fs::write(
            dir.join("sample.rs"),
            "//! proves HW-VER-0002\n//! proves HW-VER-0001\nfn f() {}\n",
        )
        .expect("a fixture file");

        let resolver = CommentScan::new(
            &dir,
            "HW-VER-",
            ["HW-VER-0001", "HW-VER-0002"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        let outcome = resolver.resolve_for("sample.rs", "HW-VER-0001");
        std::fs::remove_dir_all(&dir).ok();

        assert!(matches!(outcome, Binding::Resolved { .. }), "{outcome:?}");
    }

    /// The bare `resolve` names no asserter, so it binds nothing, even where
    /// the file cites a minted identifier.
    #[test]
    fn the_bare_resolve_binds_no_citation() {
        let dir = scratch("bare-resolve");
        std::fs::write(dir.join("sample.rs"), "//! proves HW-VER-0001\nfn f() {}\n")
            .expect("a fixture file");

        let resolver = CommentScan::new(
            &dir,
            "HW-VER-",
            std::collections::BTreeSet::from(["HW-VER-0001".to_string()]),
        );
        let Binding::Unresolved(why) = resolver.resolve("sample.rs") else {
            panic!("the bare resolve bound a citation");
        };
        std::fs::remove_dir_all(&dir).ok();

        assert!(why.contains("the document that asserts the edge"), "{why}");
    }

    /// The asserter's own citation binds only if the asserter is minted. A
    /// different minted identifier beside it does not stand in for it, which
    /// is #967 in a second shape.
    #[test]
    fn an_unminted_asserter_cited_beside_a_minted_identifier_is_refused() {
        let dir = scratch("unminted-beside-minted");
        std::fs::write(
            dir.join("sample.rs"),
            "//! proves HW-VER-0001\n//! proves HW-VER-9999\nfn f() {}\n",
        )
        .expect("a fixture file");

        let resolver = CommentScan::new(
            &dir,
            "HW-VER-",
            std::collections::BTreeSet::from(["HW-VER-0001".to_string()]),
        );
        let Binding::Unresolved(why) = resolver.resolve_for("sample.rs", "HW-VER-9999") else {
            panic!("an unminted asserter bound because another citation is minted");
        };
        std::fs::remove_dir_all(&dir).ok();

        assert!(why.contains("HW-VER-9999"), "{why}");
        assert!(why.contains("no document mints it"), "{why}");
    }

    /// An asserter whose identifier lacks the declared prefix can never be
    /// cited, so the refusal names the identifier and the prefix and asks
    /// for no citation.
    #[test]
    fn an_asserter_outside_the_prefix_is_refused_and_names_the_prefix() {
        let dir = scratch("asserter-outside-prefix");
        std::fs::write(dir.join("sample.rs"), "//! proves HW-VER-0001\nfn f() {}\n")
            .expect("a fixture file");

        let resolver = CommentScan::new(
            &dir,
            "HW-VER-",
            ["HW-VER-0001", "HW-SPEC-x"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        let Binding::Unresolved(why) = resolver.resolve_for("sample.rs", "HW-SPEC-x") else {
            panic!("an asserter outside the prefix resolved");
        };
        std::fs::remove_dir_all(&dir).ok();

        assert!(why.contains("HW-SPEC-x"), "{why}");
        assert!(why.contains("does not start `HW-VER-`"), "{why}");
    }
}
