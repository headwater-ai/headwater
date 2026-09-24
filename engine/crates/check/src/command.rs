// SPDX-License-Identifier: Apache-2.0
//! A Document-origin check: a page for an adopter that tells them to run a
//! program the consumer surface does not declare.
//!
//! # The decision this implements
//!
//! [HW-DR-0077](../../../../docs/decisions/0077-the-consumer-surface-is-what-an-adopter-receives-runs-and-must-have-installed-and-it-is-a-closed-and-declared-list.md)
//! rules that the consumer surface is a closed list, and clause 1 of its
//! Decision puts "Git, `sh` and the download and unpack tools of the operating
//! system" on it as platform prerequisites that "the surface declares each one
//! by name". The `commands` member of `surface` is that list, and this rule
//! holds the shell blocks of every page on `adopter_documents` against it.
//!
//! The engine names no program of its own. A second list here would be a
//! second copy of a rule, and a corpus with no `commands` member generates no
//! instance, for the reason [`crate::surface`] gives: a list the engine
//! supplies reports zero by construction.
//!
//! # Which blocks it reads
//!
//! A fenced code block whose info string has `sh`, `shell`, `bash` or
//! `console` as its first word, compared exactly, so `SH` and `shell-session`
//! are not read. Over this repository's adopter pages on the day
//! the rule landed, 67 of 70 fenced blocks carried no info string, and most of
//! those were program output, YAML or a file listing. A rule that read an
//! untagged block would report each of them, so the author states which blocks
//! are commands by tagging them, and an untagged or indented block is a stated
//! limit (#1051).
//!
//! # What a command is
//!
//! A line of the block, joined with every line that a trailing `\` continues it
//! onto. The rule reads the first word of the command, and the first word after
//! each `|`, `||`, `&&`, `&` and `;` that no quote encloses. It skips:
//!
//! - a blank line and a line that opens with `#`,
//! - a `$ ` prompt before the command, and in a `console` block every line
//!   that has no prompt, because that line is output,
//! - an assignment such as `NAME=value` before the program,
//! - a reserved word of the shell grammar, such as `if` or `then`, before the
//!   program, and a segment that opens with `for`, `case`, `fi`, `done` or
//!   `esac`, whose words are not a program,
//! - the body of a here-document.
//!
//! A program is compared by the last segment of its path, so `./headwater` and
//! `~/bin/headwater` are `headwater`. A program that another program runs, as
//! `sed` is in `xargs sed`, and a command inside `$( )` are not read. The
//! contract states both as limits.

use crate::finding::{Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{DocumentCheck, DocumentView};
use crate::shape::Shape;
use crate::surface::glob_matches;
use headwater_doc::body::{BlockKind, Ownership};

pub const RULE: &str = "surface.command.undeclared";

/// The info strings that mark a block as commands to run. This is the syntax
/// of a Markdown fence, and not a list of programs.
const SHELLS: [&str; 4] = ["sh", "shell", "bash", "console"];

/// The check, generated from the `surface` block of the taxonomy.
pub struct Undeclared {
    adopter_documents: Vec<String>,
    commands: Vec<String>,
}

impl Undeclared {
    /// The generation step. A surface that names no adopter document or no
    /// command generates no instance.
    pub fn over(shape: &Shape) -> Self {
        Undeclared {
            adopter_documents: shape.surface.adopter_documents.clone(),
            commands: shape.surface.commands.clone(),
        }
    }

    fn declared(&self) -> bool {
        !self.adopter_documents.is_empty() && !self.commands.is_empty()
    }

    fn for_an_adopter(&self, path: &str) -> bool {
        self.adopter_documents
            .iter()
            .any(|glob| glob_matches(glob, path))
    }
}

impl DocumentCheck for Undeclared {
    const RULE: &'static str = self::RULE;
    const VERSION: u32 = 1;
    const NEEDS_BODY: bool = true;

    fn instantiates(&self, _kind: &str) -> bool {
        self.declared()
    }

    fn selects(&self, path: &str) -> bool {
        self.for_an_adopter(path)
    }

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
        if !self.declared() || !self.for_an_adopter(view.path()) {
            return Outcome::Passed;
        }
        let mut findings = Vec::new();
        for block in view
            .body()
            .map(|body| body.blocks.as_slice())
            .unwrap_or(&[])
        {
            let Some(info) = block.info.as_deref() else {
                continue;
            };
            if block.kind != BlockKind::Code || !SHELLS.contains(&info) {
                continue;
            }
            let Some(first) = block.runs.first() else {
                continue;
            };
            let text: String = block
                .runs
                .iter()
                .filter(|run| run.ownership == Ownership::Code)
                .map(|run| run.text.as_str())
                .collect();
            for (line, program) in programs(&text, info == "console") {
                let name = program.rsplit('/').next().unwrap_or(program.as_str());
                if self.commands.iter().any(|declared| declared == name) {
                    continue;
                }
                findings.push(Finding {
                    rule: self::RULE,
                    severity: Severity::Error,
                    obligation: None,
                    path: view.path().to_string(),
                    line: first.span.start.line + line,
                    column: 1,
                    message: format!(
                        "a shell block runs `{name}`, and `commands` of the consumer surface does not declare it (HW-DR-0077)"
                    ),
                    remediation: format!(
                        "add `{name}` to `surface.commands` if an adopter must have it installed, name a declared command that does this, or mark the step as a deliberate exception with `<!-- headwater allow={} scope=block reason=accepted_deviation ... -->`",
                        self::RULE
                    ),
                    patch: None,
                });
            }
        }
        findings.sort_by_key(|finding| (finding.line, finding.column));
        Outcome::failed(findings)
    }
}

/// Every program a shell block runs, with the line of the block, counted from
/// zero, that the command holding it starts on.
fn programs(text: &str, console: bool) -> Vec<(usize, String)> {
    let mut found = Vec::new();
    // The joined command so far, and for each line joined into it the byte
    // offset it starts at and its line number.
    let mut command = String::new();
    let mut starts: Vec<(usize, usize)> = Vec::new();
    let mut heredoc: Option<String> = None;
    for (number, raw) in text.lines().enumerate() {
        if let Some(end) = &heredoc {
            if raw.trim() == end {
                heredoc = None;
            }
            continue;
        }
        let continuing = !starts.is_empty();
        let line = match (continuing, console) {
            (true, true) => raw.trim_start().strip_prefix("> ").unwrap_or(raw),
            (true, false) => raw,
            (false, _) => {
                let trimmed = raw.trim_start();
                match (trimmed.strip_prefix("$ "), console) {
                    (Some(rest), _) => rest,
                    (None, true) => continue,
                    (None, false) => raw,
                }
            }
        };
        let trimmed = line.trim();
        if !continuing && (trimmed.is_empty() || trimmed.starts_with('#')) {
            continue;
        }
        starts.push((command.len(), number));
        match trimmed.strip_suffix('\\') {
            Some(head) => {
                command.push_str(head);
                command.push(' ');
            }
            None => {
                command.push_str(trimmed);
                for (offset, segment) in segments(&command) {
                    if let Some(program) = program_of(segment) {
                        let line = starts
                            .iter()
                            .rev()
                            .find(|(start, _)| *start <= offset)
                            .map_or(number, |(_, line)| *line);
                        found.push((line, program));
                    }
                }
                heredoc = heredoc_end(&command);
                command.clear();
                starts.clear();
            }
        }
    }
    found
}

/// The parts of a command between the operators that start a new program, each
/// with its byte offset. An operator inside quotes is part of an argument.
fn segments(command: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let bytes = command.as_bytes();
    let mut at = 0;
    while at < bytes.len() {
        let c = bytes[at] as char;
        if escaped {
            escaped = false;
            at += 1;
            continue;
        }
        match (quote, c) {
            (Some('\''), '\'') | (Some('"'), '"') => quote = None,
            (Some('"'), '\\') | (None, '\\') => escaped = true,
            (Some(_), _) => {}
            (None, '\'' | '"') => quote = Some(c),
            (None, '|' | '&' | ';') => {
                // `>&` and `&>` redirect a stream and start no program.
                let redirect = (c == '&')
                    && ((at > 0 && bytes[at - 1] == b'>') || bytes.get(at + 1) == Some(&b'>'));
                if !redirect {
                    out.push((start, &command[start..at]));
                    while at + 1 < bytes.len() && matches!(bytes[at + 1], b'|' | b'&' | b';') {
                        at += 1;
                    }
                    start = at + 1;
                }
            }
            (None, _) => {}
        }
        at += 1;
    }
    out.push((start, &command[start..]));
    out
}

/// The program a segment runs, if it names one.
fn program_of(segment: &str) -> Option<String> {
    for word in segment.split_whitespace() {
        let word = word.trim_start_matches(['(', '{']);
        match word {
            "" | "!" | "if" | "then" | "else" | "elif" | "while" | "until" | "do" | "time" => {
                continue;
            }
            "for" | "case" | "select" | "fi" | "done" | "esac" | "}" | ")" | "in" => {
                return None;
            }
            _ => {}
        }
        if is_assignment(word) {
            continue;
        }
        let word = word.trim_matches(['"', '\'', ')', '}']);
        if word.is_empty() || word.starts_with(['$', '<', '>']) {
            return None;
        }
        return Some(word.to_string());
    }
    None
}

fn is_assignment(word: &str) -> bool {
    let Some((name, _)) = word.split_once('=') else {
        return false;
    };
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// The word that ends a here-document this command opens, if it opens one.
fn heredoc_end(command: &str) -> Option<String> {
    let at = command.find("<<")?;
    let rest = &command[at + 2..];
    if rest.starts_with('<') {
        return None;
    }
    let rest = rest.strip_prefix('-').unwrap_or(rest).trim_start();
    let word: String = rest
        .chars()
        .take_while(|c| !c.is_whitespace() && !matches!(c, ';' | '|' | '&' | ')'))
        .filter(|c| !matches!(c, '\'' | '"'))
        .collect();
    (!word.is_empty()).then_some(word)
}

#[cfg(test)]
mod tests {
    use super::programs;

    fn names(text: &str, console: bool) -> Vec<(usize, String)> {
        programs(text, console)
    }

    fn pairs(list: &[(usize, &str)]) -> Vec<(usize, String)> {
        list.iter().map(|(l, n)| (*l, n.to_string())).collect()
    }

    #[test]
    fn a_prompt_a_comment_and_a_blank_line_are_skipped() {
        assert_eq!(
            names("# set up\n\n$ headwater check\nFOO=1 git init\n", false),
            pairs(&[(2, "headwater"), (3, "git")])
        );
    }

    #[test]
    fn every_program_of_a_pipeline_is_read_and_a_quoted_bar_is_not_an_operator() {
        let block = "grep -rlZ -E 'a|b' docs \\\n  | xargs -0 sed -i \\\n      -e 's#a#b#g'\n";
        assert_eq!(names(block, false), pairs(&[(0, "grep"), (1, "xargs")]));
        assert_eq!(
            names("cd x && make; ls | wc -l 2>&1\n", false),
            pairs(&[(0, "cd"), (0, "make"), (0, "ls"), (0, "wc")])
        );
    }

    #[test]
    fn a_console_block_reads_only_prompted_lines() {
        assert_eq!(
            names("$ headwater check\nwrote x\n0 findings\n", true),
            pairs(&[(0, "headwater")])
        );
    }

    #[test]
    fn grammar_and_a_heredoc_body_are_not_programs() {
        let block = "if headwater check; then\n  echo ok\nfi\ncat > x <<'EOF'\nrm -rf /\nEOF\nfor f in a b; do tar x; done\n";
        assert_eq!(
            names(block, false),
            pairs(&[(0, "headwater"), (1, "echo"), (3, "cat"), (6, "tar")])
        );
    }
}
