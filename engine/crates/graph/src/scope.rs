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

use crate::anchors::{Binding, Resolvers};
use crate::declarations::Declarations;
use headwater_meta::pattern::Pattern;

/// One pattern of a declared scope, with the anchor kind that declared it.
#[derive(Clone, Debug)]
pub struct Member {
    pub anchor_kind: String,
    pub resolver: String,
    pub pattern: Pattern,
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
    pub pattern: String,
    /// The entries the pattern matched, sorted and with no duplicate. Empty
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
                    anchor.scope.iter().map(|pattern| Member {
                        anchor_kind: anchor.name.clone(),
                        resolver: anchor.resolver.clone(),
                        pattern: Pattern::new(pattern),
                    })
                })
                .collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    /// Whether any scope pattern admits `path`, a path relative to the
    /// repository root with `/` separators.
    ///
    /// The test is the pattern alone. A corpus exclusion is the walk's to
    /// apply, so a caller that holds one path and cares about exclusions asks
    /// the corpus as well.
    pub fn contains(&self, path: &str) -> bool {
        self.members
            .iter()
            .any(|member| member.pattern.matches(path))
    }

    /// Each pattern with the entries its anchor kind's resolver matched.
    pub fn reach(&self, resolvers: &Resolvers) -> Vec<Reach> {
        self.members
            .iter()
            .map(|member| Reach {
                anchor_kind: member.anchor_kind.clone(),
                pattern: member.pattern.source().to_string(),
                entries: bind(member, resolvers).unwrap_or_default(),
            })
            .collect()
    }

    /// Each pattern that matches no entry, with the resolver's reason.
    pub fn unmatched(&self, resolvers: &Resolvers) -> Vec<(String, String)> {
        self.members
            .iter()
            .filter_map(|member| match bind(member, resolvers) {
                Ok(_) => None,
                Err(why) => Some((member.pattern.source().to_string(), why)),
            })
            .collect()
    }
}

fn bind(member: &Member, resolvers: &Resolvers) -> Result<Vec<String>, String> {
    let Some(resolver) = resolvers.get(&member.resolver) else {
        return Err(format!(
            "anchor kind `{}` names the resolver `{}`, and this engine carries none by that name",
            member.anchor_kind, member.resolver
        ));
    };
    match resolver.resolve(member.pattern.source()) {
        Binding::Resolved { matched, .. } if !matched.is_empty() => Ok(matched),
        Binding::Resolved { .. } => Err(format!(
            "no entry in the source tree matches `{}`",
            member.pattern.source()
        )),
        Binding::Unresolved(why) => Err(why),
        Binding::Withheld { profile } => Err(format!(
            "the resolver `{}` withheld `{}` under the export profile `{profile}`",
            member.resolver,
            member.pattern.source()
        )),
    }
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
                assert!(scope.contains(path), "`{spelling}` does not contain `{path}`");
            }
            assert!(!scope.contains("tools-old/c.sh"), "`{spelling}`");
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
            assert!(scope.reach(&resolvers)[0].entries.is_empty(), "`{spelling}`");
            let refused = scope.unmatched(&resolvers);
            assert_eq!(refused.len(), 1, "`{spelling}`: {refused:?}");
            assert!(refused[0].1.contains("directory"), "{refused:?}");
            assert!(!scope.contains("tools/a.sh"));
        }
        // A literal file is an entry, and contains agrees.
        let scope = scope(&["./tools/a.sh"]);
        assert_eq!(scope.reach(&resolvers(&at))[0].entries, ["tools/a.sh"]);
        assert!(scope.contains("tools/a.sh"));
        let _ = std::fs::remove_dir_all(&at);
    }
}
