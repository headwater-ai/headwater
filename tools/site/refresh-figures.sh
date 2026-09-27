#!/bin/sh
# refresh-figures.sh — measure this repository, and fill the figures of the
# hand-built pages in an assembled copy of `site/` when the site is published.
#
# WHAT THIS MEASURES, AND FROM WHERE
#
#   Every figure this script writes comes from one of five sources, and each
#   one is a run rather than a memory:
#
#     1. `headwater check --root . --json`      the census, the findings, the
#                                               rules wired, the taxonomy and
#                                               the clock
#     2. `headwater check --root .`             the census breakdown that the
#                                               JSON does not carry (untyped,
#                                               excluded, not a document) and
#                                               the obligation register
#     3. `docs/interfaces/README.md`            the verb rows and the group
#                                               headings. That file is a generated
#                                               projection (`verb_index`), and
#                                               `headwater generate --check`
#                                               holds it, so it is a run too
#     4. `engine/crates/generate/src/profile.rs` the emitter split, read off the
#                                               `Emitter` enum and `is_built`
#     5. `headwater conformance --root .`       the level this repository reaches
#
#   Nothing here is typed by a person. A figure appears in the HTML inside a
#   `<span data-figure="KEY">` element, and this script rewrites the text of
#   every such element from the measurement above. A key with no measurement is
#   an error, and a measurement that reaches no page is an error too. The
#   second half of that is the denominator: a renamed marker, a moved page or
#   an empty `site/` leaves every figure unused, and a check with no
#   denominator passes over a site that proves nothing.
#   `tools/site/refresh-site-tokens.sh` refuses an unmarked page for the same
#   reason, and HW-DR-0050 rules it: a page that opts out of the register is
#   refused rather than skipped.
#
#   WHAT THAT DENOMINATOR STILL DOES NOT CATCH, measured on 2026-09-06:
#
#     `used` is a union across pages, so a figure lost on ONE page is invisible
#     while any other page still carries it. Renaming the marker attribute on
#     `site/how-it-works/index.html` alone gives exit 0 at `34 used across 8
#     pages`; renaming it on all eight refuses. Five of the eight
#     pages carry no figure at all, and nothing here notices which page holds
#     which key. The fix is a declaration of what each page owes, which this
#     script has nowhere to read.
#
#     The element pattern requires the text to hold no `<`, so a figure with a
#     nested child element is not matched by it. `<b data-figure="rules.wired"
#     ><b>99</b></b>` once served 99 where the run said 29, at exit 0. An
#     off-shape key such as `rules.wired2` matched nothing for the same reason.
#     Since #1273 the count of `data-figure` occurrences on a page is held
#     against the count the pattern matched, and a difference fails.
#
# WHEN A FIGURE IS MEASURED (#1273)
#
#   A figure is measured when the site is published, and never committed. The
#   pages under `site/` carry each `data-figure` element empty. The CI deploy
#   on a push to `main` assembles the site into `.headwater/site-deploy`, and
#   `--into` fills the copies there from a run of the engine at that commit.
#   So no pull request that adds a document moves a byte under `site/`, and
#   two such pull requests no longer conflict on the pages.
#
#   Until #1273 this script wrote the figures into the committed pages, each
#   page held a fold over the corpus, and `.gitattributes` declared the three
#   pages that carried one `-merge`. Every pull request conflicted with every
#   other one on those pages, so the merge queue landed them one at a time.
#   The decision that supersedes HW-DR-0039 records the move.
#
#   There is no clock partition any more. The eight figures that read the
#   clock (`run.date` and the seven that read the findings list) were compared
#   against a committed value and excused where only the clock moved. Nothing
#   committed is compared against a run now, so that machinery went too.
#
# USAGE
#
#   sh tools/site/refresh-figures.sh --check       measure, write nothing, and
#                                                    exit 1 on a committed value,
#                                                    an unknown key, a figure on
#                                                    no page, or tutorial or
#                                                    landing drift
#   sh tools/site/refresh-figures.sh --into <dir>  measure, and fill the pages
#                                                    under <dir>, never site/.
#                                                    It fails as --check does,
#                                                    less the committed value
#   sh tools/site/refresh-figures.sh --blank       empty every figure under
#                                                    site/. It measures nothing
#                                                    and needs no engine. After
#                                                    a conflict on a page, take
#                                                    either side's prose, then
#                                                    run this
#   sh tools/site/refresh-figures.sh --print       measure, and print the table
#
#   `tools/site/deploy-site.sh` runs `--into`, and CI runs `--check` and
#   `--into` on every event. `tools/site/figures-fixtures.sh` holds both.
#
# EXIT STATUS
#
#   0  the pages carry the figures this mode asks for
#   1  a committed value, an unknown key, a figure on no page, a marker this
#      cannot fill, or drift
#   2  no engine of either profile is built, or the arguments are wrong
#   3  the engine is older than the engine sources, so this cannot measure. It
#      writes nothing in any mode, and it names the build command rather than
#      itself.
#
set -eu

MODE=
INTO=
case "${1:-}" in
  --check) MODE=check ;;
  --print) MODE=print ;;
  --blank) MODE=blank ;;
  --into)
    MODE=into
    INTO=${2:-}
    if [ -z "$INTO" ]; then
      echo "refresh-figures.sh: --into needs the assembled directory to fill" >&2
      exit 2
    fi
    ;;
  "")
    echo "refresh-figures.sh: name a mode: --check, --into <dir>, --blank or --print" >&2
    echo "  The committed pages carry no measured figure since #1273, so there is" >&2
    echo "  no mode that writes one into site/." >&2
    exit 2
    ;;
  *) echo "refresh-figures.sh: unknown argument '$1'" >&2; exit 2 ;;
esac

ROOT=$(cd "$(dirname "$0")/../.." && pwd)
if [ "$MODE" = into ]; then
  case "$INTO" in
    /*) ;;
    *) INTO="$PWD/$INTO" ;;
  esac
  if [ ! -d "$INTO" ]; then
    echo "refresh-figures.sh: $INTO is not a directory" >&2
    exit 2
  fi
fi
cd "$ROOT"

# `--blank` measures nothing, so it needs no engine. It is the one command that
# resolves a conflict on a page: take either side's prose, then blank.
if [ "$MODE" = blank ]; then
  python3 - <<'PY'
import pathlib, re

root = pathlib.Path.cwd()
pattern = re.compile(
    r'(<(\w+)\b[^>]*\bdata-figure="[^"]*"[^>]*>)([^<]*)(</\2>)')
blanked = 0
for page in sorted(root.glob("site/**/*.html")):
    before = page.read_text()
    after, n = pattern.subn(lambda m: m.group(1) + m.group(4), before)
    if after != before:
        page.write_text(after)
        blanked += 1
    if before.count("data-figure=") != n:
        raise SystemExit("refresh-figures.sh: %s carries a data-figure element "
                         "that holds another element, which this cannot blank"
                         % page.relative_to(root))
print("blanked the figures on %d page%s" % (blanked, "" if blanked == 1 else "s"))
PY
  exit 0
fi

# Either profile builds the engine these figures are measured with, and the
# newer answers. `.githooks/pre-commit` runs this script, so a root that reads
# one path and no other refuses every commit on a checkout built the other way
# rather than merely declining to measure.
HW_RELEASE="$ROOT/engine/target/release/headwater"
HW_DEV_RELEASE="$ROOT/engine/target/dev-release/headwater"
HW="$HW_RELEASE"
if [ -x "$HW_DEV_RELEASE" ] && { [ ! -x "$HW" ] || [ "$HW_DEV_RELEASE" -nt "$HW" ]; }; then
  HW="$HW_DEV_RELEASE"
fi
if [ ! -x "$HW" ]; then
  echo "refresh-figures.sh: no engine of either profile under $ROOT/engine/target" >&2
  echo "  build it: cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked" >&2
  exit 2
fi

# The binary must not be behind the engine sources beside it. Nothing else in
# this tree asks that question, and the failure it lets through is the one this
# script is least able to survive: a binary that predates a rule cannot see the
# rule, counts one fewer wired rule than the page correctly states, and calls
# every figure that reads the findings stale. The page is right and the report
# is wrong. In `write` mode the script then puts the wrong number on the page.
# Measured on 6c12f87: a binary a few hours old reported 12 stale figures over
# 3 pages, and a rebuild at that same commit reported none.
#
# The guard sits above the mode split on purpose, so `--check`, `--print` and
# the writing path cannot disagree about whether a run is trustworthy. It asks
# the same mtime question `cargo` asks, so a tree that `cargo build` calls fresh
# always passes here. `engine/target` is pruned: it holds the binary itself and
# whatever the build wrote after it. HW-DR-0039 rules the three outcomes, and
# `3` is this one — `1` is a disagreement and `2` is no engine of either profile.
#
# `find` runs inside a command substitution rather than as a test, because
# `set -e` would end the script on a non-zero status before any of this printed.
BEHIND=$(find engine -name target -prune -o \
    \( -name '*.rs' -o -name 'Cargo.toml' -o -name 'Cargo.lock' \) \
    -newer "$HW" -print 2>/dev/null | sort | head -3)
if [ -n "$BEHIND" ]; then
  echo "refresh-figures.sh: cannot tell whether a figure is stale. The engine" >&2
  echo "  under engine/target is older than the engine sources beside it, so a" >&2
  echo "  run of it measures this corpus with an engine the tree has moved past." >&2
  echo "  A figure it calls stale may be a figure this corpus holds." >&2
  echo "  newer than that binary:" >&2
  printf '%s\n' "$BEHIND" | sed 's/^/        /' >&2
  echo "  Build the engine again, then run this again:" >&2
  echo "        cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked" >&2
  exit 3
fi

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

"$HW" check --root . --json > "$WORK/check.json" 2>/dev/null || true
"$HW" check --root . > "$WORK/check.txt" 2>/dev/null || true
"$HW" conformance --root . > "$WORK/conformance.txt" 2>/dev/null || true

python3 - "$MODE" "$WORK" "$HW" "$INTO" <<'PY'
import json, re, subprocess, sys, pathlib

mode, work, hw, into = sys.argv[1], pathlib.Path(sys.argv[2]), sys.argv[3], sys.argv[4]
root = pathlib.Path.cwd()

check = json.loads((work / "check.json").read_text())
text = (work / "check.txt").read_text()
conf = (work / "conformance.txt").read_text()

fig = {}
src = {}


def put(key, value, source):
    fig[key] = str(value)
    src[key] = source


# --- 1. the census, from `headwater check --json` -------------------------
cov = check["coverage"]
put("census.seen", cov["seen"], "check --json .coverage.seen")
put("census.typed", cov["classified"], "check --json .coverage.classified")
put("census.checked", cov["checked"], "check --json .coverage.checked")
put("census.generated", cov["generated"], "check --json .coverage.generated")
put("census.instances", cov["instances"], "check --json .coverage.instances")
put("census.unaccounted", len(cov["unaccounted"]),
    "check --json .coverage.unaccounted, its length")

# --- 2. the census lines the JSON does not carry, from the text census ----
census = re.search(r"\ncensus\n(.*?)\n\n", text, re.S)
if not census:
    sys.exit("refresh-figures.sh: no census block in `headwater check`")
census = census.group(1)


def census_line(label):
    m = re.search(r"^\s*(\d+)\s+" + re.escape(label) + r"\s*$", census, re.M)
    if not m:
        sys.exit("refresh-figures.sh: no census line for %r" % label)
    return int(m.group(1))


put("census.untyped", census_line("untyped"), "check census, `N untyped`")
put("census.excluded", census_line("excluded"), "check census, `N excluded`")
put("census.notdoc", census_line("not a document"),
    "check census, `N not a document`")

# The census accounts for every file it saw. This is that identity, checked
# rather than assumed, and it is what `0 unaccounted for` means.
parts = ["census.typed", "census.generated", "census.untyped",
         "census.excluded", "census.notdoc"]
total = sum(int(fig[p]) for p in parts)
if total != int(fig["census.seen"]):
    sys.exit("refresh-figures.sh: the census does not add up: %d != %d"
             % (total, int(fig["census.seen"])))

# --- 3. the findings, from `headwater check --json` -----------------------
findings = check["findings"]
suppressed = [f for f in findings if f.get("escape") == "suppression"]
reported = [f for f in findings if f.get("escape") != "suppression"]
put("findings.raised", len(findings), "check --json .findings, its length")
put("findings.reported", len(reported),
    "check --json .findings, those with escape != suppression")
put("findings.suppressed", len(suppressed),
    "check --json .findings, those with escape == suppression")
put("findings.errors", len([f for f in reported if f["severity"] == "error"]),
    "check --json, reported findings with severity error")
put("findings.advisory", len([f for f in reported if f["severity"] == "warn"]),
    "check --json, reported findings with severity warn")
put("rules.fired", len({f["rule"] for f in findings}),
    "check --json .findings, distinct rule names")
put("rules.wired", len(check["rules"]), "check --json .rules, its length")

m = re.search(r"^\s*(\d+) findings hidden by (\d+) directives\s*$", text, re.M)
if not m:
    sys.exit("refresh-figures.sh: no suppressions block in `headwater check`")
if int(m.group(1)) != len(suppressed):
    sys.exit("refresh-figures.sh: the two suppression counts disagree")
put("findings.directives", m.group(2),
    "check, `N findings hidden by M directives`")

# --- 4. the obligation register, from the text register block -------------
m = re.search(
    r"^\s*(\d+) obligations: (\d+) verified, (\d+) gap, (\d+) unverifiable,"
    r" (\d+) with no disposition\s*$", text, re.M)
if not m:
    sys.exit("refresh-figures.sh: no register line in `headwater check`")
put("oblig.total", m.group(1), "check register, `N obligations:`")
put("oblig.verified", m.group(2), "check register, the same line")
put("oblig.gap", m.group(3), "check register, the same line")
put("oblig.unverifiable", m.group(4), "check register, the same line")
put("oblig.nodisposition", m.group(5), "check register, the same line")

# --- 5. the taxonomy in force and the clock -------------------------------
put("taxonomy.package", check["taxonomy"]["package"],
    "check --json .taxonomy.package")
put("taxonomy.version", check["taxonomy"]["version"],
    "check --json .taxonomy.version")
put("taxonomy.lock", check["taxonomy"]["lock"][:19] + "…",
    "check --json .taxonomy.lock, its first 19 characters")
put("run.date", check["clock"], "check --json .clock")

# --- 6. the command surface, from the generated verb index ----------------
# The index states no count since #1058, because a count is a fold that a
# text merge writes wrong. So this reads the rows: one row for each verb the
# binary dispatches, and the mark `**no contract**` in the last cell of a verb
# nothing describes.
verbs = (root / "docs/interfaces/README.md").read_text()
rows = re.findall(r"^\| `[^`]+` \|.*$", verbs, re.M)
if not rows:
    sys.exit("refresh-figures.sh: docs/interfaces/README.md holds no verb row "
             "in the form this script reads")
bare = [row for row in rows if "**no contract**" in row]
put("verbs.count", len(rows),
    "docs/interfaces/README.md, a generated `verb_index` projection, its rows")
put("verbs.contracts", len(rows) - len(bare),
    "the same rows, less the ones marked `**no contract**`")
put("verbs.nocontract", len(bare), "the rows marked `**no contract**`")
put("verbs.groups", len(re.findall(r"^## ", verbs, re.M)),
    "docs/interfaces/README.md, its `## ` headings, which are the groups "
    "`headwater --help` prints")

# --- 7. the emitter split, from the engine source ------------------------
prof = (root / "engine/crates/generate/src/profile.rs").read_text()
m = re.search(r"pub enum Emitter \{(.*?)\n\}", prof, re.S)
if not m:
    sys.exit("refresh-figures.sh: no `Emitter` enum in profile.rs")
variants = re.findall(r"^\s{4}([A-Z]\w*),\s*$", m.group(1), re.M)
m = re.search(r"pub fn is_built\(self\) -> bool \{\n(.*?)\n    \}", prof, re.S)
if not m:
    sys.exit("refresh-figures.sh: no `is_built` in profile.rs")
# Every arm of that `match`, whichever way rustfmt wrapped its patterns. The
# two lists are compared with the variants below rather than trusted, so a
# wildcard arm or a variant nobody judged is a refusal and never a miscount.
built, unbuilt = [], []
for pats, verdict in re.findall(
        r"((?:\s*Emitter::\w+\s*\|?)+)\s*=>\s*(true|false)\s*,", m.group(1)):
    (built if verdict == "true" else unbuilt).extend(
        re.findall(r"Emitter::(\w+)", pats))
if sorted(built + unbuilt) != sorted(variants):
    sys.exit("refresh-figures.sh: the arms of `is_built` in profile.rs do not "
             "name every `Emitter` variant exactly once")
put("emitters.total", len(variants),
    "engine/crates/generate/src/profile.rs, the `Emitter` variants")
put("emitters.built", len(built),
    "engine/crates/generate/src/profile.rs, the arms of `is_built`")
put("emitters.waiting", len(variants) - len(built),
    "the two counts above, subtracted")

# --- 8. the conformance level --------------------------------------------
m = re.search(r"^(L\d|no level) (reached|level reached), against (\S+) (\S+)",
              conf, re.M)
if not m:
    sys.exit("refresh-figures.sh: no verdict line in `headwater conformance`")
put("conformance.level", "no level" if m.group(1) == "no level" else m.group(1),
    "conformance, its verdict line")

# --- fill, check, or print -----------------------------------------------
# One element with a `data-figure` attribute, holding text and nothing else.
# The element name is captured so the closing tag has to match it, which keeps
# a nested element from being read as a figure.
pattern = re.compile(
    r'(<(\w+)\b[^>]*\bdata-figure="([a-z]+\.[a-z]+)"[^>]*>)([^<]*)(</\2>)')

if mode == "print":
    width = max(len(k) for k in fig)
    for k in sorted(fig):
        print("%-*s  %-12s  %s" % (width, k, fig[k], src[k]))
    raise SystemExit(0)

# `--check` reads the committed pages and writes nothing. `--into` fills the
# copies in an assembled directory and never touches `site/`. The two read the
# same markers, so a key that `--check` accepts is a key `--into` fills.
base = pathlib.Path(into) if mode == "into" else root / "site"
pages = sorted(base.glob("**/*.html"))
if not pages:
    sys.exit("refresh-figures.sh: no page under %s, so there is nothing to "
             "fill and nothing to check" % base)


def rel(p):
    try:
        return p.relative_to(root)
    except ValueError:
        return p


# Since #1273 the committed pages carry no measured value. A figure is measured
# when the site is published, into an assembled copy, so a value in a committed
# marker is a fold that two branches merge wrong, and it is refused. The clock
# partition and the G0 measurement that excused a clock-only difference are
# gone with the committed values: there is nothing committed to compare a run
# against. HW-DR-0039's successor records the change.
used, unknown, filled, unmatched = set(), [], [], []
for page in pages:
    before = page.read_text()

    def sub(m):
        key = m.group(3)
        if key not in fig:
            unknown.append((page, key))
            return m.group(0)
        used.add(key)
        if mode == "check" and m.group(4) != "":
            filled.append((page, key, m.group(4)))
        return m.group(1) + fig[key] + m.group(5)

    after, matched = pattern.subn(sub, before)
    # The pattern needs the element to hold text alone and the key to be two
    # lower-case words. A marker that fails either is counted here, because it
    # is otherwise read as no marker at all and served as it stands.
    if before.count("data-figure=") != matched:
        unmatched.append((page, before.count("data-figure=") - matched))
    if mode == "into" and after != before:
        page.write_text(after)

for page, key in unknown:
    print("unknown figure key %s in %s" % (key, rel(page)), file=sys.stderr)
for page, n in unmatched:
    print("%d data-figure element%s in %s that this cannot fill: a nested "
          "element, or a key that is not two lower-case words"
          % (n, "" if n == 1 else "s", rel(page)), file=sys.stderr)
filled_pages = {}
for page, key, value in filled:
    filled_pages.setdefault(page, []).append((key, value))
for page, entries in filled_pages.items():
    key, value = entries[0]
    print("%s carries %d measured figure%s, the first %s holding %r"
          % (rel(page), len(entries), "" if len(entries) == 1 else "s", key,
             value), file=sys.stderr)
if filled_pages:
    print("  the committed pages carry no measured value, because a figure is "
          "measured when the site is published. Run: "
          "sh tools/site/refresh-figures.sh --blank", file=sys.stderr)

# --- the tutorial page, held against the tutorial document ----------------
# The tutorial page copies commands and output out of
# docs/tutorials/your-first-governed-corpus.md, which is the document that
# .claude/tutorial/fixtures.sh runs against a scratch repository. A block on
# the page that is no longer in that document is drift, and this is what
# catches it. Nothing here rewrites the page: the remedy is to regenerate the
# blocks from the document, and never to edit the page.
#
# The page is not optional. HW-DR-0037 governs it by name, so a page that is
# gone takes 43 checked blocks with it, and that is an error rather than a
# skip. The same reasoning as the denominator guard below, one layer up.
import html as _html

drift = []
tutpage = root / "site/tutorial/index.html"
if not tutpage.exists():
    sys.exit("refresh-figures.sh: site/tutorial/index.html is not there, and "
             "HW-DR-0037 governs that page by name, so its absence is an error "
             "and not a skip")
tut = (root / "docs/tutorials/your-first-governed-corpus.md").read_text()
page = tutpage.read_text()
blocks = re.findall(r'<pre class="[^"]*\bverbatim\b[^"]*">(.*?)</pre>',
                    page, re.S)
for block in blocks:
    if _html.unescape(block).rstrip("\n") not in tut:
        drift.append(_html.unescape(block).split("\n")[0][:70])
m = re.search(r"<span data-tutorial-date>([^<]*)</span>", page)
stated = m.group(1) if m else None
m = re.search(r"The date of the run is (\d{4}-\d{2}-\d{2})\.", tut)
real = m.group(1) if m else None
if stated != real:
    drift.append("the run date: page says %r, the document says %r"
                 % (stated, real))
for d in drift:
    print("tutorial drift: %s" % d, file=sys.stderr)
print("%d verbatim tutorial blocks checked against the document, %d adrift"
      % (len(blocks), len(drift)))

# --- the landing page's quotation of HW-DR-0037 --------------------------
# The hero card quotes the front matter of the record that governs these
# pages. A field that record no longer carries is drift of the same kind the
# tutorial check catches, so it is caught the same way.
#
# `landing` is the landing check's own list, and it is printed rather than a
# tail of `drift`. A slice of `drift` here printed the last three TUTORIAL
# entries under the label `landing drift:` on every run where the tutorial had
# any, which is a refusal naming a check that did not fail.
landing = []
home = root / "site/index.html"
if not home.exists():
    sys.exit("refresh-figures.sh: site/index.html is not there, and HW-DR-0037 "
             "governs that page by name, so its absence is an error and not a "
             "skip")
rec = next(root.glob("docs/decisions/0037-*.md")).read_text()
m = re.search(r'<pre class="quotes-0037">(.*?)</pre>', home.read_text(), re.S)
if not m:
    landing.append("site/index.html no longer quotes HW-DR-0037")
else:
    quoted = _html.unescape(re.sub(r"<[^>]+>", "", m.group(1)))
    for line in quoted.split("\n"):
        if line.strip() in ("", "---"):
            continue
        if line not in rec:
            landing.append("HW-DR-0037 no longer carries %r" % line.strip())
for d in landing:
    print("landing drift: %s" % d, file=sys.stderr)
drift.extend(landing)

# --- the denominator ------------------------------------------------------
# A measured figure that reaches no page is the whole check contributing
# nothing, silently. An empty `site/`, a renamed marker attribute and a moved
# page all produce it, and all three used to exit 0.
never = sorted(set(fig) - used)
if never:
    print("measured but on no page: %s" % ", ".join(never), file=sys.stderr)
    print("  a figure that reaches no page means a page stopped carrying it. "
          "HW-DR-0050 refuses a page that opts out of the register rather than "
          "skipping it, and this refuses the same thing for a figure.",
          file=sys.stderr)

verb = "filled" if mode == "into" else "checked"
print("%d figures measured, %d used across %d pages, %s under %s, run of %s"
      % (len(fig), len(used), len(pages), verb, rel(base), fig["run.date"]))

if unknown or unmatched or filled or drift or never:
    raise SystemExit(1)
PY
