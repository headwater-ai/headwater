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
//!
//! Two readings sit beside the resolver's, both ruled by the owner on
//! 2026-09-25. A scope counts only files git does not ignore, so a cache that
//! one host wrote and CI never sees moves no figure: [`Ignored`] carries the
//! list and [`Scope::ignoring`] applies it. A taxonomy with no tree beside it,
//! which is what a taxonomy published as its own repository is, has nothing to
//! count: [`Scope::tree_is_absent`] says so, and `taxonomy validate` prints a
//! notice rather than refusing each pattern.

use crate::anchors::{normalize, Binding, Resolvers};
use crate::declarations::Declarations;
use headwater_meta::pattern::Pattern;
use std::path::Path;

/// The paths git's ignore rules exclude under a tree, relative to its root.
///
/// [`headwater_vcs::ignored`] is the reader, so git decides what it ignores
/// and this crate matches no ignore pattern itself. A whole ignored directory
/// arrives as one entry ending in `/`. Empty where the tree is not in a git
/// repository, because no ignore rule binds a tree git does not see.
///
/// Only a verb that is not the check, the gate or the probe reads this: spec 12
/// keeps every version control command off that loop, so `Graph::build`
/// counts the scope with nothing ignored and `taxonomy audit` drops the
/// ignored entries afterward with [`Reach::retain_unignored`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ignored {
    entries: Vec<String>,
}

impl Ignored {
    /// What git ignores under `base`, which is the root the resolver's paths
    /// are relative to.
    pub fn read(base: &Path) -> Ignored {
        Ignored {
            entries: headwater_vcs::ignored(base),
        }
    }

    /// Whether git ignores `path`, a path relative to the same root.
    pub fn covers(&self, path: &str) -> bool {
        self.entries
            .iter()
            .any(|entry| match entry.strip_suffix('/') {
                Some(directory) => path.starts_with(entry.as_str()) || path == directory,
                None => path == entry,
            })
    }
}

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
    /// What the count leaves out. Empty unless a caller set it with
    /// [`Scope::ignoring`].
    pub ignored: Ignored,
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
            ignored: Ignored::default(),
        }
    }

    /// The same scope, counting no entry that `ignored` covers.
    pub fn ignoring(self, ignored: Ignored) -> Scope {
        Scope { ignored, ..self }
    }

    /// Whether no tree lies beside the taxonomy for this scope to count.
    ///
    /// `beside` is what [`tree_directories`] found at the root. The tree is
    /// absent only when the scope declares at least one pattern, `beside` is
    /// empty, and the root of every pattern is missing too: the literal
    /// directory a wildcard pattern walks from, or the file a literal pattern
    /// names. So a tree whose every pattern is misspelled is still a tree, and
    /// `taxonomy validate` refuses each pattern. A pattern the resolver cannot
    /// read at all is no evidence either way.
    pub fn tree_is_absent(&self, resolvers: &Resolvers, beside: &[String]) -> bool {
        !self.members.is_empty()
            && beside.is_empty()
            && !self.members.iter().any(|member| {
                let (Ok(pattern), Some(resolver)) =
                    (member.pattern.as_ref(), resolvers.get(&member.resolver))
                else {
                    return false;
                };
                if pattern.is_literal() {
                    matches!(resolver.resolve(pattern.source()), Binding::Resolved { .. })
                } else {
                    let root = pattern.literal_prefix();
                    !root.is_empty() && resolver.names_directory(&root)
                }
            })
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
                entries: bind(member, resolvers, &self.ignored).unwrap_or_default(),
            })
            .collect()
    }

    /// Each pattern that matches no file, with the reason.
    pub fn unmatched(&self, resolvers: &Resolvers) -> Vec<(String, String)> {
        self.members
            .iter()
            .filter_map(|member| match bind(member, resolvers, &self.ignored) {
                Ok(_) => None,
                Err(why) => Some((member.written.clone(), why)),
            })
            .collect()
    }
}

/// The directories at the top of `base` that could hold a tree, sorted.
///
/// This asks the root and never the scope, so a misspelled pattern cannot
/// hide a tree. A directory does not count when it is:
///
/// - the first segment of a taxonomy source in `sources`, the paths the
///   resolution read, because the taxonomy is not the tree it describes;
/// - a dot-directory, because `.git`, `.github` and an editor's settings sit
///   beside a taxonomy published on its own as well as beside a tree;
/// - a directory git ignores, because a build output is not a tree.
///
/// A file at the top of the root does not count either, because a README and
/// a license sit beside a published taxonomy too. A pattern whose root is one
/// of these directories still makes the tree present, which
/// [`Scope::tree_is_absent`] decides.
pub fn tree_directories(base: &Path, sources: &[String], ignored: &Ignored) -> Vec<String> {
    let taxonomy: Vec<&str> = sources
        .iter()
        .filter_map(|source| {
            let source = source.trim_start_matches("./");
            source.split_once('/').map(|(first, _)| first)
        })
        .collect();
    let Ok(entries) = std::fs::read_dir(base) else {
        return Vec::new();
    };
    let mut found: Vec<String> = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| !name.starts_with('.'))
        .filter(|name| !taxonomy.contains(&name.as_str()))
        .filter(|name| !ignored.covers(&format!("{name}/")))
        .collect();
    found.sort();
    found
}

impl Reach {
    /// Drop every entry git ignores, for a reach taken with nothing ignored.
    pub fn retain_unignored(&mut self, ignored: &Ignored) {
        self.entries.retain(|path| !ignored.covers(path));
    }
}

fn bind(member: &Member, resolvers: &Resolvers, ignored: &Ignored) -> Result<Vec<String>, String> {
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
    // everything under it, or for nothing when it is empty. The resolver
    // answers which of the two a literal names, because a walk of
    // `<literal>/**` finds nothing under an empty directory either.
    if pattern.is_literal() && resolver.names_directory(source) {
        return Err(format!(
            "`{source}` names a directory, and a scope pattern admits files: write `{source}/**`"
        ));
    }
    let kept: Vec<String> = matched
        .into_iter()
        .filter(|path| !ignored.covers(path))
        .collect();
    if kept.is_empty() {
        return Err(format!(
            "every entry `{source}` matches is one git ignores, and the scope counts none of them"
        ));
    }
    Ok(kept)
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
                assert!(
                    scope.contains(path),
                    "`{spelling}` does not contain `{path}`"
                );
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
            assert!(
                scope.reach(&resolvers)[0].entries.is_empty(),
                "`{spelling}`"
            );
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

    /// A scope counts only what git does not ignore, so a cache written on one
    /// host and absent on another moves no figure (#951, owner ruling
    /// 2026-09-25).
    #[test]
    fn an_entry_git_ignores_is_outside_the_count_and_a_pattern_of_only_such_entries_is_refused() {
        let at = tree("ignored");
        let cache = at.join("tools/site/__pycache__/b.cpython-312.pyc");
        std::fs::create_dir_all(cache.parent().expect("a parent")).expect("the cache is made");
        std::fs::write(&cache, "").expect("the cache writes");
        std::fs::write(at.join(".gitignore"), "__pycache__/\n").expect("the ignore file writes");
        let init = std::process::Command::new("git")
            .args(["init", "-q"])
            .current_dir(&at)
            .status()
            .expect("git runs");
        assert!(init.success());

        // With nothing ignored, the walk admits the cache.
        let everything = scope(&["tools/**"]).reach(&resolvers(&at));
        assert_eq!(everything[0].entries.len(), 3, "{everything:?}");

        let ignored = Ignored::read(&at);
        let scope = scope(&["tools/**", "tools/site/__pycache__/**"]).ignoring(ignored.clone());
        let reach = scope.reach(&resolvers(&at));
        assert_eq!(reach[0].entries, ["tools/a.sh", "tools/site/b.py"]);
        assert!(reach[1].entries.is_empty(), "{reach:?}");
        let refused = scope.unmatched(&resolvers(&at));
        assert_eq!(refused.len(), 1, "{refused:?}");
        assert_eq!(refused[0].0, "tools/site/__pycache__/**");
        assert!(refused[0].1.contains("ignore"), "{refused:?}");

        // A reach taken with nothing ignored drops the same entry afterward.
        let mut late = everything;
        for reach in &mut late {
            reach.retain_unignored(&ignored);
        }
        assert_eq!(late[0].entries, ["tools/a.sh", "tools/site/b.py"]);
        let _ = std::fs::remove_dir_all(&at);
    }

    /// A taxonomy with no tree beside it has nothing to count, so the scope
    /// says the tree is absent rather than calling every pattern unmatched
    /// (#951, owner ruling 2026-09-25). Whether a tree is there is asked of
    /// the root, never of the patterns: a tree whose every pattern is
    /// misspelled is still a tree, and each pattern is refused.
    #[test]
    fn a_scope_has_no_tree_beside_it_only_where_the_root_holds_none() {
        let none = Ignored::default();

        // A tree is there, and no pattern names a root that exists in it.
        let at = tree("absent");
        let over_tree = resolvers(&at);
        let beside = tree_directories(&at, &[], &none);
        assert_eq!(beside, ["tools", "tools-old"]);
        for patterns in [&["tool/**"][..], &["engine/**", "site/**", "README.md"]] {
            assert!(
                !scope(patterns).tree_is_absent(&over_tree, &beside),
                "{patterns:?} beside a tree"
            );
        }
        let _ = std::fs::remove_dir_all(&at);

        // The root of a taxonomy published on its own: the taxonomy's own
        // sources, dot-directories, top-level files, and a directory git
        // ignores.
        let bare =
            std::env::temp_dir().join(format!("headwater-graph-scope-{}-bare", std::process::id()));
        let _ = std::fs::remove_dir_all(&bare);
        for file in [
            ".headwater/overlay.yml",
            ".github/workflows/ci.yml",
            "README.md",
            "bundles/one/bundle.yml",
            "build/out.txt",
        ] {
            let path = bare.join(file);
            std::fs::create_dir_all(path.parent().expect("a parent")).expect("the root is made");
            std::fs::write(path, "").expect("the file writes");
        }
        let sources = vec![
            ".headwater/overlay.yml".to_string(),
            "bundles/one/bundle.yml".to_string(),
        ];
        let ignored = Ignored {
            entries: vec!["build/".to_string()],
        };
        let beside = tree_directories(&bare, &sources, &ignored);
        assert!(beside.is_empty(), "{beside:?}");
        let over_bare = resolvers(&bare);
        assert!(scope(&["tool/**"]).tree_is_absent(&over_bare, &beside));
        // A pattern whose root is there makes the tree present, even under a
        // dot-directory.
        assert!(!scope(&[".github/**"]).tree_is_absent(&over_bare, &beside));
        assert!(
            !scope(&[]).tree_is_absent(&over_bare, &beside),
            "no scope claims no tree"
        );
        // One directory that is not the taxonomy's makes a tree.
        assert_eq!(tree_directories(&bare, &sources, &none), ["build"]);
        let _ = std::fs::remove_dir_all(&bare);
    }
}
