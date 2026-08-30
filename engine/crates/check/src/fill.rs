// SPDX-License-Identifier: Apache-2.0
//! Shared line wrapping for human-readable reports.
//!
//! A report is laid out at a fixed width and must not hand-wrap its own text.
//! The wrapping logic is shared so every text report uses the same width and
//! the same word-breaking rules.

/// The width every report this engine lays out at, in characters.
pub const WIDTH: usize = 80;

/// Wrap a sentence block to the report width, adding a leading indent to each
/// line.
pub fn filled(text: &str, indent: usize) -> String {
    let pad = " ".repeat(indent);
    let mut out = String::new();
    for paragraph in text.split('\n') {
        let mut line = pad.clone();
        let mut filled = false;
        for word in paragraph.split_whitespace() {
            match filled {
                false => {
                    line.push_str(word);
                    filled = true;
                }
                true => {
                    if line.chars().count() + 1 + word.chars().count() <= WIDTH {
                        line.push(' ');
                        line.push_str(word);
                    } else {
                        out.push_str(&line);
                        out.push('\n');
                        line = pad.clone();
                        line.push_str(word);
                    }
                }
            }
        }
        out.push_str(&line);
        out.push('\n');
    }
    out
}
