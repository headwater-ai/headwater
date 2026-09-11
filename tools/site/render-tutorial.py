#!/usr/bin/env python3
"""render-tutorial.py — write `site/tutorial/index.html` from
`docs/tutorials/your-first-governed-corpus.md`.

WHY THIS SCRIPT EXISTS

  Before this script, `site/tutorial/index.html` was typed by hand as a
  second copy of the tutorial, and nothing kept the two in step.
  `tools/site/refresh-figures.sh` already checked one direction of that drift —
  that every command or output block quoted on the page still appears in the
  document — but a step (or a paragraph inside one) added to the document
  with nothing added to the page passed that check silently, because the
  check has nothing to compare an ADDITION against. That is exactly what
  happened to the *Where to go next* section: `tools/headwater-bootstrap.sh`
  landed in the document and never reached the page.

  This script closes that direction too, by removing the second copy. The
  document is the only place tutorial prose is written by hand from here on;
  the page is this script's output, checked into git like any other
  generated file HW-DR-0037 admits (the "interpolating" form: a build runs on
  the author's side of the commit, and the committed bytes are what
  Cloudflare serves — the same move `tools/site/refresh-figures.sh` and
  `tools/site/refresh-site-tokens.sh` already make, and HW-DR-0039 records it).

WHAT THIS READS, AND THE RULE IT APPLIES TO EACH FENCED BLOCK

  The document's headings, prose paragraphs and fenced code blocks, walked
  one step at a time. Every split — into top-level `## ` sections, into
  `### Step N —` steps, into paragraphs — tracks fence state as it goes and
  never treats a line inside a fence as a heading, because step 8 quotes a
  scaffolded document whose own body carries `## Context`, `## Decision` and
  `## Consequences`, and a naive split on `\n## ` cuts the document there.

  A fence's role is read off the paragraph nearest above it, in this order:

    1. the fence shares its paragraph with the fence before it (nothing but
       a blank line stands between them)                    -> that fence's
       output, whatever role the fence before it carried
    2. the paragraph opens `**Check.**`                      -> a command's
       output. A one-line output under a `**Check.**` paragraph that ends in
       `:` is folded into that paragraph's own sentence instead of a block
       of its own, matching the one-line convention *Before you start*
       states.
    3. the paragraph opens `Trimmed`                          -> a command's
       output, and every fence in the run this starts carries a trimnote
    4. the paragraph contains "with these" or "with the" and ends in a
       number word plus "line:" or "lines:"                  -> the line(s)
       a reader is being told to write into a file, not output
    5. anything else, including no paragraph at all           -> a command

  This reproduces every block classification the hand-built page carried
  except one: step 5's `grep 'version:'` block, which the page styled as an
  edit block because it doubles as the line to type. Rule 2 reads it as
  output here instead, which keeps the check that produced it visible rather
  than treating a verification command as invisible scaffolding. Documented
  rather than special-cased, because a special case keyed to one step number
  is exactly the kind of hand-maintained fact this script exists to remove.

USAGE

  python3 tools/site/render-tutorial.py            write site/tutorial/index.html
  python3 tools/site/render-tutorial.py --check    write nothing; exit 1 if the
                                               page on disk would change

  Run `sh tools/site/refresh-figures.sh --check` afterward. It still holds the
  page to the document at the level it always has — every block on the page
  is a substring of the document — and now the two can never disagree,
  because the page has no content this script did not read out of the
  document.
"""
import html
import re
import sys
import pathlib

ROOT = pathlib.Path(__file__).resolve().parents[2]
DOC_PATH = ROOT / "docs/tutorials/your-first-governed-corpus.md"
PAGE_PATH = ROOT / "site/tutorial/index.html"

# The dot marker each step's gutter carries. Every other step gets the plain
# filled dot and no label. This table is the one fact about the page's
# reading, rather than its content, that the document states nowhere and so
# cannot be read out of it.
STEP_DOT = {4: (True, "refusal"), 5: (True, "refusal"),
            12: (False, "break"), 13: (False, "fix")}

NUM_WORDS = ["one", "two", "three", "four", "five", "six", "seven", "eight",
             "nine", "ten"]
EDIT_CUE = re.compile(
    r"with these|with the .*\b(%s) lines?:$" % "|".join(NUM_WORDS))


def esc(text):
    """Escape for HTML text content, leaving quotes alone — the convention
    the hand-built page already used throughout its `<pre>` blocks."""
    return html.escape(text, quote=False)


# --------------------------------------------------------------------------
# inline markdown -> HTML, restricted to the forms this document uses:
# `code`, **bold**, *em*, and [text](url). The link is kept as plain text:
# every occurrence in this document points at a spec part outside this
# script's job to re-derive a site URL for, and dropping the href rather
# than guessing one is the same refusal this repository states elsewhere
# against a hand-typed fact with no source.
def inline(text):
    escaped = esc(text)
    escaped = re.sub(r"`([^`]+)`", r"<code>\1</code>", escaped)
    escaped = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", escaped)
    escaped = re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", escaped)
    escaped = re.sub(r"(?<!\*)\*([^*]+)\*(?!\*)", r"<em>\1</em>", escaped)
    return escaped


def para_tag(text, cls=None):
    open_tag = '<p class="%s">' % cls if cls else "<p>"
    return open_tag + inline(text) + "</p>"


# --------------------------------------------------------------------------
# fence-aware structural splitting. Every function below tracks whether it
# is inside a ``` fence and never treats a line inside one as a heading or a
# paragraph boundary.
def strip_front_matter(text):
    lines = text.split("\n")
    if lines[0] != "---":
        sys.exit("render-tutorial: no front matter")
    end = lines.index("---", 1)
    return "\n".join(lines[end + 1:])


def top_level_sections(body):
    """`{heading text: body}` for every `## ` heading not inside a fence,
    plus the text before the first one."""
    lines = body.split("\n")
    sections, order = {}, []
    name, buf, inside = None, [], False
    intro = []
    for line in lines:
        if line.strip() == "```":
            inside = not inside
            (buf if name is not None else intro).append(line)
            continue
        if not inside and line.startswith("## "):
            if name is not None:
                sections[name] = "\n".join(buf)
            name = line[3:].strip()
            buf = []
            continue
        (buf if name is not None else intro).append(line)
    if name is not None:
        sections[name] = "\n".join(buf)
    return "\n".join(intro), sections


def split_steps(steps_body):
    """`[(number, title, body)]` for every `### Step N — Title` heading not
    inside a fence."""
    lines = steps_body.split("\n")
    steps, current, buf, inside = [], None, [], False
    for line in lines:
        if line.strip() == "```":
            inside = not inside
            if current is not None:
                buf.append(line)
            continue
        if not inside:
            m = re.match(r"^### Step (\d+) — (.+)$", line)
            if m:
                if current is not None:
                    steps.append((current[0], current[1], "\n".join(buf)))
                current = (int(m.group(1)), m.group(2).strip())
                buf = []
                continue
        if current is not None:
            buf.append(line)
    if current is not None:
        steps.append((current[0], current[1], "\n".join(buf)))
    return steps


class Fence:
    def __init__(self, text, paragraph, is_new_paragraph):
        self.text = text.rstrip("\n")
        self.paragraph = paragraph
        self.is_new_paragraph = is_new_paragraph
        self.role = None
        self.trimmed = False


def walk_fences(chunk):
    """Every fenced block in `chunk`, each with the paragraph nearest above
    it and whether that paragraph is new since the fence before it. Mirrors
    `.claude/tutorial/drive.py:read_blocks`, which the same document already
    answers to in CI."""
    lines = chunk.split("\n")
    fences = []
    current, inside, paragraph, fresh = [], False, "", True
    for line in lines:
        if line.strip() == "```":
            if inside:
                fences.append(Fence("\n".join(current), paragraph, fresh))
                current, fresh = [], False
            inside = not inside
            continue
        if inside:
            current.append(line)
            continue
        if line.strip():
            paragraph = line.strip()
            fresh = True
    return fences


def classify(fences):
    prev_trimmed = False
    for f in fences:
        p = f.paragraph.lower().lstrip("*")
        if not f.is_new_paragraph:
            f.role = "out"
            f.trimmed = prev_trimmed
        elif p.startswith("check."):
            f.role, f.trimmed = "out", False
        elif p.startswith("trimmed"):
            f.role, f.trimmed = "out", True
        elif EDIT_CUE.search(p):
            f.role, f.trimmed = "edit", False
        else:
            f.role, f.trimmed = "cmd", False
        prev_trimmed = f.trimmed


PROSE_STOP = ("check.", "trimmed")


def paragraphs(chunk, want):
    """Every paragraph in `chunk` outside a fence. `want` is `"prose"` (every
    paragraph that is not a `**Check.**` sentence and not a `Trimmed`
    lead-in) or `"check"` (only `**Check.**` sentences), in document
    order."""
    out, para, inside = [], [], False
    for line in chunk.split("\n") + [""]:
        if line.strip() == "```":
            inside = not inside
            continue
        if inside:
            continue
        if line.strip():
            para.append(line.strip())
            continue
        if not para:
            continue
        text = " ".join(para)
        para = []
        low = text.lower().lstrip("*")
        is_check = low.startswith("check.")
        if want == "check" and is_check:
            out.append(text)
        elif want == "prose" and not low.startswith(PROSE_STOP):
            out.append(text)
    return out


def fold_one_line_checks(check_sentences, fences):
    """`X prints:` followed by a one-line, non-trimmed output fence becomes
    `X prints Y.`, with no separate block for that fence. Every other check
    sentence, and every fence that does not qualify, is untouched."""
    out_fences = [f for f in fences if f.role == "out"]
    folded = set()
    rendered = []
    i = 0
    for sentence in check_sentences:
        if sentence.rstrip().endswith(":") and i < len(out_fences):
            f = out_fences[i]
            if "\n" not in f.text and not f.trimmed:
                sentence = sentence.rstrip()[:-1] + ", `%s`." % f.text
                folded.add(id(f))
                i += 1
                rendered.append(sentence)
                continue
        rendered.append(sentence)
    return rendered, folded


def blocklabel_for(role, line_count):
    if role == "edit":
        return "the line to write" if line_count == 1 else "the lines to write"
    if role == "out":
        return "what that prints"
    return None


def render_right_column(fences, folded):
    visible = [f for f in fences if id(f) not in folded]
    out = []
    for i, f in enumerate(visible):
        lines = f.text.split("\n")
        if len(lines) > 1:
            label = blocklabel_for(f.role, len(lines))
            if label:
                out.append('          <p class="blocklabel">%s</p>' % label)
        out.append('          <pre class="%s verbatim">%s</pre>'
                    % (f.role, esc(f.text)))
        # The trimnote closes a run of trimmed blocks right where the run
        # ends, rather than trailing every block in the step — a step can
        # run a command after a trimmed block (step 16 does), and the note
        # belongs to the trim, not to the step.
        at_run_end = f.trimmed and (
            i + 1 >= len(visible) or not visible[i + 1].trimmed)
        if at_run_end:
            out.append('          <p class="trimnote">Output trimmed to the '
                        "lines that matter — the full run prints more.</p>")
    return "\n".join(out)


def step_section(number, title, chunk):
    fences = walk_fences(chunk)
    classify(fences)
    checks = paragraphs(chunk, "check")
    checks, folded = fold_one_line_checks(checks, fences)
    prose = paragraphs(chunk, "prose")

    hollow, marker = STEP_DOT.get(number, (False, None))
    dot_span = '<span class="dot%s"></span>' % (" hollow" if hollow else "")
    label_span = ('<span class="marklabel">%s</span>' % marker
                  if marker else "")

    prose_html = "\n".join("        " + para_tag(p, "prose") for p in prose)
    check_html = ""
    if checks:
        pieces = [inline(re.sub(r"^\*\*Check\.\*\*\s*", "", c)) for c in checks]
        check_html = ('        <p class="check"><span class="checkword">'
                       'Check</span> <span>%s</span></p>' % " ".join(pieces))

    right = render_right_column(fences, folded)

    return """<section class="step">
  <div class="bar steprow">
    <div class="gutter" aria-hidden="true">
      %s
      %s
      <span class="stem"></span>
    </div>
    <div class="stepgrid">
      <div>
        <h2><span class="n">%02d</span>%s</h2>
%s
%s
      </div>
      <div>
%s
      </div>
    </div>
  </div>
</section>""" % (dot_span, label_span, number, inline(title), prose_html,
                  check_html, right)


def render_before_you_start(chunk):
    fences = walk_fences(chunk)
    classify(fences)
    cmd = next(f for f in fences if f.role == "cmd" and "\n" in f.text)
    checks = paragraphs(chunk, "check")
    check_html = ""
    if checks:
        body = re.sub(r"^\*\*Check\.\*\*\s*", "", checks[0])
        check_html = ('<p class="check"><span class="checkword">Check</span> '
                       "<span>%s</span></p>" % inline(body))
    return esc(cmd.text), check_html


def render_where_next(chunk):
    """Every `**Bold lead.**` paragraph in *Where to go next* becomes one
    sub-block, in document order, with whatever fenced commands sit under it
    before the next such paragraph — mechanically, rather than by naming the
    sub-blocks that existed when this script was written. That is what keeps
    a future addition here (this section is exactly where the document's own
    bootstrap addition landed with nothing on the page to receive it) from
    repeating the miss."""
    pieces = re.split(r"\n(?=\*\*[^*]+\*\*)", chunk.strip("\n"))
    blocks = []
    for piece in pieces:
        if not re.match(r"\*\*[^*]+\*\*", piece):
            continue
        fences = walk_fences(piece)
        classify(fences)
        text_paras = paragraphs(piece, "prose")
        html_out = ['    <p style="font-size: 1.05rem; max-width: 44rem;">'
                    + inline(text_paras[0]) + "</p>"] if text_paras else []
        for p in text_paras[1:]:
            html_out.append("    " + para_tag(p))
        for f in fences:
            if f.role in ("cmd", "out"):
                html_out.append('    <pre class="%s verbatim">%s</pre>'
                                 % (f.role, esc(f.text)))
        blocks.append("\n".join(html_out))
    return "\n\n".join(blocks)


def main():
    check_only = "--check" in sys.argv[1:]
    intro, sections = top_level_sections(strip_front_matter(DOC_PATH.read_text()))
    steps = split_steps(sections["Steps"])

    steps_html = "\n\n".join(step_section(n, t, c) for n, t, c in steps)
    where_next_html = render_where_next(sections["Where to go next"])

    if not PAGE_PATH.exists():
        sys.exit("render-tutorial: site/tutorial/index.html is not there")
    current = PAGE_PATH.read_text()

    # Only the step sections and the where-to-next section are regenerated;
    # the shell (head, nav, hero, before-you-start, glossary, footer) is
    # carried over from the file on disk untouched, so this script owns
    # exactly the two places the drift happened and no others.
    steps_start = current.index('<section class="step">')
    steps_end = current.index('<section class="tint">')
    next_marker = ('<section>\n  <div class="bar pad">\n    '
                   '<h2 class="label">Where to go next')
    next_start = current.index(next_marker)
    next_end = current.index("</section>\n\n<footer>")

    new = (current[:steps_start] + steps_html + "\n\n"
           + current[steps_end:next_start]
           + '<section>\n  <div class="bar pad">\n    '
             '<h2 class="label">Where to go next</h2>\n'
           + where_next_html + "\n  </div>\n"
           + current[next_end:])

    if check_only:
        if new != current:
            sys.stdout.write("render-tutorial: site/tutorial/index.html is stale\n")
            return 1
        print("render-tutorial: site/tutorial/index.html matches the document")
        return 0

    PAGE_PATH.write_text(new)
    print("render-tutorial: wrote site/tutorial/index.html")
    return 0


if __name__ == "__main__":
    sys.exit(main())
