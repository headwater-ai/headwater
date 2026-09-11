#!/bin/sh
# What holds `tools/run/run-census.sh`.
#
# The tool's whole value is that it counts the way the evaluation counts: one
# usage record per `message.id`, a call grouped by what it is for rather than
# by its text. Both are held here against a transcript written by hand whose
# right answer is known: five lines, four turns, because one response is
# written as two lines that carry one usage record, and two `gh pr view` calls
# in one turn that cost one turn and not two.
#
# Run it from anywhere:
#     sh tools/run/run-census-fixtures.sh
#
# It needs `jq`, exits 3 without it so CI can skip with a warning, and writes
# only under a temporary directory.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/run/run-census.sh"

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
same 'a session with no agent transcripts beside it has no fleet' 'no agent transcripts beside the session file, so no fleet' "$(printf '%s\n' "$out" | tail -1)"

# A heredoc body is data a command writes, not a command it runs. Turn h1
# appends a ledger entry that quotes `for`, `cargo test` and a `gh pr view`
# call inside its body; none of those are mentions the parent made. Turn h2
# runs a real `gh pr view`, so the table still counts what is actually run.
cat > "$scratch/heredoc.jsonl" <<'EOF'
{"type":"assistant","message":{"id":"h1","usage":{"cache_read_input_tokens":1000},"content":[{"type":"tool_use","name":"Bash","input":{"command":"cat >> file.md << 'EOF2'\nfor this cargo test gh pr view is quoted, not run\nEOF2"}}]}}
{"type":"assistant","message":{"id":"h2","usage":{"cache_read_input_tokens":1000},"content":[{"type":"tool_use","name":"Bash","input":{"command":"gh pr view 9"}}]}}
EOF

hout=$(sh "$tool" "$scratch/heredoc.jsonl" 2>&1); status=$?
hrow() { printf '%s\n' "$hout" | awk -v g="$1" '/^Bash, by what/ { exit } $1 == g && ($2 == "" || $2 !~ /^[a-z]/) { print; exit } $1" "$2 == g { print; exit }' | tr -s ' '; }
hmrow() { printf '%s\n' "$hout" | awk -v g="$1" 'on && ($1 == g || $1" "$2 == g) { print; exit } /^Bash, by what/ { on = 1 }' | tr -s ' '; }
same 'the heredoc census exits 0' 0 "$status"
same 'a heredoc body is stripped, so its cat call is the whole verb' 'cat 1 1 1000 50%' "$(hrow cat)"
same 'the real call outside the heredoc still files as its own verb' 'gh pr 1 1 1000 50%' "$(hrow 'gh pr')"
same 'the mentions table does not count a poll word quoted inside the body' 'gh pr 1 1 1000 50%' "$(hmrow 'gh pr')"
same '  the by-tool table counts both calls' 'Bash 2 2 2000 100%' "$(hrow Bash)"

# The fleet. A parent that dispatches a builder at 00:00 and a verifier at
# 00:10, rules at 00:45 with no tool call, compacts at 00:50, dispatches an
# integrator at 01:00, and at 01:10 dispatches a type the harness refuses. The
# builder runs to 00:30 and spawns a depth-two agent of its own; the verifier
# runs to 00:40; the integrator to 01:20. So the span is 80 minutes, 20 of
# them with no agent in flight, and the agent-minutes are 80 over 80, one in
# flight on average. The builder's transcript carries a truncated line, the
# shape the measured run left in one of its files, and it is skipped. Five
# turns over one agent of each type is five turns per agent.
mkdir -p "$scratch/fleet/subagents"
cat > "$scratch/fleet.jsonl" <<'EOF'
{"type":"assistant","timestamp":"2026-01-01T00:00:00.000Z","message":{"id":"f1","usage":{"cache_read_input_tokens":1000},"content":[{"type":"tool_use","id":"tA","name":"Agent","input":{"subagent_type":"hw-build","prompt":"a"}}]}}
{"type":"assistant","timestamp":"2026-01-01T00:10:00.000Z","message":{"id":"f2","usage":{"cache_read_input_tokens":1000},"content":[{"type":"tool_use","id":"tB","name":"Agent","input":{"subagent_type":"hw-verify","prompt":"b"}}]}}
{"type":"assistant","timestamp":"2026-01-01T00:45:00.000Z","message":{"id":"f3","usage":{"cache_read_input_tokens":1000},"content":[{"type":"text","text":"ruling"}]}}
{"type":"user","timestamp":"2026-01-01T00:50:00.000Z","isCompactSummary":true,"message":{"role":"user","content":"summary"}}
{"type":"assistant","timestamp":"2026-01-01T01:00:00.000Z","message":{"id":"f4","usage":{"cache_read_input_tokens":1000},"content":[{"type":"tool_use","id":"tC","name":"Agent","input":{"subagent_type":"hw-integrate","prompt":"c"}}]}}
{"type":"assistant","timestamp":"2026-01-01T01:10:00.000Z","message":{"id":"f5","usage":{"cache_read_input_tokens":1000},"content":[{"type":"tool_use","id":"tD","name":"Agent","input":{"subagent_type":"hw-build","prompt":"d"}}]}}
{"type":"user","timestamp":"2026-01-01T01:10:01.000Z","message":{"role":"user","content":[{"type":"tool_result","tool_use_id":"tD","is_error":true,"content":"Agent type not found."}]},"toolUseResult":"Error: Agent type not found."}
EOF
printf '%s\n' '{"agentType":"hw-build","description":"a","toolUseId":"tA","spawnDepth":1}' > "$scratch/fleet/subagents/agent-a1.meta.json"
cat > "$scratch/fleet/subagents/agent-a1.jsonl" <<'EOF'
{"type":"user","timestamp":"2026-01-01T00:00:00.000Z","message":{"role":"user","content":"a"}}
{"type":"assistant","timestamp":"2026-01-01T00:12:00.000Z","message":{"id":"a1m1","content":[{"type":"text","text":"working"}]}}
{"type":"assistant","timestamp":"2026-01-01T00:29:00.000Z","message":{"id":"a1m2","content":[{"type":"text","text":"a line the harness cut sho
{"type":"assistant","timestamp":"2026-01-01T00:30:00.000Z","message":{"id":"a1m3","content":[{"type":"text","text":"BRANCH: x"}]}}
EOF
printf '%s\n' '{"agentType":"hw-verify","description":"b","toolUseId":"tB","spawnDepth":1}' > "$scratch/fleet/subagents/agent-b1.meta.json"
cat > "$scratch/fleet/subagents/agent-b1.jsonl" <<'EOF'
{"type":"user","timestamp":"2026-01-01T00:10:00.000Z","message":{"role":"user","content":"b"}}
{"type":"assistant","timestamp":"2026-01-01T00:40:00.000Z","message":{"id":"b1m1","content":[{"type":"text","text":"VERDICT: PASS"}]}}
EOF
printf '%s\n' '{"agentType":"hw-integrate","description":"c","toolUseId":"tC","spawnDepth":1}' > "$scratch/fleet/subagents/agent-c1.meta.json"
cat > "$scratch/fleet/subagents/agent-c1.jsonl" <<'EOF'
{"type":"user","timestamp":"2026-01-01T01:00:00.000Z","message":{"role":"user","content":"c"}}
{"type":"assistant","timestamp":"2026-01-01T01:20:00.000Z","message":{"id":"c1m1","content":[{"type":"text","text":"MERGED: 1"}]}}
EOF
printf '%s\n' '{"agentType":"general-purpose","description":"e","toolUseId":"tE","spawnDepth":2}' > "$scratch/fleet/subagents/agent-e1.meta.json"
cat > "$scratch/fleet/subagents/agent-e1.jsonl" <<'EOF'
{"type":"user","timestamp":"2026-01-01T00:15:00.000Z","message":{"role":"user","content":"e"}}
{"type":"assistant","timestamp":"2026-01-01T00:20:00.000Z","message":{"id":"e1m1","content":[{"type":"text","text":"found"}]}}
EOF

fout=$(sh "$tool" "$scratch/fleet.jsonl" 2>&1); status=$?
same 'the fleet census exits 0' 0 "$status"
fleet() { printf '%s\n' "$fout" | awk -v g="$1" 'on && $1 == g { print; exit } /^fleet$/ { on = 1 }' | tr -s ' '; }
same 'agents are counted from the transcripts, a refused dispatch is not one, and depth two is set aside' 'dispatched 4 by Agent calls, 3 agents at depth one, 1 deeper' "$(fleet dispatched)"
same 'the span runs from the first agent to the last' 'span 1.3 h from the first agent'"'"'s start to the last agent'"'"'s end' "$(fleet span)"
same 'the idle share, and the largest window with nothing in flight' 'idle 25% of the span with no agent in flight, 0.3 h, largest window 20 min' "$(fleet idle)"
same 'the mean concurrency is agent-minutes over the span' 'mean concurrency 1 agents in flight' "$(fleet mean)"
same 'one compaction, with the gap on each side of it' ' 2026-01-01T00:50:00Z 5 min since the parent'"'"'s last turn, 10 min to its next dispatch' "$(fleet 2026-01-01T00:50:00Z)"
same 'a truncated line in an agent transcript is skipped, and the builder still spans 30 minutes' 'hw-build 1 30 5' "$(fleet hw-build)"
same '  the integrator row is the parent'"'"'s turns per merge' 'hw-integrate 1 20 5' "$(fleet hw-integrate)"

sh "$tool" "$scratch/missing.jsonl" >/dev/null 2>&1; status=$?
same 'a missing file is refused with usage' 2 "$status"

printf '\n%s passed, %s failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
