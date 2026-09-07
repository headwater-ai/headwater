#!/bin/sh
# What holds `DEVELOPING.md`, the contributor loop.
#
# Run it from anywhere:
#     sh tools/developing-fixtures.sh
#
# # WHY THIS PAGE NEEDS A SUITE AT ALL
#
# `DEVELOPING.md` sits at the repository root, outside `docs/`, so no rule of
# this engine reads it and `mkdocs build --strict` never opens it. Its prose
# answers to nobody, exactly as `README.md`'s does. What this suite holds is
# not the prose. It is the one section of the page that is a copy of another
# file: the list of what CI runs.
#
# A hand-kept gate list goes stale silently, and the measurement is on record.
# The best hand-written one this repository produced named 8 script
# invocations on 2026-09-06; the workflow ran 15 that day. Nothing reddened,
# because a list that is merely wrong changes no exit status. So the list on
# the page is derived from `.github/workflows/ci.yml` here, in both
# directions, and a gate added to CI without a line on the page fails this
# suite the day it lands.
#
# # THE SECOND WORKFLOW PARSER IN `tools/`, AND WHY IT IS NOT DUPLICATION
#
# `tools/build-declaration-fixtures.sh` already walks every `run:` value of
# the workflow. It asks a different question of what it finds — whether a
# cargo invocation carries `--locked` — and it reports offenders rather than
# enumerating a population. Sharing the walk would mean one suite sourcing the
# other, and then a defect in the shared file takes both suites down together
# and neither can be run on its own. The walk is short, so this file
# reimplements it in the same shape rather than importing it. A reviewer
# meeting two `run:` walks in `tools/` is meeting a deliberate second copy of
# eleven lines, not an accident.
#
# The shape, which is the part worth copying: a YAML comment line is skipped
# before anything else, so a shell comment inside a run block is skipped too;
# `run:` is found only where everything before it is blank or a list dash, so
# `name: … run: …` does not hit; a block scalar (`|`, `>`, `|-`) is followed
# by every subsequent line indented at least as far as the `run:` key. A
# line-wise grep of the file cannot do any of that, and it would put
# `.claude/hooks/fixtures-live.sh` in the population, which appears in the
# workflow only inside a comment saying no job runs it.
#
# # THE THREE POPULATIONS, AND THE BOUNDARY ON THE THIRD
#
# From every `run:` value, and from every code block in the page's gate
# section, through the same extractor so that both sides are read by one set
# of rules:
#
#   script  a path under `.githooks/`, `.claude/` or `tools/` ending `.sh` or
#           `.py`, and EXECUTED rather than merely named — the argument to
#           `sh`, `bash`, `python3` or `python`, or a command word beginning
#           `./`, or the first word of a command. A bare path written in prose
#           is not a member, which is why the page may say in prose that
#           `.claude/hooks/fixtures-live.sh` is not a gate.
#   cargo   `cargo` plus the following word, so `cargo --version` and
#           `cargo fmt` are two members and every flag after the first word is
#           ignored.
#   verb    `headwater` plus the following words up to the first flag, so
#           `taxonomy resolve` is one member and `check` is one member however
#           many times and however many ways CI runs it.
#
# **Flags on a verb are deliberately not read.** CI runs `check` with `--now`,
# `--change`, `--format` and `--no-cache` in five combinations, and none of
# those is a gate a contributor can add or remove: they are a pinned clock, a
# change manifest and an output shape. Reading them would make the page name
# five strings nobody types, and would redden on a flag reordering that
# changed no gate. The direction that goes stale is a gate arriving or
# leaving, and that is what the verb population is cut to catch. The page's
# prose says what the flags are for, and no case here reads that sentence.
#
# `.githooks/change-manifest` is outside the script population because it
# carries no extension. It is a producer the workflow calls, not a gate with
# an exit status of its own, and the page says so.
#
# # THE FLOOR GUARD
#
# Every case below is a set comparison, and two empty sets are equal. A parser
# that silently stopped matching would report a clean run over nothing. Case
# group 1 therefore asserts the derived population is not implausibly small
# before any comparison is believed.

set -u

root=$(cd "$(dirname "$0")/.." && pwd)

page=$root/DEVELOPING.md
workflow=$root/.github/workflows/ci.yml
section='What CI runs'

for f in "$page" "$workflow" "$root/CONTRIBUTING.md" "$root/engine/Cargo.toml"; do
    if [ ! -f "$f" ]; then
        echo "missing: $f" >&2
        echo "  every case below reads it, so this suite stops rather than" >&2
        echo "  reporting a row of passes over a file it could not open." >&2
        exit 1
    fi
done

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

passed=0
failed=0

pass() {
    passed=$((passed + 1))
    echo "  ok    $1"
}

fail() {
    failed=$((failed + 1))
    echo "  FAIL  $1"
    echo "          $2"
}

# same NAME EXPECTED ACTUAL
same() {
    if [ "$2" = "$3" ]; then
        pass "$1"
    else
        fail "$1" "expected \`$2\`, got \`$3\`"
    fi
}

# ---------------------------------------------------------------------------
# The extractor. Reads command text on stdin, one command per line, and prints
# `<kind> <member>` for each population member it finds. A line ending in a
# backslash is joined to the next before anything is read, so a command split
# over a continuation is one command here.
# ---------------------------------------------------------------------------
members() {
    awk '
        # Strip the shell punctuation a word can be wrapped in — quotes of
        # either kind, parentheses, braces, backticks, a trailing semicolon —
        # without naming any of them, so a form nobody thought of is stripped
        # too. A path keeps its dots and its slashes and nothing else.
        function bare(t) {
            while (length(t) > 0 && t ~ /^[^A-Za-z0-9_.\/]/) t = substr(t, 2)
            while (length(t) > 0 && t ~ /[^A-Za-z0-9_.\/]$/) t = substr(t, 1, length(t) - 1)
            return t
        }
        function is_script(t) {
            return t ~ /^(\.\/)?(\.githooks|\.claude|tools)\/[^ ]+\.(sh|py)$/
        }
        function scan_scripts(text,   n, w, i, prev, atstart, t) {
            n = split(text, w, /[ \t]+/)
            atstart = 1
            prev = ""
            for (i = 1; i <= n; i++) {
                if (w[i] == "") continue
                t = bare(w[i])
                if (t != "" && is_script(t)) {
                    if (atstart \
                        || t ~ /^\.\// \
                        || prev ~ /(^|[^A-Za-z0-9_-])(sh|bash|python3|python)$/) {
                        sub(/^\.\//, "", t)
                        print "script " t
                    }
                }
                atstart = (w[i] ~ /(&|\||;|\{|\()$/)
                prev = bare(w[i])
            }
        }
        function scan_cargo(text,   rest, w) {
            rest = text
            while (match(rest, /(^|[^A-Za-z0-9_.\/-])cargo[ \t]+[^ \t]/)) {
                rest = substr(rest, RSTART + RLENGTH - 1)
                if (match(rest, /^[^ \t]+/))
                    w = substr(rest, RSTART, RLENGTH)
                if (w ~ /^[a-z-]+$/ || w == "--version")
                    print "cargo " w
            }
        }
        function scan_verb(text,   rest, out, w) {
            rest = text
            while (match(rest, /(^|[^A-Za-z0-9_-])headwater[ \t]+[a-z]/)) {
                rest = substr(rest, RSTART + RLENGTH - 1)
                out = ""
                while (match(rest, /^[a-z][a-z-]*([ \t]|$)/)) {
                    w = substr(rest, RSTART, RLENGTH)
                    sub(/[ \t]+$/, "", w)
                    out = (out == "") ? w : out " " w
                    rest = substr(rest, RSTART + RLENGTH)
                    sub(/^[ \t]+/, "", rest)
                }
                # No word followed the verb name after all — `headwater
                # check.txt` in a filename, say. Advance one character rather
                # than looping on the same position forever.
                if (out != "") print "verb " out
                else rest = substr(rest, 2)
            }
        }
        function scan(text) {
            scan_scripts(text)
            scan_cargo(text)
            scan_verb(text)
        }
        {
            t = $0
            sub(/[ \t]+$/, "", t)
            if (t ~ /\\$/) { buf = buf substr(t, 1, length(t) - 1) " "; next }
            scan(buf t)
            buf = ""
        }
        END { if (buf != "") scan(buf) }
    ' | sort -u
}

# workflow_text FILE — every `run:` value, one command line per output line.
workflow_text() {
    awk '
        /^[ \t]*#/ { next }
        {
            if (inblock) {
                if ($0 ~ /^[ \t]*$/) next
                if (match($0, /[^ \t]/) - 1 >= blockind) { print; next }
                inblock = 0
            }
            p = index($0, "run:")
            if (p > 0 && substr($0, 1, p - 1) ~ /^[ \t-]*$/) {
                rest = substr($0, p + 4)
                sub(/^[ \t]+/, "", rest)
                if (rest ~ /^[|>][-+0-9]*$/) { inblock = 1; blockind = p; next }
                print rest
            }
        }
    ' "$1"
}

# page_text FILE HEADING — every code-block line under `## HEADING`, indented
# or fenced. Prose is not read: a path named in a sentence is a mention, and
# the page has to be able to say that a script is NOT a gate.
page_text() {
    awk -v want="$2" '
        $0 == "## " want { inside = 1; next }
        /^## / { inside = 0 }
        inside == 0 { next }
        /^[ \t]*```/ { fence = !fence; next }
        fence { print; next }
        /^[ \t]*$/ { next }
        /^(    |\t)/ { print }
    ' "$1"
}

# judge KIND WORKFLOW PAGE — prints the members CI runs that the page omits on
# stdout, one per line, and the members the page names that CI does not run on
# file descriptor 3's stand-in (a second call with `extra`).
side() {
    # side KIND FILE  ->  the members of KIND in FILE's extracted text
    grep "^$1 " | sed "s/^$1 //" | sort -u
}

# ---------------------------------------------------------------------------
# The live populations.
# ---------------------------------------------------------------------------
workflow_text "$workflow" | members > "$scratch/ci.all"
page_text "$page" "$section" | members > "$scratch/page.all"

for kind in script cargo verb; do
    side "$kind" < "$scratch/ci.all" > "$scratch/ci.$kind"
    side "$kind" < "$scratch/page.all" > "$scratch/page.$kind"
done

echo "the gate list on the page is not implausibly small"

n_script=$(wc -l < "$scratch/ci.script" | tr -d ' ')
n_cargo=$(wc -l < "$scratch/ci.cargo" | tr -d ' ')
n_verb=$(wc -l < "$scratch/ci.verb" | tr -d ' ')

if [ "$n_script" -ge 12 ] && [ "$n_cargo" -ge 4 ] && [ "$n_verb" -ge 4 ]; then
    pass "the workflow parse found a plausible population ($n_script scripts, $n_cargo cargo, $n_verb verbs)"
else
    fail "the workflow parse found a plausible population" \
        "$n_script scripts, $n_cargo cargo, $n_verb verbs — too few to have parsed the file"
fi

echo
echo "what CI runs and what the page says are one set"

for kind in script cargo verb; do
    missing=$(comm -23 "$scratch/ci.$kind" "$scratch/page.$kind" | tr '\n' ' ')
    extra=$(comm -13 "$scratch/ci.$kind" "$scratch/page.$kind" | tr '\n' ' ')
    same "every $kind gate CI runs is named on the page" "" "$missing"
    same "  and every $kind gate the page names, CI runs" "" "$extra"
done

echo
echo "the page's other claims about files that exist"

floor_page=$(sed -n 's/.*Rust \([0-9][0-9]*\.[0-9][0-9]*\) or later.*/\1/p' "$page" | head -1)
floor_ws=$(sed -n 's/^rust-version = "\([0-9][0-9.]*\)".*/\1/p' "$root/engine/Cargo.toml" | head -1)
same "the floor the page states is the floor \`[workspace.package]\` declares" \
    "$floor_ws" "$floor_page"
if [ -n "$floor_ws" ]; then
    pass "  and the workspace declaration was read ($floor_ws)"
else
    fail "  and the workspace declaration was read" \
        "no \`rust-version\` found in engine/Cargo.toml, so the case above compared two empty strings"
fi

# `-F`, because in a basic regular expression `\(` and `\)` are grouping and
# the pattern would match `]DEVELOPING.md` — a shape no Markdown link has, so
# the case failed on a correct file the first time it ran.
link=$(grep -c -F '](DEVELOPING.md)' "$root/CONTRIBUTING.md")
if [ "$link" -ge 1 ]; then
    pass "\`CONTRIBUTING.md\` reaches this page in one hop"
else
    fail "\`CONTRIBUTING.md\` reaches this page in one hop" \
        "no link to DEVELOPING.md in CONTRIBUTING.md"
fi

# Every relative link on the page resolves to a file that exists. Read per
# occurrence and not per line: this repository forbids hard-wrapped Markdown,
# so a paragraph is one long line carrying several links.
dead=""
seen=0
for target in $(grep -o ']([^)]*)' "$page" | sed 's/^](//; s/)$//'); do
    case "$target" in
        http://* | https://* | mailto:* | '#'*) continue ;;
    esac
    seen=$((seen + 1))
    file=${target%%#*}
    [ -n "$file" ] || continue
    [ -e "$root/$file" ] || dead="$dead $target"
done
same "every relative link on the page resolves to a file that exists" "" "$dead"
if [ "$seen" -ge 4 ]; then
    pass "  over every relative link on the page ($seen read)"
else
    fail "  over every relative link on the page" \
        "$seen links read — too few to have parsed the page"
fi

# ---------------------------------------------------------------------------
# The refusal arms. Every case above is a comparison that passes on a green
# tree forever, so none of them is evidence that the judge can say no.
# ---------------------------------------------------------------------------
echo
echo "the judge refuses what it claims to refuse"

# A workflow pinned to what CI ran on 2026-09-06, written in the shapes the
# real file uses: a plain value, a block scalar, a block scalar behind a `cd`,
# a step whose only mention of a script is a shell comment, and a YAML comment.
fixture_workflow="$scratch/ci-2026-09-06.yml"
{
    echo 'jobs:'
    echo '  engine:'
    echo '    steps:'
    echo '      - name: Format'
    echo '        run: cargo fmt --check'
    echo '      - name: Lint'
    echo '        run: cargo clippy --all-targets --locked -- -D warnings'
    echo '      - name: Test'
    echo '        run: cargo test --locked'
    echo '      - name: The toolchain'
    echo '        run: cargo --version'
    echo '  headwater:'
    echo '    steps:'
    echo '      - name: Build'
    echo '        run: cargo build --release -p headwater-cli --locked'
    echo '      - name: The commit gate'
    echo '        run: sh .githooks/fixtures.sh'
    echo '      - name: The harness hooks'
    echo '        run: sh .claude/hooks/fixtures.sh'
    echo '      - name: The skills'
    echo '        run: sh .claude/skills/fixtures.sh'
    echo '      - name: The tutorial'
    echo '        run: sh .claude/tutorial/fixtures.sh'
    echo '      - name: The claim store'
    echo '        run: sh tools/id-store-fixtures.sh'
    echo '      - name: The build declarations'
    echo '        run: sh tools/build-declaration-fixtures.sh'
    echo '      - name: The first screen'
    echo '        run: sh tools/readme-fixtures.sh'
    echo '      - name: The command an outsider runs'
    echo '        run: sh tools/engine-readme-fixtures.sh'
    echo '      - name: The crawler files'
    echo '        run: sh tools/refresh-crawler-files.sh --check'
    echo '      - name: The visual register'
    echo '        run: sh tools/refresh-site-tokens.sh --check'
    # A block scalar behind a `cd`, which a line-anchored reader loses.
    echo '      - name: The two halves of the site'
    echo '        run: |'
    echo '          cd "$GITHUB_WORKSPACE"'
    echo '          sh tools/assemble-site.sh --check'
    echo '      - name: Every in-site fragment'
    echo '        run: |'
    echo '          python3 tools/check-site-fragments.py'
    echo '          sh tools/site-fragments-fixtures.sh'
    echo '      - name: Every served page'
    echo '        run: |'
    echo '          python3 tools/check-site-console.py || status=$?'
    echo '          sh tools/site-console-fixtures.sh || status=$?'
    # The `fixtures-live.sh` case: named only in a comment, and not a gate.
    echo '      - name: The harness binding'
    echo '        run: |'
    echo '          # sh .claude/hooks/fixtures-live.sh is not run here: it'
    echo '          # spends real credits against a real login.'
    echo '          echo skipped'
    # Two commented-out steps rather than a sentence in a comment, because a
    # sentence carries no `run:` at all and would stay out of the population
    # however badly the parser were broken — a case nothing holds.
    #
    # The first is held by two guards in series: the comment skip fires on it,
    # and if that were removed the "everything before `run:` is blank or a list
    # dash" guard still refuses the `#` in front of `run:`. The second is held
    # by the prefix guard ALONE, because its first non-blank character is the
    # dash and the comment skip never sees it. So this case reddens when the
    # prefix guard is widened, which is a different mutation from the one the
    # case above catches, and the two are independent.
    echo '      # run: sh tools/never-run-fixtures.sh'
    echo '      - # run: sh tools/never-listed-fixtures.sh'
    echo '      - name: The verbs'
    echo '        run: |'
    echo '          ./engine/target/release/headwater taxonomy resolve --check'
    echo '          ./engine/target/release/headwater generate --check'
    echo '          ./engine/target/release/headwater export --check'
    echo '          ./engine/target/release/headwater conformance --level L0'
    echo '          ./engine/target/release/headwater check --no-cache --now "$NOW" \'
    echo '            --change "$manifest" --format markdown > change.md'
} > "$fixture_workflow"

workflow_text "$fixture_workflow" | members > "$scratch/fx.all"
side script < "$scratch/fx.all" > "$scratch/fx.script"
side cargo < "$scratch/fx.all" > "$scratch/fx.cargo"
side verb < "$scratch/fx.all" > "$scratch/fx.verb"

# An EXACT count, and it must never be softened to `>=`. The floor guard above
# catches a parser that stopped matching; it does not catch a parser that
# dropped a whole class from both sides at once. Delete `.py` from the script
# pattern and every bidirectional comparison stays green, because the page and
# the workflow lose the same two members together. This line is what notices.
same "the pinned workflow parses to the fifteen scripts it ran that day" \
    15 "$(wc -l < "$scratch/fx.script" | tr -d ' ')"
same "  and a script named only in a shell comment is not a gate" \
    "" "$(grep -c 'fixtures-live' "$scratch/fx.script" | sed 's/^0$//')"
same "  and a step commented out in the YAML is not a gate, in either shape" \
    "" "$(grep -c 'never-run-fixtures\|never-listed-fixtures' "$scratch/fx.script" | sed 's/^0$//')"
same "  and a gate behind a \`cd\` inside a block scalar is found" \
    "tools/assemble-site.sh" "$(grep '^tools/assemble-site' "$scratch/fx.script")"
same "  and the five cargo commands are five" \
    5 "$(wc -l < "$scratch/fx.cargo" | tr -d ' ')"
same "  and a verb is its name, whatever flags follow it" \
    "check conformance export generate taxonomy resolve" \
    "$(tr '\n' '|' < "$scratch/fx.verb" | sed 's/|/ /g; s/ *$//')"

# The page issue #592 wrote on 2026-09-06, seeded from a literal rather than
# from the live issue. It named 8 of the 15 gates above. The judge must name
# the other 7 — no fewer, or the extraction is broken, and no more, or the
# population is over-broad.
stale_page="$scratch/stale.md"
{
    echo '# Developing Headwater'
    echo
    echo '## What CI runs'
    echo
    echo '    cargo fmt --check'
    echo '    cargo clippy --all-targets --locked -- -D warnings'
    echo '    cargo test --locked'
    echo '    cargo --version'
    echo '    cargo build --release -p headwater-cli --locked'
    echo
    echo '    headwater taxonomy resolve --check'
    echo '    headwater generate --check'
    echo '    headwater export --check'
    echo '    headwater conformance --level L0'
    echo '    headwater check'
    echo
    echo 'The suites, and a prose mention of one that is not a gate:'
    echo '`.claude/hooks/fixtures-live.sh` is not run by any job.'
    echo
    echo '    sh .githooks/fixtures.sh'
    echo '    sh .claude/hooks/fixtures.sh'
    echo '    sh .claude/skills/fixtures.sh'
    echo '    sh .claude/tutorial/fixtures.sh'
    echo '    sh tools/id-store-fixtures.sh'
    echo '    sh tools/refresh-crawler-files.sh --check'
    echo '    sh tools/refresh-site-tokens.sh --check'
    echo '    sh tools/site-fragments-fixtures.sh'
} > "$stale_page"

page_text "$stale_page" "$section" | members > "$scratch/stale.all"
side script < "$scratch/stale.all" > "$scratch/stale.script"

omitted=$(comm -23 "$scratch/fx.script" "$scratch/stale.script" | tr '\n' ' ' | sed 's/ *$//')
same "the eight-item list of 2026-09-06 is reported as seven omissions, by name" \
    "tools/assemble-site.sh tools/build-declaration-fixtures.sh tools/check-site-console.py tools/check-site-fragments.py tools/engine-readme-fixtures.sh tools/readme-fixtures.sh tools/site-console-fixtures.sh" \
    "$omitted"
same "  and the eight it did name are not reported as extras" \
    "" "$(comm -13 "$scratch/fx.script" "$scratch/stale.script" | tr '\n' ' ' | sed 's/ *$//')"
same "  and a path named in prose rather than in a code block is a mention, not a claim" \
    "" "$(grep -c 'fixtures-live' "$scratch/stale.script" | sed 's/^0$//')"

# The same stale page against the LIVE workflow, stated as a relative
# invariant rather than a count: CI has only grown since that day, so those
# seven must still be reported, and an absolute number written here would go
# stale the next time a gate lands.
still=$(comm -23 "$scratch/ci.script" "$scratch/stale.script" | tr '\n' ' ')
missing_now=""
for s in tools/assemble-site.sh tools/build-declaration-fixtures.sh \
    tools/check-site-console.py tools/check-site-fragments.py \
    tools/engine-readme-fixtures.sh tools/readme-fixtures.sh \
    tools/site-console-fixtures.sh; do
    case " $still " in
        *" $s "*) ;;
        *) missing_now="$missing_now $s" ;;
    esac
done
same "  and the live workflow still reports every one of those seven" "" "$missing_now"

# The other direction: a page naming a gate that CI does not run.
invented="$scratch/invented.md"
{
    echo '## What CI runs'
    echo
    echo '    sh tools/nonexistent-fixtures.sh'
} > "$invented"
page_text "$invented" "$section" | members | side script > "$scratch/inv.script"
same "a gate the page invents is reported in the other direction" \
    "tools/nonexistent-fixtures.sh" \
    "$(comm -13 "$scratch/fx.script" "$scratch/inv.script" | tr '\n' ' ' | sed 's/ *$//')"

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
