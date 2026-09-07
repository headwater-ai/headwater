#!/bin/sh
# What holds `tools/run-census.sh`.
#
# The tool's whole value is that it counts the way the evaluation counts: one
# usage record per `message.id`, a call grouped by what it is for rather than
# by its text. Both are held here against a transcript written by hand whose
# right answer is known: five lines, four turns, because one response is
# written as two lines that carry one usage record, and two `gh pr view` calls
# in one turn that cost one turn and not two.
#
# Run it from anywhere:
#     sh tools/run-census-fixtures.sh
#
# It needs `jq`, exits 3 without it so CI can skip with a warning, and writes
# only under a temporary directory.

set -u

root=$(cd "$(dirname "$0")/.." && pwd)
tool="$root/tools/run-census.sh"

if ! command -v jq >/dev/null 2>&1; then
    echo "no \`jq\` on the path, and the tool under test reads JSON with it." >&2
    exit 3
fi

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

passed=0
failed=0
pass() { printf 'ok   %s\n' "$1"; passed=$((passed + 1)); }
fail() { printf 'FAIL %s\n  %s\n' "$1" "$2"; failed=$((failed + 1)); }
same() {
    if [ "$2" = "$3" ]; then pass "$1"; else fail "$1" "expected \`$2\`, got \`$3\`"; fi
}

# Turn 1: two lines, one usage record (1000), a Read and a Bash `gh pr view`.
# Turn 2: one line (2000), a second `gh pr view` and a `cargo test`.
# Turn 3: one line (4000), `HEADWATER_BLESS=1 cargo test` behind an assignment.
# Turn 4: one line (8000), the engine by path, and a user line to ignore.
# Turn 5: `cd … && gh pr view`, the compound shape the measured run wrote.
# Turn 6: `set -e; cargo test … | tail`, the other one.
# Turn 7: a `gh pr view` inside an `until` loop, which the verb table files
# under `until` and the mentions table counts as the poll it is.
cat > "$scratch/session.jsonl" <<'EOF'
{"type":"assistant","message":{"id":"m1","usage":{"cache_read_input_tokens":1000},"content":[{"type":"tool_use","name":"Read","input":{"file_path":"x"}}]}}
{"type":"assistant","message":{"id":"m1","usage":{"cache_read_input_tokens":1000},"content":[{"type":"tool_use","name":"Bash","input":{"command":"gh pr view 1 --json mergeable"}}]}}
{"type":"assistant","message":{"id":"m2","usage":{"cache_read_input_tokens":2000},"content":[{"type":"tool_use","name":"Bash","input":{"command":"gh pr view 2"}},{"type":"tool_use","name":"Bash","input":{"command":"cargo test --workspace"}}]}}
{"type":"user","message":{"content":"ignored"}}
{"type":"assistant","message":{"id":"m3","usage":{"cache_read_input_tokens":4000},"content":[{"type":"tool_use","name":"Bash","input":{"command":"HEADWATER_BLESS=1 cargo test -p x > out 2>&1"}}]}}
{"type":"assistant","message":{"id":"m4","usage":{"cache_read_input_tokens":8000},"content":[{"type":"tool_use","name":"Bash","input":{"command":"engine/target/dev-release/headwater check --strict"}},{"type":"text","text":"done"}]}}
{"type":"assistant","message":{"id":"m5","usage":{"cache_read_input_tokens":16000},"content":[{"type":"tool_use","name":"Bash","input":{"command":"cd /home/x/repo && gh pr view 3 --json mergeable -q .mergeable"}}]}}
{"type":"assistant","message":{"id":"m6","usage":{"cache_read_input_tokens":32000},"content":[{"type":"tool_use","name":"Bash","input":{"command":"set -e; cargo test --workspace 2>&1 | tail -3"}}]}}
{"type":"assistant","message":{"id":"m7","usage":{"cache_read_input_tokens":64000},"content":[{"type":"tool_use","name":"Bash","input":{"command":"until [ \"$(gh pr view 4 --json mergeable -q .mergeable)\" != \"UNKNOWN\" ]; do sleep 30; done"}}]}}
EOF

out=$(sh "$tool" "$scratch/session.jsonl" 2>&1); status=$?
same 'the census exits 0' 0 "$status"
same 'turns are counted per message id, not per line' 'turns 7  tool calls 9  cache reads 127000' "$(printf '%s\n' "$out" | head -1)"

# A row of the first table that names the group, or of a named table.
row() { printf '%s\n' "$out" | awk -v g="$1" '/^Bash, by what/ { exit } $1 == g && ($2 == "" || $2 !~ /^[a-z]/) { print; exit } $1" "$2 == g { print; exit }' | tr -s ' '; }
mrow() { printf '%s\n' "$out" | awk -v g="$1" 'on && ($1 == g || $1" "$2 == g) { print; exit } /^Bash, by what/ { on = 1 }' | tr -s ' '; }

same 'gh pr view is found behind a cd, and counted per turn' 'gh pr 3 3 19000 15%' "$(row 'gh pr')"
same 'cargo test behind an assignment, and behind set -e, groups with the bare one' 'cargo test 3 3 38000 29.9%' "$(row 'cargo test')"
same 'the engine named by path groups as headwater' 'headwater check 1 1 8000 6.3%' "$(row 'headwater check')"
same 'a poll inside a loop is filed under the loop by the verb table' 'until 1 1 64000 50.4%' "$(row 'until')"
same 'the by-tool table counts Bash calls' 'Bash 8 7 127000 100%' "$(row 'Bash')"
same '  and the Read' 'Read 1 1 1000 0.8%' "$(row 'Read')"
same 'the mentions table counts the poll inside the loop as a poll' 'gh pr 4 4 83000 65.4%' "$(mrow 'gh pr')"
same '  and the loop as a loop' 'until 1 1 64000 50.4%' "$(mrow 'until')"
same 'no call is grouped under cd or set' 0 "$(printf '%s\n' "$out" | awk '$1 == "cd" || $1 == "set"' | wc -l | tr -d ' ')"

sh "$tool" "$scratch/missing.jsonl" >/dev/null 2>&1; status=$?
same 'a missing file is refused with usage' 2 "$status"

printf '\n%s passed, %s failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
