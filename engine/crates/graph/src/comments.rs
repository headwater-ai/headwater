// SPDX-License-Identifier: Apache-2.0
//! The comment bytes of a Rust source file, and nothing else.
//!
//! [`engine/crates/check/src/fragment.rs`](../../../../engine/crates/check/src/fragment.rs)
//! carried this walk first, as a `#[cfg(test)]`-only primitive that resolves a
//! comment's own Markdown links against the corpus slugger. [`CommentScan`](crate::anchors::CommentScan)
//! is the second reader, and it does not resolve a link: it looks for an
//! identifier-shaped token in the same bytes. `CLAUDE.md`'s rule that no
//! second copy of a rule lives in a script applies to code as much as to
//! prose, so the walk moved here, where both readers can call it, rather than
//! being hand-rolled a second time for a resolver that ships in the binary.
//!
//! # What a comment is, here
//!
//! A `//` inside a string literal is not a comment, and a raw string's
//! terminator is decided by its own hash count rather than by the first `"`
//! after it. So this walks the file byte by byte and tracks the four states
//! that can hide a `//`: a line comment, a block comment (which nests, as Rust
//! allows), a string that may span lines, and a raw string.

/// Where a comment sits in a Rust source file.
enum State {
    Code,
    Block(usize),
    Str,
    Raw(usize),
}

/// The comment text of a Rust file, one output line per input line.
///
/// Every non-comment byte becomes nothing, so a reported line number is the
/// line of the `.rs` file and needs no second mapping. The comment marker and
/// the indentation in front of it go too: a doc comment inside an `impl`
/// block is indented, and a Markdown reader takes four spaces for a code
/// block and stops parsing the links inside it. A resolver that instead scans
/// for a bare identifier token cares about neither, and reads the same output.
pub fn rust_comment_text(src: &str) -> String {
    let mut out = String::new();
    let mut state = State::Code;
    for line in src.lines() {
        let mut comment = String::new();
        let bytes: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < bytes.len() {
            match state {
                State::Block(depth) => {
                    if bytes[i..].starts_with(&['*', '/']) {
                        state = if depth == 1 {
                            State::Code
                        } else {
                            State::Block(depth - 1)
                        };
                        i += 2;
                    } else if bytes[i..].starts_with(&['/', '*']) {
                        state = State::Block(depth + 1);
                        comment.push_str("/*");
                        i += 2;
                    } else {
                        comment.push(bytes[i]);
                        i += 1;
                    }
                }
                State::Str => {
                    if bytes[i] == '\\' {
                        i += 2;
                    } else if bytes[i] == '"' {
                        state = State::Code;
                        i += 1;
                    } else {
                        i += 1;
                    }
                }
                State::Raw(hashes) => {
                    if bytes[i] == '"'
                        && bytes[i + 1..]
                            .iter()
                            .take(hashes)
                            .filter(|c| **c == '#')
                            .count()
                            == hashes
                    {
                        state = State::Code;
                        i += 1 + hashes;
                    } else {
                        i += 1;
                    }
                }
                State::Code => {
                    if bytes[i..].starts_with(&['/', '/']) {
                        comment.push_str(&bytes[i..].iter().collect::<String>());
                        break;
                    } else if bytes[i..].starts_with(&['/', '*']) {
                        state = State::Block(1);
                        i += 2;
                    } else if bytes[i] == '"' {
                        state = State::Str;
                        i += 1;
                    } else if bytes[i] == 'r'
                        && bytes[i + 1..]
                            .first()
                            .is_some_and(|c| *c == '"' || *c == '#')
                    {
                        let hashes = bytes[i + 1..].iter().take_while(|c| **c == '#').count();
                        if bytes.get(i + 1 + hashes) == Some(&'"') {
                            state = State::Raw(hashes);
                            i += 2 + hashes;
                        } else {
                            i += 1;
                        }
                    } else if bytes[i] == '\'' {
                        // A char literal, or a lifetime. Only the first can
                        // hold a quote, so only the first is consumed.
                        let closes = match bytes.get(i + 1) {
                            Some('\\') => bytes.get(i + 3) == Some(&'\''),
                            Some(_) => bytes.get(i + 2) == Some(&'\''),
                            None => false,
                        };
                        i += if closes {
                            if bytes.get(i + 1) == Some(&'\\') {
                                4
                            } else {
                                3
                            }
                        } else {
                            1
                        };
                    } else {
                        i += 1;
                    }
                }
            }
        }
        let text = comment.trim_start();
        let text = text
            .strip_prefix("///")
            .or_else(|| text.strip_prefix("//!"))
            .or_else(|| text.strip_prefix("//"))
            .unwrap_or(text);
        out.push_str(text.trim_start());
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::rust_comment_text;

    /// A string literal is not a comment, even where it holds `//`.
    ///
    /// `engine/crates/check/src/fragment.rs`'s `comment_links` module carries
    /// the rest of this walk's behavior, over the corpus this repository
    /// ships and over trees it writes for the purpose, and that coverage
    /// moved with the function rather than being repeated here.
    #[test]
    fn a_string_literal_hides_a_comment_marker() {
        let out = rust_comment_text("let s = \"[a](../../../docs/nope.md)\"; // and\n");
        assert_eq!(out, "and\n");
    }
}
