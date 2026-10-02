//! The agent support that `headwater init --harness` writes, compiled in.
//!
//! **The set is text in the binary, and the binary writes it out.** An
//! adopter receives the engine and nothing else, so a skill that lived in the
//! checkout of the repository that maintains this engine reached no adopter.
//! Each file is read here with `include_str!`, the way the meta-schema is, so
//! one release of the binary and one version of the set are the same thing.
//!
//! **No file of the set names the corpus that maintains this engine.** A skill
//! that cites one of its decisions, a path under its `docs/`, or a kind that
//! only its package declares sends an adopter's agent after a file the
//! adopter does not have. [`foreign`] is the list of what a shipped file must
//! not hold, and the unit tests below hold every member to it.
//!
//! **The last line records that the step wrote the file.** It carries the
//! digest of every byte before it, so a later release can tell a file it may
//! replace from a file somebody edited. [`classify`] is that decision, and
//! `docs/interfaces/headwater-init.md` states it under "The harness step".

/// What a member is, which decides where a harness looks for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Form {
    /// A `SKILL.md` under a directory named for the skill.
    Skill,
    /// An agent definition, one file named for the agent.
    Agent,
}

/// One file of the set, as the binary carries it.
#[derive(Debug, Clone, Copy)]
pub struct Member {
    /// The name the harness addresses it by, and the `name` of its front matter.
    pub name: &'static str,
    /// A skill or an agent.
    pub form: Form,
    /// The text, before the line that records the install.
    pub text: &'static str,
}

/// The shipped set. The hooks are not in it: which vehicle a hook ships in is
/// not decided (#1578).
pub const SET: [Member; 5] = [
    Member {
        name: "headwater-orient",
        form: Form::Skill,
        text: include_str!("../harness/skills/headwater-orient/SKILL.md"),
    },
    Member {
        name: "headwater-authoring",
        form: Form::Skill,
        text: include_str!("../harness/skills/headwater-authoring/SKILL.md"),
    },
    Member {
        name: "headwater-taxonomy",
        form: Form::Skill,
        text: include_str!("../harness/skills/headwater-taxonomy/SKILL.md"),
    },
    Member {
        name: "headwater-sweep",
        form: Form::Skill,
        text: include_str!("../harness/skills/headwater-sweep/SKILL.md"),
    },
    Member {
        name: "headwater-maintainer",
        form: Form::Agent,
        text: include_str!("../harness/agents/headwater-maintainer.md"),
    },
];

impl Member {
    /// Every path, relative to the repository root, that a harness of spec 16
    /// reads this member from.
    ///
    /// Claude Code reads `.claude/`, and Copilot reads it too. Codex reads
    /// `.agents/skills/` and states no agent file, so an agent gets one path.
    pub fn paths(&self) -> Vec<String> {
        match self.form {
            Form::Skill => vec![
                format!(".claude/skills/{}/SKILL.md", self.name),
                format!(".agents/skills/{}/SKILL.md", self.name),
            ],
            Form::Agent => vec![format!(".claude/agents/{}.md", self.name)],
        }
    }
}

/// The opening of the line that records the install, up to the digest.
const RECORD_OPEN: &str = "<!-- installed by headwater init --harness, digest ";
/// The close of that line.
const RECORD_CLOSE: &str = " -->";

/// The bytes the step writes for a member: its text, and the record line.
pub fn installed(text: &str) -> String {
    format!(
        "{text}{RECORD_OPEN}{}{RECORD_CLOSE}\n",
        headwater_hash::digest(text.as_bytes())
    )
}

/// One path the step writes, and the bytes it writes there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    /// Relative to the repository root.
    pub path: String,
    /// What [`installed`] makes of the member.
    pub bytes: String,
}

/// Every file of the set, in the order of [`SET`].
pub fn files() -> Vec<File> {
    SET.iter()
        .flat_map(|member| {
            let bytes = installed(member.text);
            member.paths().into_iter().map(move |path| File {
                path,
                bytes: bytes.clone(),
            })
        })
        .collect()
}

/// What the step found at one path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Found {
    /// No file is there, so the step writes one.
    Absent,
    /// The bytes of this release are there, so the step writes nothing.
    Current,
    /// An earlier release wrote the file and nobody edited it, so the step
    /// writes this release over it.
    Earlier,
    /// Somebody edited the file, or something else wrote it, so the step
    /// refuses.
    Foreign,
}

/// Decide one path from the bytes already there.
///
/// A file is the step's own only where its last line is the record and the
/// digest in the record is the digest of every byte before that line. An
/// edit anywhere above the record moves that digest, and a file with no
/// record was never the step's.
pub fn classify(existing: Option<&[u8]>, current: &str) -> Found {
    let Some(existing) = existing else {
        return Found::Absent;
    };
    if existing == current.as_bytes() {
        return Found::Current;
    }
    let Ok(text) = std::str::from_utf8(existing) else {
        return Found::Foreign;
    };
    let Some(body) = text.strip_suffix('\n') else {
        return Found::Foreign;
    };
    let start = body.rfind('\n').map_or(0, |at| at + 1);
    let (above, last) = body.split_at(start);
    let recorded = last
        .strip_prefix(RECORD_OPEN)
        .and_then(|rest| rest.strip_suffix(RECORD_CLOSE));
    match recorded {
        Some(digest) if digest == headwater_hash::digest(above.as_bytes()) => Found::Earlier,
        _ => Found::Foreign,
    }
}

/// What a shipped file must not hold, each with the reason.
///
/// An identifier, a path or an internal skill of the repository that
/// maintains this engine names something an adopter does not have. The kind
/// names that only its package declares are held by the fixture that binds a
/// repository to another package (`tests/init_harness.rs`), because a kind
/// name is also an English word and only a lock says which ones a corpus has.
pub const FOREIGN: [(&str, &str); 12] = [
    ("HW-", "an identifier of the corpus that maintains this engine"),
    ("docs/", "a path into that corpus"),
    ("engine/", "a path into the engine workspace"),
    ("tools/", "a path into that repository's tools"),
    (".githooks", "that repository's git hooks"),
    ("taxonomy-source", "that repository's package source"),
    (".claude/hooks", "a hook the set does not ship"),
    ("hw-", "an internal agent or skill of the build order"),
    ("ste-editor", "an internal skill, for a language regime an adopter may not declare"),
    ("repo-cleanup", "an internal skill"),
    ("headwater-engine", "an internal skill, for building the engine"),
    ("headwater-product-owner", "an internal agent"),
];

/// Every entry of [`FOREIGN`] that a text holds, with its reason.
pub fn foreign(text: &str) -> Vec<(&'static str, &'static str)> {
    FOREIGN
        .iter()
        .copied()
        .filter(|(needle, _)| text.contains(needle))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_member_of_the_set_holds_a_name_of_the_corpus_that_maintains_this_engine() {
        let found: Vec<String> = SET
            .iter()
            .flat_map(|member| {
                foreign(member.text)
                    .into_iter()
                    .map(move |(needle, why)| format!("{} holds `{needle}`: {why}", member.name))
            })
            .collect();
        assert!(found.is_empty(), "{}", found.join("\n"));
    }

    #[test]
    fn a_text_that_cites_a_decision_of_this_corpus_is_refused() {
        let found = foreign("as HW-DR-0052 rules, a settled document is current");
        assert_eq!(found.first().map(|(needle, _)| *needle), Some("HW-"));
        assert!(foreign("run `headwater explain <path>`").is_empty());
    }

    #[test]
    fn no_member_is_named_as_internal() {
        for member in SET {
            assert!(
                member.name.starts_with("headwater-") && foreign(member.name).is_empty(),
                "{} is named as an internal file",
                member.name
            );
        }
    }

    #[test]
    fn every_member_declares_the_name_the_harness_addresses_it_by() {
        for member in SET {
            let declared = member
                .text
                .lines()
                .find_map(|line| line.strip_prefix("name: "))
                .map(str::trim);
            assert_eq!(declared, Some(member.name));
            assert!(member.text.ends_with('\n'), "{} ends with a newline", member.name);
        }
    }

    #[test]
    fn the_set_is_nine_paths_and_no_two_are_one() {
        let paths: Vec<String> = files().into_iter().map(|file| file.path).collect();
        let distinct: std::collections::BTreeSet<&String> = paths.iter().collect();
        assert_eq!(paths.len(), 9);
        assert_eq!(distinct.len(), 9);
    }

    #[test]
    fn a_file_is_the_steps_own_only_while_its_record_agrees_with_it() {
        let current = installed("---\nname: x\n---\nnew\n");
        let earlier = installed("---\nname: x\n---\nold\n");
        assert_eq!(classify(None, &current), Found::Absent);
        assert_eq!(classify(Some(current.as_bytes()), &current), Found::Current);
        assert_eq!(classify(Some(earlier.as_bytes()), &current), Found::Earlier);
        let edited = earlier.replacen("old", "mine", 1);
        assert_eq!(classify(Some(edited.as_bytes()), &current), Found::Foreign);
        let unrecorded = "---\nname: x\n---\nold\n";
        assert_eq!(classify(Some(unrecorded.as_bytes()), &current), Found::Foreign);
        let appended = format!("{earlier}a line after the record\n");
        assert_eq!(classify(Some(appended.as_bytes()), &current), Found::Foreign);
    }
}
