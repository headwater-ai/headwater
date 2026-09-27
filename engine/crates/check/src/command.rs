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
pub(crate) const SHELLS: [&str; 4] = ["sh", "shell", "bash", "console"];

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
    /// 2 reports the rest of a block as unreadable once a line ends inside a
    /// quote, a pair of backticks, a `$(` or a `${` (#1135). The first edition read
    /// each line alone and could read a quoted chain as one program. 3 reads
    /// a here-document end word, `$(( ))`, a `\` before a CR and a closer of
    /// the other kind as the shell does, and names the first program inside
    /// `$( )`, backticks, `<( )` and `>( )` (#1135).
    const VERSION: u32 = 3;
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
            for (line, read) in programs(&text, info == "console") {
                let (message, remediation) = match read {
                    Read::Unreadable => (
                        "a line of this shell block ends inside a quote, a `$'…'`, a pair of backticks, a `$(`, a `$((`, a `<(`, a `>(` or a `${`, so the rule cannot read the rest of the block (HW-DR-0077)".to_string(),
                        format!(
                            "close the quote, the `$'…'`, the backticks, the `$(`, the `$((`, the `<(`, the `>(` or the `${{` on the line that opens it, or mark the step as a deliberate exception with `<!-- headwater allow={} scope=block reason=accepted_deviation ... -->`",
                            self::RULE
                        ),
                    ),
                    Read::Program(program) => {
                        let name = program.rsplit('/').next().unwrap_or(program.as_str());
                        if self.commands.iter().any(|declared| declared == name) {
                            continue;
                        }
                        (
                            format!(
                                "a shell block runs `{name}`, and `commands` of the consumer surface does not declare it (HW-DR-0077)"
                            ),
                            format!(
                                "add `{name}` to `surface.commands` if an adopter must have it installed, name a declared command that does this, or mark the step as a deliberate exception with `<!-- headwater allow={} scope=block reason=accepted_deviation ... -->`",
                                self::RULE
                            ),
                        )
                    }
                };
                findings.push(Finding {
                    rule: self::RULE,
                    severity: Severity::Error,
                    obligation: None,
                    path: view.path().to_string(),
                    line: first.span.start.line + line,
                    column: 1,
                    message,
                    remediation,
                    patch: None,
                });
            }
        }
        findings.sort_by_key(|finding| (finding.line, finding.column));
        Outcome::failed(findings)
    }
}

/// What the rule reads at one line of a shell block.
#[derive(Debug, PartialEq, Eq)]
enum Read {
    /// A program that a command runs.
    Program(String),
    /// The rest of the block, from a line that ends inside a quote or a span.
    /// The rule reads nothing after it.
    Unreadable,
}

/// Every program a shell block runs, with the line of the block, counted from
/// zero, that the command holding it starts on.
///
/// Only a trailing `\` outside quotes and comments joins a line to the next.
/// A `\` before a CR joins nothing, because the shell reads it as an escape of
/// the CR. A line that ends inside a quote or a span joins nothing (#1135):
/// the rule reads what the line runs, then reports the rest of the block as
/// unreadable and stops. A checker that joined such a line
/// would have to lex the shell, and each construct it misread would hide the
/// lines after it rather than flag them.
fn programs(text: &str, console: bool) -> Vec<(usize, Read)> {
    let mut found = Vec::new();
    // The joined command so far, and for each line joined into it the byte
    // offset it starts at and its line number.
    let mut command = String::new();
    let mut starts: Vec<(usize, usize)> = Vec::new();
    let mut heredoc: Option<String> = None;
    // `lines` drops the CR of a CRLF line end. The shell does not: a `\`
    // before it escapes the CR, so it joins nothing.
    let carriage: Vec<bool> = text.split('\n').map(|line| line.ends_with('\r')).collect();
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
        command.push_str(trimmed);
        let scanned = scan(&command);
        match scanned.open {
            Some(open) if open != '#' => {
                read_command(&command, &starts, number, &mut found);
                found.push((number, Read::Unreadable));
                return found;
            }
            None if scanned.continued && !carriage.get(number).copied().unwrap_or(false) => {
                command.pop();
                command.push(' ');
                continue;
            }
            _ => {}
        }
        read_command(&command, &starts, number, &mut found);
        heredoc = heredoc_end(&command);
        command.clear();
        starts.clear();
    }
    // A last trailing `\` leaves a command the block does not finish. It is
    // still read.
    if let Some(&(_, last)) = starts.last() {
        read_command(&command, &starts, last, &mut found);
    }
    found
}

/// Push every program of one joined command, each with the line it starts on.
fn read_command(
    command: &str,
    starts: &[(usize, usize)],
    number: usize,
    found: &mut Vec<(usize, Read)>,
) {
    for (offset, segment) in segments(command) {
        if let Some(program) = program_of(segment) {
            let line = starts
                .iter()
                .rev()
                .find(|(start, _)| *start <= offset)
                .map_or(number, |(_, line)| *line);
            found.push((line, Read::Program(program)));
        }
    }
}

/// The parts of a command between the operators that start a new program, each
/// with its byte offset. An operator inside quotes is part of an argument.
fn segments(command: &str) -> Vec<(usize, &str)> {
    scan(command).segments
}

/// What one pass over a command finds.
struct Scan<'a> {
    /// The parts between the operators that start a new program, each with
    /// its byte offset.
    segments: Vec<(usize, &'a str)>,
    /// What stops the command from ending cleanly: the innermost span still
    /// open (`'`, `"`, `a` for `$'…'`, `` ` ``, `(` for `$(`, `<(` and `>(`,
    /// `m` for `$((`, `p` for a `(` inside `$((`, `{` for `${`), `#` for a
    /// comment, or nothing. Every value but `#` leaves the rest of the block
    /// unreadable.
    open: Option<char>,
    /// The byte offset of the first `<<` outside quotes, comments and
    /// `$(( ))`.
    heredoc: Option<usize>,
    /// Whether the command ends in a `\\` that escapes nothing, outside every
    /// span and comment: the one thing that joins the next line.
    continued: bool,
}

/// Read one command as a shell splits it, with one stack of open spans.
///
/// A single quote, `$'…'`, backticks, a double quote, `$(`, `<(`, `>(`,
/// `$((` and `${` each open a span, and each closes only the span of its own
/// kind at the top of the stack. A `)` inside `${…}` and a `}` inside `$( )`
/// are text, as the shell reads them. `$((` closes on `))`, and a `(` inside
/// it opens a span that a `)` closes. A `<<` inside `$(( ))` is a shift. The
/// first word after `$(`, `<(`, `>(` or an opening backtick names a program,
/// and the text after the closer, up to the next operator, is an argument of
/// the command around it. Inside single quotes nothing is special but the closing quote. Inside
/// `$'…'` and backticks a backslash escapes the next character and no quote
/// opens. Inside double quotes a backslash escapes, and `$(`, `${` and
/// backticks open. A `#` opens a comment only when no span is open, at the
/// start of the command or after a blank, `;`, `&` or `|`. An operator starts
/// a new program only when no quote is open.
fn scan(command: &str) -> Scan<'_> {
    let mut out = Vec::new();
    // Where the part that names the next program starts, or `None` after the
    // closer of a substitution, where the text up to the next operator is an
    // argument of the command around it.
    let mut start = Some(0);
    let mut stack: Vec<char> = Vec::new();
    let mut heredoc = None;
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
        let top = stack.last().copied();
        let quoted = stack.iter().any(|span| matches!(span, '\'' | '"' | 'a'));
        let arithmetic = stack.contains(&'m');
        let next = bytes.get(at + 1).copied();
        match (top, c) {
            (Some('\''), '\'') | (Some('a'), '\'') | (Some('"'), '"') => {
                stack.pop();
            }
            (Some('`'), '`') => {
                stack.pop();
                close(command, &mut start, at, &mut out);
            }
            (Some('\''), _) => {}
            (Some('a' | '`' | '"' | 'm' | 'p') | None | Some('(' | '{'), '\\') => escaped = true,
            (Some('a'), _) => {}
            (Some('`' | '"' | 'm' | 'p') | None | Some('(' | '{'), '$')
                if next == Some(b'(') && bytes.get(at + 2) == Some(&b'(') =>
            {
                stack.push('m');
                at += 2;
            }
            (Some('`' | '"' | 'm' | 'p') | None | Some('(' | '{'), '$') if next == Some(b'(') => {
                stack.push('(');
                open(command, &mut start, at, 2, &mut out);
                at += 1;
            }
            (Some('`' | '"' | 'm' | 'p') | None | Some('(' | '{'), '$') if next == Some(b'{') => {
                stack.push('{');
                at += 1;
            }
            (Some('"' | 'm' | 'p') | None | Some('(' | '{'), '`') => {
                stack.push('`');
                open(command, &mut start, at, 1, &mut out);
            }
            (Some('"'), _) => {}
            (None | Some('(' | '{'), '<' | '>') if !quoted && next == Some(b'(') => {
                stack.push('(');
                open(command, &mut start, at, 2, &mut out);
                at += 1;
            }
            (None | Some('(' | '{'), '$') if next == Some(b'\'') => {
                stack.push('a');
                at += 1;
            }
            (None | Some('(' | '{' | 'm' | 'p'), '\'' | '"') => stack.push(c),
            (Some('m' | 'p'), '(') => stack.push('p'),
            (Some('p'), ')') => {
                stack.pop();
            }
            (Some('m'), ')') if next == Some(b')') => {
                stack.pop();
                at += 1;
            }
            (Some('('), ')') => {
                stack.pop();
                close(command, &mut start, at, &mut out);
            }
            (Some('{'), '}') => {
                stack.pop();
            }
            (None, '#')
                if at == 0 || matches!(bytes[at - 1], b' ' | b'\t' | b';' | b'&' | b'|') =>
            {
                if let Some(from) = start {
                    out.push((from, &command[from..at]));
                }
                return Scan {
                    segments: out,
                    open: Some('#'),
                    heredoc,
                    continued: false,
                };
            }
            (None | Some('(' | '{'), '<')
                if !quoted && !arithmetic && heredoc.is_none() && next == Some(b'<') =>
            {
                heredoc = Some(at);
                at += 1;
            }
            (None | Some('(' | '{' | '`'), '|' | '&' | ';') if !quoted => {
                // `>&` and `&>` redirect a stream and start no program.
                let redirect =
                    (c == '&') && ((at > 0 && bytes[at - 1] == b'>') || next == Some(b'>'));
                if !redirect {
                    if let Some(from) = start {
                        out.push((from, &command[from..at]));
                    }
                    while at + 1 < bytes.len() && matches!(bytes[at + 1], b'|' | b'&' | b';') {
                        at += 1;
                    }
                    start = Some(at + 1);
                }
            }
            _ => {}
        }
        at += 1;
    }
    if let Some(from) = start {
        out.push((from, &command[from..]));
    }
    Scan {
        segments: out,
        open: stack.last().copied(),
        heredoc,
        continued: escaped && stack.is_empty(),
    }
}

/// A substitution opens at `at` with an opener `width` bytes long: the part
/// before it ends, and the first word inside it names a program.
fn open<'a>(
    command: &'a str,
    start: &mut Option<usize>,
    at: usize,
    width: usize,
    out: &mut Vec<(usize, &'a str)>,
) {
    if let Some(from) = *start {
        out.push((from, &command[from..at]));
    }
    *start = Some(at + width);
}

/// A substitution closes at `at`: its last part ends, and the text after it
/// up to the next operator is an argument of the command around it.
fn close<'a>(
    command: &'a str,
    start: &mut Option<usize>,
    at: usize,
    out: &mut Vec<(usize, &'a str)>,
) {
    if let Some(from) = *start {
        out.push((from, &command[from..at]));
    }
    *start = None;
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
        // A word that opens with `#` is text inside a span, where no comment
        // opens, and it names no program.
        if word.is_empty() || word.starts_with(['$', '<', '>', '#']) {
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
/// A `<<` inside quotes or in a comment opens none.
fn heredoc_end(command: &str) -> Option<String> {
    let at = scan(command).heredoc?;
    let rest = &command[at + 2..];
    if rest.starts_with('<') {
        return None;
    }
    let rest = rest.strip_prefix('-').unwrap_or(rest).trim_start();
    // The shell removes quotes and each backslash from the word. A `$'…'`
    // quotes as `'…'` does, and an unquoted blank or operator ends the word.
    let mut word = String::new();
    let mut quote: Option<char> = None;
    let mut chars = rest.chars().peekable();
    while let Some(c) = chars.next() {
        match (quote, c) {
            (Some(q), _) if c == q => quote = None,
            (Some('"'), '\\') => {
                if let Some(escaped) = chars.next() {
                    word.push(escaped);
                }
            }
            (Some(_), _) => word.push(c),
            (None, '\'' | '"') => quote = Some(c),
            (None, '$') if chars.peek() == Some(&'\'') => {}
            (None, '\\') => {
                if let Some(escaped) = chars.next() {
                    word.push(escaped);
                }
            }
            (None, _)
                if c.is_whitespace() || matches!(c, ';' | '|' | '&' | '(' | ')' | '<' | '>') =>
            {
                break;
            }
            (None, _) => word.push(c),
        }
    }
    (!word.is_empty()).then_some(word)
}

#[cfg(test)]
mod tests {
    use super::{programs, Read};

    fn names(text: &str, console: bool) -> Vec<(usize, String)> {
        programs(text, console)
            .into_iter()
            .map(|(line, read)| match read {
                Read::Program(name) => (line, name),
                Read::Unreadable => (line, UNREADABLE.to_string()),
            })
            .collect()
    }

    /// What `names` writes for a remainder the rule cannot read.
    const UNREADABLE: &str = "<unreadable>";

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

    /// Rule 1 (#1135): a line that ends inside a quote joins nothing. The
    /// rule reads what the line runs before the quote and reports the rest of
    /// the block as unreadable, so the chain after the quote is flagged rather
    /// than hidden.
    #[test]
    fn a_quote_open_at_the_end_of_a_line_leaves_the_rest_of_the_block_unreadable() {
        let block = "printf 'a\nheadwater check' && echo RAN-2 && printf 'b'\n";
        assert_eq!(
            names(block, false),
            pairs(&[(0, "printf"), (0, UNREADABLE)])
        );
        assert_eq!(
            names("echo ok\nprintf \"a\nnpm install\n", false),
            pairs(&[(0, "echo"), (1, "printf"), (1, UNREADABLE)])
        );
    }

    /// Rule 1: an open `$'…'`, an open pair of backticks and an unclosed `$(`
    /// leave the rest of the block unreadable in the same way.
    #[test]
    fn every_open_span_leaves_the_rest_of_the_block_unreadable() {
        assert_eq!(
            names("printf $'a\nheadwater check' && echo RAN\n", false),
            pairs(&[(0, "printf"), (0, UNREADABLE)])
        );
        assert_eq!(
            names("echo `date\nnpm install\n", false),
            pairs(&[(0, "echo"), (0, "date"), (0, UNREADABLE)])
        );
        assert_eq!(
            names("echo $(date\nnpm install\n", false),
            pairs(&[(0, "echo"), (0, "date"), (0, UNREADABLE)])
        );
    }

    /// Rule 1: a quote the block never closes is flagged.
    #[test]
    fn a_quote_the_block_never_closes_is_flagged() {
        assert_eq!(
            names("echo ok && npm install '\n", false),
            pairs(&[(0, "echo"), (0, "npm"), (0, UNREADABLE)])
        );
    }

    /// Rule 2: a trailing `\` joins the next line, and one in a comment does
    /// not.
    #[test]
    fn a_trailing_backslash_joins_and_one_in_a_comment_does_not() {
        assert_eq!(
            names("headwater check \\\n  && npm install\n", false),
            pairs(&[(0, "headwater"), (1, "npm")])
        );
        assert_eq!(
            names("headwater check # note \\\nnpm install\n", false),
            pairs(&[(0, "headwater"), (1, "npm")])
        );
        // An escaped backslash at the end of a line is a literal backslash,
        // and it joins nothing (X4 of the third verify).
        assert_eq!(
            names("echo a\\\\\nnpm ci\n", false),
            pairs(&[(0, "echo"), (1, "npm")])
        );
    }

    /// Rule 3 (#1135): inside `$'…'` a backslash escapes the next character,
    /// so `\'` does not close the quote.
    #[test]
    fn a_backslash_escapes_a_quote_inside_an_ansi_c_quote() {
        assert_eq!(
            names("printf $'\\'' && echo RAN-3\n", false),
            pairs(&[(0, "printf"), (0, "echo")])
        );
    }

    /// Rule 3: a quote inside backticks on one line opens nothing.
    #[test]
    fn a_quote_inside_backticks_opens_nothing() {
        assert_eq!(
            names("echo `echo it's` && npm install\nnpm ci\n", false),
            pairs(&[(0, "echo"), (0, "echo"), (0, "npm"), (1, "npm")])
        );
    }

    /// Rule 4: a `#` after a blank, `;`, `&` or `|` opens a comment, so an
    /// apostrophe in it opens nothing and the comment is no program.
    #[test]
    fn a_hash_after_a_blank_or_an_operator_opens_a_comment() {
        assert_eq!(
            names("headwater check # it's here\nnpm install\n", false),
            pairs(&[(0, "headwater"), (1, "npm")])
        );
        assert_eq!(
            names("echo a &&# it's\nnpm install\n", false),
            pairs(&[(0, "echo"), (1, "npm")])
        );
    }

    /// Rule 4: a `#` after `)`, `<` or `>` is part of a word. So the chain
    /// after `$(date)#x` is read, and an apostrophe after `)#` is flagged.
    #[test]
    fn a_hash_after_a_parenthesis_or_a_redirect_is_part_of_a_word() {
        assert_eq!(
            names("echo $(date)#x && npm install\n", false),
            pairs(&[(0, "echo"), (0, "date"), (0, "npm")])
        );
        assert_eq!(
            names("(echo a)#'\nnpm install\n", false),
            pairs(&[(0, "echo"), (0, UNREADABLE)])
        );
        assert_eq!(
            names("echo hi >#it's\nnpm install\n", false),
            pairs(&[(0, "echo"), (0, UNREADABLE)])
        );
    }

    /// Rule 5: a `<<` inside quotes or in a comment opens no here-document,
    /// and an apostrophe in a here-document body is not read.
    #[test]
    fn a_heredoc_opens_only_outside_quotes_and_comments() {
        assert_eq!(
            names("printf '<<EOF' && echo hi\nnpm install\n", false),
            pairs(&[(0, "printf"), (0, "echo"), (1, "npm")])
        );
        assert_eq!(
            names("echo hi # see <<EOF\nnpm install\n", false),
            pairs(&[(0, "echo"), (1, "npm")])
        );
        assert_eq!(
            names("cat <<'EOF'\ndon't\nEOF\nnpm install && echo RAN\n", false),
            pairs(&[(0, "cat"), (3, "npm"), (3, "echo")])
        );
    }

    /// A `$(` inside double quotes opens a span of its own, so a double quote
    /// inside it is a quote of its own and the apostrophe in that quote opens
    /// nothing. The line closes cleanly and the chain after it is read (M4).
    #[test]
    fn a_double_quote_inside_a_quoted_substitution_is_a_quote_of_its_own() {
        assert_eq!(
            names("echo \"$(echo \"it's\")\" && npm install\nnpm ci\n", false),
            pairs(&[(0, "echo"), (0, "echo"), (0, "npm"), (1, "npm")])
        );
    }

    /// Inside `${…}` a `#` is literal and opens no comment, so the chain
    /// after the expansion is read (P1e, X1, X2 of the third verify).
    #[test]
    fn a_hash_inside_a_parameter_expansion_opens_no_comment() {
        assert_eq!(
            names("echo ${x:-a #b} && npm ci\n", false),
            pairs(&[(0, "echo"), (0, "npm")])
        );
        assert_eq!(
            names("git log ${x:-a #b} && npm ci\n", false),
            pairs(&[(0, "git"), (0, "npm")])
        );
        assert_eq!(
            names("echo ${x// #/-} && npm ci\n", false),
            pairs(&[(0, "echo"), (0, "npm")])
        );
    }

    /// Rule 1: a line that ends with a `${` still open leaves the rest of
    /// the block unreadable, and so does a comment inside an open `$(`.
    #[test]
    fn an_open_expansion_or_a_comment_inside_an_open_substitution_is_flagged() {
        assert_eq!(
            names("echo ${x:-a\nnpm ci\n", false),
            pairs(&[(0, "echo"), (0, UNREADABLE)])
        );
        assert_eq!(
            names("x=$(ls # c\nnpm ci\n", false),
            pairs(&[(0, "ls"), (0, UNREADABLE)])
        );
    }

    /// The open spans are one stack. A `}` inside a `$( )` does not close the
    /// `${` around it, and a `)` inside `${…}` does not close a `$(` around
    /// it: each is text (W1 of the fourth verify of #1195).
    #[test]
    fn a_closer_that_does_not_match_the_innermost_span_is_text() {
        assert_eq!(
            names("echo ${x:-$(echo }) #c} && npm ci\nnpm i\n", false),
            pairs(&[(0, "echo"), (0, "echo"), (0, "npm"), (1, "npm")])
        );
        assert_eq!(
            names("echo $(echo ${x:-a)}) #c && npm ci\nnpm i\n", false),
            pairs(&[(0, "echo"), (0, "echo"), (1, "npm")])
        );
    }

    /// A span closed by its own kind is read through: after `$(echo a)` the
    /// `${` is still open, so ` #c` is part of the word and the chain is read.
    #[test]
    fn a_span_closed_by_its_own_kind_reads_on() {
        assert_eq!(
            names("echo ${x:-$(echo a) #c} && npm ci\n", false),
            pairs(&[(0, "echo"), (0, "echo"), (0, "npm")])
        );
        assert_eq!(
            names("echo \"a)\" && npm ci\n", false),
            pairs(&[(0, "echo"), (0, "npm")])
        );
        assert_eq!(
            names("{ echo a; } #c && npm ci\nnpm i\n", false),
            pairs(&[(0, "echo"), (1, "npm")])
        );
    }

    /// An operator inside backticks starts a new program, as it does on
    /// `main`, so the rule reads it.
    #[test]
    fn an_operator_inside_backticks_starts_a_program() {
        assert_eq!(
            names("echo `a && npm ci`\n", false),
            pairs(&[(0, "echo"), (0, "a"), (0, "npm")])
        );
    }

    /// The decisive case of #1135: a backslash in the end word is removed, as
    /// the shell removes it, so the body ends at `EOF` and the command after it
    /// is read. Before the fix the end word was `\EOF`, and every later line of
    /// the block was skipped as body.
    #[test]
    fn a_backslash_in_a_heredoc_end_word_is_removed() {
        assert_eq!(
            names("cat <<\\EOF\nbody\nEOF\nnpm ci\n", false),
            pairs(&[(0, "cat"), (3, "npm")])
        );
        assert_eq!(
            names("cat <<E\\OF\nx\nEOF\nnpm ci\n", false),
            pairs(&[(0, "cat"), (3, "npm")])
        );
    }

    /// A redirect attached to the end word is not part of it, and `$'…'`
    /// quotes the end word as `'…'` does.
    #[test]
    fn a_heredoc_end_word_stops_at_a_redirect_and_reads_an_ansi_c_quote() {
        assert_eq!(
            names("cat <<EOF>out.yml\nbody\nEOF\nnpm ci\n", false),
            pairs(&[(0, "cat"), (3, "npm")])
        );
        assert_eq!(
            names("cat <<$'EOF'\nx\nEOF\nnpm ci\n", false),
            pairs(&[(0, "cat"), (3, "npm")])
        );
    }

    /// A `<<` inside `$(( ))` is a shift and opens no here-document, and
    /// `$(( ))` closes on its own `))`.
    #[test]
    fn a_shift_inside_arithmetic_opens_no_heredoc() {
        assert_eq!(
            names("echo $((1<<2))\nnpm ci\n", false),
            pairs(&[(0, "echo"), (1, "npm")])
        );
        assert_eq!(
            names("echo $(( (1<<2) | 3 )) && npm ci\n", false),
            pairs(&[(0, "echo"), (0, "npm")])
        );
        assert_eq!(
            names("echo $((1\nnpm ci\n", false),
            pairs(&[(0, "echo"), (0, UNREADABLE)])
        );
    }

    /// A `\` before a CR does not join the next line: the shell reads the
    /// backslash as an escape of the CR. So in a block with CRLF line ends the
    /// next line is a command of its own.
    #[test]
    fn a_backslash_before_a_carriage_return_joins_nothing() {
        assert_eq!(
            names("echo a \\\r\nnpm ci\r\n", false),
            pairs(&[(0, "echo"), (1, "npm")])
        );
        assert_eq!(names("echo a \\\nnpm ci\n", false), pairs(&[(0, "echo")]));
    }

    /// A `)` inside `${…}` and a `}` inside `$( )` are text, as the shell reads
    /// them, so each of the four shapes the fifth verify of #1195 found reads
    /// on.
    #[test]
    fn a_closer_of_the_other_kind_is_text() {
        assert_eq!(
            names("echo ${x:-)} && npm ci\nnpm i\n", false),
            pairs(&[(0, "echo"), (0, "npm"), (1, "npm")])
        );
        assert_eq!(
            names("echo \"$(echo })\" && npm ci\nnpm i\n", false),
            pairs(&[(0, "echo"), (0, "echo"), (0, "npm"), (1, "npm")])
        );
        assert_eq!(
            names("echo ${x:-$((1))} && npm ci\nnpm i\n", false),
            pairs(&[(0, "echo"), (0, "npm"), (1, "npm")])
        );
        assert_eq!(
            names("echo $(echo \"$(echo })\") && npm ci\nnpm i\n", false),
            pairs(&[
                (0, "echo"),
                (0, "echo"),
                (0, "echo"),
                (0, "npm"),
                (1, "npm")
            ])
        );
    }

    /// A `}` does not close a `$(`. If any closer closed any span, the `}`
    /// would close the `$(`, the `)` would close the `${`, and ` #c} && npm
    /// ci` would be a comment that hides `npm`.
    #[test]
    fn a_brace_does_not_close_a_substitution() {
        assert_eq!(
            names("echo ${x:-$(echo }) #c} && npm ci\n", false),
            pairs(&[(0, "echo"), (0, "echo"), (0, "npm")])
        );
        assert_eq!(
            names("echo $(echo }) && npm ci\n", false),
            pairs(&[(0, "echo"), (0, "echo"), (0, "npm")])
        );
    }

    /// The first word inside `$( )`, a pair of backticks, `<( )` and `>( )`
    /// is a program, and the text after the closer is an argument of the
    /// command around it.
    #[test]
    fn the_first_word_of_a_substitution_is_a_program() {
        assert_eq!(
            names("echo `whoami`\n", false),
            pairs(&[(0, "echo"), (0, "whoami")])
        );
        assert_eq!(
            names("echo $(whoami) done\n", false),
            pairs(&[(0, "echo"), (0, "whoami")])
        );
        assert_eq!(
            names("cat <(whoami)\n", false),
            pairs(&[(0, "cat"), (0, "whoami")])
        );
        assert_eq!(
            names("diff <(npm install) <(echo b)\n", false),
            pairs(&[(0, "diff"), (0, "npm"), (0, "echo")])
        );
        assert_eq!(
            names("echo $(curl x | sh)\n", false),
            pairs(&[(0, "echo"), (0, "curl"), (0, "sh")])
        );
        assert_eq!(
            names("tee >(gzip > a.gz) < x\n", false),
            pairs(&[(0, "tee"), (0, "gzip")])
        );
        assert_eq!(
            names("printf '$(whoami)' \"`id`\"\n", false),
            pairs(&[(0, "printf"), (0, "id")])
        );
    }
}
