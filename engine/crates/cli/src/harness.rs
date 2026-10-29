// SPDX-License-Identifier: Apache-2.0
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
//! A Markdown file carries the line as an HTML comment and a hook script as a
//! `#` comment, so [`Form::record`] names the form.
//!
//! **A JSON file carries no record, so the step merges into it.** The hook
//! entries in `.claude/settings.json` and the server in `.mcp.json` sit beside
//! entries the adopter wrote. [`merge`] adds each entry the file lacks and
//! never removes, reorders or changes one.

/// What a member is, which decides where a harness looks for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Form {
    /// A `SKILL.md` under a directory named for the skill.
    Skill,
    /// An agent definition, one file named for the agent.
    Agent,
    /// A hook script, one `sh` file under `.claude/hooks/`, which the hook
    /// entries of [`merge`] call.
    Hook,
}

impl Form {
    /// The opening and the close of the line that records the install, around
    /// the digest. A hook script is `sh`, which reads no HTML comment.
    pub fn record(self) -> (&'static str, &'static str) {
        match self {
            Form::Skill | Form::Agent => (RECORD_OPEN, RECORD_CLOSE),
            Form::Hook => ("# installed by headwater init --harness, digest ", ""),
        }
    }
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

/// The shipped skills and agent. The hooks are [`HOOKS`]: the owner ruled in
/// #1868 that `init` ships them as `sh` scripts, and #1871 carries it.
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

/// The shipped hook scripts. Each one finds the binary on the `PATH` and calls
/// verbs only, and `lib.sh` is what the other five read.
pub const HOOKS: [Member; 6] = [
    Member {
        name: "intent",
        form: Form::Hook,
        text: include_str!("../harness/hooks/intent.sh"),
    },
    Member {
        name: "read",
        form: Form::Hook,
        text: include_str!("../harness/hooks/read.sh"),
    },
    Member {
        name: "write",
        form: Form::Hook,
        text: include_str!("../harness/hooks/write.sh"),
    },
    Member {
        name: "review",
        form: Form::Hook,
        text: include_str!("../harness/hooks/review.sh"),
    },
    Member {
        name: "lib",
        form: Form::Hook,
        text: include_str!("../harness/hooks/lib.sh"),
    },
    Member {
        name: "touch",
        form: Form::Hook,
        text: include_str!("../harness/hooks/touch.sh"),
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
            Form::Hook => vec![format!(".claude/hooks/{}.sh", self.name)],
        }
    }
}

/// The opening of the line that records the install, up to the digest.
const RECORD_OPEN: &str = "<!-- installed by headwater init --harness, digest ";
/// The close of that line.
const RECORD_CLOSE: &str = " -->";

/// The bytes the step writes for a Markdown member: its text, and the record
/// line.
pub fn installed(text: &str) -> String {
    installed_as(text, Form::Skill)
}

/// The bytes the step writes for a member of any form.
pub fn installed_as(text: &str, form: Form) -> String {
    let (open, close) = form.record();
    format!(
        "{text}{open}{}{close}\n",
        headwater_hash::digest(text.as_bytes())
    )
}

/// One path the step writes, and the bytes it writes there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    /// Relative to the repository root.
    pub path: String,
    /// What [`installed_as`] makes of the member.
    pub bytes: String,
    /// The form of the member, which decides the form of the record line.
    pub form: Form,
}

/// Every file of the set, in the order of [`SET`] and then [`HOOKS`].
pub fn files() -> Vec<File> {
    SET.iter()
        .chain(HOOKS.iter())
        .flat_map(|member| {
            let bytes = installed_as(member.text, member.form);
            let form = member.form;
            member.paths().into_iter().map(move |path| File {
                path,
                bytes: bytes.clone(),
                form,
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
///
/// The step writes `\n` line endings. Git checks a file out with `\r\n` in
/// their place under `core.autocrlf=true`, so the decision reads every
/// `\r\n` of the bytes found as `\n` before it compares them. A `\r` that
/// ends no line is a changed byte and stays one.
pub fn classify(existing: Option<&[u8]>, current: &str) -> Found {
    classify_as(existing, current, Form::Skill)
}

/// [`classify`], for a member whose record line has the form of `form`.
pub fn classify_as(existing: Option<&[u8]>, current: &str, form: Form) -> Found {
    let (record_open, record_close) = form.record();
    let Some(existing) = existing else {
        return Found::Absent;
    };
    let read_as_lf = lf_line_endings(existing);
    let existing = read_as_lf.as_slice();
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
        .strip_prefix(record_open)
        .and_then(|rest| rest.strip_suffix(record_close));
    match recorded {
        Some(digest) if digest == headwater_hash::digest(above.as_bytes()) => Found::Earlier,
        _ => Found::Foreign,
    }
}

/// The bytes with each `\r\n` written as `\n`, and every other byte kept.
fn lf_line_endings(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut rest = bytes.iter().peekable();
    while let Some(&byte) = rest.next() {
        if byte == b'\r' && rest.peek() == Some(&&b'\n') {
            continue;
        }
        out.push(byte);
    }
    out
}

/// One hook entry that the step adds to `.claude/settings.json`.
#[derive(Debug, Clone, Copy)]
pub struct HookEntry {
    /// The event of Claude Code the entry answers.
    pub event: &'static str,
    /// The tool names the entry matches, where the event takes a matcher.
    pub matcher: Option<&'static str>,
    /// The member of [`HOOKS`] that the entry calls.
    pub hook: &'static str,
    /// The seconds Claude Code waits for the hook.
    pub timeout: u32,
}

/// The hook entries, in the order a fresh `.claude/settings.json` holds them.
pub const HOOK_ENTRIES: [HookEntry; 6] = [
    HookEntry {
        event: "UserPromptSubmit",
        matcher: None,
        hook: "intent",
        timeout: 15,
    },
    HookEntry {
        event: "PreToolUse",
        matcher: Some("Write|Edit"),
        hook: "write",
        timeout: 15,
    },
    HookEntry {
        event: "PreToolUse",
        matcher: Some("Write|Edit"),
        hook: "touch",
        timeout: 15,
    },
    HookEntry {
        event: "PreToolUse",
        matcher: Some("Read"),
        hook: "read",
        timeout: 15,
    },
    HookEntry {
        event: "PostToolUse",
        matcher: Some("Write|Edit"),
        hook: "write",
        timeout: 15,
    },
    HookEntry {
        event: "Stop",
        matcher: None,
        hook: "review",
        timeout: 120,
    },
];

impl HookEntry {
    /// The command of the entry. It names the script under the project
    /// directory, and it says so on standard error when the script is gone,
    /// rather than failing with nothing said.
    pub fn command(&self) -> String {
        format!(
            "p=\"$CLAUDE_PROJECT_DIR/.claude/hooks/{}.sh\"; if [ -f \"$p\" ]; then sh \"$p\"; else \
             echo \"hook script not found: $p\" >&2; exit 1; fi",
            self.hook
        )
    }
}

/// A JSON file that the step merges its entries into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Config {
    /// `.claude/settings.json`, which holds the hook entries.
    Settings,
    /// `.mcp.json`, which holds the `headwater` MCP server.
    Mcp,
}

/// Both JSON files, in the order the step decides them.
pub const CONFIGS: [Config; 2] = [Config::Settings, Config::Mcp];

impl Config {
    /// Relative to the repository root.
    pub fn path(self) -> &'static str {
        match self {
            Config::Settings => ".claude/settings.json",
            Config::Mcp => ".mcp.json",
        }
    }
}

/// The `headwater` member of `mcpServers`: the binary on the `PATH`, at the
/// `mcp` verb.
pub fn mcp_server() -> headwater_yaml::json::Json {
    use headwater_yaml::json::Json;
    Json::object([
        ("command", Json::string("headwater")),
        ("args", Json::Array(vec![Json::string("mcp")])),
    ])
}

/// What the step found in one JSON file, and what it writes there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Merged {
    /// The file holds every entry of this release, so the step writes nothing.
    Current,
    /// The step writes these bytes: the file was absent, or it lacked this
    /// many entries.
    Write {
        bytes: String,
        absent: bool,
        added: usize,
    },
    /// The step cannot merge, and says why.
    Foreign(String),
}

/// Decide one JSON file from the bytes already there.
///
/// The file is read with the YAML loader, because JSON is a subset of the
/// core schema it implements. Every member keeps its place, and the step only
/// appends. When it appends, it writes the whole value again, indented by two
/// spaces. So a second run over what it wrote finds every entry and writes
/// nothing.
pub fn merge(config: Config, existing: Option<&[u8]>) -> Merged {
    use headwater_yaml::json::Json;
    let mut members = match existing {
        None => Vec::new(),
        Some(bytes) => {
            let Ok(text) = std::str::from_utf8(bytes) else {
                return Merged::Foreign("it is not UTF-8 text".to_string());
            };
            if text.trim().is_empty() {
                Vec::new()
            } else {
                match headwater_yaml::load(text).map(|node| json_of(&node.value)) {
                    Ok(Json::Object(members)) => members,
                    _ => return Merged::Foreign("it is not one JSON object".to_string()),
                }
            }
        }
    };
    let added = match config {
        Config::Settings => merge_settings(&mut members),
        Config::Mcp => merge_mcp(&mut members),
    };
    match added {
        Err(why) => Merged::Foreign(why),
        Ok(0) if existing.is_some() => Merged::Current,
        Ok(added) => Merged::Write {
            bytes: Json::Object(members).render_pretty(),
            absent: existing.is_none(),
            added,
        },
    }
}

/// A loaded value as JSON. A quoted scalar is a string, and a plain one, a
/// number, `true`, `false` or `null`, is written back as it stood.
fn json_of(value: &headwater_yaml::Value) -> headwater_yaml::json::Json {
    use headwater_yaml::json::Json;
    use headwater_yaml::Value;
    match value {
        Value::Scalar(scalar) if scalar.is_plain() => Json::Raw(scalar.text.clone()),
        Value::Scalar(scalar) => Json::String(scalar.text.clone()),
        Value::Seq(items) => Json::Array(items.iter().map(|item| json_of(&item.value)).collect()),
        Value::Map(mapping) => Json::Object(
            mapping
                .iter()
                .map(|entry| (entry.key.value.clone(), json_of(&entry.value.value)))
                .collect(),
        ),
    }
}

/// The member `key` of an object, appended with `empty` where it is absent.
fn member<'a>(
    members: &'a mut Vec<(String, headwater_yaml::json::Json)>,
    key: &str,
    empty: headwater_yaml::json::Json,
) -> &'a mut headwater_yaml::json::Json {
    let at = match members.iter().position(|(name, _)| name == key) {
        Some(at) => at,
        None => {
            members.push((key.to_string(), empty));
            members.len() - 1
        }
    };
    &mut members[at].1
}

/// Add each hook entry the file lacks, and count them.
fn merge_settings(root: &mut Vec<(String, headwater_yaml::json::Json)>) -> Result<usize, String> {
    use headwater_yaml::json::Json;
    let mut added = 0;
    for entry in HOOK_ENTRIES {
        let Json::Object(events) = member(root, "hooks", Json::Object(Vec::new())) else {
            return Err("its `hooks` member is not an object".to_string());
        };
        let Json::Array(groups) = member(events, entry.event, Json::Array(Vec::new())) else {
            return Err(format!(
                "its `hooks.{}` member is not an array",
                entry.event
            ));
        };
        let command = entry.command();
        let hook = Json::Object(vec![
            ("type".to_string(), Json::string("command")),
            ("command".to_string(), Json::string(command.clone())),
            ("timeout".to_string(), Json::Raw(entry.timeout.to_string())),
        ]);
        let group = groups.iter_mut().find_map(|group| match group {
            Json::Object(members) if matcher_of(members) == entry.matcher => Some(members),
            _ => None,
        });
        match group {
            Some(members) => {
                let Json::Array(hooks) = member(members, "hooks", Json::Array(Vec::new())) else {
                    return Err(format!(
                        "a group of `hooks.{}` holds a `hooks` member that is not an array",
                        entry.event
                    ));
                };
                let present = hooks.iter().any(|hook| match hook {
                    Json::Object(members) => members.iter().any(|(name, value)| {
                        name == "command" && *value == Json::String(command.clone())
                    }),
                    _ => false,
                });
                if !present {
                    hooks.push(hook);
                    added += 1;
                }
            }
            None => {
                let mut members = Vec::new();
                if let Some(matcher) = entry.matcher {
                    members.push(("matcher".to_string(), Json::string(matcher)));
                }
                members.push(("hooks".to_string(), Json::Array(vec![hook])));
                groups.push(Json::Object(members));
                added += 1;
            }
        }
    }
    Ok(added)
}

/// The `matcher` of a group, where it is a string.
fn matcher_of(members: &[(String, headwater_yaml::json::Json)]) -> Option<&str> {
    members.iter().find_map(|(name, value)| match value {
        headwater_yaml::json::Json::String(text) if name == "matcher" => Some(text.as_str()),
        _ => None,
    })
}

/// Add the `headwater` server where the file lacks it, and refuse one that
/// differs.
fn merge_mcp(root: &mut Vec<(String, headwater_yaml::json::Json)>) -> Result<usize, String> {
    use headwater_yaml::json::Json;
    let Json::Object(servers) = member(root, "mcpServers", Json::Object(Vec::new())) else {
        return Err("its `mcpServers` member is not an object".to_string());
    };
    let ours = mcp_server();
    match servers.iter().find(|(name, _)| name == "headwater") {
        Some((_, theirs)) if same(theirs, &ours) => Ok(0),
        Some(_) => Err(
            "its `headwater` member of `mcpServers` differs from the entry of this release"
                .to_string(),
        ),
        None => {
            servers.push(("headwater".to_string(), ours));
            Ok(1)
        }
    }
}

/// Whether two values are one, with the members of an object in any order.
fn same(left: &headwater_yaml::json::Json, right: &headwater_yaml::json::Json) -> bool {
    use headwater_yaml::json::Json;
    match (left, right) {
        (Json::Object(a), Json::Object(b)) => {
            a.len() == b.len()
                && a.iter().all(|(key, value)| {
                    b.iter()
                        .any(|(other, theirs)| other == key && same(value, theirs))
                })
        }
        (Json::Array(a), Json::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(x, y)| same(x, y))
        }
        _ => left == right,
    }
}

/// What a shipped file must not hold, each with the reason.
///
/// An identifier, a path or an internal skill of the repository that
/// maintains this engine names something an adopter does not have. The kind
/// names that only its package declares are held by the fixture that binds a
/// repository to another package (`tests/init_harness.rs`), because a kind
/// name is also an English word and only a lock says which ones a corpus has.
pub const FOREIGN: [(&str, &str); 11] = [
    (
        "HW-",
        "an identifier of the corpus that maintains this engine",
    ),
    ("docs/", "a path into that corpus"),
    ("engine/", "a path into the engine workspace"),
    ("tools/", "a path into that repository's tools"),
    (".githooks", "that repository's git hooks"),
    ("taxonomy-source", "that repository's package source"),
    ("hw-", "an internal agent or skill of the build order"),
    (
        "ste-editor",
        "an internal skill, for a language regime an adopter may not declare",
    ),
    ("repo-cleanup", "an internal skill"),
    (
        "headwater-engine",
        "an internal skill, for building the engine",
    ),
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
            assert!(
                member.text.ends_with('\n'),
                "{} ends with a newline",
                member.name
            );
        }
    }

    #[test]
    fn the_set_is_fifteen_paths_and_no_two_are_one() {
        let paths: Vec<String> = files().into_iter().map(|file| file.path).collect();
        let distinct: std::collections::BTreeSet<&String> = paths.iter().collect();
        assert_eq!(paths.len(), 15);
        assert_eq!(distinct.len(), 15);
        for config in CONFIGS {
            assert!(!paths.iter().any(|path| path == config.path()));
        }
    }

    #[test]
    fn no_hook_holds_a_name_of_the_corpus_that_maintains_this_engine_and_each_entry_calls_one() {
        let found: Vec<String> = HOOKS
            .iter()
            .flat_map(|member| {
                foreign(member.text)
                    .into_iter()
                    .map(move |(needle, why)| format!("{} holds `{needle}`: {why}", member.name))
            })
            .collect();
        assert!(found.is_empty(), "{}", found.join("\n"));
        for member in HOOKS {
            assert!(member.text.starts_with("#!/bin/sh\n"), "{}", member.name);
            assert!(member.text.ends_with('\n'), "{}", member.name);
        }
        for entry in HOOK_ENTRIES {
            assert!(
                HOOKS.iter().any(|member| member.name == entry.hook),
                "{} names no shipped hook",
                entry.hook
            );
        }
        for member in HOOKS.iter().filter(|member| member.name != "lib") {
            assert!(
                member.text.contains(". \"${0%/*}/lib.sh\""),
                "{} reads lib.sh",
                member.name
            );
        }
    }

    #[test]
    fn a_hook_carries_its_record_as_a_shell_comment() {
        let current = installed_as("#!/bin/sh\nnew\n", Form::Hook);
        assert!(!current.ends_with(" -->\n"));
        assert!(current.lines().last().is_some_and(
            |line| line.starts_with("# installed by headwater init --harness, digest sha256:")
        ));
        let earlier = installed_as("#!/bin/sh\nold\n", Form::Hook);
        assert_eq!(
            classify_as(Some(current.as_bytes()), &current, Form::Hook),
            Found::Current
        );
        assert_eq!(
            classify_as(Some(earlier.as_bytes()), &current, Form::Hook),
            Found::Earlier
        );
        assert_eq!(
            classify_as(
                Some(earlier.replacen("old", "mine", 1).as_bytes()),
                &current,
                Form::Hook
            ),
            Found::Foreign
        );
        // The Markdown record on a script is not the step's.
        let wrong = installed("#!/bin/sh\nold\n");
        assert_eq!(
            classify_as(Some(wrong.as_bytes()), &current, Form::Hook),
            Found::Foreign
        );
        assert_eq!(
            classify(Some(earlier.as_bytes()), &installed("x\n")),
            Found::Foreign
        );
    }

    #[test]
    fn a_fresh_settings_file_holds_every_entry_and_a_second_merge_writes_nothing() {
        let Merged::Write {
            bytes,
            absent,
            added,
        } = merge(Config::Settings, None)
        else {
            panic!("an absent file is written");
        };
        assert!(absent);
        assert_eq!(added, HOOK_ENTRIES.len());
        assert_eq!(
            merge(Config::Settings, Some(bytes.as_bytes())),
            Merged::Current
        );
        for entry in HOOK_ENTRIES {
            assert!(bytes.contains(&headwater_yaml::json::Json::string(entry.command()).render()));
        }
        let Merged::Write { bytes, .. } = merge(Config::Mcp, None) else {
            panic!("an absent file is written");
        };
        assert_eq!(merge(Config::Mcp, Some(bytes.as_bytes())), Merged::Current);
        assert!(bytes.contains("\"command\": \"headwater\""));
        assert!(bytes.contains("\"mcp\""));
    }

    #[test]
    fn a_merge_keeps_every_entry_of_the_adopter_in_its_place() {
        let theirs = r#"{"model": "x", "hooks": {"PreToolUse": [{"matcher": "Write|Edit", "hooks": [{"type": "command", "command": "mine.sh", "timeout": 5}]}, {"matcher": "Bash", "hooks": []}]}, "env": {"A": "1"}}"#;
        let Merged::Write {
            bytes,
            absent,
            added,
        } = merge(Config::Settings, Some(theirs.as_bytes()))
        else {
            panic!("a file that lacks entries is written");
        };
        assert!(!absent);
        assert_eq!(added, HOOK_ENTRIES.len());
        let order = [
            "\"model\"",
            "\"hooks\"",
            "mine.sh",
            "write.sh",
            "touch.sh",
            "\"Bash\"",
            "read.sh",
            "\"env\"",
        ];
        let at: Vec<usize> = order
            .iter()
            .map(|needle| {
                bytes
                    .find(needle)
                    .unwrap_or_else(|| panic!("{needle} is kept"))
            })
            .collect();
        assert!(at.windows(2).all(|pair| pair[0] < pair[1]), "{bytes}");
        assert!(bytes.contains("\"timeout\": 5"));
        assert_eq!(
            merge(Config::Settings, Some(bytes.as_bytes())),
            Merged::Current
        );
    }

    #[test]
    fn a_file_that_is_not_an_object_or_holds_another_headwater_server_is_refused() {
        for text in ["[1, 2]", "not json {", "{\"hooks\": []}"] {
            assert!(
                matches!(
                    merge(Config::Settings, Some(text.as_bytes())),
                    Merged::Foreign(_)
                ),
                "{text}"
            );
        }
        let other = r#"{"mcpServers": {"headwater": {"command": "./engine/target/release/headwater", "args": ["mcp"]}}}"#;
        assert!(matches!(
            merge(Config::Mcp, Some(other.as_bytes())),
            Merged::Foreign(_)
        ));
        let reordered = r#"{"mcpServers": {"other": {"command": "x"}, "headwater": {"args": ["mcp"], "command": "headwater"}}}"#;
        assert_eq!(
            merge(Config::Mcp, Some(reordered.as_bytes())),
            Merged::Current
        );
        let without = r#"{"mcpServers": {"other": {"command": "x"}}}"#;
        let Merged::Write { bytes, added, .. } = merge(Config::Mcp, Some(without.as_bytes()))
        else {
            panic!("a file without the server is written");
        };
        assert_eq!(added, 1);
        assert!(bytes.find("\"other\"") < bytes.find("\"headwater\""));
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
        assert_eq!(
            classify(Some(unrecorded.as_bytes()), &current),
            Found::Foreign
        );
        let appended = format!("{earlier}a line after the record\n");
        assert_eq!(
            classify(Some(appended.as_bytes()), &current),
            Found::Foreign
        );
        // This release's bytes with more after them are not this release.
        let grown = format!("{current}a line after the record\n");
        assert_eq!(classify(Some(grown.as_bytes()), &current), Found::Foreign);
        // Nor are they with more before them.
        let prefixed = format!("a line before the text\n{current}");
        assert_eq!(
            classify(Some(prefixed.as_bytes()), &current),
            Found::Foreign
        );
    }

    /// What git writes for a file it checks out with `core.autocrlf=true`.
    fn crlf(text: &str) -> String {
        text.replace('\n', "\r\n")
    }

    #[test]
    fn a_copy_git_checked_out_with_crlf_line_endings_is_the_steps_own() {
        for file in files() {
            assert!(
                !file.bytes.contains('\r'),
                "{} ships a carriage return, so a CRLF reading would change what it means",
                file.path
            );
            let checked_out = crlf(&file.bytes);
            assert_eq!(
                classify(Some(checked_out.as_bytes()), &file.bytes),
                Found::Current,
                "{} with CRLF line endings",
                file.path
            );
        }
        let current = installed("---\nname: x\n---\nnew\n");
        // Each `\r\n` is read alone, so mixed endings are still this release:
        // a file whose first line alone is CRLF, so that it ends in LF, and a
        // file whose first line alone is LF, so that it ends in CRLF.
        let first_line_only = current.replacen('\n', "\r\n", 1);
        assert_eq!(
            classify(Some(first_line_only.as_bytes()), &current),
            Found::Current
        );
        let all_but_first = crlf(&current).replacen("\r\n", "\n", 1);
        assert_eq!(
            classify(Some(all_but_first.as_bytes()), &current),
            Found::Current
        );
        let earlier = crlf(&installed("---\nname: x\n---\nold\n"));
        assert_eq!(classify(Some(earlier.as_bytes()), &current), Found::Earlier);
        // One changed byte above the record is an edit, in either ending.
        let edited = earlier.replacen("old", "mine", 1);
        assert_eq!(classify(Some(edited.as_bytes()), &current), Found::Foreign);
        let edited_current = crlf(&current).replacen("new", "neW", 1);
        assert_eq!(
            classify(Some(edited_current.as_bytes()), &current),
            Found::Foreign
        );
        // A carriage return that ends no line is a changed byte, not an ending.
        let stray = crlf(&current).replacen("new", "ne\rw", 1);
        assert_eq!(classify(Some(stray.as_bytes()), &current), Found::Foreign);
        // This release's bytes with more after them are not this release.
        let grown = crlf(&format!("{current}a line after the record\n"));
        assert_eq!(classify(Some(grown.as_bytes()), &current), Found::Foreign);
    }
}
