#!/bin/sh
# What holds the tools a campaign runs before it spends: `tools/probe/ablate.sh`,
# the leak check of `tools/probe/seal.sh`, `tools/probe/recordings.py`, the dry
# runs of `tools/probe/campaign-dry-run.sh` and `tools/probe/additive-dry-run.sh`,
# the layer gate of the tuning round, and `tools/probe/phrasing.sh`.
#
#     sh tools/probe/campaign-fixtures.sh
#
# These cases were the second half of `tools/probe/probe-record-fixtures.sh`
# until 2026-11-22. That suite then took 22 minutes on `ubuntu-latest`, and
# 82% of the time went to about 50 cases here, each of which rebuilt the same
# trees from a 90 MB `git archive`. So this suite sets
# `HW_CAMPAIGN_TREE_CACHE` for every dry run it starts, and the first dry run
# of a set of inputs builds the trees that every later one with the same
# inputs reads. `tools/probe/dry-run-trees.sh` states the key, and the cases
# of "The trees of a dry run" below hold that a changed input misses it.
#
# Its batch cases start sessions through the recorder's confinement, so CI
# runs it through `tools/probe/ci-confine.sh` as it runs the recorder's suite.
# It needs `jq`, exits 3 without it so CI can skip with a warning, and writes
# only under a temporary directory.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
driver="$root/tools/probe/probe-record.sh"
# The newer of the two profiles answers, by the rule in
# `tools/repo/resolve-engine.sh` (#990). With neither built, `engine` names a
# path that is not executable, and each case that needs one skips on that.
. "$root/tools/repo/resolve-engine.sh"
engine=$(hw_resolve_engine_bin "$root") || engine="$root/engine/target/release/headwater"

if ! command -v jq >/dev/null 2>&1; then
    echo "no \`jq\` on the path, and the tools under test read JSON with it." >&2
    exit 3
fi

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

passed=0
failed=0
. "$root/tools/probe/fixture-cases.sh"

HW_CAMPAIGN_TREE_CACHE=$scratch/trees
export HW_CAMPAIGN_TREE_CACHE
printf 'answer the question\n' > "$scratch/task.md"

# ---------------------------------------------------------------------------
# `tools/probe/ablate.sh`, which produces an absent-arm tree from the tier's
# declared ablation (#1010).
#
# Two tiers remove two different trees, and the difference between them is
# the only thing that measures what the documents under `docs/` do. So the
# cases hold both ablations against the checkout's own `.headwater/probe.yml`,
# the refusal of a tier with no absent arm, and the refusal of an entry that
# would take `rm -rf` outside the workspace. Every refusal is asserted to
# leave the workspace untouched.
# ---------------------------------------------------------------------------
ablate="$root/tools/probe/ablate.sh"
fresh_workspace() {
    rm -rf "$scratch/ablate-ws"
    mkdir -p "$scratch/ablate-ws/.claude" "$scratch/ablate-ws/.githooks" \
        "$scratch/ablate-ws/.headwater" "$scratch/ablate-ws/docs/spec" \
        "$scratch/ablate-ws/docs/probes" "$scratch/ablate-ws/docs/probe-runs" \
        "$scratch/ablate-ws/engine/crates/census/fixtures" \
        "$scratch/ablate-ws/site/tutorial" "$scratch/ablate-ws/tools"
    : > "$scratch/ablate-ws/CLAUDE.md"
    : > "$scratch/ablate-ws/.headwater/export.json"
    : > "$scratch/ablate-ws/.headwater/taxonomy.lock"
    : > "$scratch/ablate-ws/docs/spec/05.md"
    : > "$scratch/ablate-ws/docs/probes/p.md"
    : > "$scratch/ablate-ws/engine/crates/census/fixtures/corpus.census"
    : > "$scratch/ablate-ws/site/tutorial/index.html"
    : > "$scratch/ablate-ws/site/index.html"
}
kept() {
    # $1 name, then the paths that must still exist, then `--`, then the
    # paths that must be gone.
    name=$1
    shift
    missing=""
    while [ "$#" -gt 0 ] && [ "$1" != "--" ]; do
        [ -e "$scratch/ablate-ws/$1" ] || missing="$missing $1"
        shift
    done
    [ "$#" -gt 0 ] && shift
    stayed=""
    for gone in "$@"; do
        [ -e "$scratch/ablate-ws/$gone" ] && stayed="$stayed $gone"
    done
    if [ -z "$missing$stayed" ]; then
        pass "$name"
    else
        fail "$name" "removed:${missing:- nothing wrong}; kept:${stayed:- nothing wrong}"
    fi
}

fresh_workspace
sh "$ablate" campaign "$scratch/ablate-ws" >/dev/null 2>"$scratch/ablate.err"
same "ablate.sh produces the campaign's absent arm" "0" "$?"
kept "the campaign removes the four governance paths and the instrument, and keeps the rest of docs/" \
    docs docs/spec/05.md engine/crates/census/fixtures/corpus.census site/tutorial/index.html tools \
    -- CLAUDE.md .claude .githooks .headwater docs/probes docs/probe-runs

fresh_workspace
sh "$ablate" documentation "$scratch/ablate-ws" >/dev/null 2>"$scratch/ablate.err"
same "ablate.sh produces the documentation tier's absent arm" "0" "$?"
kept "the documentation tier removes the four governance paths, docs/ and the two copies outside it" \
    engine/crates/census/fixtures site/index.html tools \
    -- CLAUDE.md .claude .githooks .headwater docs \
    engine/crates/census/fixtures/corpus.census site/tutorial/index.html

fresh_workspace
sh "$ablate" --present "$scratch/ablate-ws" >/dev/null 2>"$scratch/ablate.err"
same "ablate.sh produces a present arm" "0" "$?"
kept "a present arm loses the instrument and nothing else" \
    CLAUDE.md .claude .githooks .headwater/taxonomy.lock docs/spec/05.md \
    engine/crates/census/fixtures/corpus.census site/tutorial/index.html \
    -- docs/probes docs/probe-runs .headwater/export.json

same "ablate.sh lists the instrument the checkout declares" \
    "docs/probes docs/probe-runs docs/probe-results .headwater/export.json" \
    "$(sh "$ablate" --instrument | tr '\n' ' ' | sed 's/ $//')"

printf 'tiers:\n  regression:\n    budget_cents: 1\n    session_cost_cents: 1\n    repetitions: 1\n    arms: [present]\n' \
    > "$scratch/no-instrument.yml"
fresh_workspace
HW_PROBE_YML="$scratch/no-instrument.yml" sh "$ablate" --present "$scratch/ablate-ws" \
    >/dev/null 2>"$scratch/ablate.err"
same "a present arm with no instrument declared exits 0" "0" "$?"
kept "and removes nothing, and never the workspace itself" \
    CLAUDE.md .claude docs docs/probes engine tools --

# The engine reads a declaration whole or refuses it, so each declaration
# below is a complete one.
printf 'instrument: [docs/probes] # the shelf\ntiers:\n  campaign:\n    budget_cents: 100\n    session_cost_cents: 25\n    repetitions: 58\n    arms: [present, absent]\n    ablation: [docs] # the documents\n' \
    > "$scratch/commented.yml"
same "ablate.sh reads past a trailing YAML comment, as the engine does" "docs/probes" \
    "$(HW_PROBE_YML="$scratch/commented.yml" sh "$ablate" --instrument 2>&1)"

# A block sequence at four spaces is YAML the engine reads. The hand-written
# reader this script carried matched two spaces alone, printed nothing and
# exited 0, so every arm kept the answer key (#1472, clause 8).
printf 'instrument:\n    - docs/probes\n    - docs/probe-runs\ntiers:\n  campaign:\n    budget_cents: 100\n    session_cost_cents: 25\n    repetitions: 58\n    arms: [present, absent]\n    ablation:\n        - docs\n' \
    > "$scratch/indented.yml"
same "ablate.sh reads a four-space instrument block, as the engine does" "docs/probes docs/probe-runs" \
    "$(HW_PROBE_YML="$scratch/indented.yml" sh "$ablate" --instrument | tr '\n' ' ' | sed 's/ $//')"
fresh_workspace
HW_PROBE_YML="$scratch/indented.yml" sh "$ablate" campaign "$scratch/ablate-ws" \
    >/dev/null 2>"$scratch/ablate.err"
same "ablate.sh produces an absent arm from a four-space declaration" "0" "$?"
kept "and removes the instrument and the ablation it declares" \
    CLAUDE.md .claude .headwater engine tools -- docs

# A declaration the engine refuses is not an empty instrument: `--instrument`
# fails, and a caller that loops over its output fails closed on the status.
printf 'instrument: [docs/probes, ../outside]\ntiers:\n  campaign:\n    budget_cents: 100\n    session_cost_cents: 25\n    repetitions: 58\n    arms: [present, absent]\n    ablation: [docs]\n' \
    > "$scratch/refused-instrument.yml"
HW_PROBE_YML="$scratch/refused-instrument.yml" sh "$ablate" --instrument \
    >"$scratch/ablate.out" 2>"$scratch/ablate.err"
same "ablate.sh --instrument fails on a declaration the engine refuses" "2" "$?"
same "and prints no path" "" "$(cat "$scratch/ablate.out")"

# With no built engine there is nothing to read the declaration with, and the
# script refuses rather than print an empty instrument. A copy of the script
# under a root with no `engine/target` stands for a checkout that never built.
mkdir -p "$scratch/unbuilt/tools/probe"
cp "$ablate" "$scratch/unbuilt/tools/probe/ablate.sh"
HW_PROBE_YML="$scratch/commented.yml" sh "$scratch/unbuilt/tools/probe/ablate.sh" --instrument \
    >"$scratch/ablate.out" 2>"$scratch/ablate.err"
same "ablate.sh --instrument with no built engine exits nonzero" "2" "$?"
same "and prints no path" "" "$(cat "$scratch/ablate.out")"
present "and it names the build" "Build it first" "$scratch/ablate.err"

# The two refusals of the driver below are asserted with a stub harness first
# on PATH. It records that it ran and exits nonzero, so a refusal that
# regresses shows up as a marker file here and never as a paid session.
mkdir -p "$scratch/refuse-bin"
cat > "$scratch/refuse-bin/claude" <<STUB
#!/bin/sh
: > "$scratch/harness-ran"
exit 1
STUB
chmod +x "$scratch/refuse-bin/claude"

fresh_workspace
rm -f "$scratch/harness-ran"
PATH="$scratch/refuse-bin:$PATH" sh "$driver" --probe PROBE-FIX-opened --session x \
    --task-file "$scratch/task.md" --workspace "$scratch/ablate-ws" \
    >/dev/null 2>"$scratch/driver-instrument.err"
same "the driver refuses a workspace that still holds the instrument" "8" "$?"
present "and it names the path and the command that prepares the arm" \
    "sh tools/probe/ablate.sh --present" "$scratch/driver-instrument.err"
same "and no session started" "no" "$([ -e "$scratch/harness-ran" ] && echo yes || echo no)"

fresh_workspace
rm -f "$scratch/harness-ran"
HW_PROBE_YML="$scratch/no-such-probe.yml" PATH="$scratch/refuse-bin:$PATH" sh "$driver" \
    --probe PROBE-FIX-opened --session x --task-file "$scratch/task.md" \
    --workspace "$scratch/ablate-ws" >/dev/null 2>"$scratch/driver-unread.err"
same "the driver fails closed when the instrument cannot be read" "8" "$?"
same "and no session started" "no" "$([ -e "$scratch/harness-ran" ] && echo yes || echo no)"

fresh_workspace
sh "$ablate" regression "$scratch/ablate-ws" >/dev/null 2>"$scratch/ablate.err"
same "ablate.sh refuses the regression tier, which runs no absent arm" "2" "$?"
present "and it names the tier" "\`regression\` tier does not run the \`absent\` arm" "$scratch/ablate.err"
kept "and the refused workspace is untouched" \
    CLAUDE.md .claude .githooks .headwater docs docs/probes engine tools --

fresh_workspace
sh "$ablate" sweep "$scratch/ablate-ws" >/dev/null 2>"$scratch/ablate.err"
same "ablate.sh refuses a tier the declaration does not carry" "2" "$?"

sh "$ablate" "$scratch/ablate-ws" >/dev/null 2>"$scratch/ablate.err"
same "ablate.sh refuses the old one-argument form" "2" "$?"
present "and prints the two-argument usage" "<tier> <workspace>" "$scratch/ablate.err"

for entry in 'docs/../..' '/etc' '..' '""'; do
    cat > "$scratch/unsafe-probe.yml" <<YML
tiers:
  campaign:
    budget_cents: 100
    session_cost_cents: 25
    repetitions: 58
    arms: [present, absent]
    ablation: [CLAUDE.md, $entry]
YML
    fresh_workspace
    HW_PROBE_YML="$scratch/unsafe-probe.yml" sh "$ablate" campaign "$scratch/ablate-ws" \
        >/dev/null 2>"$scratch/ablate.err"
    same "ablate.sh refuses the ablation entry $entry" "2" "$?"
    kept "and removes nothing, not even the safe entry before it" \
        CLAUDE.md .claude .githooks .headwater docs docs/probes engine tools --
done

cat > "$scratch/block-probe.yml" <<'YML'
tiers:
  documentation:
    budget_cents: 100
    session_cost_cents: 25
    repetitions: 58
    arms: [present, absent]
    ablation:
      - docs
      - ".claude"
YML
fresh_workspace
HW_PROBE_YML="$scratch/block-probe.yml" sh "$ablate" documentation "$scratch/ablate-ws" \
    >/dev/null 2>"$scratch/ablate.err"
same "ablate.sh reads a block-sequence ablation" "0" "$?"
kept "and removes exactly its entries" CLAUDE.md .githooks .headwater engine -- docs .claude

# ---------------------------------------------------------------------------
# The leak check (#1472). The status probe's answer is in text the harness
# loads into every session, and no search by the probe's identifier or slug
# finds it, because that text names the ruling and never the probe. So the
# check reads the leak strings each probe declares, over the always-loaded set of a
# copy of this repository's present tree.
# ---------------------------------------------------------------------------
seal="$root/tools/probe/seal.sh"
status_probe=HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request
rm -rf "$scratch/leak-ws"
mkdir -p "$scratch/leak-ws/.claude"
cp "$root/CLAUDE.md" "$scratch/leak-ws/CLAUDE.md"
cp -R "$root/.claude/skills" "$root/.claude/agents" "$scratch/leak-ws/.claude/"

# The probe's own name is nowhere in that text, which is the whole defect: a
# search by name passes the tree.
if grep -rqF "$status_probe" "$scratch/leak-ws/CLAUDE.md" "$scratch/leak-ws/.claude/skills/hw-corpus/SKILL.md"; then
    fail "the always-loaded text does not name the status probe" "it does, so a search by name would find the leak"
else
    pass "the always-loaded text does not name the status probe"
fi

# THE decisive case: a declaration that keeps no leak string. The check names the
# internal hw-corpus skill with the ruling's identifier, and exits 1. Since #1578 the
# shipped authoring skill names no ruling, and hw-corpus carries the sentence.
awk '/^leaks_kept:/ { skip = 1; next } skip && /^[^ #]/ { skip = 0 } !skip' \
    "$root/.headwater/probe.yml" > "$scratch/leak-probe.yml"
HW_PROBE_YML="$scratch/leak-probe.yml" sh "$seal" --leak "$scratch/leak-ws" "$status_probe" \
    > "$scratch/leak.out" 2> "$scratch/leak.err"
same "the leak check fails the present tree for the status probe" "1" "$?"
present "and names the hw-corpus skill with the leak string HW-DR-0052" \
    "leak $status_probe .claude/skills/hw-corpus/SKILL.md HW-DR-0052" "$scratch/leak.out"
present "and the leak string \`status: current\` too" \
    "leak $status_probe .claude/skills/hw-corpus/SKILL.md status: current" "$scratch/leak.out"

# The committed declaration keeps the leak string on purpose (#1472, outcome b): the
# probe is reported on its own line and never in a sufficiency rate.
sh "$seal" --leak "$scratch/leak-ws" "$status_probe" > "$scratch/kept.out" 2> "$scratch/kept.err"
same "a leak string the declaration keeps does not fail the check" "0" "$?"
present "and the check still prints it, as kept" \
    "kept $status_probe .claude/skills/hw-corpus/SKILL.md HW-DR-0052" "$scratch/kept.out"

# A description is the one line a harness loads. A leak string in the body of a skill
# is not always loaded, so it is no leak.
rm -rf "$scratch/leak-body"
mkdir -p "$scratch/leak-body/.claude/skills/s"
printf -- '---\nname: s\ndescription: Nothing to see.\n---\n\nThe ruling is HW-DR-0052.\n' \
    > "$scratch/leak-body/.claude/skills/s/SKILL.md"
HW_PROBE_YML="$scratch/leak-probe.yml" sh "$seal" --leak "$scratch/leak-body" "$status_probe" \
    > "$scratch/body.out" 2>&1
same "a leak string in the body of a skill and not in its description is no leak" "0" "$?"
# A folded description is read to its end.
printf -- '---\nname: s\ndescription: >\n  Nothing to see, and\n  the ruling is HW-DR-0052.\n---\n' \
    > "$scratch/leak-body/.claude/skills/s/SKILL.md"
HW_PROBE_YML="$scratch/leak-probe.yml" sh "$seal" --leak "$scratch/leak-body" "$status_probe" \
    > "$scratch/folded.out" 2>&1
same "a leak string on the second line of a folded description is a leak" "1" "$?"
# Each channel on its own (verify round 1): an agent definition's description
# and a command's description are loaded into every session too.
rm -rf "$scratch/leak-body/.claude/skills"
mkdir -p "$scratch/leak-body/.claude/agents"
printf -- '---\nname: a\ndescription: An agent that cites HW-DR-0052.\n---\n' \
    > "$scratch/leak-body/.claude/agents/a.md"
HW_PROBE_YML="$scratch/leak-probe.yml" sh "$seal" --leak "$scratch/leak-body" "$status_probe" \
    > "$scratch/agent.out" 2>&1
same "a leak string in an agent definition's description alone is a leak" "1" "$?"
present "and names the agent definition" "leak $status_probe .claude/agents/a.md HW-DR-0052" "$scratch/agent.out"
rm -rf "$scratch/leak-body/.claude/agents"
mkdir -p "$scratch/leak-body/.claude/commands"
printf -- '---\ndescription: A command that says status: current.\n---\n\nBody.\n' \
    > "$scratch/leak-body/.claude/commands/c.md"
HW_PROBE_YML="$scratch/leak-probe.yml" sh "$seal" --leak "$scratch/leak-body" "$status_probe" \
    > "$scratch/command.out" 2>&1
same "a leak string in a command's description is a leak" "1" "$?"
present "and names the command" "leak $status_probe .claude/commands/c.md status: current" "$scratch/command.out"
rm -rf "$scratch/leak-body/.claude/commands"

# Verify round 2: the harness lists a skill or a command as its description
# followed by its `when_to_use:`, and a command with no description by the
# first line of its body. Each channel alone, and a namespaced command, and a
# file with CRLF line endings.
leak_one() {
    # $1 name, $2 relative path, $3 file content (printf format), $4 expected exit
    rm -rf "$scratch/leak-one"
    mkdir -p "$scratch/leak-one/$(dirname "$2")"
    # shellcheck disable=SC2059
    printf -- "$3" > "$scratch/leak-one/$2"
    HW_PROBE_YML="$scratch/leak-probe.yml" sh "$seal" --leak "$scratch/leak-one" "$status_probe" \
        > "$scratch/leak-one.out" 2>&1
    same "$1" "$4" "$?"
}
leak_one "a leak string in a skill's when_to_use is a leak" \
    .claude/skills/s/SKILL.md '---\nname: s\ndescription: Nothing.\nwhen_to_use: When HW-DR-0052 applies.\n---\n' 1
leak_one "a leak string in a command's when_to_use is a leak" \
    .claude/commands/c.md '---\ndescription: Nothing.\nwhen_to_use: >\n  When the ruling\n  HW-DR-0052 applies.\n---\n' 1
leak_one "the first body line of a command with no front matter is a leak" \
    .claude/commands/c.md 'HW-DR-0052 says it.\n\nMore.\n' 1
leak_one "the first body line of a command whose front matter holds no description is a leak" \
    .claude/commands/c.md '---\nargument-hint: <n>\n---\n\n# HW-DR-0052 says it\n' 1
leak_one "a later body line of a command is not loaded, and is no leak" \
    .claude/commands/c.md '---\nargument-hint: <n>\n---\n\nNothing here.\n\nHW-DR-0052 later.\n' 0
leak_one "the body of a command that has a description is not loaded, and is no leak" \
    .claude/commands/c.md '---\ndescription: Nothing.\n---\n\nHW-DR-0052 says it.\n' 0
leak_one "a command under a namespace directory is read" \
    .claude/commands/ns/c.md '---\ndescription: The HW-DR-0052 command.\n---\n' 1
leak_one "a command with CRLF line endings is read" \
    .claude/commands/c.md '---\r\ndescription: The HW-DR-0052 command.\r\n---\r\n' 1
leak_one "a skill with CRLF line endings is read" \
    .claude/skills/s/SKILL.md '---\r\nname: s\r\ndescription: The HW-DR-0052 skill.\r\n---\r\n' 1

# Verify round 3: eight kinds of always-loaded text a hand-written reader
# missed, each planted alone and each reported, and each with CRLF line
# endings too. `leak.py` reads front matter with a YAML parser and the memory
# set as the harness names it.
leak_kind() {
    # $1 name, $2 relative path, $3 file content (printf format), $4 the
    # `<where>` the hit must name
    leak_one "$1" "$2" "$3" 1
    present "and names $4" "leak $status_probe $4 HW-DR-0052" "$scratch/leak-one.out"
    leak_one "$1, with CRLF line endings" "$2" "$(printf '%s' "$3" | sed 's/\\n/\\r\\n/g')" 1
    present "and names $4" "leak $status_probe $4 HW-DR-0052" "$scratch/leak-one.out"
}
leak_kind "(A) a plain multi-line description whose second line holds the leak string" \
    .claude/skills/s/SKILL.md '---\nname: s\ndescription: Nothing to see, and\n  the ruling is HW-DR-0052.\n---\n' \
    .claude/skills/s/SKILL.md
leak_kind "(B) a double-quoted multi-line description" \
    .claude/agents/a.md '---\nname: a\ndescription: "Nothing to see, and\n  the ruling is HW-DR-0052."\n---\n' \
    .claude/agents/a.md
leak_kind "(C) .claude/CLAUDE.md" .claude/CLAUDE.md 'The ruling is HW-DR-0052.\n' .claude/CLAUDE.md
leak_kind "(D) CLAUDE.local.md" CLAUDE.local.md 'The ruling is HW-DR-0052.\n' CLAUDE.local.md
leak_kind "(K) a when-to-use key, with a hyphen" \
    .claude/agents/a.md '---\nname: a\ndescription: Nothing.\nwhen-to-use: When HW-DR-0052 applies.\n---\n' \
    .claude/agents/a.md
leak_kind "(L) AGENTS.md as a regular file" AGENTS.md 'The ruling is HW-DR-0052.\n' AGENTS.md
leak_kind "(M) a file under .claude/rules/" .claude/rules/zz.md 'The ruling is HW-DR-0052.\n' .claude/rules/zz.md
leak_kind "(M) a file under a subdirectory of .claude/rules/" \
    .claude/rules/area/zz.md '---\npaths: ["src/**"]\n---\n\nThe ruling is HW-DR-0052.\n' .claude/rules/area/zz.md

# (E) A file CLAUDE.md imports with `@<path>`, and a file that one imports in
# turn. An `@` inside a code span is no import.
leak_import() {
    # $1 name, $2 CLAUDE.md content, $3 expected exit
    rm -rf "$scratch/leak-one"
    mkdir -p "$scratch/leak-one/notes"
    printf -- "$2" > "$scratch/leak-one/CLAUDE.md"
    printf 'See @deeper.md for more.\n' > "$scratch/leak-one/notes/extra.md"
    printf 'The ruling is HW-DR-0052.\n' > "$scratch/leak-one/notes/deeper.md"
    HW_PROBE_YML="$scratch/leak-probe.yml" sh "$seal" --leak "$scratch/leak-one" "$status_probe" \
        > "$scratch/leak-one.out" 2>&1
    same "$1" "$3" "$?"
}
leak_import "(E) a file an @ import in CLAUDE.md reaches is read" 'Read @notes/extra.md first.\n' 1
present "and names the imported file, relative to the file that imports it" \
    "leak $status_probe notes/deeper.md HW-DR-0052" "$scratch/leak-one.out"
leak_import "(E) the same import with CRLF line endings" 'Read @notes/extra.md first.\r\n' 1
# Verify round 4: the harness cuts an import at its first `#`.
leak_import "an import with a fragment reads the file before the #" 'Read @notes/extra.md#top first.\n' 1
present "and names the file it reaches" "leak $status_probe notes/deeper.md HW-DR-0052" "$scratch/leak-one.out"
leak_import "an @ inside a code span is no import" 'Write `see @notes/extra.md now` to import it.\n' 0
leak_import "an @ inside a fenced code block is no import" '```\nsee @notes/extra.md\n```\n' 0

# AGENTS.md as a link to CLAUDE.md, as in this repository, is one file and
# one line.
rm -rf "$scratch/leak-one"
mkdir -p "$scratch/leak-one"
printf 'The ruling is HW-DR-0052.\n' > "$scratch/leak-one/CLAUDE.md"
ln -s CLAUDE.md "$scratch/leak-one/AGENTS.md"
HW_PROBE_YML="$scratch/leak-probe.yml" sh "$seal" --leak "$scratch/leak-one" "$status_probe" \
    > "$scratch/leak-one.out" 2>&1
same "AGENTS.md as a link to CLAUDE.md is read once" "leak $status_probe CLAUDE.md HW-DR-0052" \
    "$(grep HW-DR-0052 "$scratch/leak-one.out")"

# A leak string that a line break of a literal block splits is still one
# leak string, because every text is read with its whitespace runs as one space.
leak_one "a leak string split by a line break of a literal block is a leak" \
    .claude/skills/s/SKILL.md '---\nname: s\ndescription: |\n  The status:\n  current one.\n---\n' 1

# Front matter that does not parse is read whole, never skipped.
leak_one "front matter that does not parse is read whole" \
    .claude/skills/s/SKILL.md '---\nname: s\ndescription: [HW-DR-0052\n---\n' 1

# A probe that declares no leak string is printed as one the check cannot see, and
# does not pass as clean by saying nothing.
sh "$seal" --leak "$scratch/leak-ws" HW-PROBE-no-such-probe > "$scratch/undeclared.out" 2>&1
same "a probe with no leak string does not fail the check" "0" "$?"
present "and is printed as undeclared" "undeclared HW-PROBE-no-such-probe" "$scratch/undeclared.out"

# The host-level text (#1467, clause 7). A session loads the `CLAUDE.md` and
# the skills of its configuration directory too, so `--config <dir>` reads
# them under `config:<path>`. A campaign's directory is empty by
# construction, and this case shows the check would read a planted answer.
rm -rf "$scratch/leak-bare" "$scratch/leak-config"
mkdir -p "$scratch/leak-bare" "$scratch/leak-config/skills/planted"
printf 'Settle it under HW-DR-0052.\n' > "$scratch/leak-config/CLAUDE.md"
printf -- '---\nname: planted\ndescription: A settled decision goes in at status: current.\n---\n' \
    > "$scratch/leak-config/skills/planted/SKILL.md"
HW_PROBE_YML="$scratch/leak-probe.yml" sh "$seal" --leak --config "$scratch/leak-config" "$scratch/leak-bare" "$status_probe" \
    > "$scratch/leak-config.out" 2> "$scratch/leak-config.err"
same "a leak string in the configuration directory's CLAUDE.md fails the check" "1" "$?"
present "and is named under config:CLAUDE.md" \
    "leak $status_probe config:CLAUDE.md HW-DR-0052" "$scratch/leak-config.out"
present "and a skill there is read as always-loaded text" \
    "leak $status_probe config:skills/planted/SKILL.md status: current" "$scratch/leak-config.out"
rm -rf "$scratch/leak-config"
mkdir -p "$scratch/leak-config"
HW_PROBE_YML="$scratch/leak-probe.yml" sh "$seal" --leak --config "$scratch/leak-config" "$scratch/leak-bare" "$status_probe" \
    > "$scratch/leak-config.out" 2> "$scratch/leak-config.err"
same "an empty configuration directory leaks nothing" "0" "$?"

# The MCP tools. A workspace that declares a server has the descriptions of
# its tools loaded too, and the check lists them through the engine.
if [ -x "$engine" ]; then
    rm -rf "$scratch/leak-mcp"
    mkdir -p "$scratch/leak-mcp"
    cp -R "$root/.headwater" "$scratch/leak-mcp/.headwater"
    printf '{"mcpServers":{}}\n' > "$scratch/leak-mcp/.mcp.json"
    printf 'leaks:\n  HW-PROBE-x: [Resolve a task description]\n' > "$scratch/mcp-probe.yml"
    HW_PROBE_YML="$scratch/mcp-probe.yml" sh "$seal" --leak "$scratch/leak-mcp" HW-PROBE-x \
        > "$scratch/mcp.out" 2> "$scratch/mcp.err"
    same "a leak string in an MCP tool's description is a leak" "1" "$?"
    present "and names the tool" "leak HW-PROBE-x mcp:route Resolve a task description" "$scratch/mcp.out"
    # `--bytes` counts the same tool descriptions (#1776): one line for each
    # tool, and a total that is their sum, because the tree holds nothing else.
    HW_PROBE_YML="$scratch/mcp-probe.yml" sh "$seal" --leak --bytes "$scratch/leak-mcp" HW-PROBE-x \
        > "$scratch/mcp-bytes.out" 2> "$scratch/mcp-bytes.err"
    same "--bytes over a declared server exits 0" "0" "$?"
    mcp_route=$(awk '$1 == "bytes" && $2 == "mcp:route" { print $3 }' "$scratch/mcp-bytes.out")
    mcp_sum=$(awk '$1 == "bytes" && $2 ~ /^mcp:/ { s += $3 } END { print s + 0 }' "$scratch/mcp-bytes.out")
    if [ -n "$mcp_route" ] && [ "$mcp_route" -gt 0 ] && [ "$mcp_sum" = "$(awk '$1 == "bytes" && $2 == "total" { print $3 }' "$scratch/mcp-bytes.out")" ]; then
        pass "and counts each MCP tool's description, which the total sums"
    else
        fail "and counts each MCP tool's description, which the total sums" "$(head -c 300 "$scratch/mcp-bytes.out")"
    fi
    # A tool counts as its whole definition, name and input schema with the
    # descriptions, in compact JSON (verify round 1 of #1776). The expected
    # figure comes from the server's own `tools/list`, read here apart from
    # leak.py.
    mcp_def=$(printf '%s\n%s\n%s\n' \
        '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}' \
        '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
        '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
        | "$engine" mcp --root "$scratch/leak-mcp" 2>/dev/null \
        | python3 -I -c 'import json, sys
for line in sys.stdin:
    try:
        m = json.loads(line)
    except ValueError:
        continue
    if m.get("id") == 2:
        for t in m["result"]["tools"]:
            if t.get("name") == "route":
                print(len(json.dumps(t, separators=(",", ":"), ensure_ascii=False).encode("utf-8")))')
    same "and counts the route tool as its whole definition, schema included" "$mcp_def" "$mcp_route"
    rm -f "$scratch/leak-mcp/.mcp.json"
    HW_PROBE_YML="$scratch/mcp-probe.yml" sh "$seal" --leak "$scratch/leak-mcp" HW-PROBE-x \
        > "$scratch/nomcp.out" 2>&1
    same "and with no server declared the tools are not loaded" "0" "$?"
else
    printf 'note no engine, so the MCP leak cases did not run.\n'
fi

# The scenario channels (#1774, clause 5). A scenario of the additive campaign
# is checked over every skill description and body, and over the text the
# intent hook injects for each probe's task. `--scenario` reads them, so the
# always-loaded contract above keeps its meaning without it.
HW_PROBE_YML="$scratch/leak-probe.yml" sh "$seal" --leak --scenario "$scratch/leak-body-s" "$status_probe" \
    > "$scratch/scenario-nows.out" 2>&1
same "--scenario refuses a workspace that is not a directory" "2" "$?"
rm -rf "$scratch/leak-body-s"
mkdir -p "$scratch/leak-body-s/.claude/skills/s"
printf -- '---\nname: s\ndescription: Nothing to see.\n---\n\nThe ruling is HW-DR-0052.\n' \
    > "$scratch/leak-body-s/.claude/skills/s/SKILL.md"
HW_PROBE_YML="$scratch/leak-probe.yml" sh "$seal" --leak "$scratch/leak-body-s" "$status_probe" \
    > "$scratch/body-s.out" 2>&1
same "without --scenario a leak string in a skill body is still no leak" "0" "$?"
HW_PROBE_YML="$scratch/leak-probe.yml" sh "$seal" --leak --scenario "$scratch/leak-body-s" "$status_probe" \
    > "$scratch/body-s.out" 2>&1
same "with --scenario a leak string in the body of a skill is a leak" "1" "$?"
present "and names the skill file" "leak $status_probe .claude/skills/s/SKILL.md HW-DR-0052" "$scratch/body-s.out"
HW_PROBE_YML="$scratch/leak-probe.yml" sh "$seal" --leak --config "$scratch/leak-config" --scenario "$scratch/leak-body-s" "$status_probe" \
    > "$scratch/body-s2.out" 2>&1
same "--scenario reads after --config too" "1" "$?"
# An agent body is outside the clause, and is no leak even with --scenario.
rm -rf "$scratch/leak-body-s/.claude/skills"
mkdir -p "$scratch/leak-body-s/.claude/agents"
printf -- '---\nname: a\ndescription: Nothing.\n---\n\nThe ruling is HW-DR-0052.\n' \
    > "$scratch/leak-body-s/.claude/agents/a.md"
HW_PROBE_YML="$scratch/leak-probe.yml" sh "$seal" --leak --scenario "$scratch/leak-body-s" "$status_probe" \
    > "$scratch/agent-s.out" 2>&1
same "with --scenario an agent body is still no leak, the known limit" "0" "$?"

# The harness classification: a hit in a file of the `tooling` list, in a
# tree that holds it, is internal text the seal cannot remove (#1872). It prints as
# `harness` and does not fail the check. The same text in another skill is a
# leak.
rm -rf "$scratch/leak-harness"
mkdir -p "$scratch/leak-harness/.claude/skills/hw-corpus" "$scratch/leak-harness/.claude/skills/other"
printf -- '---\nname: hw-corpus\ndescription: Nothing.\n---\n\nThe ruling is HW-DR-0052.\n' \
    > "$scratch/leak-harness/.claude/skills/hw-corpus/SKILL.md"
HW_PROBE_YML="$scratch/leak-probe.yml" sh "$seal" --leak --scenario "$scratch/leak-harness" "$status_probe" \
    > "$scratch/harness.out" 2>&1
same "with --scenario a hit in a tooling skill body does not fail the check" "0" "$?"
present "and prints as harness" "harness $status_probe .claude/skills/hw-corpus/SKILL.md HW-DR-0052" "$scratch/harness.out"
cp "$scratch/leak-harness/.claude/skills/hw-corpus/SKILL.md" "$scratch/leak-harness/.claude/skills/other/SKILL.md"
HW_PROBE_YML="$scratch/leak-probe.yml" sh "$seal" --leak --scenario "$scratch/leak-harness" "$status_probe" \
    > "$scratch/harness2.out" 2>&1
same "the same text in a skill of no harness path is a leak" "1" "$?"
present "and names that skill" "leak $status_probe .claude/skills/other/SKILL.md HW-DR-0052" "$scratch/harness2.out"
HW_PROBE_YML="$scratch/leak-probe.yml" sh "$seal" --leak "$scratch/leak-harness" "$status_probe" \
    > "$scratch/harness3.out" 2>&1
same "without --scenario the harness skill's body is not read at all" "0" "$?"
absent "and nothing is printed for it" "harness $status_probe" "$scratch/harness3.out"

# The hook's injected text. The tree's `.claude/hooks/intent.sh` runs as the
# harness runs it, on the probe's `## Task`, read from this checkout, and its
# standard output is the channel `hook:<probe>`. A stub hook stands in here,
# and keeps what it was handed.
ruling_probe=HW-PROBE-what-a-session-writes-points-back-at-the-ruling-it-rests-on
rm -rf "$scratch/leak-hook"
mkdir -p "$scratch/leak-hook/.claude/hooks"
printf '#!/bin/sh\ncat > "%s"\nprintf "Pointer: the corpus-wide fold\\n"\nprintf "HW-DR-0049 on stderr\\n" >&2\n' \
    "$scratch/hook-stdin" > "$scratch/leak-hook/.claude/hooks/intent.sh"
sh "$seal" --leak "$scratch/leak-hook" "$ruling_probe" > "$scratch/hook0.out" 2>&1
same "without --scenario the hook does not run" "0" "$?"
rm -f "$scratch/hook-stdin"
sh "$seal" --leak --scenario "$scratch/leak-hook" "$ruling_probe" > "$scratch/hook.out" 2>&1
same "with --scenario a leak string in the hook's output is a leak" "1" "$?"
present "and names the channel hook:<probe>" "leak $ruling_probe hook:$ruling_probe corpus-wide fold" "$scratch/hook.out"
absent "and the hook's standard error is no channel" "HW-DR-0049" "$scratch/hook.out"
task_line=$(awk '/^## Task/ { f = 1; next } /^## / { f = 0 } f && NF { print; exit }' \
    "$root/docs/probes/what-a-session-writes-points-back-at-the-ruling-it-rests-on.md" | cut -c1-40)
if [ -n "$task_line" ] && grep -qF "$task_line" "$scratch/hook-stdin" 2>/dev/null && grep -q '"UserPromptSubmit"' "$scratch/hook-stdin"; then
    pass "the hook is handed the probe's task in a UserPromptSubmit envelope"
else
    fail "the hook is handed the probe's task in a UserPromptSubmit envelope" "$(head -c 300 "$scratch/hook-stdin" 2>/dev/null)"
fi
# A hook string of one probe is no leak for another: each probe's sessions
# meet the text the hook injects for its own task.
printf 'leaks:\n  %s: [Pointer]\n  %s: [nothing-matches-this]\n' "$status_probe" "$ruling_probe" > "$scratch/hook-probe.yml"
HW_PROBE_YML="$scratch/hook-probe.yml" sh "$seal" --leak --scenario "$scratch/leak-hook" "$ruling_probe" \
    > "$scratch/hook2.out" 2>&1
same "a string of a probe not checked is no leak in another probe's hook text" "0" "$?"
# A probe with no task in this checkout cannot be put to the hook, so the
# scenario text is not whole.
printf 'leaks:\n  HW-PROBE-no-such-probe: [Pointer]\n' > "$scratch/hook-probe.yml"
HW_PROBE_YML="$scratch/hook-probe.yml" sh "$seal" --leak --scenario "$scratch/leak-hook" HW-PROBE-no-such-probe \
    > "$scratch/hook3.out" 2>&1
same "a probe whose task the checkout does not hold stops a scenario check with 3" "3" "$?"

# The hook naming the record in force (#1874, parent ruling of 2026-10-30).
# For a probe with the `superseded_target` axis, a hit in its own hook channel
# on the whole identifier of a `current` document is the pointer that
# scenarios 4 and 6 measure: it prints as `treatment` and does not fail. Every
# other hook hit still fails, and the always-loaded check does not change.
stamp_probe=HW-PROBE-a-session-that-reviews-an-agent-s-acceptance-stamp-cites-the-ruling-in-force-and-not-the-draft-that-the-pull-request-names
rm -rf "$scratch/leak-force"
mkdir -p "$scratch/leak-force/.claude/hooks"
printf '#!/bin/sh\ncat > /dev/null\nprintf "Pointer: against HW-DR-0034, the draft HW-DR-0069 and the corpus-wide fold\\n"\n' \
    > "$scratch/leak-force/.claude/hooks/intent.sh"
force_case() {
    # force_case <yml> <probe> <out>: a scenario check of the stub hook tree.
    HW_PROBE_YML="$1" sh "$seal" --leak --scenario "$scratch/leak-force" "$2" > "$3" 2>&1
}
printf 'leaks:\n  %s: [HW-DR-0034]\n' "$stamp_probe" > "$scratch/force.yml"
force_case "$scratch/force.yml" "$stamp_probe" "$scratch/force1.out"
same "the hook naming the record in force for a superseded-target probe does not fail" "0" "$?"
present "and prints as treatment" "treatment $stamp_probe hook:$stamp_probe HW-DR-0034" "$scratch/force1.out"
printf 'leaks:\n  %s: [corpus-wide fold]\n' "$stamp_probe" > "$scratch/force.yml"
force_case "$scratch/force.yml" "$stamp_probe" "$scratch/force2.out"
same "a phrase in the hook text of a superseded-target probe is still a leak" "1" "$?"
present "and prints as leak" "leak $stamp_probe hook:$stamp_probe corpus-wide fold" "$scratch/force2.out"
printf 'leaks:\n  %s: [HW-DR-0069]\n' "$stamp_probe" > "$scratch/force.yml"
force_case "$scratch/force.yml" "$stamp_probe" "$scratch/force3.out"
same "the identifier of a superseded record in the hook text is still a leak" "1" "$?"
present "and prints as leak" "leak $stamp_probe hook:$stamp_probe HW-DR-0069" "$scratch/force3.out"
printf 'leaks:\n  %s: [against HW-DR-0034]\n' "$stamp_probe" > "$scratch/force.yml"
force_case "$scratch/force.yml" "$stamp_probe" "$scratch/force4.out"
same "a string that holds the identifier and more is still a leak" "1" "$?"
printf 'leaks:\n  %s: [HW-DR-0034]\n' "$ruling_probe" > "$scratch/force.yml"
force_case "$scratch/force.yml" "$ruling_probe" "$scratch/force5.out"
same "the same identifier for a probe without the axis is still a leak" "1" "$?"
present "and prints as leak" "leak $ruling_probe hook:$ruling_probe HW-DR-0034" "$scratch/force5.out"
printf 'In force: HW-DR-0034\n' > "$scratch/leak-force/CLAUDE.md"
printf 'leaks:\n  %s: [HW-DR-0034]\n' "$stamp_probe" > "$scratch/force.yml"
HW_PROBE_YML="$scratch/force.yml" sh "$seal" --leak "$scratch/leak-force" "$stamp_probe" > "$scratch/force6.out" 2>&1
same "without --scenario the identifier in an always-loaded file is still a leak" "1" "$?"
present "and prints as leak" "leak $stamp_probe CLAUDE.md HW-DR-0034" "$scratch/force6.out"
force_case "$scratch/force.yml" "$stamp_probe" "$scratch/force7.out"
same "with --scenario the always-loaded hit still fails beside the hook's treatment" "1" "$?"
present "and the always-loaded hit prints as leak" "leak $stamp_probe CLAUDE.md HW-DR-0034" "$scratch/force7.out"
present "and the hook hit still prints as treatment" "treatment $stamp_probe hook:$stamp_probe HW-DR-0034" "$scratch/force7.out"

# `[]` is a declaration with no answer string: the two `opened` probes of the
# additive set key on an act, and no string states it. The check prints
# nothing for such a probe, and a probe with no entry is still undeclared.
printf 'leaks:\n  HW-PROBE-x: []\n  HW-PROBE-y:\n' > "$scratch/empty-probe.yml"
HW_PROBE_YML="$scratch/empty-probe.yml" sh "$seal" --leak "$scratch/leak-ws" HW-PROBE-x HW-PROBE-y HW-PROBE-z \
    > "$scratch/empty.out" 2>&1
same "a probe declared with [] does not fail the check" "0" "$?"
absent "and is not printed as undeclared" "undeclared HW-PROBE-x" "$scratch/empty.out"
present "an entry with no value is still undeclared" "undeclared HW-PROBE-y" "$scratch/empty.out"
present "and a missing entry is still undeclared" "undeclared HW-PROBE-z" "$scratch/empty.out"

# The bytes the layers put in front of the model (#1776, the bytes column of
# HW-DR-0111's table). `--bytes` counts what the harness hands every session,
# and nothing it loads on demand: a document under `docs/` and a skill body
# leave the total where it was, and a `CLAUDE.md` of K raw bytes raises it by
# exactly K, multi-byte characters and whitespace runs included.
bytes_total() {
    awk '$1 == "bytes" && $2 == "total" { print $3 }' "$1"
}
rm -rf "$scratch/bytes-ws"
mkdir -p "$scratch/bytes-ws/.claude/skills/s" "$scratch/bytes-ws/.claude/agents" "$scratch/bytes-ws/.claude/hooks"
printf -- '---\nname: s\ndescription: Nothing to see.\n---\n\nShort body.\n' > "$scratch/bytes-ws/.claude/skills/s/SKILL.md"
printf -- '---\nname: a\ndescription: An agent.\n---\n\nAgent body.\n' > "$scratch/bytes-ws/.claude/agents/a.md"
printf '#!/bin/sh\ncat > /dev/null\nprintf "Pointer: x\\n"\n' > "$scratch/bytes-ws/.claude/hooks/intent.sh"
sh "$seal" --leak --scenario --bytes "$scratch/bytes-ws" "$ruling_probe" > "$scratch/bytes0.out" 2> "$scratch/bytes0.err"
same "--bytes measures a tree and exits 0" "0" "$?"
t0=$(bytes_total "$scratch/bytes0.out")
present "and counts the skill listing, name and description" "bytes .claude/skills/s/SKILL.md 17" "$scratch/bytes0.out"
present "and counts the agent listing" "bytes .claude/agents/a.md 11" "$scratch/bytes0.out"
present "and counts the hook's text for the probe on its own line" "bytes hook:$ruling_probe 11" "$scratch/bytes0.out"
same "and the total is the always-loaded lines alone, not the hook's" "28" "$t0"
mkdir -p "$scratch/bytes-ws/docs"
printf 'A document of the corpus, never loaded whole.\n' > "$scratch/bytes-ws/docs/d.md"
printf -- '---\nname: s\ndescription: Nothing to see.\n---\n\nA much longer body that loads on demand, and only on demand.\n' \
    > "$scratch/bytes-ws/.claude/skills/s/SKILL.md"
sh "$seal" --leak --scenario --bytes "$scratch/bytes-ws" "$ruling_probe" > "$scratch/bytes1.out" 2>&1
same "a document under docs/ and a longer skill body leave the total unchanged" "$t0" "$(bytes_total "$scratch/bytes1.out")"
# Octal escapes, because the `printf` of dash reads no `\x`: an em dash and an
# e with an acute accent, 5 bytes in 2 characters.
printf 'Rule  one.\n\n\342\200\224 caf\303\251\n' > "$scratch/bytes-ws/CLAUDE.md"
k=$(wc -c < "$scratch/bytes-ws/CLAUDE.md" | tr -d ' ')
same "the CLAUDE.md of this case holds 22 bytes in 19 characters" "22" "$k"
sh "$seal" --leak --scenario --bytes "$scratch/bytes-ws" "$ruling_probe" > "$scratch/bytes2.out" 2>&1
same "a CLAUDE.md of K raw bytes raises the total by exactly K" "$((t0 + k))" "$(bytes_total "$scratch/bytes2.out")"
present "and prints its own line" "bytes CLAUDE.md $k" "$scratch/bytes2.out"
sh "$seal" --leak --bytes "$scratch/bytes-ws" "$ruling_probe" > "$scratch/bytes3.out" 2>&1
same "--bytes without --scenario counts the same always-loaded total" "$((t0 + k))" "$(bytes_total "$scratch/bytes3.out")"
present "and still counts the hook's text, which loads with or without --scenario" "bytes hook:$ruling_probe 11" "$scratch/bytes3.out"
printf 'leaks:\n  %s: [Rule]\n' "$ruling_probe" > "$scratch/bytes-probe.yml"
HW_PROBE_YML="$scratch/bytes-probe.yml" sh "$seal" --leak --bytes "$scratch/bytes-ws" "$ruling_probe" > "$scratch/bytes4.out" 2>&1
same "--bytes checks no leak string, so a hit does not fail it" "0" "$?"
absent "and prints no leak line" "leak $ruling_probe" "$scratch/bytes4.out"
# Each other text the harness loads into every session raises the total by its
# own bytes (verify round 1 of #1776): an imported memory file, a rule under
# `.claude/rules`, a command listing, and the CLAUDE.md and skill listing of a
# `--config` directory. Each has its own line, and the total is checked whole.
printf 'Rule  one.\n\n\342\200\224 caf\303\251\n@imp.md\n' > "$scratch/bytes-ws/CLAUDE.md"
printf 'Imported \303\251.\n' > "$scratch/bytes-ws/imp.md"
mkdir -p "$scratch/bytes-ws/.claude/rules" "$scratch/bytes-ws/.claude/commands"
printf 'A rule \342\200\224 kept.\n' > "$scratch/bytes-ws/.claude/rules/r.md"
printf -- '---\ndescription: A command.\n---\n\nCommand body.\n' > "$scratch/bytes-ws/.claude/commands/c.md"
k_claude=$(wc -c < "$scratch/bytes-ws/CLAUDE.md" | tr -d ' ')
k_imp=$(wc -c < "$scratch/bytes-ws/imp.md" | tr -d ' ')
k_rule=$(wc -c < "$scratch/bytes-ws/.claude/rules/r.md" | tr -d ' ')
sh "$seal" --leak --bytes "$scratch/bytes-ws" "$ruling_probe" > "$scratch/bytes5.out" 2>&1
present "an imported memory file counts on its own line" "bytes imp.md $k_imp" "$scratch/bytes5.out"
present "a rule under .claude/rules counts on its own line" "bytes .claude/rules/r.md $k_rule" "$scratch/bytes5.out"
present "a command counts as its listing, the description" "bytes .claude/commands/c.md 10" "$scratch/bytes5.out"
same "and the total sums the CLAUDE.md, the import, the rule and the command" "$((t0 + k_claude + k_imp + k_rule + 10))" "$(bytes_total "$scratch/bytes5.out")"
rm -rf "$scratch/bytes-config"
mkdir -p "$scratch/bytes-config/skills/h"
printf 'Host rule.\n' > "$scratch/bytes-config/CLAUDE.md"
printf -- '---\nname: h\ndescription: Host skill.\n---\n\nHost body.\n' > "$scratch/bytes-config/skills/h/SKILL.md"
k_host=$(wc -c < "$scratch/bytes-config/CLAUDE.md" | tr -d ' ')
sh "$seal" --leak --bytes --config "$scratch/bytes-config" "$scratch/bytes-ws" "$ruling_probe" > "$scratch/bytes6.out" 2>&1
present "the CLAUDE.md of a --config directory counts on its own line" "bytes config:CLAUDE.md $k_host" "$scratch/bytes6.out"
present "and so does its skill listing" "bytes config:skills/h/SKILL.md 13" "$scratch/bytes6.out"
same "and both raise the total" "$(($(bytes_total "$scratch/bytes5.out") + k_host + 13))" "$(bytes_total "$scratch/bytes6.out")"
sh "$seal" --leak "$scratch/leak-ws" \
    HW-PROBE-a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor \
    HW-PROBE-the-pointer-this-corpus-offers-for-a-task-is-the-document-a-session-opens \
    "$ruling_probe" > "$scratch/set-declared.out" 2>&1
absent "the committed declaration leaves no probe of the additive set undeclared" "undeclared" "$scratch/set-declared.out"

# ---------------------------------------------------------------------------
# The reader of the raw recordings (#1776). A finding drawn from a recording
# must count the harness's own record as evidence, and never text a session
# happened to read. A grep for "hook script not found" over the no-hook
# streams of 2026-10-03 matched 21 of 120 L3 streams, because those sessions
# read a file that quotes the string; the error itself is a
# `hook_non_blocking_error` attachment in the harness log. So session A below
# reads a file whose result quotes the string and has no hook attachment, and
# session B has the attachment and no such read. The reader must report the
# error for B alone, and say which attachment the model was shown.
# ---------------------------------------------------------------------------
reader="$root/tools/probe/recordings.py"
rec="$scratch/rec"
rm -rf "$rec"
mkdir -p "$rec/sessions/A" "$rec/sessions/B" \
    "$rec/config/A/projects/-var-tmp-hw-1659-x-ws-A" \
    "$rec/config/B/projects/-var-tmp-hw-1659-x-ws-B/u-b/subagents"
cat > "$rec/sessions/A/raw.jsonl" <<'EOF'
{"type":"system","subtype":"init","skills":["s"],"agents":["g"]}
{"type":"assistant","parent_tool_use_id":null,"message":{"content":[{"type":"text","text":"tool: Read hook script not found"},{"type":"tool_use","id":"toolu_a1","name":"Read","input":{"file_path":"/var/tmp/hw-1659/x/ws/A/tools/probe/arms"}}]}}
{"type":"user","parent_tool_use_id":null,"message":{"content":[{"type":"tool_result","tool_use_id":"toolu_a1","content":"echo \"hook script not found: $p\" >&2"}]}}
{"type":"user","parent_tool_use_id":null,"message":{"content":[{"type":"tool_use","id":"toolu_a8","name":"Write","input":{"file_path":"/not/a/call/of/the/model"}}]}}
{"type":"assistant","parent_tool_use_id":null,"message":{"content":[{"type":"tool_result","tool_use_id":"toolu_a9","content":"echo again"}]}}
EOF
cat > "$rec/config/A/projects/-var-tmp-hw-1659-x-ws-A/u-a.jsonl" <<'EOF'
{"type":"attachment","isSidechain":false,"rendered":"x","attachment":{"type":"instructions","files":[]}}
{"type":"user","isSidechain":false,"message":{"content":[{"type":"tool_result","tool_use_id":"toolu_a1","content":"hook_non_blocking_error hook script not found"}]}}
EOF
cat > "$rec/sessions/B/raw.jsonl" <<'EOF'
{"type":"assistant","parent_tool_use_id":null,"message":{"content":[{"type":"tool_use","id":"toolu_b1","name":"Skill","input":{"skill":"headwater-orient"}}]}}
{"type":"assistant","parent_tool_use_id":null,"message":{"content":[{"type":"tool_use","id":"toolu_b2","name":"Bash","input":{"command":"headwater check\t--root .\r\nx"}}]}}
{"type":"assistant","parent_tool_use_id":"toolu_b9","message":{"content":[{"type":"tool_use","id":"toolu_b3","name":"Grep","input":{"pattern":"supersedes","path":"docs"}}]}}
{"type":"assistant","parent_tool_use_id":null,"message":{"content":[{"type":"tool_use","id":"toolu_b4","name":"Glob","input":{"pattern":"docs/**/*.md"}},{"type":"tool_use","id":"toolu_b5","name":"WebFetch","input":{"url":"https://w3id.org/headwater/","prompt":"p"}}]}}
EOF
cat > "$rec/config/B/projects/-var-tmp-hw-1659-x-ws-B/u-b.jsonl" <<'EOF'
{"type":"attachment","isSidechain":false,"attachment":{"type":"hook_non_blocking_error","hookName":"UserPromptSubmit","stderr":"hook script not found: /var/tmp/hw-1659/x/ws/B/.claude/hooks/intent.sh"}}
{"type":"attachment","isSidechain":false,"rendered":"Headwater routed this task","attachment":{"type":"hook_success","hookName":"UserPromptSubmit","content":"Headwater routed this task"}}
{"type":"attachment","isSidechain":true,"attachment":{"type":"hook_additional_context","hookName":"PreToolUse:Read","content":"x"}}
EOF
cat > "$rec/config/B/projects/-var-tmp-hw-1659-x-ws-B/u-b/subagents/agent-1.jsonl" <<'EOF'
{"type":"attachment","attachment":{"type":"hook_success","hookName":"PreToolUse:Grep","content":""}}
EOF
# Session D has a harness log and no stream, as a session that died before
# its first call does. Its attachments differ in what was shown and what was
# held, so the --text column must take the shown text first.
mkdir -p "$rec/config/D/projects/-var-tmp-hw-1659-x-ws-D"
cat > "$rec/config/D/projects/-var-tmp-hw-1659-x-ws-D/u-d.jsonl" <<'EOF'
{"type":"attachment","isSidechain":false,"rendered":"shown text","attachment":{"type":"hook_additional_context","hookName":"SessionStart:startup","hookEvent":"SessionStart","content":"raw content"}}
{"type":"attachment","isSidechain":false,"rendered":"y","attachment":{"type":"instructions","files":[{"path":"/w/CLAUDE.md"},{"path":"/w/docs/CLAUDE.md"}]}}
{"type":"attachment","isSidechain":false,"rendered":"","attachment":{"type":"skill_listing","names":["s1","s2"]}}
{"type":"attachment","isSidechain":false,"attachment":{"type":"hook_success","hookName":"Stop","content":"held content"}}
EOF
rec_sum() { (cd "$rec" && find . -type f -print0 | LC_ALL=C sort -z | xargs -0 sha256sum | sha256sum); }
rec_before=$(rec_sum)
python3 -I "$reader" --batch "$rec" A B > "$scratch/rec.out" 2> "$scratch/rec.err"
same "the reader of the recordings runs on a synthetic batch" "0" "$?"
same "no hook_non_blocking_error line for a session that only read a file quoting the error" "0" \
    "$(awk -F'\t' '$1=="A" && $3=="hook_non_blocking_error"' "$scratch/rec.out" | wc -l | tr -d ' ')"
same "exactly one hook_non_blocking_error line for the session whose log holds one" "1" \
    "$(awk -F'\t' '$1=="B" && $2=="log" && $3=="hook_non_blocking_error"' "$scratch/rec.out" | wc -l | tr -d ' ')"
same "and it says the model was not shown it" "no" \
    "$(awk -F'\t' '$1=="B" && $3=="hook_non_blocking_error" {print $6}' "$scratch/rec.out")"
same "the hook_success the model was shown reads rendered yes" "yes" \
    "$(awk -F'\t' '$1=="B" && $3=="hook_success" && $7=="no" {print $6}' "$scratch/rec.out")"
same "a sidechain attachment is marked as one" "1" \
    "$(awk -F'\t' '$1=="B" && $3=="hook_success" && $7=="yes"' "$scratch/rec.out" | wc -l | tr -d ' ')"
same "a Read call's target is the file path, not the result text" \
    "A	stream	tool_use	Read	/var/tmp/hw-1659/x/ws/A/tools/probe/arms	-	no" \
    "$(awk -F'\t' '$1=="A" && $3=="tool_use"' "$scratch/rec.out")"
absent "a tool result is never an event" "echo" "$scratch/rec.out"
same "and session A has two events, its Read call and its instructions attachment" "2" \
    "$(awk -F'\t' '$1=="A"' "$scratch/rec.out" | wc -l | tr -d ' ')"
same "a sidechain line of the session's own log is marked as one" "yes" \
    "$(awk -F'\t' '$1=="B" && $3=="hook_additional_context" {print $7}' "$scratch/rec.out")"
absent "and no text block is one" "tool: Read" "$scratch/rec.out"
same "a Skill call is a skill event named for the skill" "1" \
    "$(awk -F'\t' '$1=="B" && $3=="skill" && $4=="headwater-orient"' "$scratch/rec.out" | wc -l | tr -d ' ')"
same "a Bash target keeps the command on one line" 'headwater check\t--root .\r\nx' \
    "$(awk -F'\t' '$1=="B" && $4=="Bash" {print $5}' "$scratch/rec.out")"
same "a call inside a subagent is marked sidechain" "yes" \
    "$(awk -F'\t' '$1=="B" && $4=="Grep" {print $7}' "$scratch/rec.out")"
same "a call target prefers the path to the pattern" "docs" \
    "$(awk -F'\t' '$1=="B" && $4=="Grep" {print $5}' "$scratch/rec.out")"
same "every call of a message with two is an event" "docs/**/*.md https://w3id.org/headwater/" \
    "$(awk -F'\t' '$1=="B" && ($4=="Glob" || $4=="WebFetch") {print $5}' "$scratch/rec.out" | tr '\n' ' ' | sed 's/ $//')"
python3 -I "$reader" --batch "$rec" --text B D > "$scratch/rec-text.out" 2> "$scratch/rec-text.err"
same "a session with a log and no stream is read" "0" "$?"
same "with --text, every line has eight columns, a call's included" "0" \
    "$(awk -F'\t' 'NF!=8' "$scratch/rec-text.out" | wc -l | tr -d ' ')"
same "and a call's text is -" "-" \
    "$(awk -F'\t' '$1=="B" && $4=="Bash" {print $8}' "$scratch/rec-text.out")"
same "the text is what the model was shown, before what the attachment held" "shown text" \
    "$(awk -F'\t' '$1=="D" && $3=="hook_additional_context" {print $8}' "$scratch/rec-text.out")"
same "an attachment that was not shown gives its content" "held content" \
    "$(awk -F'\t' '$1=="D" && $4=="Stop" {print $8}' "$scratch/rec-text.out")"
same "and an error gives its stderr" "hook script not found: /var/tmp/hw-1659/x/ws/B/.claude/hooks/intent.sh" \
    "$(awk -F'\t' '$1=="B" && $3=="hook_non_blocking_error" {print $8}' "$scratch/rec-text.out")"
same "a hook is named by its hookName, not its event" "SessionStart:startup" \
    "$(awk -F'\t' '$1=="D" && $3=="hook_additional_context" {print $4}' "$scratch/rec-text.out")"
same "an instructions target joins its file paths with a comma" "/w/CLAUDE.md,/w/docs/CLAUDE.md" \
    "$(awk -F'\t' '$1=="D" && $3=="instructions" {print $5}' "$scratch/rec-text.out")"
same "a skill listing joins its names, and an empty rendered key still reads shown" "s1,s2 yes" \
    "$(awk -F'\t' '$1=="D" && $3=="skill_listing" {print $5, $6}' "$scratch/rec-text.out")"
same "the reader writes nothing under the batch it reads" "$rec_before" "$(rec_sum)"
python3 -I "$reader" --batch "$rec" 'A*' A > "$scratch/rec-glob.out" 2> "$scratch/rec-glob.err"
same "a session glob selects by session id" "A" "$(cut -f1 "$scratch/rec-glob.out" | LC_ALL=C sort -u)"
same "and two globs that match one session print it once" "2" \
    "$(wc -l < "$scratch/rec-glob.out" | tr -d ' ')"
python3 -I "$reader" --batch "$rec" C > "$scratch/rec-none.out" 2> "$scratch/rec-none.err"
same "a session id that matches nothing is refused" "2" "$?"

# ---------------------------------------------------------------------------
# The component arms (#1472). Each is built from the one present tree, and
# `--diff` prints how it differs from that tree, against the committed
# declaration.
# ---------------------------------------------------------------------------
layer_tree() {
    rm -rf "$scratch/layer"
    mkdir -p "$scratch/layer/.claude/hooks" "$scratch/layer/.claude/skills/s" \
        "$scratch/layer/.claude/agents" "$scratch/layer/.headwater" "$scratch/layer/docs/spec"
    : > "$scratch/layer/CLAUDE.md"
    : > "$scratch/layer/.claude/hooks/intent.sh"
    : > "$scratch/layer/.claude/hooks/write.sh"
    : > "$scratch/layer/.claude/settings.json"
    : > "$scratch/layer/.claude/skills/s/SKILL.md"
    : > "$scratch/layer/.claude/agents/a.md"
    : > "$scratch/layer/.headwater/taxonomy.lock"
    : > "$scratch/layer/docs/spec/05.md"
}
layer_tree
sh "$ablate" campaign "$scratch/layer" no-hook > /dev/null 2> "$scratch/layer.err"
same "ablate.sh builds the no-hook arm" "0" "$?"
if [ ! -e "$scratch/layer/.claude/hooks/intent.sh" ] && [ -e "$scratch/layer/.claude/settings.json" ] \
    && [ -e "$scratch/layer/.claude/hooks/write.sh" ] && [ -e "$scratch/layer/CLAUDE.md" ]; then
    pass "and it removes the intent hook's script and keeps the settings and every other hook"
else
    fail "and it removes the intent hook's script and keeps the settings and every other hook" \
        "$(cd "$scratch/layer" && find . -type f | LC_ALL=C sort | tr '\n' ' ')"
fi

layer_tree
for arm_delta in "no-hook|- .claude/hooks/intent.sh" "no-skills|- .claude/skills" \
    "no-claude-md|- CLAUDE.md" "mcp|+ .mcp.json"; do
    arm_name=${arm_delta%%|*}
    sh "$ablate" --diff campaign "$arm_name" "$scratch/layer" > "$scratch/diff.$arm_name" 2> "$scratch/diff.err"
    same "ablate.sh --diff campaign $arm_name exits 0" "0" "$?"
    same "and its delta against present is exactly its component" "${arm_delta#*|}" "$(cat "$scratch/diff.$arm_name")"
done
sh "$ablate" --diff campaign absent "$scratch/layer" > "$scratch/diff.absent" 2>&1
same "ablate.sh --diff campaign absent exits 0" "0" "$?"
same "and its delta is the ablation the tree holds" \
    "- .claude
- .headwater
- CLAUDE.md" "$(cat "$scratch/diff.absent")"
if [ -e "$scratch/layer/.claude/hooks/intent.sh" ]; then
    pass "--diff leaves the present tree as it was"
else
    fail "--diff leaves the present tree as it was" "it removed intent.sh from the present tree"
fi

rm -f "$scratch/layer/CLAUDE.md"
sh "$ablate" --diff campaign no-claude-md "$scratch/layer" > "$scratch/diff.same" 2> "$scratch/diff.same.err"
same "an arm whose tree does not differ from present fails --diff" "1" "$?"
present "and says so" "no different from the present tree" "$scratch/diff.same.err"

# A present tree that already declares a server: the mcp arm changes a file
# its delta does not add, and `--diff` names the change rather than passing it.
layer_tree
printf '{}\n' > "$scratch/layer/.mcp.json"
sh "$ablate" --diff campaign mcp "$scratch/layer" > "$scratch/diff.changed" 2>&1
same "an arm that changes a path its delta does not name fails --diff" "1" "$?"
present "and prints the change as unexpected" "unexpected ? Files" "$scratch/diff.changed"

layer_tree
mkdir -p "$scratch/layer/docs/probes"
sh "$ablate" --diff campaign no-hook "$scratch/layer" > /dev/null 2> "$scratch/diff.instrument.err"
same "--diff refuses a tree that still holds the instrument" "2" "$?"

# A block-sequence delta reads as the flow one does, because the engine reads
# both (verify round 1), and a delta that is neither names its own field.
awk '/^      no-hook: / { print "      no-hook:"; print "        - .claude/hooks/intent.sh"; next } { print }' \
    "$root/.headwater/probe.yml" > "$scratch/block-delta.yml"
layer_tree
HW_PROBE_YML="$scratch/block-delta.yml" sh "$ablate" --diff campaign no-hook "$scratch/layer" \
    > "$scratch/block-delta.out" 2> "$scratch/block-delta.err"
same "a block-sequence delta builds its arm" "0" "$?"
same "and its delta is the one path" "- .claude/hooks/intent.sh" "$(cat "$scratch/block-delta.out")"
# Verify round 2: the other forms the engine reads. Each builds the one path,
# and never reads as empty or as another arm's delta.
delta_form() {
    # $1 name, $2 awk program that rewrites the declaration
    awk "$2" "$root/.headwater/probe.yml" > "$scratch/form.yml"
    layer_tree
    HW_PROBE_YML="$scratch/form.yml" sh "$ablate" --diff campaign no-hook "$scratch/layer" \
        > "$scratch/form.out" 2> "$scratch/form.err"
    same "$1 builds its arm" "0" "$?"
    same "and its delta is the one path" "- .claude/hooks/intent.sh" "$(cat "$scratch/form.out")"
}
delta_form "a block-sequence delta at ten spaces" \
    '/^      no-hook: / { print "      no-hook:"; print "          - .claude/hooks/intent.sh"; next } { print }'
delta_form "a block-sequence delta at the key's own column" \
    '/^      no-hook: / { print "      no-hook:"; print "      - .claude/hooks/intent.sh"; next } { print }'
delta_form "a flow-sequence delta over several lines" \
    '/^      no-hook: / { print "      no-hook: ["; print "        .claude/hooks/intent.sh"; print "      ]"; next } { print }'
delta_form "a block-sequence delta followed by another arm's block sequence" \
    '/^      no-hook: / { print "      no-hook:"; print "        - .claude/hooks/intent.sh"; next }
     /^      no-skills: / { print "      no-skills:"; print "        - .claude/skills"; next } { print }'
awk '/^      no-hook: / { print "      no-hook: [.claude/hooks/intent.sh"; next } /^      no-skills: / { next } /^      no-claude-md: / { next } /^      mcp: / { next } { print }' \
    "$root/.headwater/probe.yml" > "$scratch/open-flow.yml"
HW_PROBE_YML="$scratch/open-flow.yml" sh "$ablate" --delta campaign no-hook > /dev/null 2> "$scratch/open-flow.err"
same "a flow sequence that never closes is refused" "2" "$?"
present "with the engine's reason, not as an empty delta" \
    "did not parse as YAML" "$scratch/open-flow.err"
absent "and never as an empty delta" "empty delta" "$scratch/open-flow.err"

# Verify round 3, finding 3: four forms the engine plans and a hand-written
# reader got wrong. The delta now comes from `headwater probe plan --delta`,
# so each builds the arm the engine reads. The last one also holds the arm
# after it: no-skills must still read as its own delta.
delta_form "a delta under a components key that carries a comment" \
    '/^    components:/ { print "    components: # the component arms"; next } { print }'
delta_form "a delta under a quoted key" \
    '/^      no-hook: / { print "      \"no-hook\": [.claude/hooks/intent.sh]"; next } { print }'
delta_form "a flow-sequence delta with a trailing comma" \
    '/^      no-hook: / { print "      no-hook: [.claude/hooks/intent.sh, ]"; next } { print }'
delta_form "a multi-line flow delta with a comment before its closing bracket" \
    '/^      no-hook: / { print "      no-hook: ["; print "        .claude/hooks/intent.sh, # the intent hook"; print "        # nothing else"; print "      ]"; next } { print }'
HW_PROBE_YML="$scratch/form.yml" sh "$ablate" --delta campaign no-skills > "$scratch/form-next.out" 2> "$scratch/form-next.err"
same "and the arm after it reads as its own delta" "declared - .claude/skills" "$(cat "$scratch/form-next.out")"

sed 's/^      no-hook: .*/      no-hook: .claude\/hooks\/intent.sh/' "$root/.headwater/probe.yml" > "$scratch/scalar-delta.yml"
HW_PROBE_YML="$scratch/scalar-delta.yml" sh "$ablate" --delta campaign no-hook \
    > /dev/null 2> "$scratch/scalar-delta.err"
same "a delta that is not a sequence is refused" "2" "$?"
present "and the refusal names the delta, not the ablation" \
    "the \`no-hook\` delta of the \`campaign\` tier in .headwater/probe.yml is not a sequence of paths" "$scratch/scalar-delta.err"

# Verify round 4: a delta entry that holds a line break would print as two
# lines of `--delta`, and the arm would lose a path nobody declared. The
# engine refuses the entry, so no tree is built.
sed 's/^      no-hook: .*/      no-hook: [".claude\/hooks\/intent.sh\\n- CLAUDE.md"]/' "$root/.headwater/probe.yml" > "$scratch/newline-delta.yml"
layer_tree
HW_PROBE_YML="$scratch/newline-delta.yml" sh "$ablate" campaign "$scratch/layer" no-hook \
    > /dev/null 2> "$scratch/newline-delta.err"
same "a delta entry that holds a line break is refused" "2" "$?"
present "and the refusal names the control character" "no control character" "$scratch/newline-delta.err"
if [ -e "$scratch/layer/CLAUDE.md" ] && [ -e "$scratch/layer/.claude/hooks/intent.sh" ]; then
    pass "and it removes nothing"
else
    fail "and it removes nothing" "$(cd "$scratch/layer" && find . -type f | LC_ALL=C sort | tr '\n' ' ')"
fi

layer_tree
sh "$ablate" campaign "$scratch/layer" no-docs > /dev/null 2> "$scratch/undeclared.err"
same "ablate.sh refuses an arm the tier declares no delta for" "2" "$?"
present "and names it" "\`--arm no-docs\` names no arm" "$scratch/undeclared.err"
if [ -e "$scratch/layer/.claude/hooks/intent.sh" ] && [ -e "$scratch/layer/CLAUDE.md" ]; then
    pass "and it removes nothing"
else
    fail "and it removes nothing" "$(cd "$scratch/layer" && find . -type f | LC_ALL=C sort | tr '\n' ' ')"
fi
layer_tree
sh "$ablate" regression "$scratch/layer" no-hook > /dev/null 2> "$scratch/not-run.err"
same "ablate.sh refuses a component arm the tier does not run" "2" "$?"
present "and says the tier does not run it" "does not run the \`no-hook\` arm" "$scratch/not-run.err"

layer_tree
sh "$ablate" campaign "$scratch/layer" mcp > /dev/null 2> "$scratch/mcp.err"
same "ablate.sh builds the mcp arm" "0" "$?"
if cmp -s "$root/tools/probe/arms/mcp/.mcp.json" "$scratch/layer/.mcp.json"; then
    pass "and it adds the declared server file from tools/probe/arms/mcp/"
else
    fail "and it adds the declared server file from tools/probe/arms/mcp/" "the .mcp.json differs or is missing"
fi

# ---------------------------------------------------------------------------
# The scenarios of the additive campaign (#1774), as HW-DR-0112 cuts them
# (#1872). The decisive case: each scenario's tree differs from its base by
# exactly the layers the decision names, and no scenario holds this
# repository's own tooling, in a present tree that holds the Codex copies,
# the census fixture, a built binary and the whole `settings.json` with more
# than one hook. A builder that leaves `AGENTS.md` or `.agents/` in plain
# code fails the second assertion, and one that lets scenario 6 keep
# `CLAUDE.md` or a `hw-*` skill fails the third. Either is the #1712
# problem: a scenario that differs from its base by more than its named
# layers.
#
# The table is the decision's, written here by hand, so a declaration that
# drifts from it fails here as well as in the engine's own suite. The paths of
# each layer are the committed declaration's, read from scenario 0's delta.
# ---------------------------------------------------------------------------
scenario_layers() {
    case $1 in
        0) echo "" ;;
        1) echo "docs" ;;
        2) echo "docs engine" ;;
        3) echo "docs engine skills" ;;
        4) echo "docs engine hooks" ;;
        5) echo "docs engine mcp" ;;
        6) echo "docs engine skills hooks mcp" ;;
    esac
}
scenario_base() {
    case $1 in
        1) echo 0 ;; 2) echo 1 ;; 3|4|5|6) echo 2 ;;
    esac
}
scenario_tree() {
    rm -rf "$scratch/scen"
    mkdir -p "$scratch/scen"
    # Plain code, which every scenario holds.
    for plain in src/main.rs engine/Cargo.toml engine/crates/census/src/lib.rs tools/t.sh .github/workflows/ci.yml; do
        mkdir -p "$scratch/scen/$(dirname "$plain")"
        printf 'plain %s\n' "$plain" > "$scratch/scen/$plain"
    done
    # The shapes the adjudication of #1774 names.
    for held in docs/spec/05.md docs/decisions/0001.md .headwater/taxonomy.lock CLAUDE.md AGENTS.md \
        .agents/skills/x/SKILL.md .codex/hooks.json engine/crates/census/fixtures/corpus.census \
        site/tutorial/index.html engine/target/dev-release/headwater .githooks/pre-commit \
        .claude/skills/headwater-orient/SKILL.md .claude/skills/hw-corpus/SKILL.md \
        .claude/agents/headwater-maintainer.md .claude/agents/hw-build.md .claude/commands/next.md \
        .claude/hooks/intent.sh .claude/hooks/lib.sh .claude/hooks/write.sh \
        .github/agents/headwater-maintainer.agent.md .github/hooks/hooks.json .github/prompts/next.prompt.md; do
        mkdir -p "$scratch/scen/$(dirname "$held")"
        printf 'held %s\n' "$held" > "$scratch/scen/$held"
    done
    cp "$root/.claude/settings.json" "$scratch/scen/.claude/settings.json"
    # The Copilot copy of the standing instructions is a link to CLAUDE.md, so
    # a scenario that keeps it holds a second path to CLAUDE.md that --diff
    # cannot see: the link itself does not change.
    ln -s ../CLAUDE.md "$scratch/scen/.github/copilot-instructions.md"
    # The engine's own copies of the skills text are real files, so that a
    # scenario that keeps them can be compared with this checkout's (#1837).
    mkdir -p "$scratch/scen/engine/crates/cli"
    cp -a "$root/engine/crates/cli/harness" "$scratch/scen/engine/crates/cli/harness"
    # Every other path a layer names, so each layer is in the tree.
    while IFS=' ' read -r sign layer path; do
        [ "$sign" = - ] || continue
        [ "$layer" != mcp ] || continue
        [ -e "$scratch/scen/$path" ] && continue
        mkdir -p "$scratch/scen/$(dirname "$path")"
        printf 'held %s\n' "$path" > "$scratch/scen/$path"
    done < "$scratch/scen-zero.delta"
}
"$engine" probe plan --root "$root" --scenario 0 --delta > "$scratch/scen-zero.delta" 2> "$scratch/scen-zero.err"
same "the engine prints plain code's delta" "0" "$?"
scenario_tree
if grep -q '"command":' "$scratch/scen/.claude/settings.json" &&
    [ "$(grep -c '"command":' "$scratch/scen/.claude/settings.json")" -gt 1 ]; then
    pass "the present tree's settings.json wires more than one hook"
else
    fail "the present tree's settings.json wires more than one hook" "$(cat "$scratch/scen/.claude/settings.json")"
fi

# Assertion 1: against plain code, each scenario differs by exactly the paths
# of its layers, and against its base, by the paths of the layers it adds:
# one layer, or the three of scenario 6, the declared exception.
for n in 0 1 2 3 4 5 6; do
    : > "$scratch/scen-expected"
    for layer in $(scenario_layers "$n"); do
        sed -n "s/^- $layer /+ /p" "$scratch/scen-zero.delta" >> "$scratch/scen-expected"
    done
    LC_ALL=C sort -u "$scratch/scen-expected" -o "$scratch/scen-expected"
    sh "$ablate" --diff --scenario "$n" "$scratch/scen" > "$scratch/scen-diff.$n" 2> "$scratch/scen-diff.$n.err"
    status=$?
    if [ "$n" = 0 ]; then
        same "scenario 0 against itself builds no difference and fails --diff" "1" "$status"
        present "and says so" "no different from scenario 0" "$scratch/scen-diff.0.err"
        continue
    fi
    same "ablate.sh --diff --scenario $n exits 0" "0" "$status"
    same "and its difference from scenario 0 is exactly its layers' paths" \
        "$(cat "$scratch/scen-expected")" "$(LC_ALL=C sort "$scratch/scen-diff.$n")"

    base=$(scenario_base "$n")
    added=
    for layer in $(scenario_layers "$n"); do
        case " $(scenario_layers "$base") " in
            *" $layer "*) ;;
            *) added="$added $layer" ;;
        esac
    done
    added=$(echo $added)
    sh "$ablate" --diff --scenario "$n" "$scratch/scen" "$base" > "$scratch/scen-base.$n" 2> "$scratch/scen-base.$n.err"
    same "ablate.sh --diff --scenario $n against its base $base exits 0" "0" "$?"
    if [ "$n" = 6 ]; then
        same "and it adds the three layers of the exception to its base" "skills hooks mcp" "$added"
    else
        same "and it adds one layer to its base" "1" "$(echo $added | wc -w | tr -d ' ')"
    fi
    stray=$(while IFS=' ' read -r sign path; do
        owned=
        for layer in $added; do
            grep -qxF -- "- $layer $path" "$scratch/scen-zero.delta" && owned=yes
        done
        [ -n "$owned" ] || printf '%s %s; ' "$sign" "$path"
    done < "$scratch/scen-base.$n")
    same "and every path it differs by belongs to an added layer, $added" "" "$stray"
    if [ ! -s "$scratch/scen-base.$n" ]; then
        fail "and it differs from its base at all" "it printed nothing"
    fi
done
same "scenario 6 differs from 2 by the settings file the hooks layer writes" "+ .claude/settings.json" \
    "$(grep -F 'settings.json' "$scratch/scen-base.6")"

# Assertion 2: plain code holds no Headwater path.
cp -a "$scratch/scen" "$scratch/scen-zero"
sh "$ablate" --scenario 0 "$scratch/scen-zero" > /dev/null 2> "$scratch/scen-zero-build.err"
same "ablate.sh --scenario 0 builds plain code" "0" "$?"
left=
for headwater_path in docs .headwater CLAUDE.md AGENTS.md .claude .agents .codex .githooks .mcp.json \
    engine/crates/census/fixtures/corpus.census site/tutorial engine/target engine/crates/cli/harness \
    .github/copilot-instructions.md .github/agents .github/hooks .github/prompts; do
    if [ -e "$scratch/scen-zero/$headwater_path" ] || [ -L "$scratch/scen-zero/$headwater_path" ]; then
        left="$left $headwater_path"
    fi
done
same "and it holds no Headwater path, and no directory one left empty" "" "$left"
if [ -f "$scratch/scen-zero/src/main.rs" ] && [ -f "$scratch/scen-zero/engine/crates/census/src/lib.rs" ] &&
    [ -f "$scratch/scen-zero/tools/t.sh" ] && [ -f "$scratch/scen-zero/.github/workflows/ci.yml" ]; then
    pass "and it keeps the code, the engine source included"
else
    fail "and it keeps the code, the engine source included" \
        "$(cd "$scratch/scen-zero" && find . -type f | LC_ALL=C sort | tr '\n' ' ')"
fi

# Assertion 3: the hooks layer writes the settings file `headwater init
# --harness` writes, six entries over four events, and keeps the six scripts.
# No scenario keeps the tooling: scenario 6, what `headwater init` ships,
# holds every layer and none of this repository's own instructions, skills,
# commands or Git hooks.
cp -a "$scratch/scen" "$scratch/scen-four"
sh "$ablate" --scenario 4 "$scratch/scen-four" > /dev/null 2> "$scratch/scen-four.err"
same "ablate.sh --scenario 4 builds the hooks scenario" "0" "$?"
same "and its settings.json wires six hooks" "6" \
    "$(grep -c '"command":' "$scratch/scen-four/.claude/settings.json" 2>/dev/null)"
if cmp -s "$root/tools/probe/arms/hooks/.claude/settings.json" "$scratch/scen-four/.claude/settings.json"; then
    pass "and it is the hooks layer's arms file"
else
    fail "and it is the hooks layer's arms file" "it differs from tools/probe/arms/hooks/.claude/settings.json"
fi
for event in UserPromptSubmit PreToolUse PostToolUse Stop; do
    present "and it wires $event" "\"$event\"" "$scratch/scen-four/.claude/settings.json"
done
missing=
for script in intent read write review lib touch; do
    [ -f "$scratch/scen-four/.claude/hooks/$script.sh" ] || missing="$missing $script"
done
same "and it keeps the six scripts" "" "$missing"
if [ ! -e "$scratch/scen-four/.claude/hooks/derived.sh" ] && [ ! -e "$scratch/scen-four/AGENTS.md" ] &&
    [ ! -e "$scratch/scen-four/CLAUDE.md" ] && [ ! -e "$scratch/scen-four/.githooks" ]; then
    pass "and no hook of this repository's own, Codex copy, CLAUDE.md or Git hook"
else
    fail "and no hook of this repository's own, Codex copy, CLAUDE.md or Git hook" \
        "$(cd "$scratch/scen-four" && find .claude AGENTS.md CLAUDE.md .githooks -type f 2>/dev/null | LC_ALL=C sort | tr '\n' ' ')"
fi
rm -rf "$scratch/scen-four"
cp -a "$scratch/scen" "$scratch/scen-six"
sh "$ablate" --scenario 6 "$scratch/scen-six" > /dev/null 2> "$scratch/scen-six.err"
same "ablate.sh --scenario 6 builds what headwater init ships" "0" "$?"
if cmp -s "$root/tools/probe/arms/hooks/.claude/settings.json" "$scratch/scen-six/.claude/settings.json"; then
    pass "and its settings.json is the hooks layer's, not this repository's"
else
    fail "and its settings.json is the hooks layer's, not this repository's" "it differs from the arms file"
fi
left=
for tooling in CLAUDE.md AGENTS.md .codex .githooks .claude/commands .claude/skills/hw-corpus \
    .claude/skills/headwater-engine .claude/agents/hw-build.md .claude/hooks/derived.sh \
    .github/copilot-instructions.md .github/agents .github/hooks .github/prompts; do
    if [ -e "$scratch/scen-six/$tooling" ] || [ -L "$scratch/scen-six/$tooling" ]; then
        left="$left $tooling"
    fi
done
same "and it holds none of the tooling" "" "$left"
if [ -f "$scratch/scen-six/.claude/skills/headwater-orient/SKILL.md" ] &&
    [ -f "$scratch/scen-six/.claude/hooks/review.sh" ] && [ -f "$scratch/scen-six/.mcp.json" ] &&
    [ -f "$scratch/scen-six/.github/workflows/ci.yml" ]; then
    pass "and it holds a shipped skill, a shipped hook and the server, and keeps the workflows"
else
    fail "and it holds a shipped skill, a shipped hook and the server, and keeps the workflows" \
        "$(cd "$scratch/scen-six" && find .claude .mcp.json .github -type f 2>/dev/null | LC_ALL=C sort | tr '\n' ' ')"
fi
rm -rf "$scratch/scen-six"

# Assertion 4 (#1837): the engine's own copies of the skills text under
# `engine/crates/cli/harness/` go with the skills layer. A scenario without
# the layer holds neither directory, and keeps the rest of the engine source.
# A scenario with it keeps both as they are in this checkout, and each file
# is its `.claude/` mirror less the one marker line `headwater init --harness`
# appends to the mirror.
cp -a "$scratch/scen" "$scratch/scen-three"
sh "$ablate" --scenario 2 "$scratch/scen-three" > /dev/null 2> "$scratch/scen-three.err"
same "ablate.sh --scenario 2 builds the engine scenario" "0" "$?"
left=
for harness_dir in skills agents; do
    [ -e "$scratch/scen-three/engine/crates/cli/harness/$harness_dir" ] && left="$left $harness_dir"
done
same "and it holds neither of the engine's copies of the skills text" "" "$left"
if [ -f "$scratch/scen-three/engine/crates/census/src/lib.rs" ]; then
    pass "and it keeps the rest of the engine source"
else
    fail "and it keeps the rest of the engine source" "engine/crates/census/src/lib.rs is gone"
fi
rm -rf "$scratch/scen-three"
cp -a "$scratch/scen" "$scratch/scen-four"
sh "$ablate" --scenario 3 "$scratch/scen-four" > /dev/null 2> "$scratch/scen-four.err"
same "ablate.sh --scenario 3 builds the skills scenario" "0" "$?"
for harness_dir in skills agents; do
    if [ -d "$scratch/scen-four/engine/crates/cli/harness/$harness_dir" ] &&
        diff -r "$root/engine/crates/cli/harness/$harness_dir" \
            "$scratch/scen-four/engine/crates/cli/harness/$harness_dir" > "$scratch/scen-four.diff" 2>&1; then
        pass "and it keeps engine/crates/cli/harness/$harness_dir as this checkout has it"
    else
        fail "and it keeps engine/crates/cli/harness/$harness_dir as this checkout has it" \
            "$(head -5 "$scratch/scen-four.diff")"
    fi
done
unequal=
for copy in skills/headwater-orient/SKILL.md skills/headwater-authoring/SKILL.md \
    skills/headwater-taxonomy/SKILL.md skills/headwater-sweep/SKILL.md agents/headwater-maintainer.md; do
    sed '${/^<!-- installed by headwater init --harness, digest sha256:[0-9a-f]* -->$/d;}' \
        "$root/.claude/$copy" > "$scratch/scen-mirror"
    cmp -s "$scratch/scen-mirror" "$scratch/scen-four/engine/crates/cli/harness/$copy" || unequal="$unequal $copy"
done
same "and each of the five is its .claude/ mirror less the init marker" "" "$unequal"
rm -rf "$scratch/scen-four"
sh "$ablate" --diff --scenario 3 "$scratch/scen" 2 > "$scratch/scen-four-three" 2> "$scratch/scen-four-three.err"
same "ablate.sh --diff --scenario 3 against 2 exits 0" "0" "$?"
for harness_dir in skills agents; do
    present "and it adds engine/crates/cli/harness/$harness_dir" "+ engine/crates/cli/harness/$harness_dir" \
        "$scratch/scen-four-three"
done
absent "and it names nothing unexpected" "unexpected" "$scratch/scen-four-three"

# The refusals, each before anything is removed.
sh "$ablate" --scenario 7 "$scratch/scen" > /dev/null 2> "$scratch/scen-14.err"
same "ablate.sh refuses a scenario the declaration does not carry" "2" "$?"
present "and the engine names the scenarios it does carry" "declares no scenario 7. It declares 0, 1, 2, 3, 4, 5, 6" "$scratch/scen-14.err"
[ -f "$scratch/scen/CLAUDE.md" ] && pass "and it removes nothing" || fail "and it removes nothing" "CLAUDE.md is gone"
sh "$ablate" --scenario 0 "$root/tools" > /dev/null 2> "$scratch/scen-inside.err"
same "ablate.sh refuses to build a scenario inside this checkout" "6" "$?"
mkdir -p "$scratch/scen/docs/probes"
sh "$ablate" --diff --scenario 1 "$scratch/scen" > /dev/null 2> "$scratch/scen-instrument.err"
same "--diff --scenario refuses a tree that still holds the instrument" "2" "$?"
rm -rf "$scratch/scen/docs/probes"
# A declaration whose hooks layer writes a file `tools/probe/arms/` does not
# hold is refused before the tree is touched.
sed 's/^    arms: \[\.claude\/settings\.json\]$/    arms: [.claude\/settings.local.json]/' "$root/.headwater/probe.yml" \
    > "$scratch/scen-missing-arm.yml"
cp -a "$scratch/scen" "$scratch/scen-missing"
HW_PROBE_YML="$scratch/scen-missing-arm.yml" sh "$ablate" --scenario 4 "$scratch/scen-missing" \
    > /dev/null 2> "$scratch/scen-missing.err"
same "ablate.sh refuses a written file that tools/probe/arms/ does not hold" "2" "$?"
present "and names it" "tools/probe/arms/hooks/.claude/settings.local.json is not there to copy" "$scratch/scen-missing.err"
[ -f "$scratch/scen-missing/CLAUDE.md" ] && pass "and it removes nothing" || fail "and it removes nothing" "CLAUDE.md is gone"
# A present tree that already holds a server file: plain code and scenario 2
# remove it, and scenario 5 holds the mcp layer's file in its place, so 5
# differs from 2 by the one path.
printf '{}\n' > "$scratch/scen/.mcp.json"
sh "$ablate" --diff --scenario 5 "$scratch/scen" 2 > "$scratch/scen-mcp.out" 2>&1
same "a present server file is replaced by the mcp layer's, and --diff passes it" "0" "$?"
same "and names the one path" "+ .mcp.json" "$(cat "$scratch/scen-mcp.out")"
rm -f "$scratch/scen/.mcp.json"
# A difference outside the expected paths fails --diff: plain code against
# scenario 1 removes the documents, and nothing expects a removal.
sh "$ablate" --diff --scenario 0 "$scratch/scen" 1 > "$scratch/scen-reverse.out" 2> "$scratch/scen-reverse.err"
same "--diff --scenario fails a difference it does not expect" "1" "$?"
present "and names it" "unexpected - docs/spec/05.md" "$scratch/scen-reverse.out"
# A workspace that still holds the instrument loses it in every scenario,
# also one that holds the documents layer.
cp -a "$scratch/scen" "$scratch/scen-instrumented"
mkdir -p "$scratch/scen-instrumented/docs/probes"
printf 'probe\n' > "$scratch/scen-instrumented/docs/probes/p.md"
sh "$ablate" --scenario 6 "$scratch/scen-instrumented" > /dev/null 2> "$scratch/scen-instrumented.err"
same "ablate.sh --scenario 6 builds a workspace that holds the instrument" "0" "$?"
if [ ! -e "$scratch/scen-instrumented/docs/probes" ] && [ -f "$scratch/scen-instrumented/docs/spec/05.md" ]; then
    pass "and it removes the instrument and keeps the documents"
else
    fail "and it removes the instrument and keeps the documents" \
        "$(cd "$scratch/scen-instrumented" && find docs -type f | LC_ALL=C sort | tr '\n' ' ')"
fi
rm -rf "$scratch/scen" "$scratch/scen-zero" "$scratch/scen-missing" \
    "$scratch/scen-instrumented"

# ---------------------------------------------------------------------------
# The dry run of the layer campaign (#1472): the committed spec, planned,
# priced, its trees built and diffed, and the leak check run, with a harness
# on the path that records any call to it.
# ---------------------------------------------------------------------------
if [ -x "$engine" ]; then
    mkdir -p "$scratch/dry-bin"
    printf '#!/bin/sh\n: > "%s/claude-called"\nexit 7\n' "$scratch" > "$scratch/dry-bin/claude"
    chmod +x "$scratch/dry-bin/claude"
    rm -f "$scratch/claude-called"
    PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$root/tools/probe/layer-campaign.spec" --max-sessions 500 > "$scratch/dry.out" 2> "$scratch/dry.err"
    same "the dry run of the layer campaign exits 0" "0" "$?"
    if [ -e "$scratch/claude-called" ]; then
        fail "and it calls no model" "the harness on the path was called"
    else
        pass "and it calls no model"
    fi
    present "it prints the power calculation, uncorrected and corrected" \
        "needs 325 sessions per arm, 353 with the Fleiss continuity correction" "$scratch/dry.out"
    present "and prices a discovery line at the powered repetitions" \
        "campaign no-hook discovery: 3 probes x 118 repetitions" "$scratch/dry.out"
    present "and sums each arm" "arm campaign mcp: 624 sessions, \$312.00" "$scratch/dry.out"
    present "and holds the campaign tier to its ceiling" \
        "tier campaign: 3744 sessions, \$1872.00 against a ceiling of \$2052.00: inside it" "$scratch/dry.out"
    present "and prints the total" "total: 3894 sessions, \$1947.00 against the \$2182.00 the tiers declare" "$scratch/dry.out"
    present "and sums each tier and category with its share of the total" \
        "category campaign discovery: 2124 sessions, \$1062.00, 54.5% of the total" "$scratch/dry.out"
    present "and sums a leak-kept line apart from its category" \
        "category campaign sufficiency (leak-kept): 360 sessions, \$180.00" "$scratch/dry.out"
    present "and prints the stagger of --max-sessions, rounded up where 500 does not divide 3894" \
        "slices: 3894 sessions in 8 slices of at most 500" "$scratch/dry.out"
    # A bound that divides the session count gives the quotient, not one more.
    PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$root/tools/probe/layer-campaign.spec" --max-sessions 1947 \
        > "$scratch/dry-even.out" 2> "$scratch/dry-even.err"
    present "and a bound that divides the sessions evenly gives the quotient" \
        "slices: 3894 sessions in 2 slices of at most 1947" "$scratch/dry-even.out"
    present "and the delta of the no-hook arm is the hook's script alone" \
        "tree campaign no-hook: - .claude/hooks/intent.sh" "$scratch/dry.out"
    present "and the mcp arm adds its server" "tree campaign mcp: + .mcp.json" "$scratch/dry.out"
    present "and the server starts in the mcp tree and lists its tools" \
        "lists route explain" "$scratch/dry.out"
    present "and the leak check reports the status probe as kept" \
        "leak check, present tree: kept $status_probe .claude/skills/hw-corpus/SKILL.md HW-DR-0052" "$scratch/dry.out"
    present "and each leak-kept line on its own" "line 8 holds only leak-kept probes" "$scratch/dry.out"

    # A line that pools a leak-kept probe with one that is not fails the dry run,
    # and a plan over its ceiling is printed rather than fatal: fifty
    # sufficiency probes over the documentation tier's two arms at 30
    # repetitions is 3000 sessions, $1500.00 against its $130.00. Two of the
    # fifty are the harder variants of #1718, two are the patched probes
    # of the code set (#1835), eight are the probes of rows 4 and 5 (#1874),
    # fourteen are the probes of rows 6 to 9 (#1874.3), eight are the
    # probes of rows 10 and 11 (#1874.4), and eight are the probes of rows 2
    # and 3 (#1874, slice 4b).
    # The plan selects by category and not by status, so the six retired
    # sufficiency probes count too. This line names them by category and not
    # by identifier. The campaign
    # tier no longer serves: no plan of one line passes its $2052.00 (#1659).
    printf 'documentation absent sufficiency\n' > "$scratch/pooled.spec"
    PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$scratch/pooled.spec" > "$scratch/pooled.out" 2> "$scratch/pooled.err"
    same "a line that pools a leak-kept probe fails the dry run with 8, not 5" "8" "$?"
    present "and names the line" "line 1 pools a probe under \`leaks_kept:\`" "$scratch/pooled.out"
    present "and the ceiling's refusal is printed as a line" \
        "L1 3000 sessions project \$1500.00 against a declared ceiling of \$130.00" "$scratch/pooled.out"

    # Any other refusal of a plan is the batch driver's 5.
    printf 'campaign present discovery\n' > "$scratch/refused.spec"
    PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$scratch/refused.spec" > "$scratch/refused.out" 2> "$scratch/refused.err"
    same "a plan refused for another reason fails the dry run with 5" "5" "$?"

    # Batch B, the six discovery lines, at the 118 the dry run prices them
    # at (#1659): each line is planned at the tier's 30 and priced at 118.
    awk 'NF && $1 !~ /^#/ && $3 == "discovery"' "$root/tools/probe/layer-campaign.spec" > "$scratch/batch-b.spec"
    PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$scratch/batch-b.spec" --repetitions 118 > "$scratch/batch-b.out" 2> "$scratch/batch-b.err"
    same "the dry run of the discovery lines at --repetitions 118 exits 0" "0" "$?"
    present "and prices each line at 118" \
        "line 1  campaign present discovery: 3 probes x 118 repetitions (asked; the plan is at the tier's 30) = 354 sessions, \$177.00" \
        "$scratch/batch-b.out"
    present "and sums the category" "category campaign discovery: 2124 sessions, \$1062.00" "$scratch/batch-b.out"
    PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$scratch/batch-b.spec" --repetitions 119 > "$scratch/batch-b-119.out" 2> "$scratch/batch-b-119.err"
    same "and at 119 it exits 5" "5" "$?"
    present "and names the line and the price" \
        "line 1 (campaign present discovery): --repetitions 119 is above the 118 the dry run prices for it (ceil(353 / 3), powered)" \
        "$scratch/batch-b-119.out"
    # A raise below the price is priced at the count asked, not at the price
    # (verify of #1659 slice 1b).
    PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$scratch/batch-b.spec" --repetitions 100 > "$scratch/batch-b-100.out" 2> "$scratch/batch-b-100.err"
    same "the dry run of the discovery lines at --repetitions 100 exits 0" "0" "$?"
    present "and prices each line at the 100 asked, not the price of 118" \
        "line 1  campaign present discovery: 3 probes x 100 repetitions (asked; the plan is at the tier's 30) = 300 sessions, \$150.00" \
        "$scratch/batch-b-100.out"
    # A count given is the count priced, so a pilot below the tier's count
    # prices what it runs.
    PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$scratch/batch-b.spec" --repetitions 10 > "$scratch/batch-b-10.out" 2> "$scratch/batch-b-10.err"
    same "the dry run of the discovery lines at --repetitions 10 exits 0" "0" "$?"
    present "and prices a discovery line at the 10 asked, not the powered 118" \
        "campaign present discovery: 3 probes x 10 repetitions" "$scratch/batch-b-10.out"
    # A sufficiency line prices at the tier's 30, so a raise above it refuses.
    printf 'campaign present sufficiency\n' > "$scratch/flat.spec"
    PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$scratch/flat.spec" --repetitions 31 > "$scratch/flat.out" 2> "$scratch/flat.err"
    same "a dry run of a sufficiency line at --repetitions 31 exits 5" "5" "$?"
    present "and names the tier's count as the price" \
        "line 1 (campaign present sufficiency): --repetitions 31 is above the 30 the dry run prices for it (the tier's 30)" \
        "$scratch/flat.out"
    # The priced counts the batch driver reads: one line each, and no tree.
    sh "$root/tools/probe/campaign-dry-run.sh" --priced "$scratch/batch-b.spec" > "$scratch/priced.out" 2> "$scratch/priced.err"
    same "the priced mode exits 0 and prints the index, the declared and the priced count of each line, and how it priced it" \
        "0 1 30 118 ceil(353 / 3), powered|6 30 118 ceil(353 / 3), powered|6" \
        "$? $(sed -n 1p "$scratch/priced.out")|$(sed -n 6p "$scratch/priced.out")|$(wc -l < "$scratch/priced.out" | tr -d ' ')"
    printf 'documentation no-hook sufficiency\n' > "$scratch/arm.spec"
    PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$scratch/arm.spec" > "$scratch/arm.out" 2> "$scratch/arm.err"
    same "a line whose tier does not run its arm fails the dry run with 5" "5" "$?"
    # The other three causes of exit 8 (verify round 1), each on its own.
    printf 'campaign present sufficiency HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks\n' \
        > "$scratch/kept.spec"
    HW_PROBE_YML="$scratch/leak-probe.yml" PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$scratch/kept.spec" > "$scratch/dry-leak.out" 2> "$scratch/dry-leak.err"
    same "a leak string no declaration keeps fails the dry run with 8" "8" "$?"
    present "and the dry run prints the leak" \
        "leak check, present tree: leak $status_probe .claude/skills/hw-corpus/SKILL.md HW-DR-0052" "$scratch/dry-leak.out"

    sed 's/^      no-hook: .*/      no-hook: [.claude\/hooks\/no-such-hook.sh]/' "$root/.headwater/probe.yml" > "$scratch/no-such-hook.yml"
    printf 'campaign no-hook navigability\n' > "$scratch/tree.spec"
    HW_PROBE_YML="$scratch/no-such-hook.yml" PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$scratch/tree.spec" > "$scratch/dry-tree.out" 2> "$scratch/dry-tree.err"
    same "an arm whose tree is not its delta fails the dry run with 8" "8" "$?"
    present "and the dry run names the arm" "tree campaign no-hook is not its delta" "$scratch/dry-tree.out"

    printf '#!/bin/sh\nexit 0\n' > "$scratch/dry-bin/silent-server"
    chmod +x "$scratch/dry-bin/silent-server"
    printf 'campaign mcp navigability\n' > "$scratch/mcp.spec"
    HW_PROBE_ENGINE="$scratch/dry-bin/silent-server" PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$scratch/mcp.spec" > "$scratch/dry-mcp.out" 2> "$scratch/dry-mcp.err"
    same "a server that lists no tool fails the dry run with 8" "8" "$?"
    present "and the dry run says so" "listed no tool in the mcp tree" "$scratch/dry-mcp.out"

    # Verify round 5: the dry run reads `leaks_kept:` and `power:` with the
    # parser the leak check uses, so a flow sequence keeps its probes and a
    # quoted rate is refused rather than read as 0.
    awk -v a="$status_probe" -v b=HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted '
        /^leaks_kept:/ { print "leaks_kept: [" a ", " b "]"; skip = 1; next }
        skip && /^  - / { next }
        { skip = 0; print }
    ' "$root/.headwater/probe.yml" > "$scratch/flow-kept.yml"
    HW_PROBE_YML="$scratch/flow-kept.yml" PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$root/tools/probe/layer-campaign.spec" > "$scratch/flow-kept.out" 2> "$scratch/flow-kept.err"
    same "a flow-sequence leaks_kept keeps the committed dry run at 0" "0" "$?"
    present "and each leak-kept line is still on its own" "line 8 holds only leak-kept probes" "$scratch/flow-kept.out"
    printf 'campaign present sufficiency %s HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer\n' \
        "$status_probe" > "$scratch/mixed.spec"
    HW_PROBE_YML="$scratch/flow-kept.yml" PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$scratch/mixed.spec" > "$scratch/flow-mixed.out" 2> "$scratch/flow-mixed.err"
    same "a line that pools a flow-kept probe with one that is not fails the dry run with 8" "8" "$?"
    present "and names the line" "line 1 pools a probe under \`leaks_kept:\`" "$scratch/flow-mixed.out"
    HW_PROBE_YML="$scratch/leak-probe.yml" PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$scratch/mixed.spec" > /dev/null 2>&1
    same "and with no probe kept the same line is a leak, 8" "8" "$?"

    # A `leaks_kept:` that is not a sequence of identifiers is refused, never
    # read as a list of its characters or as nothing.
    awk -v a="$status_probe" '
        /^leaks_kept:/ { print "leaks_kept: " a; skip = 1; next }
        skip && /^  - / { next }
        { skip = 0; print }
    ' "$root/.headwater/probe.yml" > "$scratch/scalar-kept.yml"
    HW_PROBE_YML="$scratch/scalar-kept.yml" PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$root/tools/probe/layer-campaign.spec" > /dev/null 2> "$scratch/scalar-kept.err"
    same "a leaks_kept that is one scalar is refused with 2" "2" "$?"
    present "and the refusal says why" "is not a sequence of probe identifiers" "$scratch/scalar-kept.err"

    dry_power() {
        # $1 name, $2 sed program over the power block, $3 text of the refusal
        sed "$2" "$root/.headwater/probe.yml" > "$scratch/power.yml"
        HW_PROBE_YML="$scratch/power.yml" PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
            --spec "$root/tools/probe/layer-campaign.spec" > "$scratch/power.out" 2> "$scratch/power.err"
        same "$1 is refused with 2" "2" "$?"
        present "and the refusal says why" "$3" "$scratch/power.err"
        absent "and nothing is priced" "tier campaign:" "$scratch/power.out"
    }
    dry_power "a quoted rate" 's/^  present: 0\.15/  present: "0.15"/' "must be a number"
    dry_power "a rate that is a word" 's/^  absent: 0\.08/  absent: low/' "must be a number"
    dry_power "a rate outside (0, 1)" 's/^  present: 0\.15/  present: 15/' "strictly between 0 and 1"
    dry_power "a power block with no alpha" '/^  alpha: /d' "must be a number"

    if [ -e "$scratch/claude-called" ]; then
        fail "no refused dry run calls a model" "the harness on the path was called"
    else
        pass "no refused dry run calls a model"
    fi
else
    printf 'note no engine, so the dry-run cases did not run.\n'
fi

# ---------------------------------------------------------------------------
# The trees of a dry run (`tools/probe/dry-run-trees.sh`). Every dry run of
# this suite keeps its trees under `HW_CAMPAIGN_TREE_CACHE`, so these cases
# hold the three things that make that safe: a run that keeps nothing prints
# what a run that reads kept trees prints, the leak strings are not in the
# key, and the rest of the declaration is.
# ---------------------------------------------------------------------------
if [ -x "$engine" ]; then
    layer_spec=$root/tools/probe/layer-campaign.spec
    PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$layer_spec" --max-sessions 500 > "$scratch/dry-kept.out" 2> /dev/null
    same "a dry run over kept trees exits 0" "0" "$?"
    same "and prints what the run that built them printed" "" \
        "$(diff "$scratch/dry.out" "$scratch/dry-kept.out" | head -5)"
    env -u HW_CAMPAIGN_TREE_CACHE PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$layer_spec" --max-sessions 500 > "$scratch/dry-unkept.out" 2> /dev/null
    same "a dry run that keeps no tree exits 0" "0" "$?"
    same "and prints what the run over kept trees printed" "" \
        "$(diff "$scratch/dry-kept.out" "$scratch/dry-unkept.out" | head -5)"
    kept_keys() { find "$HW_CAMPAIGN_TREE_CACHE" -mindepth 1 -maxdepth 1 -type d | wc -l | tr -d ' '; }
    same "the cache holds one set of trees" "1" "$(kept_keys)"
    kept_key=$(find "$HW_CAMPAIGN_TREE_CACHE" -mindepth 1 -maxdepth 1 -type d | head -1)
    : > "$kept_key/present/.fixture-marker"

    # Other leak strings: the trees are the same, so the run reads them.
    awk '
        /^leaks:/ { skip = 1; print "leaks: {}"; next }
        skip && /^[^ #]/ { skip = 0 }
        skip { next }
        { print }
    ' "$root/.headwater/probe.yml" > "$scratch/trees-leaks.yml"
    same "the fixture's leak strings differ from the committed ones" "yes" \
        "$(cmp -s "$root/.headwater/probe.yml" "$scratch/trees-leaks.yml" && echo no || echo yes)"
    HW_PROBE_YML="$scratch/trees-leaks.yml" PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$layer_spec" --max-sessions 500 > /dev/null 2>&1
    same "a declaration that differs in its leak strings alone reads the kept trees" "yes" \
        "$([ -e "$kept_key/present/.fixture-marker" ] && echo yes || echo no)"

    # Another power: no tree reads it, but it is in the key, so the trees are
    # built again and the old set is removed.
    sed 's/^  present: 0\.15/  present: 0.16/' "$root/.headwater/probe.yml" > "$scratch/trees-power.yml"
    same "the fixture's power differs from the committed one" "yes" \
        "$(cmp -s "$root/.headwater/probe.yml" "$scratch/trees-power.yml" && echo no || echo yes)"
    HW_PROBE_YML="$scratch/trees-power.yml" PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$layer_spec" --max-sessions 500 > /dev/null 2>&1
    same "a declaration that differs outside its leak strings builds the trees again" "no" \
        "$([ -e "$kept_key/present/.fixture-marker" ] && echo yes || echo no)"
    same "and the cache still holds one set of trees" "1" "$(kept_keys)"
else
    printf 'note no engine, so the cases of the kept trees did not run.\n'
fi

# ---------------------------------------------------------------------------
# The dry run of the additive campaign (#1774, clauses 5 and 7): the committed
# spec priced by round, with one total, and the leak check run over every
# scenario tree. A check that read only the present tree, or no skill body, or
# no hook text, names the wrong scenarios below.
# ---------------------------------------------------------------------------
if [ -x "$engine" ]; then
    mkdir -p "$scratch/dry-bin"
    printf '#!/bin/sh\n: > "%s/claude-called"\nexit 7\n' "$scratch" > "$scratch/dry-bin/claude"
    chmod +x "$scratch/dry-bin/claude"
    rm -f "$scratch/claude-called"
    additive_spec=$root/tools/probe/additive-campaign.spec
    PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$additive_spec" > "$scratch/add.out" 2> "$scratch/add.err"
    same "the dry run of the additive campaign exits 0" "0" "$?"
    present "it prints the screening round's sum" "round screening: 616 sessions, \$308.00" "$scratch/add.out"
    # The tuning round runs in rounds (#1838): each tuning line is priced as
    # one round of 4 variants at 22 sessions, a confirmation at 154 and the
    # best so far at 154, 396 sessions over the 22 probes of `tuning`, at
    # most 4 rounds (#1874).
    present "it prints the tuning round's sum at its most rounds" "round tuning: 11088 sessions, \$5544.00 (4 rounds at most)" "$scratch/add.out"
    present "it prints the two counts HW-DR-0112 states, each rounded up over the 22 probes" \
        "the owner ruled 22 sessions for a screened variant and 145 for a confirmation, so ceil(22 / 22) = 1 and ceil(145 / 22) = 7 repetitions" "$scratch/add.out"
    present "and prices a whole round on a tuning line" \
        "line 8  additive scenario-0 tuning: a round of 4 variants x 22 probes x 1 repetitions (ceil(22 / 22)) = 88 sessions, a confirmation of 22 probes x 7 repetitions (round tuning, ceil(145 / 22)) = 154 sessions, and the best so far at 154 sessions where no measurement at that count exists: 396 sessions, \$198.00 a round" "$scratch/add.out"
    # The final round is phase B (#1881): scenarios 1 and 6 per row at 6
    # repetitions in one batch, 132 sessions each, and the other five
    # scenarios at its 3, 66 sessions each.
    present "it prints the final round's sum" "round final: 594 sessions, \$297.00" "$scratch/add.out"
    present "and one total" "total: 12298 sessions, \$6149.00" "$scratch/add.out"
    present "and the cached share of input, with no batch recorded, as no recorded sessions (#1875)" \
        "cached share: no recorded sessions" "$scratch/add.out"
    present "each round's count is computed from its rates" "so ceil(70 / 22) = 4 repetitions of each probe" "$scratch/add.out"
    present "and the final round's from its declared count" "the set held-out of 22 probes at the 3 repetitions of each probe the round declares, 66 sessions per scenario" "$scratch/add.out"
    present "a final line of a row scenario runs the round's row_repetitions (#1881)" \
        "line 16  additive scenario-1 final: 22 probes x 6 repetitions (round final, declared for a row scenario) = 132 sessions, \$66.00" "$scratch/add.out"
    present "and a final line of any other scenario its repetitions" \
        "line 17  additive scenario-2 final: 22 probes x 3 repetitions (round final, declared) = 66 sessions, \$33.00" "$scratch/add.out"
    # The owner approved the screening round at $700 (#1775). The
    # ceiling admits each plan, and the approval of each round says which
    # lines the batch runs.
    absent "the ceiling of \$700 refuses no plan" "The ceiling refuses" "$scratch/add.out"
    present "the dry run prints the screening round as approved" \
        "round screening: approved to \$700.00 over every attempt, so the batch runs its lines" "$scratch/add.out"
    # The owner approved $1,300.00 for the engine layer of the tuning round
    # on 2026-10-29 (#1877), over every attempt of that layer.
    present "and the tuning round as approved" \
        "round tuning: approved to \$1300.00 over every attempt, so the batch runs its lines" "$scratch/add.out"
    # The owner set the ceiling of phase B at $1,050.00 on 2026-11-09 (#1881).
    present "and the final round as approved" "round final: approved to \$1050.00 over every attempt, so the batch runs its lines" "$scratch/add.out"
    # An approval is a whole number of cents at or above zero. A negative or
    # a non-integer one is refused, so no typo reads as an approval or as a
    # refusal it does not state.
    for bad in -1 true 700.5; do
        sed "s/^    approved_cents: 70000$/    approved_cents: $bad/" "$root/.headwater/probe.yml" > "$scratch/approval-bad.yml"
        python3 "$root/tools/probe/declared.py" "$scratch/approval-bad.yml" approved > /dev/null 2> "$scratch/approval-bad.err"
        bad_status=$?
        same "an approved_cents of $bad is refused" "1 1" \
            "$([ "$bad_status" != 0 ] && echo 1 || echo 0) $(grep -c 'must be a whole number of cents at or above zero' "$scratch/approval-bad.err")"
    done
    # The recorder plans a `--set` line with its arm, because the plan prices
    # one arm. With no arm a final line of 55 repetitions prices every
    # scenario, 3080 sessions, past the $700 ceiling (verify round 1).
    sh "$driver" --identity-only --tier additive --arm scenario-4 --set held-out --repetitions 55 \
        > "$scratch/id-final.out" 2> "$scratch/id-final.err"
    same "the recorder plans a final line of one scenario arm inside the ceiling" "0" "$?"
    same "and its selection is the plan of that one arm" \
        "$("$engine" probe plan --root "$root" --tier additive --arm scenario-4 --set held-out --repetitions 55 2> /dev/null | sed -n 's/^selection: //p' | head -1)" \
        "$(sed -n 's/^selection: //p' "$scratch/id-final.out")"
    same "the screening spec holds the screening lines of the whole campaign, and no other" \
        "$(awk 'NF && $1 !~ /^#/ && $3 == "screening"' "$additive_spec")" \
        "$(awk 'NF && $1 !~ /^#/' "$root/tools/probe/additive-screening.spec")"
    same "and it builds and checks each of the 7 scenario trees" "7" \
        "$(sed -n 's/^tree additive \(scenario-[0-9]*\)[ :].*/\1/p' "$scratch/add.out" | LC_ALL=C sort -u | wc -l | tr -d ' ')"
    absent "no scenario tree holds a skill of this repository's own tooling" " tree: harness " "$scratch/add.out"
    absent "and no scenario tree leaks" " tree: leak " "$scratch/add.out"
    # The two row 10 probes are the superseded-target probes whose hook text
    # names HW-DR-0034, the record in force, through the draft's summary
    # (#1874). They print as treatment in each hook scenario, and in no other.
    same "the hook's pointer to the record in force prints as treatment in scenarios 4 and 6 alone" "4 4 6 6" \
        "$(sed -n 's/^leak check, scenario-\([0-9]*\) tree: treatment .* HW-DR-0034$/\1/p' "$scratch/add.out" | LC_ALL=C sort -n | tr '\n' ' ' | sed 's/ $//')"

    # additive_leaks <file> <probe> <string>: the committed declaration with
    # one leak string, on one probe, and `[]` on every other probe of the sets.
    additive_leaks() {
        awk -v probe="$2" -v string="$3" '
            /^leaks:/ { skip = 1; next }
            /^leaks_kept:/ { skip = 1; next }
            skip && /^[^ #]/ { skip = 0 }
            skip { next }
            { print }
            /^  (tuning|held-out):/ { inset = 1; next }
            inset && /^    - / { ids[++n] = substr($0, 7) }
            inset && !/^    - / { inset = 0 }
            END {
                print "leaks:"
                for (i = 1; i <= n; i++) {
                    if (ids[i] == probe) printf "  %s: [\"%s\"]\n", ids[i], string
                    else printf "  %s: []\n", ids[i]
                }
            }
        ' "$root/.headwater/probe.yml" > "$1"
    }
    # The decisive case: the only leak string is in the body of a skill of the
    # `skills` layer, and in no description, memory file or hook text. It is
    # read from the orient body at run time, as the last sentence of the
    # first paragraph of its start-of-task gate, so a rewrite of the skill
    # cannot leave a planted string that no tree holds any more. The section
    # holds more than one paragraph, and only the first is the gate.
    body_string=$(sed -n '/^## Before you change a file$/,/^## /{/^[A-Z]/p;}' "$root/.claude/skills/headwater-orient/SKILL.md" | head -n 1 | sed 's/.*\. //')
    same "the orient body has a start-of-task gate to plant a leak from" "yes" \
        "$([ -n "$body_string" ] && echo yes || echo no)"
    # A member of `tuning`, so the screening spec plans it (#1874).
    body_probe=HW-PROBE-a-session-adds-a-count-of-findings-by-severity-to-the-json-report-of-the-check
    additive_leaks "$scratch/add-body.yml" "$body_probe" "$body_string"
    HW_PROBE_YML="$scratch/add-body.yml" PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$additive_spec" > "$scratch/add-body.out" 2> "$scratch/add-body.err"
    same "a leak string in a skills-layer skill body fails the additive dry run with 8" "8" "$?"
    same "and it names every scenario that holds skills, and no other" "3 6" \
        "$(sed -n "s/^leak check, scenario-\([0-9]*\) tree: leak .*headwater-orient\/SKILL.md $body_string$/\1/p" "$scratch/add-body.out" | LC_ALL=C sort -n | tr '\n' ' ' | sed 's/ $//')"
    present "and prints the round sums all the same" "total: 12298 sessions" "$scratch/add-body.out"

    # The hook's text: the intent hook opens the text it injects for the
    # pointer probe's task with its own first sentence, and no file of a hook
    # scenario's always-loaded text or skill bodies holds that sentence. The
    # pointer probe is a member of `tuning` (#1874), so every scenario plans it.
    hook_string="Headwater routed this task to the documents that govern it"
    pointer_probe=HW-PROBE-a-session-records-as-owed-the-corpus-size-axis-that-the-additive-campaign-defers
    held_probe=HW-PROBE-a-session-records-as-owed-that-nothing-reports-a-pass-rate-for-each-phrasing-of-a-probe
    additive_leaks "$scratch/add-hook.yml" "$pointer_probe" "$hook_string"
    HW_PROBE_YML="$scratch/add-hook.yml" PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$additive_spec" > "$scratch/add-hook.out" 2> "$scratch/add-hook.err"
    same "a leak string in the hook's text fails the additive dry run with 8" "8" "$?"
    same "and it names every scenario that holds the hooks, and no other" "4 6" \
        "$(sed -n "s|^leak check, scenario-\([0-9]*\) tree: leak $pointer_probe hook:$pointer_probe $hook_string$|\1|p" "$scratch/add-hook.out" | LC_ALL=C sort -n | tr '\n' ' ' | sed 's/ $//')"
    same "and no scenario leaks through another channel" "0" \
        "$(grep ' tree: leak ' "$scratch/add-hook.out" | grep -vc " hook:$pointer_probe ")"
    # The same phrase planted on a probe with the `superseded_target` axis, a
    # member of `tuning`: a hook hit that is not the identifier of the record
    # in force still fails in scenarios 4 and 6 (#1874).
    stamp_probe=HW-PROBE-a-session-that-reviews-an-agent-s-acceptance-stamp-cites-the-ruling-in-force-and-not-the-draft-that-the-pull-request-names
    additive_leaks "$scratch/add-force.yml" "$stamp_probe" "$hook_string"
    HW_PROBE_YML="$scratch/add-force.yml" PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$additive_spec" > "$scratch/add-force.out" 2> "$scratch/add-force.err"
    same "a phrase in a superseded-target probe's hook text fails the additive dry run with 8" "8" "$?"
    same "and it names scenarios 4 and 6, and no other" "4 6" \
        "$(sed -n "s|^leak check, scenario-\([0-9]*\) tree: leak $stamp_probe hook:$stamp_probe $hook_string$|\1|p" "$scratch/add-force.out" | LC_ALL=C sort -n | tr '\n' ' ' | sed 's/ $//')"

    printf 'additive scenario-6 final\n' > "$scratch/add-twelve.spec"
    # The refusals of a spec.
    printf 'additive scenario-3 final\ncampaign present structural\n' > "$scratch/add-mixed.spec"
    PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$scratch/add-mixed.spec" > /dev/null 2> "$scratch/add-mixed.err"
    same "a spec that mixes additive lines with another tier is refused with 5" "5" "$?"
    present "and the refusal says why" "mixes 1 lines of the \`additive\` tier with 1 of other tiers" "$scratch/add-mixed.err"
    PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$scratch/add-twelve.spec" --repetitions 3 > /dev/null 2> "$scratch/add-reps.err"
    same "an additive spec refuses --repetitions with 2" "2" "$?"
    present "and says the rounds fix the count" "Each round's count is computed from its rates" "$scratch/add-reps.err"
    printf 'additive scenario-3 pilot\n' > "$scratch/add-round.spec"
    PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$scratch/add-round.spec" > "$scratch/add-round.out" 2> /dev/null
    same "a line of a round that rounds: does not declare is refused with 5" "5" "$?"
    present "and the refusal names the rounds declared" "declares no round \`pilot\`. It declares screening tuning final" "$scratch/add-round.out"
    printf 'additive scenario-3 screening %s\n' "$held_probe" > "$scratch/add-outside.spec"
    PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$scratch/add-outside.spec" > "$scratch/add-outside.out" 2> /dev/null
    same "a line that leaves out a probe of another set is refused with 5" "5" "$?"
    printf 'additive no-hook final\n' > "$scratch/add-arm.spec"
    PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$scratch/add-arm.spec" > "$scratch/add-arm.out" 2> /dev/null
    same "an additive line of a component arm is refused with 5" "5" "$?"
    # The count is computed from the round's rates: other rates price another
    # final round. And the plan's own count of a line's probes must agree with
    # the set `declared.py` reads, so a declaration whose set the plan does
    # not select is refused.
    sed '/^    repetitions: 3$/{N;N;s/.*/    rates: [0.95, 0.80]/;}' "$root/.headwater/probe.yml" > "$scratch/add-rates.yml"
    HW_PROBE_YML="$scratch/add-rates.yml" PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$scratch/add-twelve.spec" > "$scratch/add-rates.out" 2> /dev/null
    present "other rates for the final round price another count" "0.95 against 0.8, needs 88 sessions per scenario" "$scratch/add-rates.out"
    present "and plan it" "22 probes x 4 repetitions (round final, ceil(88 / 22)) = 88 sessions" "$scratch/add-rates.out"
    awk '/^  tuning:/ && !done { print; getline; done = 1; next } { print }' "$root/.headwater/probe.yml" > "$scratch/add-short.yml"
    printf 'additive scenario-5 screening\n' > "$scratch/add-short.spec"
    HW_PROBE_YML="$scratch/add-short.yml" PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --dry-run \
        --spec "$scratch/add-short.spec" > "$scratch/add-short.out" 2> /dev/null
    same "a set the plan counts otherwise than the declaration read is refused with 5" "5" "$?"
    present "and the refusal names both counts" "the plan selects 22 probes, and the set tuning less the probes the line leaves out holds 21" "$scratch/add-short.out"
    # A tuning line runs one stage of a round (#1838), and the batch reads
    # the stage from HEADWATER_TUNING_STAGE, which tools/probe/try.sh sets.
    # With none, the priced counts exit 2, so no batch runs a tuning line at
    # a count nobody chose. No count of 1 or 7 is typed anywhere: each is
    # ceil(22 / 22) or ceil(145 / 22) over the 22 probes of the set (#1874).
    PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign-dry-run.sh" --priced "$additive_spec" \
        > "$scratch/add-priced.out" 2> "$scratch/add-priced.err"
    same "--priced of a tuning line with no stage exits 2" "2" "$?"
    present "and names the stage it lacks" "HEADWATER_TUNING_STAGE names none" "$scratch/add-priced.err"
    HEADWATER_TUNING_STAGE=screen PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign-dry-run.sh" --priced "$additive_spec" \
        > "$scratch/add-priced.out" 2> "$scratch/add-priced.err"
    same "--priced of the screen reads each additive line" "0 21" "$? $(wc -l < "$scratch/add-priced.out" | tr -d ' ')"
    same "and gives each its round's count, 1 repetition on a screened tuning line, and 3 and 6 on a final line of scenario 0 and 1" "4 1 3 6" \
        "$(awk '$1 == 1 || $1 == 8 || $1 == 15 || $1 == 16 { print $3 }' "$scratch/add-priced.out" | tr '\n' ' ' | sed 's/ $//')"
    HEADWATER_TUNING_STAGE=confirm PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign-dry-run.sh" --priced "$additive_spec" \
        > "$scratch/add-priced.out" 2> "$scratch/add-priced.err"
    same "--priced of the confirmation gives a tuning line 7 repetitions" "0 7" \
        "$? $(awk '$1 == 8 { print $3 }' "$scratch/add-priced.out")"
    HEADWATER_TUNING_STAGE=final PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign-dry-run.sh" --priced "$additive_spec" \
        > "$scratch/add-priced.out" 2> "$scratch/add-priced.err"
    same "--priced of a stage that is neither screen nor confirm exits 2" "2" "$?"
    sed 's/^    screen: 22$/    screen: 30/; s/^    confirm: 145$/    confirm: 200/' "$root/.headwater/probe.yml" > "$scratch/add-counts.yml"
    HW_PROBE_YML="$scratch/add-counts.yml" HEADWATER_TUNING_STAGE=confirm PATH="$scratch/dry-bin:$PATH" \
        sh "$root/tools/probe/campaign-dry-run.sh" --priced "$additive_spec" > "$scratch/add-priced.out" 2> "$scratch/add-priced.err"
    same "a confirmation declared at 200 is priced at ceil(200 / 22) = 10 repetitions, read and not typed" "0 10" \
        "$? $(awk '$1 == 8 { print $3 }' "$scratch/add-priced.out")"
    HW_PROBE_YML="$scratch/add-counts.yml" HEADWATER_TUNING_STAGE=screen PATH="$scratch/dry-bin:$PATH" \
        sh "$root/tools/probe/campaign-dry-run.sh" --priced "$additive_spec" > "$scratch/add-priced.out" 2> "$scratch/add-priced.err"
    same "and a screen declared at 30 at ceil(30 / 22) = 2" "0 2" \
        "$? $(awk '$1 == 8 { print $3 }' "$scratch/add-priced.out")"
    # The seven fields of the tuning round's rounds are declared together,
    # and each is read with PyYAML and refused when it does not hold.
    for bad in 'screen: 0|`screen` of the `tuning` round' 'confirm: "144"|`confirm` of the `tuning` round' \
        'variants: -1|`variants` of the `tuning` round' 'stop_after_empty: 0|`stop_after_empty` of the `tuning` round' \
        'order: [skills, nope]|names the layer nope' 'order: [skills, skills]|names a layer twice' \
        'ceiling_cents: 130001|above the round'"'"'s `approved_cents` of 130000' \
        'base_ceiling_cents: -1|`base_ceiling_cents` of the `tuning` round' \
        'base_ceiling_cents: "25000"|`base_ceiling_cents` of the `tuning` round' \
        'base_ceiling_cents: 30001|together 130001, above the round'"'"'s `approved_cents` of 130000'; do
        bad_field=${bad%%:*}
        bad_line=${bad%%|*}
        sed "s/^    $bad_field: .*/    $bad_line/" "$root/.headwater/probe.yml" > "$scratch/stage-bad.yml"
        python3 "$root/tools/probe/declared.py" "$scratch/stage-bad.yml" tuning > /dev/null 2> "$scratch/stage-bad.err"
        same "declared.py tuning refuses $bad_line with 2" "2" "$?"
        present "and says why" "${bad#*|}" "$scratch/stage-bad.err"
    done
    grep -v '^    variants: 4$' "$root/.headwater/probe.yml" > "$scratch/stage-bad.yml"
    python3 "$root/tools/probe/declared.py" "$scratch/stage-bad.yml" tuning > /dev/null 2> "$scratch/stage-bad.err"
    same "declared.py tuning refuses six of the seven fields with 2" "2" "$?"
    present "and names the one missing" "and not \`variants\`" "$scratch/stage-bad.err"
    grep -v '^    base_ceiling_cents: ' "$root/.headwater/probe.yml" > "$scratch/stage-bad.yml"
    python3 "$root/tools/probe/declared.py" "$scratch/stage-bad.yml" tuning > /dev/null 2> "$scratch/stage-bad.err"
    same "declared.py tuning refuses a round with no base ceiling with 2" "2" "$?"
    present "and names it" "and not \`base_ceiling_cents\`" "$scratch/stage-bad.err"
    python3 "$root/tools/probe/declared.py" "$root/.headwater/probe.yml" tuning > "$scratch/stage-good.out"
    same "declared.py tuning prints the base ceiling of the declaration" "base_ceiling_cents 30000" \
        "$(grep '^base_ceiling_cents ' "$scratch/stage-good.out")"

    # The batch runs the same check before each invocation (clause 5: "the
    # check runs again before each round"). A stub `cargo` stands in for the
    # build, with this checkout's engine where the build writes it, and a stub
    # harness records any call. The batch needs a clean checkout.
    if [ -z "$(git -C "$root" status --porcelain --untracked-files=no 2>/dev/null)" ]; then
        mkdir -p "$scratch/gate-bin" "$scratch/gate-target/dev-release"
        cp "$engine" "$scratch/gate-target/dev-release/headwater"
        printf '#!/bin/sh\nexit 0\n' > "$scratch/gate-bin/cargo"
        printf '#!/bin/sh\n: > "%s/claude-called"\nexit 7\n' "$scratch" > "$scratch/gate-bin/claude"
        printf '#!/bin/sh\nexit 0\n' > "$scratch/gate-bin/bwrap"
        chmod +x "$scratch/gate-bin/cargo" "$scratch/gate-bin/claude" "$scratch/gate-bin/bwrap"
        rm -rf "$scratch/gate-out"
        HW_PROBE_YML="$scratch/add-body.yml" CARGO_TARGET_DIR="$scratch/gate-target" PATH="$scratch/gate-bin:$PATH" \
            sh "$root/tools/probe/campaign.sh" --out "$scratch/gate-out" --model claude-haiku-4-5 \
            --spec "$root/tools/probe/additive-screening.spec" > /dev/null 2> "$scratch/gate.err"
        same "a batch of the screening spec stops with 8 on a leak in a scenario tree" "8" "$?"
        present "and names the leak" "leak check, scenario-3 tree: leak $body_probe" "$scratch/gate.err"
        same "and it asked the harness nothing, not even its version, and planned no line" "no no" \
            "$([ -e "$scratch/gate-out/claude-version" ] && echo yes || echo no) $([ -e "$scratch/gate-out/lines" ] && echo yes || echo no)"
        rm -rf "$scratch/gate-out"
        CARGO_TARGET_DIR="$scratch/gate-target" PATH="$scratch/gate-bin:$PATH" \
            sh "$root/tools/probe/campaign.sh" --out "$scratch/gate-out" --model claude-haiku-4-5 \
            --spec "$additive_spec" > /dev/null 2> "$scratch/gate-clean.err"
        same "the whole campaign's spec is still refused with 5, now on its tuning lines outside the open layer" "5" "$?"
        present "and the refusal names a tuning line of a scenario the open layer is not" "and the spec holds a tuning line of scenario-1." "$scratch/gate-clean.err"
        absent "and not the tuning round, which the owner approved (#1777)" "approved spend on the tuning round" "$scratch/gate-clean.err"
        absent "and not the screening round" "approved spend on the screening round" "$scratch/gate-clean.err"
        absent "and not the final round, which the owner approved (#1881)" "approved spend on the final round" "$scratch/gate-clean.err"
        same "and it ran no dry run, planned no line and asked the harness nothing" "no no no" \
            "$([ -e "$scratch/gate-out/leak-gate" ] && echo yes || echo no) $([ -e "$scratch/gate-out/lines" ] && echo yes || echo no) $([ -e "$scratch/gate-out/claude-version" ] && echo yes || echo no)"
        rm -rf "$scratch/gate-out"
        CARGO_TARGET_DIR="$scratch/gate-target" PATH="$scratch/gate-bin:$PATH" \
            sh "$root/tools/probe/campaign.sh" --out "$scratch/gate-out" --model claude-haiku-4-5 \
            --spec "$root/tools/probe/additive-screening.spec" --repetitions 3 > /dev/null 2> "$scratch/gate-reps.err"
        same "a batch of an additive spec refuses --repetitions with 2" "2" "$?"
        rm -rf "$scratch/gate-out"

        # The decisive case of #1775: a batch of one additive screening line,
        # past a ceiling above zero, runs at the round's count, in the
        # scenario's own tree, and records each session with `--set` and
        # `--arm`. The same batch refuses a line of a round nobody approved
        # with 5, before it builds a tree or asks the harness anything. The
        # harness answers `--version` alone, and `bwrap` is a stub, so no
        # session reaches a model.
        mkdir -p "$scratch/run-bin" "$scratch/run-host"
        printf '#!/bin/sh\nexit 0\n' > "$scratch/run-bin/cargo"
        printf '#!/bin/sh\nexit 0\n' > "$scratch/run-bin/bwrap"
        printf '#!/bin/sh\nif [ "${1:-}" = --version ]; then echo "2.1.288 (Claude Code)"; exit 0; fi\n: > "%s/run-session-called"\nexit 7\n' \
            "$scratch" > "$scratch/run-bin/claude"
        chmod +x "$scratch/run-bin/cargo" "$scratch/run-bin/bwrap" "$scratch/run-bin/claude"
        printf 'additive scenario-4 screening\n' > "$scratch/run-screen.spec"
        rm -rf "$scratch/run-out"
        HEADWATER_PROBE_HOST_CONFIG="$scratch/run-host" CARGO_TARGET_DIR="$scratch/gate-target" PATH="$scratch/run-bin:$PATH" \
            sh "$root/tools/probe/campaign.sh" --out "$scratch/run-out" --model claude-haiku-4-5 \
            --spec "$scratch/run-screen.spec" --max-sessions 1 > /dev/null 2> "$scratch/run.err"
        run_status=$?
        same "a batch of one screening line passes the plan and starts a session" "yes" \
            "$([ "$run_status" != 5 ] && [ -e "$scratch/run-out/claude-version" ] && echo yes || echo no)"
        same "its jobs are 22 probes x the round's 4 repetitions, not the tier's 55" "88" \
            "$(grep -c ' additive scenario-4 screening ' "$scratch/run-out/jobs" 2>/dev/null)"
        present "and the line is planned at the round's count" "22 probes x 1 arms x 4 repetitions = 88 sessions" "$scratch/run-out/plans/L1"
        # The tree is the scenario's: every path of a layer scenario 4 does
        # not hold is gone, and the hooks layer it holds is there.
        "$engine" probe plan --root "$root" --scenario 4 --delta > "$scratch/run-delta" 2> /dev/null
        same "the tree of the line is the scenario-4 tree: no path of a layer it lacks is left" "0" \
            "$(sed -n 's/^- [^ ]* //p' "$scratch/run-delta" | while IFS= read -r p; do
                [ -e "$scratch/run-out/trees/additive-scenario-4/$p" ] && echo "$p"; done | wc -l | tr -d ' ')"
        same "and the layers it holds are there" "yes" \
            "$(awk '$1 == "=" && $2 == "hooks" { print $3 }' "$scratch/run-delta" | head -1 | {
                read -r p; [ -n "$p" ] && [ -e "$scratch/run-out/trees/additive-scenario-4/$p" ] && echo yes || echo no; })"
        run_session=$(ls "$scratch/run-out/sessions" 2>/dev/null | head -1)
        run_argv=$scratch/run-out/sessions/$run_session/record.argv
        same "the recorder is given the round's set and the scenario arm" "yes yes" \
            "$(grep -qx -- '--set' "$run_argv" 2>/dev/null && grep -A1 -x -- '--set' "$run_argv" | tail -1 | grep -qx tuning && echo yes || echo no) $(grep -A1 -x -- '--arm' "$run_argv" 2>/dev/null | tail -1 | grep -qx scenario-4 && echo yes || echo no)"
        absent "and no category" "--category" "$run_argv"
        # The recorder grades in the grading tree, which keeps each file a
        # command oracle holds, and never in the sealed oracle tree (#1834).
        same "the recorder grades in the batch's grading tree" "$scratch/run-out/trees/grading" \
            "$(grep -A1 -x -- '--oracle-tree' "$run_argv" 2>/dev/null | tail -1)"
        same "and the batch built it" "yes" \
            "$([ -d "$scratch/run-out/trees/grading/.headwater" ] && echo yes || echo no)"
        absent "and the recorder plans the line the batch planned" "headwater probe plan" \
            "$scratch/run-out/sessions/$run_session/record.err"
        same "the session the plan admitted is the one the batch planned" "$(sed -n 's/^selection: //p' "$scratch/run-out/plans/L1" | head -1)" \
            "$("$engine" probe plan --root "$root" --tier additive --arm scenario-4 --set tuning --repetitions 4 2> /dev/null | sed -n 's/^selection: //p' | head -1)"
        # The identity block holds the set only as its digest, so assembly
        # states the round and the set beside it. The one session did not
        # record, so assembly fails the line, and still names its round.
        sh "$root/tools/probe/campaign.sh" --out "$scratch/run-out" --assemble > /dev/null 2> "$scratch/run-assemble.err"
        same "assembly states the line's round and set beside its identity" "round: screening set: tuning" \
            "$(tr '\n' ' ' < "$scratch/run-out/assembled/L1-additive-scenario-4-screening.round" 2>/dev/null | sed 's/ $//')"
        # The owner approved the tuning round (#1880), and the layer it has
        # open is MCP on scenario 5, over its base, scenario 2. So a
        # batch of a scenario-4 tuning line, which is neither, passes the
        # round gate and is refused by the layer gate, with 5, before it
        # builds a tree or asks the harness anything. The case of a declared
        # open layer reads a scratch root, below, because the gate reads this
        # checkout's declaration and no override.
        printf 'additive scenario-4 tuning\n' > "$scratch/run-tune.spec"
        rm -rf "$scratch/run-tune-out"
        HEADWATER_PROBE_HOST_CONFIG="$scratch/run-host" CARGO_TARGET_DIR="$scratch/gate-target" PATH="$scratch/run-bin:$PATH" \
            sh "$root/tools/probe/campaign.sh" --out "$scratch/run-tune-out" --model claude-haiku-4-5 \
            --spec "$scratch/run-tune.spec" --max-sessions 1 > /dev/null 2> "$scratch/run-tune.err"
        same "a batch of a scenario-4 tuning line is refused with 5 by the layer gate" "5" "$?"
        absent "and names no refusal of the tuning round's approval" "no person has approved spend on the tuning round" "$scratch/run-tune.err"
        present "and names the open rule" "the tuning round is open for" "$scratch/run-tune.err"
        same "and it wrote no harness version, no plan, no tree and no dry run" "no no no no" \
            "$([ -e "$scratch/run-tune-out/claude-version" ] && echo yes || echo no) $([ -e "$scratch/run-tune-out/lines" ] && echo yes || echo no) $([ -e "$scratch/run-tune-out/trees" ] && echo yes || echo no) $([ -e "$scratch/run-tune-out/leak-gate" ] && echo yes || echo no)"
        rm -rf "$scratch/run-tune-out"
        # Each line's jobs are its own plan's count, not the first line's
        # (verify round 1): the regression tier runs 1 repetition and the
        # campaign tier 30, so one spec of both is 17 + 17 x 30 jobs. The
        # navigability shelf is seventeen probes, two of them from rows 6 to 9
        # of #1874, and the plan counts its retired probes too. The line reads
        # navigability and not sufficiency since #1874.3, because thirty-four
        # sufficiency probes in the campaign tier project 6120 sessions,
        # $3060.00, past its $2052.00, and the plan refuses the run.
        printf 'regression present navigability\ncampaign present navigability\n' > "$scratch/run-tiers.spec"
        rm -rf "$scratch/run-tiers-out"
        HEADWATER_PROBE_HOST_CONFIG="$scratch/run-host" CARGO_TARGET_DIR="$scratch/gate-target" PATH="$scratch/run-bin:$PATH" \
            sh "$root/tools/probe/campaign.sh" --out "$scratch/run-tiers-out" --model claude-haiku-4-5 \
            --spec "$scratch/run-tiers.spec" --max-turns 10 --max-sessions 1 > /dev/null 2> "$scratch/run-tiers.err"
        same "a spec of two tiers runs each line at its own plan's count" "17 510" \
            "$(awk '$2 == 1' "$scratch/run-tiers-out/jobs" 2>/dev/null | wc -l | tr -d ' ') $(awk '$2 == 2' "$scratch/run-tiers-out/jobs" 2>/dev/null | wc -l | tr -d ' ')"
        rm -rf "$scratch/run-out" "$scratch/run-final-out" "$scratch/run-tiers-out"
    else
        printf 'note the checkout has uncommitted changes, so the batch leak gate cases did not run.\n'
    fi

    if [ -e "$scratch/claude-called" ]; then
        fail "no additive dry run calls a model" "the harness on the path was called"
    else
        pass "no additive dry run calls a model"
    fi

    # `--scenario-tree` (#1821) builds the tree a session of one scenario
    # gets, with the batch's own two functions, and nothing else of a batch.
    # It reads `HEAD` of this checkout, so it runs on a dirty checkout too.
    rm -rf "$scratch/st-out"
    PATH="$scratch/dry-bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$scratch/st-out" --scenario-tree 0 \
        > "$scratch/st.out" 2> "$scratch/st.err"
    same "campaign.sh --scenario-tree 0 exits 0 and prints the tree's path" \
        "0 $scratch/st-out/trees/additive-scenario-0" "$? $(cat "$scratch/st.out")"
    same "and writes HEAD's commit, and nothing of a batch: no jobs, no plans, no claude-version" \
        "$(git -C "$root" rev-parse HEAD) no no no" \
        "$(cat "$scratch/st-out/head" 2>/dev/null) $([ -e "$scratch/st-out/jobs" ] && echo yes || echo no) $([ -e "$scratch/st-out/plans" ] && echo yes || echo no) $([ -e "$scratch/st-out/claude-version" ] && echo yes || echo no)"
    same "and scenario 0's tree holds no engine binary, because the engine layer owns it" "no" \
        "$([ -e "$scratch/st-out/trees/additive-scenario-0/engine/target/dev-release/headwater" ] && echo yes || echo no)"
    same "and holds the engine's sources and this script" "yes yes" \
        "$([ -d "$scratch/st-out/trees/additive-scenario-0/engine/crates/cli/src" ] && echo yes || echo no) $([ -f "$scratch/st-out/trees/additive-scenario-0/tools/probe/campaign.sh" ] && echo yes || echo no)"
    rm -rf "$scratch/st-out"
    sh "$root/tools/probe/campaign.sh" --out "$scratch/st-out" --scenario-tree 0 --at no-such-revision > /dev/null 2> "$scratch/st-bad.err"
    same "--at a name that is no commit exits 2" "2" "$?"
    sh "$root/tools/probe/campaign.sh" --out "$scratch/st-out" --scenario-tree zero > /dev/null 2> /dev/null
    same "--scenario-tree with no scenario number exits 2" "2" "$?"
    rm -rf "$scratch/st-out"
else
    printf 'note no engine, so the additive dry-run cases did not run.\n'
fi

# ---------------------------------------------------------------------------
# The layer gate of the tuning round (#1819), beside the round gate above.
#
# `rounds.tuning.open` names the one layer whose plan the owner accepted, its
# scenario, the try, its cap, the plan and the change, or `none`. Each
# declaration below is the committed one with `open: none` replaced, so a
# case reads every other key as the checkout declares it. These cases need
# no engine and no clean checkout, so no skip above hides them.
# ---------------------------------------------------------------------------
# The committed declaration with its open layer set back to `none`, so the
# cases hold whichever layer the checkout has open.
awk '/^    open:/ { print "    open: none"; skip = 1; next } skip && /^      / { next } { skip = 0; print }' "$root/.headwater/probe.yml" > "$scratch/open-none.yml"
open_decl() {
    # $1 the declaration to write, $2 the lines that replace `open: none`
    awk -v block="$2" '$0 == "    open: none" { print block; next } { print }' "$scratch/open-none.yml" > "$1"
}
open_docs1='    open:\n      layer: docs\n      scenario: 1\n      try: 1\n      cap_cents: 6000\n      plan: https://github.com/headwater-ai/headwater/issues/1777#issuecomment-6277195497\n      change: none'
open_decl "$scratch/open-docs1.yml" "$open_docs1"
declared_py=$root/tools/probe/declared.py

python3 "$declared_py" "$scratch/open-none.yml" open > "$scratch/open-none.out" 2> "$scratch/open-none.err"
same "declared.py open prints none for a declaration with open: none" "0 none" "$? $(cat "$scratch/open-none.out")"
python3 "$declared_py" "$scratch/open-docs1.yml" open > "$scratch/open-docs1.out" 2> "$scratch/open-docs1.err"
same "declared.py open prints the six fields of an open layer, on one line" \
    "0 docs 1 1 6000 https://github.com/headwater-ai/headwater/issues/1777#issuecomment-6277195497 none" \
    "$? $(cat "$scratch/open-docs1.out")"
sed 's/^      layer: docs$/      layer: engine/; s/^      scenario: 1$/      scenario: 2/; s/^      change: none$/      change: 2e3d71a7/' \
    "$scratch/open-docs1.yml" > "$scratch/open-engine3.yml"
python3 "$declared_py" "$scratch/open-engine3.yml" open > "$scratch/open-engine3.out" 2> /dev/null
same "and admits the engine layer on scenario 2, which adds engine over scenario 1" \
    "0 engine 2 1 6000 https://github.com/headwater-ai/headwater/issues/1777#issuecomment-6277195497 2e3d71a7" \
    "$? $(cat "$scratch/open-engine3.out")"
# Each refusal exits 2 and states its own reason. `name|sed|needle`.
for open_case in \
    "scenario 3 named with the docs layer|s/^      scenario: 1$/      scenario: 3/|does not add exactly the docs layer over its base" \
    "scenario 0, which has no base|s/^      scenario: 1$/      scenario: 0/|scenario 0 names no base" \
    "a scenario that scenarios: does not declare|s/^      scenario: 1$/      scenario: 99/|\`scenarios:\` declares no scenario 99" \
    "a layer that layers: does not declare|s/^      layer: docs$/      layer: nope/|\`layers:\` declares no layer nope" \
    "a cap above the round's approval|s/^      cap_cents: 6000$/      cap_cents: 130001/|above the tuning round's \`approved_cents\` of 130000" \
    "a cap of zero|s/^      cap_cents: 6000$/      cap_cents: 0/|must be a whole number of cents above zero" \
    "a try past the round's tries|s/^      try: 1$/      try: 5/|must be a whole number from 1 to the round's \`tries\`, 4" \
    "a try of zero|s/^      try: 1$/      try: 0/|must be a whole number from 1 to the round's \`tries\`, 4" \
    "a seventh field|s/^      change: none$/      change: none\n      note: x/|declares \`note\`" \
    "a missing field|/^      change: none$/d|does not declare \`change\`" \
    "a plan that is not a URL|s/^      plan: .*$/      plan: accepted/|\`plan\` of \`rounds.tuning.open\`" \
    "a change that is not a commit or none|s/^      change: none$/      change: main/|\`change\` of \`rounds.tuning.open\`" \
    "a change shorter than 7 hex characters|s/^      change: none$/      change: abc123/|\`change\` of \`rounds.tuning.open\`" \
    "a change in upper-case hex|s/^      change: none$/      change: ABCDEF1/|\`change\` of \`rounds.tuning.open\`" \
    "a plan that is not https|s/^      plan: https:/      plan: http:/|\`plan\` of \`rounds.tuning.open\`" \
    "an open that is a word, not a mapping|/^    open:$/,/^      change: none$/c\\    open: docs|is neither \`none\` nor a mapping"
do
    open_name=${open_case%%|*}; open_rest=${open_case#*|}
    open_sed=${open_rest%%|*}; open_needle=${open_rest#*|}
    sed "$open_sed" "$scratch/open-docs1.yml" > "$scratch/open-bad.yml"
    python3 "$declared_py" "$scratch/open-bad.yml" open > "$scratch/open-bad.out" 2> "$scratch/open-bad.err"
    same "declared.py open refuses $open_name, with 2 and its reason" "2 1 0" \
        "$? $(grep -cF -- "$open_needle" "$scratch/open-bad.err") $(wc -c < "$scratch/open-bad.out" | tr -d ' ')"
done
# A hooks layer names its hook moment as a seventh field, `moment`, one of
# intent, read, write and review (#1875), and `open` prints it as a seventh
# word. No other layer declares it, so the line of every other layer keeps
# its six words. The rows are the hooks layer on scenario 4, which adds the
# hooks over scenario 2.
sed 's/^      layer: docs$/      layer: hooks/; s/^      scenario: 1$/      scenario: 4/; s/^      change: none$/      change: none\n      moment: read/' \
    "$scratch/open-docs1.yml" > "$scratch/open-hooks4.yml"
python3 "$declared_py" "$scratch/open-hooks4.yml" open > "$scratch/open-hooks4.out" 2> "$scratch/open-hooks4.err"
same "declared.py open prints a hooks layer's moment as a seventh word" \
    "0 hooks 4 1 6000 https://github.com/headwater-ai/headwater/issues/1777#issuecomment-6277195497 none read" \
    "$? $(cat "$scratch/open-hooks4.out")"
for open_case in \
    "a hooks layer with no moment|open-hooks4|/^      moment: read$/d|does not declare \`moment\`" \
    "a moment outside the four|open-hooks4|s/^      moment: read$/      moment: start/|\`moment\` of \`rounds.tuning.open\` in .headwater/probe.yml is 'start'" \
    "a moment that is not a word|open-hooks4|s/^      moment: read$/      moment: [read]/|\`moment\` of \`rounds.tuning.open\` in .headwater/probe.yml is ['read']" \
    "a moment on the docs layer|open-docs1|s/^      change: none$/      change: none\n      moment: read/|only the hooks layer declares one"
do
    open_name=${open_case%%|*}; open_rest=${open_case#*|}
    open_from=${open_rest%%|*}; open_rest=${open_rest#*|}
    open_sed=${open_rest%%|*}; open_needle=${open_rest#*|}
    sed "$open_sed" "$scratch/$open_from.yml" > "$scratch/open-bad.yml"
    python3 "$declared_py" "$scratch/open-bad.yml" open > "$scratch/open-bad.out" 2> "$scratch/open-bad.err"
    same "declared.py open refuses $open_name, with 2 and its reason" "2 1 0" \
        "$? $(grep -cF -- "$open_needle" "$scratch/open-bad.err") $(wc -c < "$scratch/open-bad.out" | tr -d ' ')"
done

# `tuning_set` prints the members of the tuning round's set, one a line, in
# the order the set declares them, which `try.sh` draws and selects in.
python3 "$declared_py" "$scratch/open-docs1.yml" tuning_set > "$scratch/tuning-set.out" 2> /dev/null
same "declared.py tuning_set prints the tuning round's set, one member a line, in its order" \
    "0 $(python3 -c 'import sys, yaml; d = yaml.safe_load(open(sys.argv[1])); print(" ".join(d["sets"][d["rounds"]["tuning"]["set"]]))' "$scratch/open-docs1.yml")" \
    "$? $(tr '\n' ' ' < "$scratch/tuning-set.out" | sed 's/ $//')"

# ---------------------------------------------------------------------------
# `tools/probe/phrasing.sh` cuts a probe's task the one way the campaign and
# the tuning round both read it (#1875, HW-DR-0112's form of a phrasing).
# The prompt of a session is the body of the one phrasing it draws, without
# the heading and without any other phrasing (the recorder contract (docs/spec/the-recorder-contract.md)).
# ---------------------------------------------------------------------------
phrasing=$root/tools/probe/phrasing.sh
mkdir -p "$scratch/phr/docs/probes"
printf -- '---\nid: HW-PROBE-phr3\n---\n\n# Three\n\n## Task\n\n### Phrasing 1\n\nFind the first.\n\n### Phrasing 2\n\nLocate the second,\nover two lines.\n\n#### A note\n\nInside phrasing 2.\n\n### Phrasing 3\n\nName the third.\n\n## Target\n\nA link.\n' \
    > "$scratch/phr/docs/probes/phr3.md"
printf -- '---\nid: HW-PROBE-phr0\n---\n\n# None\n\n## Task\n\n\nThe whole task.\n\n\n## Target\n\nA link.\n' \
    > "$scratch/phr/docs/probes/phr0.md"
same "phrasing.sh count: three phrasings under ## Task" "3" "$(sh "$phrasing" count "$scratch/phr" HW-PROBE-phr3)"
same "phrasing.sh count: a task with none is 0" "0" "$(sh "$phrasing" count "$scratch/phr" HW-PROBE-phr0)"
same "phrasing.sh count: a probe that no document declares is 0" "0" "$(sh "$phrasing" count "$scratch/phr" HW-PROBE-nobody)"
sh "$phrasing" cut "$scratch/phr" HW-PROBE-phr3 2 > "$scratch/phr/cut2"
printf 'Locate the second,\nover two lines.\n\n#### A note\n\nInside phrasing 2.\n' > "$scratch/phr/want2"
same "phrasing.sh cut 2: the body of phrasing 2 alone, without its heading or another phrasing, byte for byte" \
    "same" "$(cmp -s "$scratch/phr/cut2" "$scratch/phr/want2" && echo same || echo differs)"
same "phrasing.sh cut 3: the last phrasing stops at the next ## heading" "Name the third." \
    "$(sh "$phrasing" cut "$scratch/phr" HW-PROBE-phr3 3)"
same "phrasing.sh cut 4: a phrasing the task does not hold is empty" "" \
    "$(sh "$phrasing" cut "$scratch/phr" HW-PROBE-phr3 4)"
same "phrasing.sh cut: a task with no phrasing is its whole body, trimmed" "The whole task." \
    "$(sh "$phrasing" cut "$scratch/phr" HW-PROBE-phr0)"
# `files` writes the task files that campaign.sh keeps under `<out>/tasks/`.
mkdir -p "$scratch/phr/tasks"
printf 'stale\n' > "$scratch/phr/tasks/HW-PROBE-phr3.md"
printf 'stale\n' > "$scratch/phr/tasks/HW-PROBE-phr3.4.md"
same "phrasing.sh files: prints nothing when every phrasing has a body" "" \
    "$(sh "$phrasing" files "$scratch/phr" HW-PROBE-phr3 "$scratch/phr/tasks")"
same "phrasing.sh files: writes the count, and one file for each phrasing, and no other" "3 HW-PROBE-phr3.1.md HW-PROBE-phr3.2.md HW-PROBE-phr3.3.md HW-PROBE-phr3.phrasings" \
    "$(cat "$scratch/phr/tasks/HW-PROBE-phr3.phrasings") $(cd "$scratch/phr/tasks" && ls HW-PROBE-phr3* | tr '\n' ' ' | sed 's/ $//')"
same "phrasing.sh files: the file of phrasing 2 is its cut, byte for byte" "same" \
    "$(cmp -s "$scratch/phr/tasks/HW-PROBE-phr3.2.md" "$scratch/phr/want2" && echo same || echo differs)"
# The task file of a probe whose id ends in `-1` is not a phrasing of the
# probe whose id is that id without the `-1`, so `files` for the one leaves
# the task file of the other in place.
printf 'The task of pa-1.\n' > "$scratch/phr/tasks/HW-PROBE-pa-1.md"
printf -- '---\nid: HW-PROBE-pa\n---\n\n## Task\n\n### Phrasing 1\n\nOne.\n\n### Phrasing 2\n\nTwo.\n\n## Target\n\nA link.\n' \
    > "$scratch/phr/docs/probes/pa.md"
printf 'A note.\n' > "$scratch/phr/tasks/HW-PROBE-pa.1-notes.md"
sh "$phrasing" files "$scratch/phr" HW-PROBE-pa "$scratch/phr/tasks" > /dev/null
same "phrasing.sh files: leaves the task file of a probe whose id is the probe's id and -1" "The task of pa-1." \
    "$(cat "$scratch/phr/tasks/HW-PROBE-pa-1.md" 2>/dev/null)"
same "phrasing.sh files: and leaves a file whose name holds more than digits between the probe's id and .md" "A note." \
    "$(cat "$scratch/phr/tasks/HW-PROBE-pa.1-notes.md" 2>/dev/null)"
sh "$phrasing" files "$scratch/phr" HW-PROBE-pa-1 "$scratch/phr/tasks" > /dev/null
same "phrasing.sh files: and the files of that probe leave the probe's phrasings in place" "One. Two." \
    "$(cat "$scratch/phr/tasks/HW-PROBE-pa.1.md" "$scratch/phr/tasks/HW-PROBE-pa.2.md" 2>/dev/null | tr '\n' ' ' | sed 's/ $//')"
# A fenced code block is not markdown structure: a `# ` comment, a `## `
# line and a `### Phrasing <n>` line inside a fence end no phrasing, end no
# task and count as no phrasing (CommonMark's fenced code block).
printf -- '---\nid: HW-PROBE-phrf\n---\n\n## Task\n\n### Phrasing 1\n\nRun this:\n\n```sh\n# a comment\n## not a heading\n### Phrasing 9\n```\n\nThen report.\n\n### Phrasing 2\n\n~~~~\n```\n## inside a tilde fence\n~~~~\n\n## Target\n\nA link.\n' \
    > "$scratch/phr/docs/probes/phrf.md"
printf 'Run this:\n\n```sh\n# a comment\n## not a heading\n### Phrasing 9\n```\n\nThen report.\n' > "$scratch/phr/wantf1"
same "phrasing.sh count: a heading line inside a fenced code block is no phrasing" "2" \
    "$(sh "$phrasing" count "$scratch/phr" HW-PROBE-phrf)"
sh "$phrasing" cut "$scratch/phr" HW-PROBE-phrf 1 > "$scratch/phr/cutf1"
same "phrasing.sh cut 1: a # line inside a fenced code block does not end the phrasing, byte for byte" \
    "same" "$(cmp -s "$scratch/phr/cutf1" "$scratch/phr/wantf1" && echo same || echo differs)"
same "phrasing.sh cut 2: a fence closes only on its own marker, and a ## line inside it does not end the task" \
    "~~~~ \`\`\` ## inside a tilde fence ~~~~" \
    "$(sh "$phrasing" cut "$scratch/phr" HW-PROBE-phrf 2 | tr '\n' ' ' | sed 's/ $//')"
# A fence closes on its own character alone, and on a marker at least as long
# as the one that opened it: a line of four backticks does not close a tilde
# fence, and a line of three backticks does not close a fence of four.
printf -- '---\nid: HW-PROBE-phrc\n---\n\n## Task\n\n### Phrasing 1\n\n~~~\n````\n### Phrasing 7\n~~~\n\n### Phrasing 2\n\nTwo.\n\n## Target\n\nA link.\n' \
    > "$scratch/phr/docs/probes/phrc.md"
printf -- '---\nid: HW-PROBE-phrl\n---\n\n## Task\n\n### Phrasing 1\n\n````\n```\n### Phrasing 8\n````\n\n### Phrasing 2\n\nTwo.\n\n## Target\n\nA link.\n' \
    > "$scratch/phr/docs/probes/phrl.md"
# A false close reopens a fence on the real closing line, which hides the
# next phrasing, so the cut of phrasing 2 is what tells the two apart.
same "phrasing.sh cut 2: a backtick line does not close a tilde fence" "2 Two." \
    "$(sh "$phrasing" count "$scratch/phr" HW-PROBE-phrc) $(sh "$phrasing" cut "$scratch/phr" HW-PROBE-phrc 2)"
same "phrasing.sh cut 2: a marker shorter than the opening one does not close the fence" "2 Two." \
    "$(sh "$phrasing" count "$scratch/phr" HW-PROBE-phrl) $(sh "$phrasing" cut "$scratch/phr" HW-PROBE-phrl 2)"
sh "$phrasing" files "$scratch/phr" HW-PROBE-phr0 "$scratch/phr/tasks" > /dev/null
same "phrasing.sh files: a task with no phrasing is <probe>.md, its whole task, and a count of 0" "0 The whole task." \
    "$(cat "$scratch/phr/tasks/HW-PROBE-phr0.phrasings") $(cat "$scratch/phr/tasks/HW-PROBE-phr0.md")"
printf -- '---\nid: HW-PROBE-phre\n---\n\n## Task\n\n### Phrasing 1\n\nOne.\n\n### Phrasing 2\n\n## Target\n\nA link.\n' \
    > "$scratch/phr/docs/probes/phre.md"
same "phrasing.sh files: names the file of a phrasing with no body" "$scratch/phr/tasks/HW-PROBE-phre.2.md" \
    "$(sh "$phrasing" files "$scratch/phr" HW-PROBE-phre "$scratch/phr/tasks")"
same "phrasing.sh pick: no draw spreads 6 repetitions over 3 phrasings, two each" "1 2 3 1 2 3" \
    "$(for r in 1 2 3 4 5 6; do sh "$phrasing" pick HW-PROBE-phr3 "$r" 3; done | tr '\n' ' ' | sed 's/ $//')"
same "phrasing.sh pick: a task with no phrasing is 0, its whole task" "0" "$(sh "$phrasing" pick HW-PROBE-phr0 4 0)"
printf 'draw: HW-PROBE-phr3 2\ndraw: HW-PROBE-phr0 1\ndraw: HW-PROBE-big 4\n' > "$scratch/phr/draw"
same "phrasing.sh pick: a draw gives the job the phrasing it names" "2" \
    "$(sh "$phrasing" pick HW-PROBE-phr3 1 3 "$scratch/phr/draw")"
same "phrasing.sh pick: a draw of 1 for a task with no phrasing is its whole task" "0" \
    "$(sh "$phrasing" pick HW-PROBE-phr0 1 0 "$scratch/phr/draw")"
sh "$phrasing" pick HW-PROBE-big 1 3 "$scratch/phr/draw" > /dev/null 2> "$scratch/phr/err"
same "phrasing.sh pick: a phrasing above the count exits 5" "5" "$?"
sh "$phrasing" pick HW-PROBE-phr3 2 3 "$scratch/phr/draw" > /dev/null 2> "$scratch/phr/err"
same "phrasing.sh pick: a session the draw does not name exits 5" "5" "$?"
sh "$phrasing" pick HW-PROBE-other 1 3 "$scratch/phr/draw" > /dev/null 2> "$scratch/phr/err"
same "phrasing.sh pick: a probe the draw does not name exits 5" "5" "$?"
same "phrasing.sh draw: a task with no phrasing draws 1" "1" "$(sh "$phrasing" draw 0 docs 1 HW-PROBE-phr0 1 0)"
same "phrasing.sh draw: the same key draws the same phrasing" \
    "$(sh "$phrasing" draw 7 hooks 2 HW-PROBE-phr3 1 3)" "$(sh "$phrasing" draw 7 hooks 2 HW-PROBE-phr3 1 3)"
# The key and the width of the draw, held by literals that Python's hashlib
# gave outside this script: sha256 of `7 hooks 2 HW-PROBE-phr3 1` opens
# `ca6112a8`, which is 3395359400. Over 3 phrasings that is phrasing 3. Over a
# count of 2^32 the draw is the eight digits plus 1, so a key in another
# order or a cut of seven digits draws another number.
same "phrasing.sh draw: the key 7 hooks 2 HW-PROBE-phr3 1 over 3 phrasings is phrasing 3" "3" \
    "$(sh "$phrasing" draw 7 hooks 2 HW-PROBE-phr3 1 3)"
same "phrasing.sh draw: and over a count of 2^32 it is the first eight hex digits of the sha256, plus 1" "3395359401" \
    "$(sh "$phrasing" draw 7 hooks 2 HW-PROBE-phr3 1 4294967296)"
# Over 300 keys the draw reaches every phrasing, and no phrasing takes more
# than half of them, so it is not one fixed index.
same "phrasing.sh draw: over 300 keys each of 3 phrasings is drawn, none in more than half" "3 yes" \
    "$(for k in $(seq 1 300); do sh "$phrasing" draw "$k" docs 1 HW-PROBE-phr3 1 3; done | LC_ALL=C sort | uniq -c | awk '{ n++; if ($1 > 150) big = 1 } END { print n, (big ? "no" : "yes") }')"

# `open` is a key of the tuning round alone. A round's key outside the list
# used to break `rounds` and `approved`, and with them the round gate and
# both dry runs, so each still reads a declaration with an open layer.
python3 "$declared_py" "$scratch/open-docs1.yml" rounds > /dev/null 2> /dev/null
rounds_status=$?
python3 "$declared_py" "$scratch/open-docs1.yml" approved > /dev/null 2> /dev/null
same "declared.py rounds and approved still read a declaration with an open layer" "0 0" "$rounds_status $?"
# A round may declare its own count in place of its rates (#1835): a pilot
# measures each probe's rate and no difference. Its line carries
# `repetitions <n>` where the rates stand, so the probes stay in the same
# fields, and a round that declares both or neither is refused. No round of
# the real declaration uses the form since #1874 removed the code pilot, so
# the cases read a scratch declaration that adds a pilot round over the
# held-out set.
awk '$0 == "  final:" { print "  pilot:"; print "    set: held-out"; print "    repetitions: 18"; print "    tries: 1"; print "    approved_cents: 13000" } { print }' \
    "$root/.headwater/probe.yml" > "$scratch/reps.yml"
python3 "$declared_py" "$scratch/reps.yml" rounds > "$scratch/reps-rounds.out" 2> /dev/null
same "declared.py rounds prints the pilot's declared count in place of its rates" "0 pilot held-out repetitions 18 1 22" \
    "$? $(awk '$1 == "pilot" { print $1, $2, $3, $4, $5, NF - 5 }' "$scratch/reps-rounds.out")"
same "and the final round's declared count beside it (#1881)" "final held-out repetitions 3 1 22" \
    "$(awk '$1 == "final" { print $1, $2, $3, $4, $5, NF - 5 }' "$scratch/reps-rounds.out")"
# Each refusal exits 2 and states its own reason. `name|sed|needle`.
for reps_case in \
    "a round with both rates and repetitions|s/^    repetitions: 18$/    repetitions: 18\n    rates: [0.90, 0.80]/|declares both \`rates\` and \`repetitions\`" \
    "a round with neither rates nor repetitions|/^    repetitions: 18$/d|declares neither \`rates\` nor \`repetitions\`" \
    "a count of zero|s/^    repetitions: 18$/    repetitions: 0/|must be a whole number above zero" \
    "a count that is a word|s/^    repetitions: 18$/    repetitions: many/|must be a whole number above zero" \
    "a count that is a quoted number|s/^    repetitions: 18$/    repetitions: \"18\"/|must be a whole number above zero"
do
    reps_name=${reps_case%%|*}; reps_rest=${reps_case#*|}
    reps_sed=${reps_rest%%|*}; reps_needle=${reps_rest#*|}
    sed "$reps_sed" "$scratch/reps.yml" > "$scratch/reps-bad.yml"
    python3 "$declared_py" "$scratch/reps-bad.yml" rounds > "$scratch/reps-bad.out" 2> "$scratch/reps-bad.err"
    same "declared.py rounds refuses $reps_name, with 2 and its reason" "2 1 0" \
        "$? $(grep -cF -- "$reps_needle" "$scratch/reps-bad.err") $(wc -c < "$scratch/reps-bad.out" | tr -d ' ')"
done
# A round that declares its own count may also measure some scenarios per
# row (#1881): `rows` names them and `row_repetitions` is the count a line
# of one of them runs. Phase B declares scenarios 1 and 6 at 6, so 12
# sessions of each row in one batch.
python3 "$declared_py" "$root/.headwater/probe.yml" rows > "$scratch/rows.out" 2> /dev/null
same "declared.py rows prints the final round's row count and its row scenarios" "0 final 6 1 6" "$? $(cat "$scratch/rows.out")"
sed '/^    rows: /d; /^    row_repetitions: /d' "$root/.headwater/probe.yml" > "$scratch/rows-none.yml"
python3 "$declared_py" "$scratch/rows-none.yml" rows > "$scratch/rows-none.out" 2> /dev/null
same "and nothing for a declaration whose rounds declare no rows" "0 0" "$? $(wc -c < "$scratch/rows-none.out" | tr -d ' ')"
for rows_case in \
    "rows without row_repetitions|/^    row_repetitions: 6$/d|declares \`rows\` alone" \
    "row_repetitions without rows|/^    rows: \[1, 6\]$/d|declares \`row_repetitions\` alone" \
    "a row count of zero|s/^    row_repetitions: 6$/    row_repetitions: 0/|must be a whole number above zero" \
    "a row scenario that scenarios: does not declare|s/^    rows: \[1, 6\]$/    rows: [1, 9]/|names the scenario 9, which" \
    "a row scenario named twice|s/^    rows: \[1, 6\]$/    rows: [6, 6]/|names a scenario twice" \
    "rows on a round of rates|s/^    repetitions: 3$/    rates: [0.90, 0.80]/|only a round that declares its own \`repetitions\`"
do
    rows_name=${rows_case%%|*}; rows_rest=${rows_case#*|}
    rows_sed=${rows_rest%%|*}; rows_needle=${rows_rest#*|}
    sed "$rows_sed" "$root/.headwater/probe.yml" > "$scratch/rows-bad.yml"
    python3 "$declared_py" "$scratch/rows-bad.yml" rows > "$scratch/rows-bad.out" 2> "$scratch/rows-bad.err"
    rows_status=$?
    python3 "$declared_py" "$scratch/rows-bad.yml" rounds > /dev/null 2> /dev/null
    same "declared.py rows and rounds refuse $rows_name, with 2 and its reason" "2 2 1 0" \
        "$rows_status $? $(grep -cF -- "$rows_needle" "$scratch/rows-bad.err") $(wc -c < "$scratch/rows-bad.out" | tr -d ' ')"
done
awk '$0 == "    approved_cents: 70000" { print; print "    open: none"; next } { print }' "$root/.headwater/probe.yml" > "$scratch/open-screening.yml"
python3 "$declared_py" "$scratch/open-screening.yml" approved > /dev/null 2> "$scratch/open-screening.err"
same "and refuse open on any round but tuning, with 2" "2 1" \
    "$? $(grep -c 'only the tuning round declares `open`' "$scratch/open-screening.err")"
# The invalidator list is the owner's ruling of 2026-10-20: six entries, and
# neither seal.sh nor leak.py. Each entry names at least one tracked path, so
# no entry is a typo that matches nothing.
python3 "$declared_py" "$root/.headwater/probe.yml" base_invalidators > "$scratch/invalidators.out" 2> /dev/null
same "declared.py base_invalidators prints the six entries the owner ruled" \
    "0 engine/crates/*/src/** tools/probe/probe-record.sh tools/probe/probe-transform.sh tools/probe/egress-proxy.py tools/probe/campaign.sh tools/probe/ablate.sh" \
    "$? $(tr '\n' ' ' < "$scratch/invalidators.out" | sed 's/ $//')"
same "and each entry names at least one tracked path" "0" \
    "$(while IFS= read -r entry; do
        [ -n "$(git -C "$root" ls-files -- ":(glob)$entry" 2>/dev/null | head -1)" ] || echo "$entry"
    done < "$scratch/invalidators.out" | wc -l | tr -d ' ')"

# The batch reads the gate from its own checkout's declaration, and from no
# override. So each case runs a scratch root that holds the batch, the reader
# and a declaration: the gate runs before the tools, the engine and the clean
# checkout are checked, so a pass of the gate stops later, on 3.
open_root=$scratch/open-root
mkdir -p "$open_root/tools/probe" "$open_root/.headwater"
cp "$root/tools/probe/campaign.sh" "$root/tools/probe/declared.py" "$open_root/tools/probe/"
open_batch() {
    # $1 the declaration, $2 the spec, $3 the stderr file. Prints the status.
    cp "$1" "$open_root/.headwater/probe.yml"
    rm -rf "$scratch/open-out"
    sh "$open_root/tools/probe/campaign.sh" --out "$scratch/open-out" --model claude-haiku-4-5 --spec "$2" > /dev/null 2> "$3"
    echo "$?"
}
open_wrote() {
    printf '%s %s %s %s' "$([ -e "$scratch/open-out/leak-gate" ] && echo yes || echo no)" \
        "$([ -e "$scratch/open-out/lines" ] && echo yes || echo no)" \
        "$([ -e "$scratch/open-out/trees" ] && echo yes || echo no)" \
        "$([ -e "$scratch/open-out/claude-version" ] && echo yes || echo no)"
}
# A line of a round whose approval is 0 is refused with 5 before anything
# is built. The owner approved the final round on 2026-11-09 (#1881), so the
# case reads a declaration that takes the approval back.
sed 's/^    approved_cents: 105000$/    approved_cents: 0/' "$root/.headwater/probe.yml" > "$scratch/final-unapproved.yml"
printf 'additive scenario-4 final\n' > "$scratch/run-final.spec"
same "a line of a final round that nobody approved is refused with 5" "5" \
    "$(open_batch "$scratch/final-unapproved.yml" "$scratch/run-final.spec" "$scratch/run-final.err")"
present "and the refusal names the round" "no person has approved spend on the final round" "$scratch/run-final.err"
same "and it ran no dry run, planned no line, built no tree and asked the harness nothing" "no no no no" "$(open_wrote)"
# The decisive case of #1819: a paid tuning line of a layer whose plan
# nobody accepted. With docs open on scenario 1, a tuning line of scenario 3
# is refused, and the refusal names the open layer and scenario.
printf 'additive scenario-3 tuning\n' > "$scratch/open-s3.spec"
same "with docs open on scenario 1, a tuning line of scenario 3 is refused with 5" "5" \
    "$(open_batch "$scratch/open-docs1.yml" "$scratch/open-s3.spec" "$scratch/open-s3.err")"
present "and the refusal names the open layer and scenario, and the line's scenario" \
    "the tuning round is open for the docs layer on scenario 1 and its base, scenario 0, alone, and the spec holds a tuning line of scenario-3" "$scratch/open-s3.err"
same "and it ran no dry run, planned no line, built no tree and asked the harness nothing" "no no no no" "$(open_wrote)"
# The base of the open scenario is measured again at the confirmation's
# count (#1838), so its tuning line passes the gate beside the open one.
printf 'additive scenario-0 tuning\n' > "$scratch/open-s0.spec"
open_status=$(open_batch "$scratch/open-docs1.yml" "$scratch/open-s0.spec" "$scratch/open-s0.err")
same "with docs open on scenario 1, a tuning line of scenario 0, its base, passes the gate" "yes 0" \
    "$([ "$open_status" != 5 ] && echo yes || echo no) $(grep -c 'the tuning round is open for' "$scratch/open-s0.err")"
printf 'additive scenario-0 tuning\nadditive scenario-1 tuning\nadditive scenario-3 tuning\n' > "$scratch/open-s013.spec"
same "and a spec of the base, the open scenario and scenario 3 is still refused with 5" "5" \
    "$(open_batch "$scratch/open-docs1.yml" "$scratch/open-s013.spec" "$scratch/open-s013.err")"
present "and names scenario 3 alone" "and the spec holds a tuning line of scenario-3." "$scratch/open-s013.err"
absent "and not the base" "a tuning line of scenario-0." "$scratch/open-s013.err"
# The edit that admits the base is inside a base-neutral region, as try.sh
# reads the regions, so it makes no measured base stale. The reader is the
# `regions` function of try.sh itself.
sed -n '/^regions() {$/,/^}$/p' "$root/tools/probe/try.sh" > "$scratch/regions.sh"
printf '%s\n' 'regions "$1" campaign.sh' >> "$scratch/regions.sh"
sh "$scratch/regions.sh" "$root/tools/probe/campaign.sh" > "$scratch/regions.out" 2> "$scratch/regions.err"
same "try.sh's reader of the regions reads campaign.sh" "0" "$?"
grep -n 'open_base' "$root/tools/probe/campaign.sh" | cut -d: -f1 > "$scratch/open-base.lines"
same "campaign.sh names open_base on three lines" "3" "$(wc -l < "$scratch/open-base.lines" | tr -d ' ')"
same "and each is inside a base-neutral region" "" "$(grep -vxF -f "$scratch/regions.out" "$scratch/open-base.lines")"
printf 'additive scenario-1 tuning\n' > "$scratch/open-s1.spec"
same "with open: none, a scenario-1 tuning line is refused with 5" "5" \
    "$(open_batch "$scratch/open-none.yml" "$scratch/open-s1.spec" "$scratch/open-nonespec.err")"
present "and the refusal names the open rule" "the tuning round is open for no layer" "$scratch/open-nonespec.err"
same "and it ran no dry run, planned no line, built no tree and asked the harness nothing" "no no no no" "$(open_wrote)"
open_status=$(open_batch "$scratch/open-docs1.yml" "$scratch/open-s1.spec" "$scratch/open-pass.err")
same "with docs open on scenario 1, a scenario-1 tuning line passes the gate" "yes 0" \
    "$([ "$open_status" != 5 ] && echo yes || echo no) $(grep -c 'the tuning round is open for' "$scratch/open-pass.err")"
# The gate reads tuning lines alone: a screening line runs while no layer is
# open, and a tuning line of the open scenario runs beside it.
printf 'additive scenario-4 screening\n' > "$scratch/open-screen.spec"
open_status=$(open_batch "$scratch/open-none.yml" "$scratch/open-screen.spec" "$scratch/open-screen.err")
same "with open: none, a screening line passes the gate" "yes 0" \
    "$([ "$open_status" != 5 ] && echo yes || echo no) $(grep -c 'the tuning round is open for' "$scratch/open-screen.err")"
printf 'additive scenario-4 screening\nadditive scenario-1 tuning\nadditive scenario-6 tuning\n' > "$scratch/open-mixed.spec"
same "with docs open on scenario 1, a spec that also holds scenario 6 is refused with 5" "5" \
    "$(open_batch "$scratch/open-docs1.yml" "$scratch/open-mixed.spec" "$scratch/open-mixed.err")"
present "and names scenario 6" "and the spec holds a tuning line of scenario-6" "$scratch/open-mixed.err"
absent "and not the open scenario" "a tuning line of scenario-1." "$scratch/open-mixed.err"
# A scenario whose name begins with the open one's is another scenario.
printf 'additive scenario-10 tuning\n' > "$scratch/open-s10.spec"
same "with docs open on scenario 1, a tuning line of scenario 10 is refused with 5" "5" \
    "$(open_batch "$scratch/open-docs1.yml" "$scratch/open-s10.spec" "$scratch/open-s10.err")"
present "and names scenario 10" "and the spec holds a tuning line of scenario-10." "$scratch/open-s10.err"
# A declaration the reader refuses stops the batch with the reader's own 2.
sed 's/^      try: 1$/      try: 5/' "$scratch/open-docs1.yml" > "$scratch/open-try4.yml"
same "a malformed open stops the batch with the reader's 2" "2" \
    "$(open_batch "$scratch/open-try4.yml" "$scratch/open-s1.spec" "$scratch/open-try4.err")"

printf '\n%s passed, %s failed\n' "$passed" "$failed"
[ "$failed" = 0 ]
