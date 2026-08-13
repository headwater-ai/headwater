// SPDX-License-Identifier: Apache-2.0
//! The generated-file marker: one wording, one predicate, two readers.
//!
//! [Spec 6](../../../../docs/spec/06-engine-architecture.md#projections) asks a
//! projection to carry a marker so that "the engine refuses to overwrite a file
//! that lacks the marker and did not come from a previous run". That sentence
//! names one reader, the writer of a file. There is a second one, and it is the
//! census: a generated file that lands inside the corpus root is walked like any
//! other file, and the marker is what tells the walk that this engine wrote it.
//!
//! The two readers sit on opposite sides of the engine. `headwater-generate`
//! depends on `headwater-census`, so the census cannot depend on the generator,
//! and a copy of the rule in each crate is the drift that
//! [principle 2](../../../../docs/spec/00-vision-and-scope.md#design-principles)
//! rules against: the writer and the reader would then disagree about which
//! files this engine wrote, and the disagreement would show up as a generated
//! file reported against contracts it was never held to. So the rule lives
//! here, below both, in the way `headwater-hash` holds one digest for three
//! components that must not disagree about it.
//!
//! # What the marker is, and what it is not
//!
//! It is a **claim** that this engine wrote the file. It is not proof, because
//! anything can write the line. [`kind_named`] therefore reports what the marker
//! says rather than what is true, and nothing here validates the name against a
//! declared projection kind — this crate does not know the kinds.
//!
//! Testing the claim needs the plan, so it happens in the generator:
//! `headwater generate --check` holds every marked file inside the corpus root
//! against the declaration that writes it, and reports a marked file that no
//! declaration claims. That test is what keeps the marker from becoming a line
//! an author can add to exempt a document from every check.
//!
//! # Where the marker sits is the format's business
//!
//! A commented format carries it on the first line. JSON has no comment, so a
//! JSON output carries the same sentence in a top-level member.
//! [`carries_marker`] reads whichever of the two a path admits.
//!
//! # A Markdown document with front matter has a member surface too
//!
//! The first line of such a file is `---`, and a comment above the block would
//! put the block on line 2, where no front-matter parser looks for it. So a
//! generated Markdown file that declares an identity carries the marker the way
//! JSON does: as a top-level member of the block, under the quoted key
//! [`MARKER`].
//!
//! The false-positive guard survives, because the member is read **inside the
//! block and nowhere else**. A document that quotes the marker in its prose is
//! an authored document, and this repository's own specification is one. The
//! bounded region is what keeps the two apart, and it is the same bound the
//! first-line rule uses for a file with no block.

/// The word that marks a file as this engine's output.
pub const MARKER: &str = "headwater:generated";

/// The comment syntax a path's format admits, which is where its marker goes.
enum Comment {
    /// Markdown, so an HTML comment.
    Html,
    /// YAML and anything else line-oriented.
    Hash,
    /// A format with no comment syntax. JSON is the one that reaches here.
    None,
}

fn comment_for(path: &str) -> Comment {
    match path.rsplit('.').next() {
        Some("md") | Some("markdown") => Comment::Html,
        Some("yml") | Some("yaml") | Some("toml") => Comment::Hash,
        Some("json") => Comment::None,
        _ => Comment::Hash,
    }
}

/// What the marker says, without the word that names it and without the syntax
/// that carries it.
///
/// Held apart from [`marker`] because a format with no comment carries the same
/// sentence in a different place, and the two places name the marker
/// differently. A comment has one line, so [`MARKER`] has to be a word inside
/// it. A member has a key, so [`MARKER`] is the key and a value that repeated
/// it would say the word twice. One wording, two frames, and neither of them
/// holds a second copy of the other's part.
///
/// The kind arrives as a name rather than as a type, because this crate sits
/// below the crate that enumerates the kinds. The name is also what a reader of
/// the file sees, and what [`kind_named`] reads back out.
pub fn marker_text(kind: &str) -> String {
    format!(
        "{kind}. `headwater generate` writes this file, and `headwater generate --check` holds \
         it. Edit the corpus, not this file."
    )
}

/// The marker as a front-matter member, for a generated Markdown document that
/// declares an identity.
///
/// The key is quoted because [`MARKER`] holds a colon, and an unquoted key that
/// holds one is a key that YAML reads as two. The value is quoted for the same
/// class of reason: [`marker_text`] holds a period and a backtick, and a plain
/// scalar that opens with neither is still a scalar nobody should have to
/// reason about.
pub fn marker_member(kind: &str) -> String {
    format!("\"{MARKER}\": \"{}\"", marker_text(kind))
}

/// The marker line for a kind at a path, or `None` when the format carries no
/// comment.
///
/// `None` is not a refusal. It says that the marker cannot be a line here, and
/// the emitter for that format carries [`marker_text`] structurally instead:
/// JSON reaches this arm, and the descriptor writes the sentence into a
/// top-level member. What matters to [`carries_marker`] is that the marker is
/// findable, and not which syntax holds it.
pub fn marker(kind: &str, path: &str) -> Option<String> {
    let body = format!("{MARKER} {}", marker_text(kind));
    match comment_for(path) {
        Comment::Html => Some(format!("<!-- {body} -->")),
        Comment::Hash => Some(format!("# {body}")),
        Comment::None => None,
    }
}

/// Whether a file at a path marks itself as this engine's output.
///
/// Two rules, because the false positive the first one guards against exists in
/// only one of the two formats.
///
/// **A commented format: the first line, and nowhere else.** A document that
/// quotes the marker while discussing it is an authored document, and this
/// repository's own specification is exactly such a document. Reading the first
/// line alone is what keeps a run from overwriting it, and it is what keeps the
/// census from reporting spec 6 as a file this engine generated.
///
/// **JSON: a top-level member, wherever it sits.** A JSON file is not prose, so
/// nothing in one discusses a marker, and the line rule would answer for the
/// brace that opens the object rather than for the file. The member is the
/// marker, and it is matched as a quoted key at the start of a line so that a
/// string somewhere in the document which happens to hold the word does not
/// count as one.
///
/// **Markdown, additionally: a member of the front-matter block.** The same
/// rule as JSON, bounded to the block, because a Markdown body is prose and a
/// member rule over the whole file would read a quoted marker in a fenced
/// example as this engine's own output.
pub fn carries_marker(path: &str, text: &str) -> bool {
    marker_line(path, text).is_some()
}

/// The kind the marker at a path names, when it names one.
///
/// **What the file says about itself, and never what is true.** The marker is a
/// claim, this function reports the claim, and the caller that cares whether the
/// claim holds is the one holding the plan. A marked file whose name matches no
/// declared kind still reads back here, because the alternative is a census that
/// silently forgets a file said something.
///
/// `None` where the marker carries no name: a file whose first line is the bare
/// word, which is what a hand-written marker usually is.
pub fn kind_named(path: &str, text: &str) -> Option<String> {
    let line = marker_line(path, text)?;
    // The syntax that carries the marker comes off before the name is read.
    // A reader that skipped this step answers `-->` for a file whose first line
    // is the bare word, because the terminator is the next thing on the line
    // and it is not a period.
    // A member reads the same way wherever it sits, so the two formats that
    // admit one share the arm. What tells them apart is where `marker_line`
    // looked, and that is its business rather than this function's.
    let member = line.trim_start().starts_with(&quoted());
    let after = match comment_for(path) {
        // `"headwater:generated": "shelf_index. …` — step over the quote, the
        // colon and the quote that opens the value, then stop at the quote that
        // closes it.
        _ if member => line
            .split_once(&quoted())
            .map(|(_, rest)| rest.trim_start())?
            .strip_prefix(':')?
            .trim_start()
            .strip_prefix('"')?
            .split('"')
            .next()?,
        Comment::None => return None,
        Comment::Html => {
            let body = line.split_once(MARKER).map(|(_, rest)| rest)?.trim_end();
            body.strip_suffix("-->").unwrap_or(body)
        }
        Comment::Hash => line.split_once(MARKER).map(|(_, rest)| rest)?,
    };
    // The name runs to the first period, which is where `marker_text` ends it.
    let name = after.trim_start().split('.').next()?.trim();
    match name.is_empty() {
        true => None,
        false => Some(name.to_string()),
    }
}

/// The line that carries the marker, by the rule the path's format admits.
///
/// The one place that decides what "carries a marker" means. [`carries_marker`]
/// asks whether there is such a line and [`kind_named`] reads it, so the
/// predicate and the reader cannot come to different answers about one file.
fn marker_line<'a>(path: &str, text: &'a str) -> Option<&'a str> {
    match comment_for(path) {
        Comment::None => text
            .lines()
            .find(|line| line.trim_start().starts_with(&quoted())),
        // The first line, and then the block. The two are disjoint: a first
        // line that carries the marker is a first line that is not `---`, so
        // such a file has no block for the second rule to read.
        Comment::Html => text
            .lines()
            .next()
            .filter(|line| line.contains(MARKER))
            .or_else(|| front_matter(text).find(|line| line.trim_start().starts_with(&quoted()))),
        Comment::Hash => text.lines().next().filter(|line| line.contains(MARKER)),
    }
}

/// The lines between the fences of a front-matter block, and none when the text
/// opens with no block.
///
/// The opening fence is the first line and nothing else, which is the rule
/// [`headwater_doc`] splits on. The block ends at the next fence, so a `---`
/// inside the body is out of reach whether or not the block was ever closed.
fn front_matter(text: &str) -> impl Iterator<Item = &str> {
    let mut lines = text.lines();
    let opened = lines.next() == Some("---");
    lines.take_while(|line| *line != "---").take(match opened {
        true => usize::MAX,
        false => 0,
    })
}

fn quoted() -> String {
    format!("\"{MARKER}\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_commented_format_carries_the_marker_on_the_first_line_and_nowhere_else() {
        let first = format!("<!-- {MARKER} shelf_index. -->\n\n# An index\n");
        assert!(carries_marker("docs/spec/README.md", &first));

        // The false positive this rule exists to stop: an authored document
        // that quotes the marker while discussing it. This repository's own
        // specification is such a document.
        let quoting = format!("---\ntitle: Spec 6\n---\n\nA projection carries `{MARKER}`.\n");
        assert!(!carries_marker(
            "docs/spec/06-engine-architecture.md",
            &quoting
        ));
    }

    #[test]
    fn json_carries_the_marker_as_a_member_wherever_it_sits() {
        let text =
            format!("{{\n  \"version\": \"1.0\",\n  \"{MARKER}\": \"corpus_descriptor. \"\n}}\n");
        assert!(carries_marker(".headwater/corpus.json", &text));

        // A string that holds the word is not a member that is the marker.
        let mentions = format!("{{\n  \"note\": \"we write {MARKER} here\"\n}}\n");
        assert!(!carries_marker(".headwater/corpus.json", &mentions));
    }

    #[test]
    fn the_kind_reads_back_out_of_the_line_the_writer_wrote() {
        // The property that matters: what `marker` writes, `kind_named` reads,
        // for every format. A writer and a reader that agree by construction
        // are the whole reason this crate exists.
        for (kind, path) in [
            ("shelf_index", "docs/spec/README.md"),
            ("agent_rules", "docs/rules.yml"),
            ("site_nav", "docs/nav.toml"),
        ] {
            let line = marker(kind, path).expect("a commented format");
            assert_eq!(kind_named(path, &line).as_deref(), Some(kind));
        }
        let json = format!(
            "{{\n  \"{MARKER}\": \"{}\"\n}}\n",
            marker_text("graph_export")
        );
        assert_eq!(
            kind_named("docs/graph.json", &json).as_deref(),
            Some("graph_export")
        );
    }

    #[test]
    fn a_marker_that_names_nothing_reads_back_as_nothing() {
        // A hand-written marker is usually the bare word. It still marks the
        // file, and it names no kind, and those are two different answers.
        let bare = format!("<!-- {MARKER} -->\n");
        assert!(carries_marker("docs/spec/13-open-obligations.md", &bare));
        assert_eq!(kind_named("docs/spec/13-open-obligations.md", &bare), None);
    }

    #[test]
    fn a_markdown_document_carries_the_marker_inside_its_front_matter_block() {
        let generated = format!(
            "---\nid: REG-HW-open-questions\n{}\n---\n\n## Q1\n",
            marker_member("shelf_sections")
        );
        assert!(carries_marker("docs/spec/09-open-questions.md", &generated));
        assert_eq!(
            kind_named("docs/spec/09-open-questions.md", &generated).as_deref(),
            Some("shelf_sections")
        );

        // The guard the block rule has to keep: a member is a member of the
        // block, and a body that quotes one is prose. This repository's own
        // specification is a document that does exactly that.
        let quoting = format!(
            "---\nid: SPEC-HW-engine\n---\n\nA projection writes {}.\n",
            marker_member("shelf_sections")
        );
        assert!(!carries_marker(
            "docs/spec/06-engine-architecture.md",
            &quoting
        ));

        // And a file whose body opens a second `---` block does not reach the
        // rule either, because the block ends at the first fence after the
        // first line.
        let fenced = format!(
            "---\nid: SPEC-HW-engine\n---\n\n---\n{}\n---\n",
            marker_member("shelf_sections")
        );
        assert!(!carries_marker(
            "docs/spec/06-engine-architecture.md",
            &fenced
        ));
    }

    #[test]
    fn a_file_with_no_marker_names_no_kind() {
        assert!(!carries_marker("docs/spec/01-concepts.md", "# A title\n"));
        assert_eq!(kind_named("docs/spec/01-concepts.md", "# A title\n"), None);
    }
}
