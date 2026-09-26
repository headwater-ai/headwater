#!/bin/sh
# What holds the terminal-sensing promise every interface contract states.
#
# Seventeen pages under `docs/interfaces/` carry one row:
#
#     | `--no-color` | Force plain text on both streams: ... The default already
#     senses whether each stream is a terminal, and renders color only there. |
#
# Nothing in this repository could observe that promise being kept.
# [HW-DR-0045](../../docs/decisions/0045-coloring-the-cli-and-where-the-banner-goes.md)
# rules that there is no `--color=always`, so no piped run can ever see an
# escape sequence, and every recorded fixture and every test in `engine/` runs
# headless. A renderer that takes a `ColorMode` and a call site that hands it
# `ColorMode::Plain` forever passes all of them. That is the defect this suite
# exists to catch, and it caught it: `route` and `generate --check` both
# rendered zero escape sequences under a real terminal while their contracts
# said otherwise.
#
# HW-DR-0045 forbids a *flag* that forces color into a pipe. It says nothing
# about a test attaching a terminal, so this suite attaches one:
# `script -qec "<command>" /dev/null` from util-linux runs the engine on a
# pseudo-terminal and writes what it emitted. No decision record has to move and
# no Rust dependency is added.
#
# Run it from anywhere:
#     sh tools/engine/color-fixtures.sh
#
# # What each case asserts, and why all four arms are needed
#
# Per surface, four runs:
#
#   1. under a pty, at least one escape byte — the arm that fails on a call site
#      wired to `Plain`, and the only one that can;
#   2. through a pipe, zero — the arm HW-DR-0045 protects, held here as well as
#      by the substring assertions in `engine/crates/cli/tests/width.rs`;
#   3. under a pty with `NO_COLOR=1`, zero;
#   4. under a pty with `--no-color`, zero.
#
# Arm 1 alone would pass a renderer that ignores its mode. Arms 2 to 4 alone are
# what the tree already had, and they passed a binary that emitted no color at
# all. Neither half means anything without the other.
#
# # The fifth arm, on the surfaces that admit it
#
# `strips_to_the_plain_bytes` runs the same command twice, takes the SGR
# sequences and the pty's carriage returns back off the terminal run, and
# compares the two byte for byte. Four arms say that color arrives and leaves
# when it should; this one says that nothing else moved when it did.
#
# It is the arm that catches painting before folding. `headwater_check::fill`
# measures a line in characters, so a `\033[35m` introduced before the fill is
# nine characters of text as far as the fold is concerned, and every break after
# it lands one word early. Nothing else in this repository can see that: the
# painted report and the plain report are each internally consistent, both are
# 80 columns wide, and no test compares them. A renderer that folds first and
# substitutes the painted token afterwards passes this arm, and that is the
# order `Route::render` and `paint::painted_row` both take.
#
# Standard error is dropped inside the pty command rather than outside it, so
# that the count is over the verb's own report. `err()` already colors every
# refusal on standard error under a terminal, and a pty gives both streams one,
# so a run that kept stderr would pass arm 1 on a refusal alone.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)

# Either profile builds the engine, and the newer answers. This is the rule
# `.githooks/fixtures.sh` and `.githooks/pre-commit` state for themselves; the
# three copies exist because the git half of this repository answers to no
# harness and must not reach into `.claude/` for a shared one.
release_engine="$root/engine/target/release/headwater"
dev_release_engine="$root/engine/target/dev-release/headwater"
engine=$release_engine
if [ -x "$dev_release_engine" ] && { [ ! -x "$engine" ] || [ "$dev_release_engine" -nt "$engine" ]; }; then
    engine=$dev_release_engine
fi

if [ ! -x "$engine" ]; then
    echo "no built engine of either profile, so nothing here can run."
    echo "  cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked"
    exit 1
fi

# A check that cannot run reads as a check that passes, so the skip is printed
# and it names what is missing. It is not a failure: a host with no util-linux
# `script` is a host this suite has nothing to say about, and CI runs on
# `["self-hosted", "headwater"]`, which has one.
if ! command -v script >/dev/null 2>&1; then
    echo "SKIP: no util-linux \`script\` on this host, so no pseudo-terminal can be attached."
    echo "  every case below is unrun, and this suite asserts nothing about this tree."
    exit 0
fi

passed=0
failed=0

# One run on a pseudo-terminal, with standard error dropped inside it, and the
# count of escape bytes on what the verb wrote to standard output.
# `$2` is an environment prefix, carried inside the string `script` runs rather
# than in front of this function. `VAR=value some_function` is defined by POSIX
# to leave `VAR` set in the calling shell, so a `NO_COLOR=1` written that way
# would silence every case after it and turn the arm above green for the wrong
# reason.
escapes_on_a_terminal() {
    script -qec "${2-} $1 2>/dev/null" /dev/null 2>/dev/null | grep -c "$(printf '\033')"
}

# The same command through a pipe. `grep -c` over a stream with no match exits
# 1, which `set -u` does not catch and a bare command substitution swallows, so
# the count is read and never the status.
escapes_through_a_pipe() {
    sh -c "$1 2>/dev/null" | grep -c "$(printf '\033')"
}

judge_at_least_one() {
    name=$1 got=$2
    if [ "$got" -ge 1 ]; then
        printf 'ok   %s\n' "$name"
        passed=$((passed + 1))
    else
        printf 'FAIL %s\n  expected at least one escape sequence under a terminal, got %s\n' "$name" "$got"
        failed=$((failed + 1))
    fi
}

judge_none() {
    name=$1 got=$2
    if [ "$got" -eq 0 ]; then
        printf 'ok   %s\n' "$name"
        passed=$((passed + 1))
    else
        printf 'FAIL %s\n  expected no escape sequence, got %s\n' "$name" "$got"
        failed=$((failed + 1))
    fi
}

# The whole promise, over one surface. `label` names the surface a reader knows
# it by and `command` is the invocation, already carrying `--root`.
senses_its_terminal() {
    label=$1
    command=$2
    judge_at_least_one "$label colors under a terminal" "$(escapes_on_a_terminal "$command")"
    judge_none "$label is plain through a pipe" "$(escapes_through_a_pipe "$command")"
    judge_none "$label is plain under a terminal with NO_COLOR=1" \
        "$(escapes_on_a_terminal "$command" 'NO_COLOR=1')"
    judge_none "$label is plain under a terminal with --no-color" \
        "$(escapes_on_a_terminal "$command --no-color")"
}

# The painted run and the plain run, with the color taken back off the first.
#
# The pty converts every `\n` it carries into `\r\n`, so the carriage return
# goes with the SGR sequence: it is the terminal's, not the renderer's, and a
# comparison that kept it would fail on every surface for a reason that has
# nothing to do with color.
strips_to_the_plain_bytes() {
    label=$1
    command=$2
    painted="${TMPDIR:-/tmp}/headwater-color-painted.$$"
    plain="${TMPDIR:-/tmp}/headwater-color-plain.$$"
    script -qec "$command 2>/dev/null" /dev/null 2>/dev/null \
        | sed -e "s/$(printf '\033')\[[0-9;]*m//g" -e 's/\r$//' >"$painted"
    sh -c "$command 2>/dev/null" >"$plain"
    if cmp -s "$painted" "$plain"; then
        printf 'ok   %s strips to the bytes it writes through a pipe\n' "$label"
        passed=$((passed + 1))
    else
        printf 'FAIL %s strips to the bytes it writes through a pipe\n' "$label"
        printf '  a break moved when the color arrived, so something painted before it folded:\n'
        diff "$painted" "$plain" | sed -n '1,8p' | sed 's/^/    /'
        failed=$((failed + 1))
    fi
    rm -f "$painted" "$plain"
}

# One line of a report, under a pty: at least one line that matches `pattern`
# carries an escape byte. It is for a report that composes two renderers, where
# the four arms above would pass on the first renderer's color alone while the
# second rendered in `ColorMode::Plain`.
paints_the_line() {
    label=$1
    command=$2
    pattern=$3
    got=$(script -qec "$command 2>/dev/null" /dev/null 2>/dev/null \
        | grep -F "$pattern" | grep -c "$(printf '\033')")
    if [ "$got" -ge 1 ]; then
        printf 'ok   %s paints the line that reads "%s"\n' "$label" "$pattern"
        passed=$((passed + 1))
    else
        printf 'FAIL %s paints the line that reads "%s"\n  expected at least one escape sequence on it under a terminal, got %s\n' \
            "$label" "$pattern" "$got"
        failed=$((failed + 1))
    fi
}

# The set is measured rather than listed. Every surface below writes bytes of
# its own report to standard output, its interface contract carries the sensing
# row, and it is not a machine format. `headwater completions`, `--format json`
# and the MCP server are excluded on the last of those and stay plain whatever
# stream they reach; `export --list`, `gate` and the `taxonomy` second words
# write nothing to standard output without arguments this suite would have to
# invent.
cd "$root" || exit 1

# The two this change wired, and the reason it exists. Both rendered zero
# escapes under a terminal before it.
senses_its_terminal 'route' "$engine route 'add rate limiting' --root ."
senses_its_terminal 'generate --check' "$engine generate --check --root ."

# The three that already sensed their terminal, held here against a regression.
# A later change that threads a mode through a fourth surface and drops one of
# these would otherwise land green.
senses_its_terminal 'check' "$engine check --root ."
senses_its_terminal 'explain' "$engine explain HW-DR-0045 --root ."
senses_its_terminal 'sweep plan' "$engine sweep plan --root ."

# The three this change wired, and the reason it exists. All three rendered
# zero escapes under a terminal before it, measured on the parent commit.
#
# `infer` is run without `--write`, so it reads the tree and writes nothing.
# `--owner` is passed because the report names an owner in the payload it
# prints, and a run without one prints a placeholder rather than refusing.
senses_its_terminal 'infer' "$engine infer --owner 'a color fixture' --root ."
senses_its_terminal 'capture' "$engine capture --root ."
senses_its_terminal 'conformance' "$engine conformance --root ."

# The two this change wired, and the reason it exists. Both rendered zero
# escapes under a terminal before it, measured on the parent commit `58f46a8d`,
# where every other surface on this page was already green. Both carry the
# sensing row in their interface contract, `docs/interfaces/headwater-taxonomy.md`
# and `docs/interfaces/headwater-probe.md`, and neither needs an argument this
# suite would have to invent.
senses_its_terminal 'taxonomy audit' "$engine taxonomy audit --root ."
senses_its_terminal 'probe plan' "$engine probe plan --root ."

# The one this change wired, and the first surface on this page that renders below
# `headwater-check`. `derived` composes its report in `headwater-census`, which
# `headwater-check` depends on, so the palette could not reach it at all while the
# primitives lived in `headwater-check`. It rendered zero escapes under a terminal
# before the move, measured on the parent commit, and it needs no argument this
# suite would have to invent: `derived --root .` reads the tree and writes nothing.
senses_its_terminal 'derived' "$engine derived --root ."

# Three of the eleven #479 wired, and the three whose input this suite can
# synthesize from the tree in front of it rather than from an external
# artifact. `taxonomy validate` and `taxonomy resolve --check` read this
# repository's own sources and lock, and write nothing. `gate` needs a read
# set to hold against a tree, and `headwater check --read-set` is what writes
# one — so this is a two-step case rather than a bare `senses_its_terminal`
# line, and the read set is generated fresh into a temp file rather than
# committed, because a read set is a hash of the tree it was taken over and
# goes stale the moment the tree that produced it moves.
senses_its_terminal 'taxonomy validate' "$engine taxonomy validate --root ."
senses_its_terminal 'taxonomy resolve --check' "$engine taxonomy resolve --check --root ."
gate_read_set="${TMPDIR:-/tmp}/headwater-color-gate.$$.readset"
if "$engine" check --read-set "$gate_read_set" --root . >/dev/null 2>&1; then
    senses_its_terminal 'gate' "$engine gate --read-set $gate_read_set --root ."
else
    echo "SKIP: gate — \`headwater check --read-set\` did not write one, so gate has nothing to hold"
fi
rm -f "$gate_read_set"

# `taxonomy diff`, which #1018 wired. It needs a published artifact to compare
# the lock against, and `taxonomy publish --from` writes one from the maintained
# source into a temp directory and nothing into this tree. The artifact is this
# repository's own package at the version the lock already holds, so the report
# says every dimension is preserved and carries no migration payload: the payload
# section and a BROKEN dimension are held by the palette unit tests beside
# `Report::render` and `Accounting::render` in `headwater-compat`, not here.
# The report pads each dimension name into a column, so the fifth arm below is
# what says the color did not shift it.
diff_artifact_dir=$(mktemp -d "${TMPDIR:-/tmp}/headwater-color-diff.XXXXXX")
diff_artifact="$diff_artifact_dir/published"
if "$engine" taxonomy publish --from taxonomy-source/headwater-standard \
    --out "$diff_artifact" --root . >/dev/null 2>&1; then
    senses_its_terminal 'taxonomy diff' "$engine taxonomy diff $diff_artifact --root ."
    strips_to_the_plain_bytes 'taxonomy diff' "$engine taxonomy diff $diff_artifact --root ."
else
    echo "FAIL taxonomy diff — \`taxonomy publish --from\` wrote no artifact to compare against"
    failed=$((failed + 1))
fi
rm -rf "$diff_artifact_dir"

# The verbs #1017 wired and could only unit-test, #1019. Every one of them
# writes, or reads an input this repository does not keep in a runnable state,
# so each runs against a scratch directory built here and removed at the end.
# Nothing below writes into `$root`: a verb that writes gets `--root` or `--out`
# inside `$scratch`, and `git status` in `$root` is the same after the suite as
# before it.
#
# An arm that has to write somewhere new on every run writes to a path that
# carries `\$\$`. The command string is run by `script` or by `sh -c`, so that
# `$$` is the pid of the inner shell, and it is a new one for each of the four
# arms.
scratch=$(mktemp -d "${TMPDIR:-/tmp}/headwater-color-scratch.XXXXXX")

# `taxonomy publish` refuses an output directory that holds files already, so
# each arm publishes into a directory of its own.
senses_its_terminal 'taxonomy publish' \
    "$engine taxonomy publish --from taxonomy-source/headwater-standard --out $scratch/publish.\$\$ --root ."

# `taxonomy vendor` installs under `.headwater/packages/` of its `--root`, so its
# root is a copy of this repository's `.headwater/` and nothing else. The
# artifact is the same package at the version that copy pins, and a second
# install of the pinned bytes reports them again rather than refusing, so one
# copy serves all four arms.
mkdir "$scratch/vendor-root"
cp -R .headwater "$scratch/vendor-root/.headwater"
if "$engine" taxonomy publish --from taxonomy-source/headwater-standard \
    --out "$scratch/pinned" --root . >/dev/null 2>&1; then
    senses_its_terminal 'taxonomy vendor' \
        "$engine taxonomy vendor $scratch/pinned --root $scratch/vendor-root"
else
    echo "FAIL taxonomy vendor — \`taxonomy publish --from\` wrote no artifact to install"
    failed=$((failed + 1))
fi

# `taxonomy migrate` and the payload branch of `taxonomy diff` need an artifact
# at a later major version that ships a payload covering the lock's version.
# The candidate is this repository's own source with both version fields moved
# to 5.0.0, `contents.bundles` pointed back at the library inside this tree
# (publishing refuses a bundle library outside the tree it reads), and one
# payload of one step. The step names a kind no document carries, so it is a
# no-op here and `migrate` without `--apply` reports it and writes nothing.
# The payload has to have a step, because `publish` refuses an empty one.
payload_source="$scratch/payload-source"
cp -R taxonomy-source/headwater-standard "$payload_source"
sed -i -e 's/^version: .*/version: 5.0.0/' \
    -e "s|^  bundles: ../../docs/taxonomies\$|  bundles: $root/docs/taxonomies\\
  migrations: migrations|" "$payload_source/package.yml"
sed -i -e 's/^version: .*/version: 5.0.0/' "$payload_source/taxonomy.yml"
mkdir "$payload_source/migrations"
printf '%s\n' 'migration:' '  format: 1' '  from: ">=4 <5"' '  to: ">=5 <6"' '' 'steps:' \
    '  - subject: kind' '    from: a_kind_no_document_carries' '    to: [decision]' \
    '    because: >-' '      A color fixture needs a payload with one step in it.' \
    >"$payload_source/migrations/4-to-5.yml"
if "$engine" taxonomy publish --from "$payload_source" \
    --out "$scratch/payload" --root . >/dev/null 2>&1; then
    senses_its_terminal 'taxonomy migrate' "$engine taxonomy migrate $scratch/payload --root ."
    # The diff over this artifact colors its header whatever the payload
    # branch does, so the four arms alone would pass a payload account rendered
    # in `ColorMode::Plain`. The line check below reads the account's own
    # heading.
    senses_its_terminal 'taxonomy diff, payload account' \
        "$engine taxonomy diff $scratch/payload --root ."
    paints_the_line 'taxonomy diff, payload account' \
        "$engine taxonomy diff $scratch/payload --root ." 'migration payload'
else
    echo "FAIL taxonomy migrate — \`taxonomy publish --from\` wrote no artifact with a payload"
    failed=$((failed + 2))
fi

# The branch of `taxonomy diff` where the candidate does not resolve under this
# repository's overlays. The candidate declares `identifier_schemes.spec_id`,
# which `.headwater/overlay.yml` also adds, and an `add` over a declared value is
# a collision that stops resolution. The version is a minor one above the lock,
# so the report is about the collision and not about the transition. Every
# byte of standard output in this branch is `Report::render`'s, so the four arms
# are about that call site alone.
unresolved_source="$scratch/unresolved-source"
cp -R taxonomy-source/headwater-standard "$unresolved_source"
sed -i -e 's/^version: .*/version: 4.99.0/' \
    -e "s|^  bundles: ../../docs/taxonomies\$|  bundles: $root/docs/taxonomies|" \
    "$unresolved_source/package.yml"
sed -i -e 's/^version: .*/version: 4.99.0/' \
    -e 's|^identifier_schemes:$|&\
  spec_id: {pattern: "{namespace}-SPEC-{slug}", allocation: minted-once}|' \
    "$unresolved_source/taxonomy.yml"
if "$engine" taxonomy publish --from "$unresolved_source" \
    --out "$scratch/unresolved" --root . >/dev/null 2>&1; then
    senses_its_terminal 'taxonomy diff, unresolved candidate' \
        "$engine taxonomy diff $scratch/unresolved --root ."
else
    echo "FAIL taxonomy diff, unresolved candidate — \`taxonomy publish --from\` wrote no artifact"
    failed=$((failed + 1))
fi

# `new` writes a document and claims an identifier, so it runs against a copy
# of the committed tree. Each run mints the next identifier, so the four arms
# write four documents into one copy and none of them collides with another.
new_root="$scratch/new-root"
mkdir "$new_root"
if git archive HEAD | tar -x -C "$new_root"; then
    senses_its_terminal 'new' \
        "$engine new decision --title 'A color fixture writes this decision' --summary 'One sentence.' --root $new_root"
else
    echo "FAIL new — \`git archive HEAD\` wrote no copy of the tree to scaffold into"
    failed=$((failed + 1))
fi

# `probe record` and `probe grade` read a transcript. The committed transcripts
# under `docs/probe-runs/` are each pinned to a lock this tree no longer
# carries, so both verbs refuse them in one plain sentence. The transcript here
# is the latest committed one with the four digests of its run identity
# replaced by the ones `probe plan` prints over this tree now. It is written
# outside the tree, so the tree digest it states stays true.
transcript="$scratch/transcript.md"
identity=$("$engine" probe plan --root . 2>/dev/null)
sed -e "s/^lock: sha256:.*/$(printf '%s\n' "$identity" | grep '^lock: sha256:')/" \
    -e "s/^tree: sha256:.*/$(printf '%s\n' "$identity" | grep '^tree: sha256:')/" \
    -e "s/^selection: sha256:.*/$(printf '%s\n' "$identity" | grep '^selection: sha256:')/" \
    -e "s/^read_set: sha256:.*/$(printf '%s\n' "$identity" | grep '^read_set: sha256:')/" \
    docs/probe-runs/regression-probe-transcript-for-2026-09-17-after-the-probe-corrections.md \
    >"$transcript"
senses_its_terminal 'probe record' "$engine probe record $transcript --root ."
senses_its_terminal 'probe grade' "$engine probe grade $transcript --root ."

# `probe stale` reads the transcripts committed in the tree it is given, and it
# writes one section for each of them. `probe_stale` paints the `## The result
# of` frame around each section, so the four arms alone would pass a
# `Staleness::render` call site that states `ColorMode::Plain`. The two line
# checks below read lines that only `Staleness::render` writes. Every committed
# transcript is pinned to a lock this tree no longer carries, so over the
# committed tree the report holds only the sentence of the unusable verdict. So
# the case runs over a copy of the tree with one more transcript in it, the one
# above with the digests `probe plan` prints over the copy. That transcript
# reads, and its section writes a member line. The file moves the tree digest
# the transcript states, and nothing here reads that digest.
stale_root="$scratch/stale-root"
mkdir "$stale_root"
if git archive HEAD | tar -x -C "$stale_root"; then
    stale_identity=$("$engine" probe plan --root "$stale_root" 2>/dev/null)
    sed -e "s/^lock: sha256:.*/$(printf '%s\n' "$stale_identity" | grep '^lock: sha256:')/" \
        -e "s/^tree: sha256:.*/$(printf '%s\n' "$stale_identity" | grep '^tree: sha256:')/" \
        -e "s/^selection: sha256:.*/$(printf '%s\n' "$stale_identity" | grep '^selection: sha256:')/" \
        -e "s/^read_set: sha256:.*/$(printf '%s\n' "$stale_identity" | grep '^read_set: sha256:')/" \
        docs/probe-runs/regression-probe-transcript-for-2026-09-17-after-the-probe-corrections.md \
        >"$stale_root/docs/probe-runs/color-fixture-transcript.md"
    senses_its_terminal 'probe stale' "$engine probe stale --root $stale_root"
    paints_the_line 'probe stale, unusable verdict' "$engine probe stale --root $stale_root" \
        'Nothing here decides whether this result is stale'
    paints_the_line 'probe stale, member line' "$engine probe stale --root $stale_root" \
        '(probe) sha256:'
else
    echo "FAIL probe stale — \`git archive HEAD\` wrote no copy of the tree to record into"
    failed=$((failed + 1))
fi

rm -rf "$scratch"

# One report surface stays outside this page, on a measured reason.
#
# `import` needs a snapshot directory with a release record over it and an
# import block in `.headwater/taxonomy.yml` pinned to that record's digest. No
# verb of this engine writes a release record over an arbitrary directory:
# `headwater-import`'s own fixtures compute one with
# `headwater_resolve::release::compute`, from Rust. A shell copy of that
# computation here would be a second implementation of the digest, and a wrong
# one would fail the case for a reason that has nothing to do with color. It is
# held by the palette unit test beside its renderer in `headwater-import`.

# The help family, which is four templates rather than one. The root screen is
# written by `first_screen`, a verb page is `clap`'s own `{options}` renderer, a
# verb with second words is `second_words`, and `headwater help <verb>` reaches
# a page through `print_help_for` rather than through the parse. `clap` decides
# what to strip from all four at write time, off the one `ColorChoice`
# `paint::color_choice` hands the tree, so a mistake there is a mistake on every
# page — and only running each of them says whether it was made.
#
# `headwater --help` is the one surface whose arm 1 would pass without any of
# this: it has printed a masthead by plain I/O since HW-DR-0045, ahead of
# `clap`'s writer. It is here for arms 2 to 4, which the masthead does not
# answer for, and the pages below are what arm 1 is evidence about.
senses_its_terminal 'headwater --help' "$engine --help"
senses_its_terminal 'check --help' "$engine check --help"
senses_its_terminal 'help check' "$engine help check"
senses_its_terminal 'taxonomy --help' "$engine taxonomy --help"

# The fifth arm. Every surface here composes a report or a page and folds it,
# and the fold is where a paint that ran too early shows up. `taxonomy audit`
# folds at two call sites through `headwater_check::filled`, so it belongs here.
# `probe plan` calls no fold, writes one line per fact and is absent for that
# reason rather than by oversight. `derived` is absent for the same reason and it
# is now the second: `Population::render` reaches nothing in `headwater_check::fill`,
# and `headwater-census` cannot depend on `headwater-check` to reach one.
strips_to_the_plain_bytes 'taxonomy audit' "$engine taxonomy audit --root ."
strips_to_the_plain_bytes 'infer' "$engine infer --owner 'a color fixture' --root ."
strips_to_the_plain_bytes 'capture' "$engine capture --root ."
strips_to_the_plain_bytes 'conformance' "$engine conformance --root ."
strips_to_the_plain_bytes 'headwater --help' "$engine --help"
strips_to_the_plain_bytes 'check --help' "$engine check --help"
strips_to_the_plain_bytes 'help check' "$engine help check"

# The machine formats, which sense nothing on purpose. `--format json` reaches
# the same renderers through `main.rs`'s `Format::Text => stdout_color(), _ =>
# ColorMode::Plain`, so a change that widened the sensing to a format a program
# parses would break a consumer rather than a reader.
judge_none 'route --json stays plain under a terminal' \
    "$(escapes_on_a_terminal "$engine route 'add rate limiting' --json --root .")"

# Completions must stay plain under a pty, because a shell script with escape
# bytes baked in is a broken script. It is a machine format like json, and it is
# the one that has to be run under a terminal to be evidence: `main`'s
# `completions` builds from the same tree the help pages are printed from, so
# the palette is one line away from it, and every case in
# `engine/crates/cli/tests/width.rs` runs headless. What holds it is that the
# call site states `ColorMode::Plain` rather than sensing a stream. Nothing
# about the sensing itself is being relied on here.
judge_none 'completions stays plain under a terminal' \
    "$(escapes_on_a_terminal "$engine completions bash --root .")"

printf '\n%s passed, %s failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
