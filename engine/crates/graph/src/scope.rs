// SPDX-License-Identifier: Apache-2.0
//! The governed scope: the part of the tree a corpus expects a `governance`
//! edge to reach (#951).
//!
//! A scope is declared on an anchor kind, as `anchors.<kind>.scope`, a list of
//! patterns in the language of
//! [HW-DR-0074](../../../../docs/decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md).
//! It is a claim by the corpus about itself, and a corpus that declares none
//! expects nothing.
//!
//! This module is the one reader of that declaration. It answers three
//! questions, and each caller asks the one it needs:
//!
//! - [`Scope::contains`]: is one path inside the scope. A pure pattern test,
//!   with no walk, so a hook can ask it of one changed path.
//! - [`Scope::unmatched`]: which pattern matches no entry of the tree.
//!   `taxonomy validate` refuses each one.
//! - [`Scope::reach`]: which entries each pattern admits. `taxonomy audit`
//!   reads this and the graph's edges.
//!
//! The last two go through the anchor kind's own resolver, so "matches an
//! entry" means for a scope exactly what it means for an anchor: the same
//! prefix walk, the same corpus exclusions, and the same refusal of a pattern
//! that opens on a wildcard. No second matcher exists here.

use crate::anchors::{normalize, Binding, Resolvers};
use crate::declarations::Declarations;
use headwater_meta::pattern::Pattern;

/// One pattern of a declared scope, with the anchor kind that declared it.
#[derive(Clone, Debug)]
pub struct Member {
    pub anchor_kind: String,
    pub resolver: String,
    /// The pattern as the taxonomy wrote it, which is what a report names.
    pub written: String,
    /// The pattern after [`crate::anchors::normalize`], the same lexical pass
    /// the resolver applies to an anchor, or the reason it refused. Both
    /// [`Scope::contains`] and the walk read this form, so `./tools/**`,
    /// `tools\**` and `tools/**` are one pattern to both of them.
    pub pattern: Result<Pattern, String>,
}

/// Every scope pattern the taxonomy declares, in declaration order.
#[derive(Clone, Debug, Default)]
pub struct Scope {
    pub members: Vec<Member>,
}

/// What one scope pattern admits from the tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reach {
    pub anchor_kind: String,
    /// The pattern as the taxonomy wrote it.
    pub pattern: String,
    /// The files the pattern matched, sorted and with no duplicate. Empty
    /// where the resolver bound nothing, which `taxonomy validate` refuses.
    pub entries: Vec<String>,
}

impl Scope {
    /// The scope the resolved taxonomy declares, over every anchor kind.
    pub fn declared(declarations: &Declarations) -> Scope {
        Scope {
            members: declarations
                .anchors
                .iter()
                .flat_map(|anchor| {
                    anchor.scope.iter().map(|written| Member {
                        anchor_kind: anchor.name.clone(),
                        resolver: anchor.resolver.clone(),
                        written: written.clone(),
                        pattern: normalize(written).map(|normalized| Pattern::new(&normalized)),
                    })
                })
                .collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    /// Whether any scope pattern admits `path`, a path relative to the
    /// repository root.
    ///
    /// The path and the patterns both pass through the resolver's lexical
    /// normalization first, so this answers as [`Scope::reach`] does for
    /// every file the walk finds. The test is the pattern alone and walks
    /// nothing. A corpus exclusion is the walk's to apply, so a caller that
    /// holds one path and cares about exclusions asks the corpus as well.
    pub fn contains(&self, path: &str) -> bool {
        let Ok(path) = normalize(path) else {
            return false;
        };
        self.members.iter().any(|member| {
            member
                .pattern
                .as_ref()
                .is_ok_and(|pattern| pattern.matches(&path))
        })
    }

    /// Each pattern with the files its anchor kind's resolver matched.
    pub fn reach(&self, resolvers: &Resolvers) -> Vec<Reach> {
        self.members
            .iter()
            .map(|member| Reach {
                anchor_kind: member.anchor_kind.clone(),
                pattern: member.written.clone(),
                entries: bind(member, resolvers).unwrap_or_default(),
            })
            .collect()
    }

    /// Each pattern that matches no file, with the reason.
    pub fn unmatched(&self, resolvers: &Resolvers) -> Vec<(String, String)> {
        self.members
            .iter()
            .filter_map(|member| match bind(member, resolvers) {
                Ok(_) => None,
                Err(why) => Some((member.written.clone(), why)),
            })
            .collect()
    }
}

fn bind(member: &Member, resolvers: &Resolvers) -> Result<Vec<String>, String> {
    let pattern = member.pattern.as_ref().map_err(Clone::clone)?;
    let Some(resolver) = resolvers.get(&member.resolver) else {
        return Err(format!(
            "anchor kind `{}` names the resolver `{}`, and this engine carries none by that name",
            member.anchor_kind, member.resolver
        ));
    };
    let source = pattern.source();
    let matched = match resolver.resolve(source) {
        Binding::Resolved { matched, .. } if !matched.is_empty() => matched,
        Binding::Resolved { .. } => {
            return Err(format!("no entry in the source tree matches `{source}`"))
        }
        Binding::Unresolved(why) => return Err(why),
        Binding::Withheld { profile } => {
            return Err(format!(
                "the resolver `{}` withheld `{source}` under the export profile `{profile}`",
                member.resolver
            ))
        }
    };
    // A literal pattern resolves when the path exists, and a directory exists.
    // A scope counts files, so a directory would be one entry standing for
    // everything under it. A file has nothing under it, so the resolver's own
    // walk of `<literal>/**` tells the two apart without a second reader of
    // the tree.
    if pattern.is_literal() {
        if let Binding::Resolved { matched: under, .. } = resolver.resolve(&format!("{source}/**"))
        {
            if !under.is_empty() {
                return Err(format!(
                    "`{source}` names a directory, and a scope pattern admits files: write `{source}/**`"
                ));
            }
        }
    }
    Ok(matched)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::declarations::AnchorKind;
    use headwater_census::walk::Corpus;
    use std::path::{Path, PathBuf};

    /// A tree of two files under `tools/`, keyed on the case name, because the
    /// cases of one target run as threads of one process.
    fn tree(label: &str) -> PathBuf {
        let at = std::env::temp_dir().join(format!(
            "headwater-graph-scope-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        for file in ["tools/a.sh", "tools/site/b.py", "tools-old/c.sh"] {
            let path = at.join(file);
            std::fs::create_dir_all(path.parent().expect("a parent")).expect("the tree is made");
            std::fs::write(path, "").expect("the file writes");
        }
        at
    }

    fn scope(patterns: &[&str]) -> Scope {
        Scope::declared(&Declarations {
            relations: Vec::new(),
            anchors: vec![AnchorKind {
                name: "code_path".to_string(),
                resolver: "source-tree".to_string(),
                pattern: None,
                scope: patterns.iter().map(|p| p.to_string()).collect(),
                span: Default::default(),
            }],
        })
    }

    fn resolvers(at: &Path) -> Resolvers {
        Resolvers::over(&Corpus::new(at.to_path_buf(), "."))
    }

    /// `contains` and `reach` answer one question, so a spelling the
    /// resolver normalizes is a spelling `contains` normalizes too.
    #[test]
    fn contains_agrees_with_reach_for_every_spelling_the_resolver_normalizes() {
        let at = tree("spellings");
        for spelling in ["tools/**", "./tools/**", "tools\\**"] {
            let scope = scope(&[spelling]);
            let reach = scope.reach(&resolvers(&at));
            assert_eq!(
                reach[0].entries,
                ["tools/a.sh", "tools/site/b.py"],
                "`{spelling}` reaches"
            );
            for path in &reach[0].entries {
                assert!(
                    scope.contains(path),
                    "`{spelling}` does not contain `{path}`"
                );
            }
            assert!(!scope.contains("tools-old/c.sh"), "`{spelling}`");
            // The path is normalized as well as the pattern, so a caller that
            // holds a path as a hook or a document writes it gets one answer.
            for path in ["./tools/a.sh", "tools\\site\\b.py", "tools//a.sh"] {
                assert!(scope.contains(path), "`{spelling}` does not contain `{path}`");
            }
            assert!(!scope.contains("./tools-old/c.sh"), "`{spelling}`");
        }
        let _ = std::fs::remove_dir_all(&at);
    }

    /// A literal pattern that names a directory admits no file, so it is
    /// refused rather than counted as one entry of the denominator.
    #[test]
    fn a_literal_pattern_that_names_a_directory_is_refused_and_counts_nothing() {
        let at = tree("directory");
        for spelling in ["tools/", "tools"] {
            let scope = scope(&[spelling]);
            let resolvers = resolvers(&at);
            assert!(
                scope.reach(&resolvers)[0].entries.is_empty(),
                "`{spelling}`"
            );
            let refused = scope.unmatched(&resolvers);
            assert_eq!(refused.len(), 1, "`{spelling}`: {refused:?}");
            assert!(refused[0].1.contains("directory"), "{refused:?}");
            assert!(!scope.contains("tools/a.sh"));
        }
        // An empty directory has nothing under it, as a file has not, and it
        // is still a directory.
        std::fs::create_dir_all(at.join("tools/zz-empty")).expect("the directory is made");
        for spelling in ["tools/zz-empty", "./tools/zz-empty/"] {
            let scope = scope(&[spelling]);
            let resolvers = resolvers(&at);
            assert!(scope.reach(&resolvers)[0].entries.is_empty(), "`{spelling}`");
            let refused = scope.unmatched(&resolvers);
            assert_eq!(refused.len(), 1, "`{spelling}`: {refused:?}");
            assert!(refused[0].1.contains("directory"), "{refused:?}");
        }
        // A literal file is an entry, and contains agrees.
        let scope = scope(&["./tools/a.sh"]);
        assert_eq!(scope.reach(&resolvers(&at))[0].entries, ["tools/a.sh"]);
        assert!(scope.contains("tools/a.sh"));
        let _ = std::fs::remove_dir_all(&at);
    }
}
