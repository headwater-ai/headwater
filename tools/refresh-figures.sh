#!/bin/sh
# refresh-figures.sh — measure this repository, and write the result into the
# hand-built pages under `site/`.
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
#     3. `docs/interfaces/README.md`            the verb count and the group
#                                               count. That file is a generated
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
#   `tools/refresh-site-tokens.sh` refuses an unmarked page for the same
#   reason, and HW-DR-0050 rules it: a page that opts out of the register is
#   refused rather than skipped.
#
#   WHAT THAT DENOMINATOR STILL DOES NOT CATCH, measured on 2026-09-06:
#
#     `used` is a union across pages, so a figure lost on ONE page is invisible
#     while any other page still carries it. Renaming the marker attribute on
#     `site/how-it-works/index.html` alone gives exit 0 at `34 used across 8
#     pages, 0 stale`; renaming it on all eight refuses. Five of the eight
#     pages carry no figure at all, and nothing here notices which page holds
#     which key. The fix is a declaration of what each page owes, which this
#     script has nowhere to read.
#
#     The element pattern requires the text to hold no `<`, so a figure with a
#     nested child element is not matched at all. `<b data-figure="rules.wired"
#     ><b>99</b></b>` serves 99 where the run says 29, at exit 0, and it is not
#     even reported as an unknown key. An off-shape key such as `rules.wired2`
#     matches nothing for the same reason and is equally silent. A count of
#     `data-figure` occurrences in the file, held against the count the pattern
#     matched, is what would catch both.
#
# WHAT `--check` COMPARES, AND WHAT IT DOES NOT
#
#   `--check` compares the 26 figures that are a function of the corpus and the
#   lock. Eight are a function of the clock as well, and those are reported and
#   never failed. `headwater generate` already states this rule of its own
#   `coverage_report` projection, in the sentence it prints on every run:
#
#     "its content is a function of the clock as well as of the corpus and the
#      lock, because a migration task lapses and a suppression expires on a
#      date. A committed copy would fail this check on a morning when nothing
#      changed."
#
#   The eight are `run.date` and the seven that read the findings list:
#   `findings.raised`, `findings.reported`, `findings.suppressed`,
#   `findings.errors`, `findings.advisory`, `findings.directives` and
#   `rules.fired`. Two dated mechanisms move a finding between reported and
#   suppressed with no file touched, and both are in this engine:
#   `check/src/adoption.rs` retires a migration task at `task.until < now`, and
#   `check/src/suppression.rs` expires an escape directive at
#   `suppression.until < now`.
#
#   Measured on 2026-09-06, each arm on a tree that nothing else touched:
#
#     the clock at 2027-07-01, past `AD-1`'s `until: 2027-06-30`
#       `findings.errors` 0->1, `findings.raised` 7->8,
#       `findings.reported` 7->8, `rules.fired` 1->2
#     one escape directive, live today and lapsed the day after its `until`
#       `findings.reported` 6->7, `findings.suppressed` 1->0,
#       `findings.advisory` 6->7, `findings.directives` 1->0
#
#   The second arm is the one that matters, because `headwater check --strict`
#   exits 0 on both sides of it. So on the morning a directive lapses, a gate
#   over these figures is the only thing that goes red, on a tree nobody
#   touched, saying a page disagrees with a run. `until` is required on every
#   directive, so every directive reaches that morning.
#
#   The page's claim is that these numbers came from a run on the date beside
#   them, and that claim stays true. Write mode rewrites all eight on every
#   run, and counts them on a line of their own.
#
# WHY THIS SCRIPT EXISTS AT ALL
#
#   HW-DR-0037 rules that a hand-built page states no figure a person typed, and
#   it admits two forms where a number belongs on such a page. The page links to
#   the generated artifact that produced the number. Or a build interpolates the
#   number from a run, and the page carries no source of its own.
#
#   The same record measures the second form as unavailable, because
#   `wrangler.jsonc` declares an asset directory and no build command, so nothing
#   runs between the commit and the served bytes. That measurement still holds.
#   This script moves the build to the other side of the commit: it runs on the
#   author's machine, before the commit, and the interpolated figure is in the
#   committed bytes. HW-DR-0039 records that move.
#
#   The distinction that makes this admissible is between a figure that is
#   hand-RUN and one that is hand-TYPED. A hand-typed figure has no source. A
#   hand-run figure has a source, a date, and a command that reproduces it — and
#   `--check` fails when the page and the run disagree.
#
# THE DISCIPLINE, AND WHAT RUNS IT
#
#   Run this before any commit that touches a page carrying a figure, and read
#   what it prints. `--check` writes nothing and exits non-zero when a page is
#   stale, which is the form to put in front of a reviewer.
#
#   Two callers run it, and a person is neither of them. `.githooks/pre-commit`
#   refuses a commit whose page disagrees with a fresh run, under
#   `HEADWATER_SKIP_FIGURE_CHECK`, and it announces the skip. The CI step named
#   "The figures on the hand-built pages came from a run" reads the committed
#   tree, has no escape hatch, and is the half that holds.
#
# USAGE
#
#   sh tools/refresh-figures.sh            measure, and write the pages
#   sh tools/refresh-figures.sh --check    measure, write nothing, and
#                                                 exit 1 on any disagreement
#   sh tools/refresh-figures.sh --print    measure, and print the table
#
set -eu

MODE=write
case "${1:-}" in
  --check) MODE=check ;;
  --print) MODE=print ;;
  "") ;;
  *) echo "refresh-figures.sh: unknown argument '$1'" >&2; exit 2 ;;
esac

ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT"

HW="$ROOT/engine/target/release/headwater"
if [ ! -x "$HW" ]; then
  echo "refresh-figures.sh: no engine at $HW" >&2
  echo "  build it: cargo build --release -p headwater-cli --manifest-path engine/Cargo.toml --locked" >&2
  exit 2
fi

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

"$HW" check --root . --json > "$WORK/check.json" 2>/dev/null || true
"$HW" check --root . > "$WORK/check.txt" 2>/dev/null || true
"$HW" conformance --root . > "$WORK/conformance.txt" 2>/dev/null || true

python3 - "$MODE" "$WORK" <<'PY'
import json, re, sys, pathlib

mode, work = sys.argv[1], pathlib.Path(sys.argv[2])
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
verbs = (root / "docs/interfaces/README.md").read_text()
m = re.search(r"dispatches (\d+) verbs\. (\d+) of them have a contract on this"
              r" shelf, and (\d+) have none", verbs)
if not m:
    sys.exit("refresh-figures.sh: docs/interfaces/README.md does not state a "
             "verb count in the form this script reads")
put("verbs.count", m.group(1),
    "docs/interfaces/README.md, a generated `verb_index` projection")
put("verbs.contracts", m.group(2), "the same sentence of the same file")
put("verbs.nocontract", m.group(3), "the same sentence of the same file")
put("verbs.groups", len(re.findall(r"^## ", verbs, re.M)),
    "docs/interfaces/README.md, its `## ` headings, which are the groups "
    "`headwater --help` prints")

# --- 7. the emitter split, from the engine source ------------------------
prof = (root / "engine/crates/generate/src/profile.rs").read_text()
m = re.search(r"pub enum Emitter \{(.*?)\n\}", prof, re.S)
if not m:
    sys.exit("refresh-figures.sh: no `Emitter` enum in profile.rs")
variants = re.findall(r"^\s{4}([A-Z]\w*),\s*$", m.group(1), re.M)
m = re.search(r"pub fn is_built\(self\) -> bool \{\s*matches!\(self, (.*?)\)",
              prof, re.S)
if not m:
    sys.exit("refresh-figures.sh: no `is_built` in profile.rs")
built = re.findall(r"Emitter::(\w+)", m.group(1))
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

# --- write, check, or print ----------------------------------------------
pages = sorted(root.glob("site/**/*.html"))
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

used, stale, unknown = set(), [], []
for page in pages:
    before = page.read_text()

    def sub(m):
        key = m.group(3)
        if key not in fig:
            unknown.append((page, key))
            return m.group(0)
        used.add(key)
        if m.group(4) != fig[key]:
            stale.append((page, key, m.group(4), fig[key]))
        return m.group(1) + fig[key] + m.group(5)

    after = pattern.sub(sub, before)
    if mode == "write" and after != before:
        page.write_text(after)

# The partition. A figure is gateable where it is a function of the corpus and
# the lock, and these eight are not: each one reads the clock, or reads the
# findings list, and two dated mechanisms move a finding between reported and
# suppressed with the tree untouched. An adoption task lapses
# (`check/src/adoption.rs`, `task.until < now`) and an escape directive expires
# (`check/src/suppression.rs`, `suppression.until < now`). `rules.fired` counts
# distinct rule names among the findings, so it moves with them.
#
# The other 26 keys are gateable. See WHAT `--check` COMPARES above for the
# measurement behind this list, and why it is eight rather than one.
CLOCK_KEYS = {
    "run.date",
    "findings.raised", "findings.reported", "findings.suppressed",
    "findings.errors", "findings.advisory", "findings.directives",
    "rules.fired",
}

# The partition names keys, so it goes stale the moment one is renamed. This is
# that list held against the run rather than trusted, the same way the page
# denominator below is.
missing = sorted(CLOCK_KEYS - set(fig))
if missing:
    sys.exit("refresh-figures.sh: the clock partition names %s, which this run "
             "does not measure" % ", ".join(missing))
stale_measured = [s for s in stale if s[1] not in CLOCK_KEYS]
stale_clock = [s for s in stale if s[1] in CLOCK_KEYS]

rel = lambda p: p.relative_to(root)
for page, key in unknown:
    print("unknown figure key %s in %s" % (key, rel(page)), file=sys.stderr)
for page, key, was, now in stale_measured:
    verb = "rewrote" if mode == "write" else "stale"
    print("%s %s in %s: %s -> %s" % (verb, key, rel(page), was, now),
          file=sys.stderr)
for page, key, was, now in stale_clock:
    verb = "rewrote" if mode == "write" else "clock only"
    print("%s %s in %s: %s -> %s" % (verb, key, rel(page), was, now),
          file=sys.stderr)

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

# The headline counts the measurements alone, and the run date is counted on
# its own line beside it. So the two numbers add up to the lines above them, in
# either mode, and neither one is silently folded into the other.
print("%d figures measured, %d used across %d pages, %d stale, run of %s"
      % (len(fig), len(used), len(pages), len(stale_measured), fig["run.date"]))
if stale_clock:
    print("%d further occurrences differ in a figure that is a function of the "
          "clock. A lapsed adoption task, an expired escape directive and the "
          "run date each move one without the tree moving, so they are counted "
          "here and they fail no check." % len(stale_clock))

if unknown or drift or never:
    raise SystemExit(1)
if mode == "check" and stale_measured:
    raise SystemExit(1)
PY
