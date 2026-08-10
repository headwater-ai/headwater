#!/usr/bin/env python3
"""Mechanical STE checks for the spec prose in this repo.

This is step 4 of the ste-editor skill ("mechanical check on the result") turned
into a script, plus the two repo rules from CLAUDE.md that are exact string
matches: no hard-wrapped Markdown, and no stock AI phrasing.

It checks the *house* profile by default: the structural rules (sentence and
paragraph limits, voice, verb forms) and spelling, but not the closed ASD-STE100
dictionary. Pass --profile strict to also check every word against
`approved-words.txt`; that profile is for procedures, not for descriptive specs.

Errors block a commit. Warnings are advisory: the passive/progressive/auxiliary
detectors are regex guesses and misfire often enough that blocking on them would
just teach everyone to pass --no-verify.

Existing violations are grandfathered through `.ste-lint-baseline.json`, keyed by
a hash of (path, rule, offending text). Edit the text and the hash stops matching,
so the sentence you touch is the sentence you have to fix.

Usage:
    tools/ste-lint.py                     # every file in scope
    tools/ste-lint.py docs/spec/01-*.md   # named files
    tools/ste-lint.py --staged            # staged content, for the pre-commit hook
    tools/ste-lint.py --update-baseline   # grandfather every current error
    tools/ste-lint.py --no-baseline       # show what the baseline is hiding
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

# --- scope -------------------------------------------------------------------
# Directories whose Markdown is held to the house profile. docs/reviews is
# excluded by CLAUDE.md: those are point-in-time records and stay as written.

SCOPE = ("docs/spec",)
EXCLUDE = ("docs/reviews", ".claude", "node_modules")

BASELINE_FILE = ".ste-lint-baseline.json"

# --- limits ------------------------------------------------------------------

MAX_SENTENCE_WORDS = 25  # rule 6.3, descriptive text
MAX_PARAGRAPH_SENTENCES = 6  # rule 6.6
MAX_HEADING_WORDS = 12

# --- vocabulary --------------------------------------------------------------

# -ise / -isation stems. American English wants -ize / -ization throughout.
_ISE_STEMS = """
apolog author capital categor central character critic custom decentral digit
emphas familiar final formal general global harmon industrial initial internation
ital item legal legitim local material maxim memor minim mobil modern moral nation
neutral normal optim organ penal person prior rational real recogn serial social
special stabil standard steril summar symbol synchron util verbal visual
""".split()

BRITISH = {}
for _stem in _ISE_STEMS:
    for _suffix, _american in (
        ("ise", "ize"),
        ("ises", "izes"),
        ("ised", "ized"),
        ("ising", "izing"),
        ("isation", "ization"),
        ("isations", "izations"),
    ):
        BRITISH[_stem + _suffix] = _stem + _american

BRITISH.update(
    {
        "analyse": "analyze", "analyses": "analyzes", "analysed": "analyzed",
        "analysing": "analyzing", "paralyse": "paralyze", "catalyse": "catalyze",
        "behaviour": "behavior", "behaviours": "behaviors",
        "behavioural": "behavioral", "behaviourally": "behaviorally",
        "colour": "color", "colours": "colors", "coloured": "colored",
        "favour": "favor", "favours": "favors", "favoured": "favored",
        "honour": "honor", "labour": "labor", "neighbour": "neighbor",
        "endeavour": "endeavor", "flavour": "flavor", "rumour": "rumor",
        "humour": "humor", "vapour": "vapor", "armour": "armor",
        "harbour": "harbor", "savour": "savor",
        "artefact": "artifact", "artefacts": "artifacts",
        "centre": "center", "centres": "centers", "centred": "centered",
        "fibre": "fiber", "litre": "liter", "metre": "meter", "metres": "meters",
        "theatre": "theater", "calibre": "caliber", "sombre": "somber",
        "defence": "defense", "offence": "offense", "pretence": "pretense",
        "licence": "license", "practise": "practice",
        "judgement": "judgment", "judgements": "judgments",
        "acknowledgement": "acknowledgment",
        "acknowledgements": "acknowledgments",
        "programme": "program", "programmes": "programs",
        "catalogue": "catalog", "catalogues": "catalogs",
        "catalogued": "cataloged", "analogue": "analog",
        "whilst": "while", "amongst": "among",
        "fulfil": "fulfill", "fulfils": "fulfills", "fulfilment": "fulfillment",
        "enrol": "enroll", "enrolment": "enrollment", "instalment": "installment",
        "sceptic": "skeptic", "sceptical": "skeptical", "scepticism": "skepticism",
        "modelling": "modeling", "modelled": "modeled",
        "labelling": "labeling", "labelled": "labeled",
        "travelling": "traveling", "travelled": "traveled",
        "cancelling": "canceling", "cancelled": "canceled",
        "signalling": "signaling", "signalled": "signaled",
        "levelled": "leveled", "totalled": "totaled", "fuelled": "fueled",
        "marvellous": "marvelous", "counsellor": "counselor",
        "focussed": "focused", "focussing": "focusing",
        "storey": "story", "tyre": "tire", "kerb": "curb", "plough": "plow",
        "draught": "draft", "cheque": "check", "aluminium": "aluminum",
        "grey": "gray",
    }
)

# CLAUDE.md: "Avoid stock AI phrasing". Exact matches only.
STOCK_PHRASES = {
    r"load[- ]bearing": "name what depends on it",
    r"first[- ]class(?:\s+citizens?)?": "say what it actually gets",
    r"battle[- ]tested": "say where it has run and for how long",
    r"north star": "name the goal",
    r"\bdelve\b": "examine, study, read",
    r"\bseamless(?:ly)?\b": "say what the joins would otherwise cost",
    r"\bholistic(?:ally)?\b": "name the parts it covers",
    r"deep dive": "detailed review",
    r"(?<![\w-])leverag(?:e|es|ed|ing)\b(?!-)": "use, apply",
    r"\brobust\b": "name the failure it survives",
}

CONTRACTIONS = re.compile(
    r"\b\w+n[’']t\b"
    r"|\b\w+[’'](?:re|ve|ll|d|m)\b"
    r"|\b(?:it|that|there|here|what|who|let|he|she|which|this)[’']s\b",
    re.IGNORECASE,
)

IRREGULAR_PARTICIPLES = (
    "known|written|given|shown|taken|made|held|built|set|kept|read|meant|sent|"
    "drawn|seen|done|found|left|put|split|cast|bound|brought|caught|chosen|"
    "driven|forgotten|got|gotten|kept|lost|meant|met|paid|run|said|sold|spent|"
    "told|thought|understood|drawn"
)

PROGRESSIVE = re.compile(
    r"\b(is|are|was|were|be|been|being|am)\s+(?:\w+ly\s+)?(\w+ing)\b", re.IGNORECASE
)
PASSIVE = re.compile(
    r"\b(is|are|was|were|be|been|being|am)\s+(?:\w+ly\s+)?"
    r"(\w+ed|" + IRREGULAR_PARTICIPLES + r")\b",
    re.IGNORECASE,
)
AUXILIARY = re.compile(
    r"\b(must|can|could|should|shall|may|might|will|would)\s+(?:not\s+)?be\b",
    re.IGNORECASE,
)

ABBREVIATIONS = (
    "e.g.", "i.e.", "cf.", "etc.", "vs.", "al.", "approx.", "Fig.", "No.",
    "Sec.", "Dr.", "Mr.", "Ms.", "St.", "Inc.", "Ltd.",
)

SEVERITY_ERROR = "error"
SEVERITY_WARN = "warn"

RULES = {
    "hard-wrap": SEVERITY_ERROR,
    "semicolon": SEVERITY_ERROR,
    "contraction": SEVERITY_ERROR,
    "british-spelling": SEVERITY_ERROR,
    "stock-phrase": SEVERITY_ERROR,
    "sentence-length": SEVERITY_ERROR,
    "heading-length": SEVERITY_WARN,
    "paragraph-sentences": SEVERITY_WARN,
    "progressive": SEVERITY_WARN,
    "passive": SEVERITY_WARN,
    "auxiliary": SEVERITY_WARN,
    "vocabulary": SEVERITY_WARN,
}


@dataclass
class Finding:
    path: str
    line: int
    rule: str
    message: str
    excerpt: str

    @property
    def severity(self) -> str:
        return RULES[self.rule]

    def key(self) -> str:
        raw = f"{self.path}\0{self.rule}\0{normalize_for_hash(self.excerpt)}"
        return hashlib.sha1(raw.encode("utf-8")).hexdigest()[:16]

    def format(self) -> str:
        return f"{self.path}:{self.line}: [{self.severity}] {self.rule}: {self.message}"


@dataclass
class Unit:
    """One logical line of prose: a paragraph, a list item, a heading, a table row."""

    line: int
    kind: str  # prose | heading | table
    text: str
    allows: set = field(default_factory=set)


def normalize_for_hash(text: str) -> str:
    return re.sub(r"\s+", " ", text).strip().lower()


# --- Markdown parsing --------------------------------------------------------

# <!-- ste-lint: allow sentence-length, passive # optional reason after the hash -->
ALLOW_COMMENT = re.compile(r"<!--\s*ste-lint:\s*allow\s+([a-z][a-z,\s-]*?)\s*(?:#[^>]*)?-->")
FENCE = re.compile(r"^\s{0,3}(```+|~~~+)")
HEADING = re.compile(r"^\s{0,3}#{1,6}\s")
TABLE_ROW = re.compile(r"^\s*\|")
HRULE = re.compile(r"^\s{0,3}(-{3,}|\*{3,}|_{3,})\s*$")
LIST_ITEM = re.compile(r"^(\s*)([-*+]|\d{1,3}[.)])\s+")
BLOCKQUOTE = re.compile(r"^\s{0,3}(>+)\s?")
HTML_BLOCK = re.compile(r"^\s{0,3}<[a-zA-Z!/]")


def parse(lines: list[str]) -> tuple[list[Unit], list[tuple[int, str]]]:
    """Return (units, hard_wraps). Code, front matter, and HTML blocks are dropped."""
    units: list[Unit] = []
    hard_wraps: list[tuple[int, str]] = []

    in_fence = False
    fence_marker = ""
    in_indented_code = False
    prev_blank = True
    prev_prose_line = 0  # line number of the previous prose line, 0 if none
    prev_ends_backslash = False
    i = 0

    # YAML front matter
    if lines and lines[0].strip() == "---":
        for j in range(1, len(lines)):
            if lines[j].strip() in ("---", "..."):
                i = j + 1
                break

    while i < len(lines):
        raw = lines[i].rstrip("\n")
        lineno = i + 1
        i += 1

        fence_match = FENCE.match(raw)
        if fence_match:
            marker = fence_match.group(1)
            if not in_fence:
                in_fence, fence_marker = True, marker[:3]
            elif marker.startswith(fence_marker):
                in_fence = False
            prev_blank, prev_prose_line = False, 0
            continue
        if in_fence:
            continue

        blank = not raw.strip()
        if blank:
            prev_blank, prev_prose_line, in_indented_code = True, 0, False
            prev_ends_backslash = False
            continue

        # Indented code: an indented block that starts after a blank line and is
        # not the continuation of a list item.
        if in_indented_code:
            if re.match(r"^(\t| {4,})", raw):
                continue
            in_indented_code = False
        if prev_blank and re.match(r"^(\t| {4,})", raw):
            in_indented_code = True
            continue

        if HRULE.match(raw) or HTML_BLOCK.match(raw):
            prev_blank, prev_prose_line = False, 0
            continue

        # A blockquote marker is structure, not prose. Strip it and let the
        # paragraph rules apply inside the quote; a bare ">" is a blank line.
        quote = BLOCKQUOTE.match(raw)
        if quote:
            stripped = raw[quote.end():]
            if not stripped.strip():
                prev_blank, prev_prose_line = True, 0
                continue
            raw = stripped

        allows = set()
        allow_match = ALLOW_COMMENT.search(raw)
        if allow_match:
            allows = {r.strip() for r in allow_match.group(1).split(",") if r.strip()}

        if HEADING.match(raw):
            units.append(Unit(lineno, "heading", raw.lstrip("# \t"), allows))
            prev_blank, prev_prose_line = False, 0
            continue

        if TABLE_ROW.match(raw):
            if not re.fullmatch(r"[\s|:\-]+", raw):
                units.append(Unit(lineno, "table", raw.strip().strip("|"), allows))
            prev_blank, prev_prose_line = False, 0
            continue

        list_match = LIST_ITEM.match(raw)
        starts_block = bool(list_match)
        text = raw[list_match.end():] if list_match else raw.strip()

        # A prose line that directly follows another prose line, and does not
        # start a new block, is a wrapped continuation. Repo rule: never wrap.
        if prev_prose_line and not starts_block and not prev_ends_backslash:
            hard_wraps.append((lineno, text[:60]))
            units[-1].text += " " + text
            units[-1].allows |= allows
        else:
            units.append(Unit(lineno, "prose", text, allows))

        prev_ends_backslash = raw.rstrip().endswith("\\")
        prev_blank, prev_prose_line = False, lineno

    return units, hard_wraps


# --- text normalization ------------------------------------------------------

def strip_markup(text: str) -> str:
    """Reduce Markdown to countable words. Code and links become single tokens."""
    text = re.sub(r"<!--.*?-->", " ", text)
    text = re.sub(r"`[^`]*`", " CODE ", text)
    text = re.sub(r"!\[[^\]]*\]\([^)]*\)", " ", text)
    text = re.sub(r"\[([^\]]*)\]\([^)]*\)", r"\1", text)
    text = re.sub(r"\[([^\]]*)\]\[[^\]]*\]", r"\1", text)
    text = re.sub(r"<https?://[^>]*>", " URL ", text)
    text = re.sub(r"https?://\S+", " URL ", text)
    text = re.sub(r"&[a-zA-Z#0-9]+;", " ", text)
    text = text.replace("**", "").replace("__", "")
    text = re.sub(r"(?<![\w*])\*(?!\s)([^*]*?)(?<!\s)\*(?![\w*])", r"\1", text)
    text = re.sub(r"(?<![\w_])_(?!\s)([^_]*?)(?<!\s)_(?![\w_])", r"\1", text)
    return text.strip()


def protect(text: str) -> str:
    for abbr in ABBREVIATIONS:
        text = re.sub(re.escape(abbr), abbr.replace(".", "\u0001"), text, flags=re.IGNORECASE)
    text = re.sub(r"(\d)\.(\d)", "\\1\u0001\\2", text)
    text = re.sub(r"\b([A-Z])\.", "\\1\u0001", text)
    # A parenthetical counts as one word (rule 6.3 help), and never splits a sentence.
    for _ in range(4):
        new = re.sub(r"\([^()]*\)", " PARENTHETICAL ", text)
        if new == text:
            break
        text = new
    return text


def restore(text: str) -> str:
    return text.replace("\u0001", ".")


def split_sentences(text: str) -> list[str]:
    protected = protect(text)
    parts = re.split(r'(?<=[.!?])["”\'’)\]]*\s+(?=[A-Z0-9“"\[(`*_])', protected)
    return [restore(p).strip() for p in parts if p.strip()]


def count_words(sentence: str) -> int:
    tokens = [t for t in re.split(r"\s+", sentence) if t]
    return sum(1 for t in tokens if re.search(r"[\w\u0001]", t))


# --- checks ------------------------------------------------------------------

QUOTED = re.compile(r'"[^"]{0,400}"|“[^”]{0,400}”')


def check_words(path: str, unit: Unit, text: str, findings: list[Finding]) -> None:
    # A direct quotation must stay verbatim, so spelling and house-style checks
    # skip quoted spans. Sentence limits still apply to the line that carries them.
    unquoted = QUOTED.sub(" ", text)

    for match in re.finditer(r"[A-Za-z][A-Za-z’'-]*", unquoted):
        word = match.group(0)
        american = BRITISH.get(word.lower())
        if american:
            findings.append(
                Finding(path, unit.line, "british-spelling",
                        f'"{word}" is British spelling; use "{american}" (rule 1.14)',
                        word.lower())
            )

    for match in CONTRACTIONS.finditer(text):
        findings.append(
            Finding(path, unit.line, "contraction",
                    f'"{match.group(0)}" is a contraction; write it out (rule 2.5)',
                    match.group(0).lower())
        )

    for pattern, advice in STOCK_PHRASES.items():
        for match in re.finditer(pattern, unquoted, re.IGNORECASE):
            findings.append(
                Finding(path, unit.line, "stock-phrase",
                        f'"{match.group(0)}" is stock AI phrasing (CLAUDE.md); {advice}',
                        match.group(0).lower())
            )

    # Rule 8.1 is about sentences. A semicolon separating items in a table cell,
    # a heading, or a bibliographic parenthetical is punctuation, not a run-on.
    if unit.kind == "prose" and ";" in protect(text):
        findings.append(
            Finding(path, unit.line, "semicolon",
                    "semicolon; split the sentence in two (rule 8.1)",
                    text[:80])
        )


def check_prose(path: str, unit: Unit, text: str, findings: list[Finding]) -> None:
    sentences = split_sentences(text)
    if len(sentences) > MAX_PARAGRAPH_SENTENCES:
        findings.append(
            Finding(path, unit.line, "paragraph-sentences",
                    f"{len(sentences)} sentences in one paragraph, limit is "
                    f"{MAX_PARAGRAPH_SENTENCES} (rule 6.6)",
                    text[:80])
        )

    for sentence in sentences:
        words = count_words(sentence)
        if words > MAX_SENTENCE_WORDS:
            findings.append(
                Finding(path, unit.line, "sentence-length",
                        f"{words} words, limit is {MAX_SENTENCE_WORDS} (rule 6.3): "
                        f"{sentence[:70]}…",
                        sentence)
            )
        for pattern, rule, note in (
            (PROGRESSIVE, "progressive", 'no "-ing" verb forms (rule 3.4)'),
            (PASSIVE, "passive", "use the active voice (rules 3.1-3.3)"),
            (AUXILIARY, "auxiliary", "no auxiliary-verb construction (rule 3.6)"),
        ):
            match = pattern.search(sentence)
            if match:
                findings.append(
                    Finding(path, unit.line, rule,
                            f'"{match.group(0)}" — {note}', match.group(0).lower())
                )


def check_vocabulary(path: str, unit: Unit, text: str, approved: dict,
                     substitutions: dict, findings: list[Finding]) -> None:
    for match in re.finditer(r"[A-Za-z][a-z’'-]{2,}", text):
        word = match.group(0).lower()
        if word in approved:
            continue
        alternative = substitutions.get(word)
        if alternative:
            findings.append(
                Finding(path, unit.line, "vocabulary",
                        f'"{word}" is not approved; use {alternative} (rule 1.1)',
                        word)
            )


def load_references() -> tuple[dict, dict]:
    base = Path(__file__).resolve().parent.parent / ".claude/skills/ste-editor/references"
    approved, substitutions = {}, {}
    approved_file = base / "approved-words.txt"
    if approved_file.exists():
        for line in approved_file.read_text(encoding="utf-8").splitlines():
            match = re.match(r"^([A-Za-z’'-]+)\s*\(([^)]*)\)", line.strip())
            if match:
                approved[match.group(1).lower()] = match.group(2)
    subs_file = base / "word-substitutions.tsv"
    if subs_file.exists():
        for line in subs_file.read_text(encoding="utf-8").splitlines():
            parts = line.split("\t")
            if len(parts) >= 2 and parts[1].strip():
                word = re.sub(r"\s*\(.*", "", parts[0]).strip().lower()
                substitutions[word] = parts[1].strip()
    return approved, substitutions


def lint(path: str, content: str, profile: str) -> list[Finding]:
    findings: list[Finding] = []
    units, hard_wraps = parse(content.splitlines())

    for line, excerpt in hard_wraps:
        findings.append(
            Finding(path, line, "hard-wrap",
                    "wrapped continuation line; one logical line per paragraph, "
                    "list item, or blockquote paragraph (CLAUDE.md)",
                    excerpt)
        )

    references = load_references() if profile == "strict" else ({}, {})

    for unit in units:
        text = strip_markup(unit.text)
        if not text:
            continue
        unit_findings: list[Finding] = []
        check_words(path, unit, text, unit_findings)
        if unit.kind == "prose":
            check_prose(path, unit, text, unit_findings)
            if profile == "strict":
                check_vocabulary(path, unit, text, *references, findings=unit_findings)
        elif unit.kind == "heading":
            words = count_words(text)
            if words > MAX_HEADING_WORDS:
                unit_findings.append(
                    Finding(path, unit.line, "heading-length",
                            f"{words}-word heading; keep headings short", text)
                )
        findings.extend(f for f in unit_findings if f.rule not in unit.allows)

    findings.sort(key=lambda f: (f.line, f.rule))
    return findings


# --- file selection ----------------------------------------------------------

def repo_root() -> Path:
    out = subprocess.run(["git", "rev-parse", "--show-toplevel"],
                         capture_output=True, text=True, check=True)
    return Path(out.stdout.strip())


def in_scope(rel: str) -> bool:
    if not rel.endswith(".md"):
        return False
    if any(rel == e or rel.startswith(e.rstrip("/") + "/") for e in EXCLUDE):
        return False
    return any(rel == s or rel.startswith(s.rstrip("/") + "/") for s in SCOPE)


def all_in_scope(root: Path) -> list[str]:
    out = subprocess.run(["git", "ls-files", "-z", "--", *SCOPE],
                         capture_output=True, text=True, cwd=root, check=True)
    return sorted(p for p in out.stdout.split("\0") if p and in_scope(p))


def staged_files(root: Path) -> list[str]:
    out = subprocess.run(
        ["git", "diff", "--cached", "--name-only", "-z", "--diff-filter=ACM"],
        capture_output=True, text=True, cwd=root, check=True)
    return sorted(p for p in out.stdout.split("\0") if p and in_scope(p))


def staged_content(root: Path, rel: str) -> str:
    out = subprocess.run(["git", "show", f":{rel}"],
                         capture_output=True, text=True, cwd=root, check=True)
    return out.stdout


# --- baseline ----------------------------------------------------------------

def load_baseline(root: Path) -> dict:
    path = root / BASELINE_FILE
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8")).get("entries", {})


def write_baseline(root: Path, findings: list[Finding]) -> None:
    entries = {
        f.key(): f"{f.path}: {f.rule}: {f.excerpt[:100]}"
        for f in findings if f.severity == SEVERITY_ERROR
    }
    payload = {
        "comment": "Pre-existing STE violations, grandfathered. Regenerate with "
                   "tools/ste-lint.py --update-baseline. Editing the offending text "
                   "invalidates its entry, so touched prose must be fixed.",
        "version": 1,
        "entries": dict(sorted(entries.items())),
    }
    (root / BASELINE_FILE).write_text(json.dumps(payload, indent=2) + "\n",
                                      encoding="utf-8")


# --- main --------------------------------------------------------------------

def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("files", nargs="*", help="files to lint (default: all in scope)")
    parser.add_argument("--staged", action="store_true",
                        help="lint staged content instead of the working tree")
    parser.add_argument("--profile", choices=("house", "strict"), default="house")
    parser.add_argument("--no-baseline", action="store_true",
                        help="report grandfathered violations too")
    parser.add_argument("--update-baseline", action="store_true",
                        help="rewrite the baseline from the current errors")
    parser.add_argument("--warnings-as-errors", action="store_true")
    parser.add_argument("--errors-only", action="store_true",
                        help="list only errors; warnings survive as a count "
                             "(what the hooks use, so advisory output stays quiet)")
    parser.add_argument("--quiet", action="store_true",
                        help="print nothing when there is nothing to report")
    args = parser.parse_args(argv)

    root = repo_root()

    if args.staged:
        targets = [(p, staged_content(root, p)) for p in staged_files(root)]
    elif args.files:
        targets = []
        for name in args.files:
            path = Path(name).resolve()
            try:
                rel = str(path.relative_to(root))
            except ValueError:
                continue
            if in_scope(rel) and path.exists():
                targets.append((rel, path.read_text(encoding="utf-8")))
    else:
        targets = [(p, (root / p).read_text(encoding="utf-8")) for p in all_in_scope(root)]

    findings: list[Finding] = []
    for rel, content in targets:
        findings.extend(lint(rel, content, args.profile))

    if args.update_baseline:
        write_baseline(root, findings)
        count = sum(1 for f in findings if f.severity == SEVERITY_ERROR)
        print(f"baseline written: {count} grandfathered error(s) across "
              f"{len(targets)} file(s)")
        return 0

    baseline = {} if args.no_baseline else load_baseline(root)
    reported, suppressed = [], 0
    for finding in findings:
        if finding.severity == SEVERITY_ERROR and finding.key() in baseline:
            suppressed += 1
            continue
        reported.append(finding)

    errors = [f for f in reported if f.severity == SEVERITY_ERROR]
    warnings = [f for f in reported if f.severity == SEVERITY_WARN]
    failing = errors + (warnings if args.warnings_as_errors else [])

    hide_warnings = args.errors_only and not args.warnings_as_errors
    listed = errors if hide_warnings else reported
    if listed or not args.quiet:
        for finding in listed:
            print(finding.format())
        sys.stdout.flush()  # keep the summary after the findings when streams differ
        summary = (f"{len(errors)} error(s), {len(warnings)} warning(s) "
                   f"in {len(targets)} file(s)")
        if suppressed:
            summary += f"; {suppressed} grandfathered"
        print(summary, file=sys.stderr if failing else sys.stdout)

    return 1 if failing else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
