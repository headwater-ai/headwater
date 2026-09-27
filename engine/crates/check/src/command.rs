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
    /// each line alone and could read a quoted chain as one program.
    const VERSION: u32 = 2;
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
                        "a line of this shell block ends inside a quote, a pair of backticks, a `$(` or a `${`, so the rule cannot read the rest of the block (HW-DR-0077)".to_string(),
                        format!(
                            "close the quote on the line that opens it, or mark the step as a deliberate exception with `<!-- headwater allow={} scope=block reason=accepted_deviation ... -->`",
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
    /// The rest of the block, from a line that ends inside a quote, a pair of
    /// backticks or a `$(`. The rule reads nothing after it.
    Unreadable,
}

/// Every program a shell block runs, with the line of the block, counted from
/// zero, that the command holding it starts on.
///
/// Only a trailing `\` outside quotes and comments joins a line to the next.
/// A line that ends inside a quote, a pair of backticks, a `$(` or a `${` joins
/// nothing (#1135): the rule reads what the line runs, then reports the rest
/// of the block as unreadable and stops. A checker that joined such a line
/// would have to lex the shell, and each construct it misread would hide the
/// lines after it rather than flag them.
fn programs(text: &str, console: bool) -> Vec<(usize, Read)> {
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
        command.push_str(trimmed);
        let scanned = scan(&command);
        match scanned.open {
            Some('\'' | '"' | 'a' | '`' | '(' | '{') => {
                read_command(&command, &starts, number, &mut found);
                found.push((number, Read::Unreadable));
                return found;
            }
            None if scanned.continued => {
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
    /// What is still open at the end: a quote (`'`, `"`, or `a` for `$'…'`),
    /// a pair of backticks (`` ` ``), a `$(` (`(`), a `${` (`{`), a comment
    /// (`#`), or nothing. A comment inside an open `$(` or `${` reports the
    /// `$(` or the `${`, because the shell reads on past the line.
    open: Option<char>,
    /// The byte offset of the first `<<` outside quotes and comments.
    heredoc: Option<usize>,
    /// Whether the command ends in a `\\` that escapes nothing, outside
    /// quotes and comments: the one thing that joins the next line.
    continued: bool,
}

/// Read one command as a shell splits it.
///
/// A backslash escapes the next character outside quotes, inside double
/// quotes, inside ANSI-C quotes (`$'…'`) and inside backticks, and is literal
/// inside single quotes. Inside backticks a quote opens nothing. A `$(`
/// outside quotes opens until its `)`, and a `${` until its `}`. A `#` at the
/// start of the command, or after a blank, `;`, `&` or `|`, opens a comment to
/// the end, so an operator, a quote or a `<<` in a comment is none of them.
/// After `)`, `<` or `>`, and anywhere inside a `${…}`, a `#` is part of a
/// word.
fn scan(command: &str) -> Scan<'_> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut quote: Option<char> = None;
    let mut depth = 0usize;
    let mut braces = 0usize;
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
        match (quote, c) {
            (Some('\'' | 'a'), '\'') | (Some('"'), '"') | (Some('`'), '`') => quote = None,
            (Some('"' | 'a' | '`') | None, '\\') => escaped = true,
            (Some(_), _) => {}
            (None, '$') if bytes.get(at + 1) == Some(&b'\'') => {
                quote = Some('a');
                at += 1;
            }
            (None, '$') if bytes.get(at + 1) == Some(&b'(') => {
                depth += 1;
                at += 1;
            }
            (None, ')') if depth > 0 => depth -= 1,
            (None, '$') if bytes.get(at + 1) == Some(&b'{') => {
                braces += 1;
                at += 1;
            }
            (None, '}') if braces > 0 => braces -= 1,
            (None, '\'' | '"' | '`') => quote = Some(c),
            (None, '#')
                if braces == 0
                    && (at == 0 || matches!(bytes[at - 1], b' ' | b'\t' | b';' | b'&' | b'|')) =>
            {
                out.push((start, &command[start..at]));
                return Scan {
                    segments: out,
                    open: Some(if depth > 0 { '(' } else { '#' }),
                    heredoc,
                    continued: false,
                };
            }
            (None, '<') if heredoc.is_none() && bytes.get(at + 1) == Some(&b'<') => {
                heredoc = Some(at);
                at += 1;
            }
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
    Scan {
        segments: out,
        open: quote
            .or((depth > 0).then_some('('))
            .or((braces > 0).then_some('{')),
        heredoc,
        continued: escaped && quote.is_none(),
    }
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
/// A `<<` inside quotes or in a comment opens none.
fn heredoc_end(command: &str) -> Option<String> {
    let at = scan(command).heredoc?;
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
            pairs(&[(0, "echo"), (0, UNREADABLE)])
        );
        assert_eq!(
            names("echo $(date\nnpm install\n", false),
            pairs(&[(0, "echo"), (0, UNREADABLE)])
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
            pairs(&[(0, "echo"), (0, "npm"), (1, "npm")])
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
            pairs(&[(0, "echo"), (0, "npm")])
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

    /// A quote inside `"$( )"` is read as a quote of its own, so the line is
    /// flagged rather than joined to the next.
    #[test]
    fn a_quote_inside_a_quoted_substitution_is_flagged() {
        assert_eq!(
            names("echo \"$(echo \"it's\")\" && npm install\nnpm ci\n", false),
            pairs(&[(0, "echo"), (0, UNREADABLE)])
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
            pairs(&[(0, UNREADABLE)])
        );
    }
}
