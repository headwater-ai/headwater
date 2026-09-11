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
