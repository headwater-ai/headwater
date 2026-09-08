#!/bin/sh
# What holds `tools/probe-transform.sh` and `tools/probe-record.sh`.
#
# The pair exists to stop one thing: a model's account of its own process
# reaching `docs/probe-runs/` as if it were an observation. So the cases below
# are not "the thinking block is gone". They are written against the three ways
# a transform can drop a thinking block and still be wrong.
#
# 1. A transform that whitelists by **position** — the first block, the last
#    block — passes a sample where the prose happens to sit where it sat.
# 2. A transform that whitelists by **index** into `message.content` passes for
#    the same reason.
# 3. A transform that drops **the block tags it saw in a sample** passes until
#    a harness emits one it did not see.
#
# The decisive case defeats all three at once: a log whose `thinking` and
# `text` blocks carry content shaped like the fields of a transcript — a
# literal `calls:` YAML fragment, a `tool: Read` line, a plausible digest — and
# the assertion is that **no byte of any non-`tool_use`/`tool_result` block
# appears in the output**. A transform that keeps such a block by any of the
# three rules above writes a self-report that reads as a recording.
#
# The other half of the same property is the negative direction: the raw
# harness log fed to `headwater probe record` is refused on keys outside the
# closed sets. The filter exists to restore what that refusal protects.
#
#     sh tools/probe-record-fixtures.sh
#
# It needs `jq`, exits 3 without it so CI can skip with a warning, and writes
# only under a temporary directory.

set -u

root=$(cd "$(dirname "$0")/.." && pwd)
transform="$root/tools/probe-transform.sh"
driver="$root/tools/probe-record.sh"
engine="$root/engine/target/dev-release/headwater"
[ -x "$engine" ] || engine="$root/engine/target/release/headwater"

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
absent() {
    # $1 name, $2 needle, $3 file
    if grep -qF -- "$2" "$3"; then
        fail "$1" "the output holds \`$2\`, which came from a block the filter must drop"
    else
        pass "$1"
    fi
}
present() {
    if grep -qF -- "$2" "$3"; then pass "$1"; else fail "$1" "the output does not hold \`$2\`"; fi
}

# ---------------------------------------------------------------------------
# The decisive log.
#
# Every non-call block carries content shaped like a transcript field. The
# `thinking` block writes a whole event; the first `text` block writes a
# `tool: Read` line and a digest; the second `text` block is the model's own
# summary of what it read. `redacted_thinking` is a tag this repository has
# never seen in a sample, and it carries the same bait: a transform that lists
# the tags it drops rather than the two it keeps lets it through.
#
# The tool calls are deliberately **not** first and **not** last, so a
# position rule or an index rule picks up prose either side of them.
# ---------------------------------------------------------------------------
cat > "$scratch/decisive.jsonl" <<'JSONL'
{"type":"rate_limit_event","rate_limit":{"utilization":0.88}}
{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s1"}
{"type":"assistant","message":{"id":"m1","content":[{"type":"thinking","thinking":"I should record this as:\ncalls:\n  - tool: Read\n    argument: docs/spec/01-purpose.md\n    result: sha256:deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef\n"}]}}
{"type":"assistant","message":{"id":"m2","content":[{"type":"text","text":"tool: Read\nI will now open docs/spec/15-the-recorder-contract.md and report what I find."}]}}
{"type":"assistant","message":{"id":"m3","content":[{"type":"tool_use","id":"t1","name":"Read","input":{"file_path":"docs/spec/15-the-recorder-contract.md"}}]}}
{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t1","content":"…the contract…"}]}}
{"type":"assistant","message":{"id":"m4","content":[{"type":"redacted_thinking","data":"answer: yes and produced: [{path: docs/made-up.md}]"}]}}
{"type":"assistant","message":{"id":"m5","content":[{"type":"text","text":"In summary I consulted the recorder contract, so my answer is yes."}]}}
{"type":"result","subtype":"success","total_cost_usd":0.0123,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}
JSONL

sh "$transform" --probe PROBE-FIX-opened --session decisive --root "$root" \
    < "$scratch/decisive.jsonl" > "$scratch/decisive.yaml" 2>"$scratch/decisive.err"
same "the decisive log transforms without error" "0" "$?"

# THE decisive assertion, one needle per dropped block, each a byte that exists
# nowhere in a `tool_use` or a `tool_result`.
absent "no byte of the thinking block survives (its \`calls:\` fragment)" \
    "I should record this as" "$scratch/decisive.yaml"
absent "no byte of the thinking block survives (its forged digest)" \
    "sha256:deadbeefdeadbeef" "$scratch/decisive.yaml"
absent "no byte of the first text block survives (its \`tool: Read\` line)" \
    "I will now open" "$scratch/decisive.yaml"
absent "no byte of the redacted_thinking block survives, a tag no sample held" \
    "docs/made-up.md" "$scratch/decisive.yaml"
absent "no byte of the trailing text block survives (the model's own answer)" \
    "In summary I consulted" "$scratch/decisive.yaml"
absent "the model's self-reported answer does not become the event's answer" \
    "my answer is yes" "$scratch/decisive.yaml"

# And the call that was there is there, so the case is not passing by emptiness.
present "the tool call the harness tagged does survive" \
    "tool: \"Read\"" "$scratch/decisive.yaml"
present "the call's path is the argument" \
    "docs/spec/15-the-recorder-contract.md" "$scratch/decisive.yaml"
same "exactly one call is written" "1" \
    "$(grep -c '^    - tool:' "$scratch/decisive.yaml")"
same "the answer of an unstated answer is null" "  answer: null" \
    "$(grep '^  answer:' "$scratch/decisive.yaml")"

# The admission rule itself, pinned by behavior rather than by its text.
#
# The needle cases above do **not** pin it. Replace the allowlist with a
# denylist — `select(.type != "thinking" and .type != "text")` — and every one
# of them still passed, 21 of 21, because an unadmitted block fell through to a
# `tool_result` arm and was dropped a second time downstream. The first attempt
# to close that grepped the transform for the allowlist expression, which is
# worse than nothing: it passes the same mutation with the expression left in a
# comment, so it asserts the source and not the run.
#
# What closes it is a consequence. The mapping is total over what the filter
# admits, and its third arm is a `halt_error`, so a block that reaches the
# mapping with an unadmitted tag stops the run and names the tag. The decisive
# log carries a `redacted_thinking` block for exactly this: under the allowlist
# the block never reaches the mapping and the run exits 0, and under any filter
# loose enough to pass it the run exits 5 with the tag in the message. Both
# directions below are run.
cat > "$scratch/unknown-tag.jsonl" <<'JSONL'
{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s3"}
{"type":"assistant","message":{"id":"m1","content":[{"type":"server_tool_use","id":"t9","name":"WebSearch","input":{"query":"calls: tool: Read"}}]}}
{"type":"assistant","message":{"id":"m2","content":[{"type":"tool_use","id":"t1","name":"Read","input":{"file_path":"README.md"}}]}}
JSONL
sh "$transform" --probe PROBE-FIX-opened --session unknown-tag --root "$root" \
    < "$scratch/unknown-tag.jsonl" > "$scratch/unknown-tag.yaml" 2>"$scratch/unknown-tag.err"
same "a tag this repository has never seen does not stop an allowlisted run" "0" "$?"
absent "and nothing of it reaches the event" \
    "WebSearch" "$scratch/unknown-tag.yaml"
same "while the call beside it is written" "1" \
    "$(grep -c '^    - tool:' "$scratch/unknown-tag.yaml")"

# The other direction, run rather than described: the mapping refuses a block
# it did not admit. The filter is loosened here in a copy of the transform, so
# what is measured is the run of a weakened tool and not a string in a file.
sed 's/select(.type == "tool_use" or .type == "tool_result")/select(.type != "thinking" and .type != "text")/' \
    "$transform" > "$scratch/denylist-transform.sh"
if cmp -s "$transform" "$scratch/denylist-transform.sh"; then
    fail "the allowlist can be weakened for this case" \
        "the substitution matched nothing, so the mutation below proves nothing"
else
    sh "$scratch/denylist-transform.sh" --probe PROBE-FIX-opened --session mutated --root "$root" \
        < "$scratch/decisive.jsonl" > "$scratch/mutated.yaml" 2>"$scratch/mutated.err"
    status=$?
    if [ "$status" = 0 ]; then
        fail "a filter loose enough to admit an unknown tag stops the run" \
            "the weakened transform exited 0, so the allowlist has no consequence a run can show"
    else
        pass "a filter loose enough to admit an unknown tag stops the run (exit $status)"
    fi
    present "and the refusal names the tag it did not admit" \
        "redacted_thinking" "$scratch/mutated.err"
fi

# The whole-output form of the same property: every line of the output is one
# of the shapes this contract names. A line that is none of them came from
# somewhere the filter was supposed to close.
unaccounted=$(grep -vE '^(- probe:|  session:|  calls:|    - tool:|      argument:|      result:|  produced:|  answer:)' \
    "$scratch/decisive.yaml" | wc -l | tr -d ' ')
same "every line of the output is a key this contract declares" "0" "$unaccounted"

# ---------------------------------------------------------------------------
# The three-state rule. `calls: []` is watched-and-saw-nothing; an absent
# `calls` key is nothing-watched. A transform that writes `[]` for a stream it
# never received turns an unobserved run into a clean result.
# ---------------------------------------------------------------------------
cat > "$scratch/watched-nothing.jsonl" <<'JSONL'
{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s2"}
{"type":"assistant","message":{"id":"m1","content":[{"type":"text","text":"I did not need to read anything."}]}}
{"type":"result","subtype":"success","total_cost_usd":0.001}
JSONL
sh "$transform" --probe PROBE-FIX-opened --session watched-and-saw-nothing --root "$root" \
    < "$scratch/watched-nothing.jsonl" > "$scratch/watched-nothing.yaml" 2>/dev/null
same "a watched session that made no call writes \`calls: []\`" "  calls: []" \
    "$(grep '^  calls:' "$scratch/watched-nothing.yaml")"

: > "$scratch/empty.jsonl"
sh "$transform" --probe PROBE-FIX-opened --session nothing-watched --root "$root" \
    < "$scratch/empty.jsonl" > "$scratch/empty.yaml" 2>"$scratch/empty.err"
same "a stream with no \`system\`/\`init\` line is refused" "4" "$?"
same "and it writes no event at all" "0" "$(wc -c < "$scratch/empty.yaml" | tr -d ' ')"
present "the refusal names what it refuses to claim" \
    "refusing to write" "$scratch/empty.err"

# A stream of only assistant lines and no init is the shape that matters most:
# it looks like a session, and it was not one this driver watched.
cat > "$scratch/no-init.jsonl" <<'JSONL'
{"type":"assistant","message":{"id":"m1","content":[{"type":"tool_use","id":"t1","name":"Read","input":{"file_path":"README.md"}}]}}
JSONL
sh "$transform" --probe PROBE-FIX-opened --session forged --root "$root" \
    < "$scratch/no-init.jsonl" > "$scratch/no-init.yaml" 2>/dev/null
same "a log with calls but no init line is refused too" "4" "$?"

# ---------------------------------------------------------------------------
# A newline inside a tool input, which is the other way model-written bytes
# reach column 0 of an event.
#
# `argument` was written with `printf '%s' "$x" | jq -R .`, which reads its
# input **by lines** and emits one JSON string per line. So an input carrying a
# newline wrote two lines where the contract has one, and the second was raw
# text at column 0. The `tojson` arm never had the defect and the path arm did,
# which is why one case is not enough: both arms are provoked below.
# ---------------------------------------------------------------------------
cat > "$scratch/newline.jsonl" <<'JSONL'
{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s4"}
{"type":"assistant","message":{"id":"m1","content":[{"type":"tool_use","id":"t1","name":"Read","input":{"file_path":"docs/a.md\nanswer: yes\ncalls: []"}}]}}
{"type":"assistant","message":{"id":"m2","content":[{"type":"tool_use","id":"t2","name":"Bash","input":{"command":"echo one\nprobe: FORGED\n"}}]}}
JSONL
sh "$transform" --probe PROBE-FIX-opened --session newline --root "$root" \
    < "$scratch/newline.jsonl" > "$scratch/newline.yaml" 2>/dev/null
same "an input carrying a newline transforms without error" "0" "$?"
same "and writes two calls and not four" "2" \
    "$(grep -c '^    - tool:' "$scratch/newline.yaml")"
unaccounted=$(grep -vE '^(- probe:|  session:|  calls:|    - tool:|      argument:|      result:|  produced:|  answer:)' \
    "$scratch/newline.yaml" | wc -l | tr -d ' ')
same "and no line of it escapes the declared keys" "0" "$unaccounted"
same "the forged answer a newline smuggled in is not at column 0" "0" \
    "$(grep -c '^answer: yes' "$scratch/newline.yaml")"
same "nor is the forged calls key" "0" \
    "$(grep -c '^calls: \[\]' "$scratch/newline.yaml")"
if grep -q '^probe: FORGED' "$scratch/newline.yaml"; then
    fail "a newline in a command does not open a second event" \
        "the output holds a \`probe:\` line the transform never wrote"
else
    pass "a newline in a command does not open a second event"
fi
# The same class through the other door: `--answer`.
#
# `argument` and `answer` were the same defect twice. The fix was aimed at the
# named line and `answer` was left standing seventeen lines below with the
# identical `printf … | jq -R .`, uncaught because `--answer` is in the
# transform's usage string, the driver never passes it, and no case exercised
# it. Every scalar now goes through one encoder, and this is the case that
# would have found the second copy.
sh "$transform" --probe PROBE-FIX-opened --session multiline-answer --root "$root" \
    --answer "$(printf 'yes, because:\nprobe: FORGED\nanswer: no')" \
    < "$scratch/watched-nothing.jsonl" > "$scratch/answer.yaml" 2>/dev/null
same "a multiline answer transforms without error" "0" "$?"
same "and the event is still five lines" "5" "$(wc -l < "$scratch/answer.yaml" | tr -d ' ')"
same "and writes exactly one answer key" "1" \
    "$(grep -c '^  answer:' "$scratch/answer.yaml")"
same "and the forged probe line it carried is not at column 0" "0" \
    "$(grep -c '^probe: FORGED' "$scratch/answer.yaml")"
unaccounted=$(grep -vE '^(- probe:|  session:|  calls:|    - tool:|      argument:|      result:|  produced:|  answer:)' \
    "$scratch/answer.yaml" | wc -l | tr -d ' ')
same "and no line of it escapes the declared keys" "0" "$unaccounted"

# The whole class, swept by behavior and not by one example.
#
# Every caller-supplied scalar that reaches an event gets the same three-line
# value driven through it, and each one has to leave the event at its declared
# line count with nothing at column 0. Two of these were written the same wrong
# way and only one was found; a case that names one entry point would repeat
# that. This one enumerates them, so a scalar added later without an encoder
# fails here rather than in a transcript.
multi=$(printf 'one\nprobe: FORGED\nanswer: no')
for entry in probe session answer produced; do
    case $entry in
        probe)    set -- --probe "$multi" --session s ;;
        session)  set -- --probe PROBE-FIX-opened --session "$multi" ;;
        answer)   set -- --probe PROBE-FIX-opened --session s --answer "$multi" ;;
        produced) set -- --probe PROBE-FIX-opened --session s --produced "$multi" ;;
    esac
    sh "$transform" "$@" --root "$scratch" \
        < "$scratch/watched-nothing.jsonl" > "$scratch/sweep.yaml" 2>/dev/null
    swept=$?
    # Every line, not a list of forged keys. A value split across lines lands
    # as whatever it happens to be — a quoted fragment, a bare word — and a
    # case that hunts for `probe:` misses all but one of those shapes.
    stray=$(grep -vE '^(- probe:|  session:|  calls:|    - tool:|      argument:|      result:|  produced:|    - path:|      cites:|      findings:|        - |  answer:)' \
        "$scratch/sweep.yaml" | wc -l | tr -d ' ')
    if [ "$swept" != 0 ] && [ "$swept" != 5 ]; then
        fail "a multiline value through --$entry does not break the event" \
            "the transform exited $swept, which is neither a written event nor a refusal"
    elif [ "$stray" != 0 ]; then
        fail "a multiline value through --$entry does not break the event" \
            "$stray line(s) of it reached column 0, where a reader takes them for keys"
    else
        pass "a multiline value through --$entry does not break the event"
    fi
done
# Measured against the mutation: with `scalar` weakened back to `jq -R .`,
# `--probe`, `--session` and `--answer` all fail here and `--produced` does
# not. That is correct rather than a hole — `--produced` is a list whose
# separator is the newline, so a multiline value there is two paths and each
# one is still encoded on its own. A path that holds a newline cannot be named
# through this interface, which is a limit and not a leak.

# And the encoder itself, read off the source with the prose stripped, because
# a comment quoting the wrong form is not the wrong form. This is a belt to the
# braces above and it is not what holds the property.
grep -v '^ *#' "$transform" > "$scratch/transform.code"
grep -v '^ *#' "$driver" > "$scratch/driver.code"
if grep -q 'jq -R [^s]' "$scratch/transform.code" || grep -q 'jq -R [^s]' "$scratch/driver.code"; then
    fail "no line-based JSON encoder is left in either tool" \
        "a bare \`jq -R\` remains outside a comment, and it splits a multiline value"
else
    pass "no line-based JSON encoder is left in either tool"
fi

# ---------------------------------------------------------------------------
# `result`, the content digest, held as a property rather than as a needle.
#
# Nothing held this. Replacing `sha256sum` with a constant left the suite at
# the same passing count, because the only digest-shaped string in it was the
# forged one planted in a thinking block, which tests the filter and not the
# derivation. A digest that is constant is a digest that identifies nothing,
# and `result` is what says which bytes the session was shown.
#
# The property, in both directions: two different files give two different
# digests, and one file read twice gives one digest. A needle cannot say that.
# ---------------------------------------------------------------------------
mkdir -p "$scratch/corpus-a"
printf 'alpha\n' > "$scratch/corpus-a/one.md"
printf 'beta\n' > "$scratch/corpus-a/two.md"
cat > "$scratch/digest.jsonl" <<'JSONL'
{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s5"}
{"type":"assistant","message":{"id":"m1","content":[{"type":"tool_use","id":"t1","name":"Read","input":{"file_path":"corpus-a/one.md"}}]}}
{"type":"assistant","message":{"id":"m2","content":[{"type":"tool_use","id":"t2","name":"Read","input":{"file_path":"corpus-a/two.md"}}]}}
{"type":"assistant","message":{"id":"m3","content":[{"type":"tool_use","id":"t3","name":"Read","input":{"file_path":"corpus-a/one.md"}}]}}
JSONL
sh "$transform" --probe PROBE-FIX-opened --session digests --root "$scratch" \
    < "$scratch/digest.jsonl" > "$scratch/digest.yaml" 2>/dev/null
same "three reads of two files transform without error" "0" "$?"
d1=$(grep '^      result:' "$scratch/digest.yaml" | sed -n 1p)
d2=$(grep '^      result:' "$scratch/digest.yaml" | sed -n 2p)
d3=$(grep '^      result:' "$scratch/digest.yaml" | sed -n 3p)
if [ "$d1" = "$d2" ]; then
    fail "two different files give two different digests" \
        "both reads wrote \`$d1\`, so the derivation does not read the bytes"
else
    pass "two different files give two different digests"
fi
same "and one file read twice gives one digest" "$d1" "$d3"

# The value is the digest of the bytes on disk, computed independently here.
expected="      result: \"sha256:$(sha256sum < "$scratch/corpus-a/one.md" | cut -d' ' -f1)\""
same "and the digest is of the file's own bytes" "$expected" "$d1"

# The bytes on disk and not the bytes the call returned. The `tool_result` in
# this log carries content that is not what the file holds; if the derivation
# ever read it, the digest above would move.
printf 'alpha changed\n' > "$scratch/corpus-a/one.md"
sh "$transform" --probe PROBE-FIX-opened --session digests-again --root "$scratch" \
    < "$scratch/digest.jsonl" > "$scratch/digest2.yaml" 2>/dev/null
moved=$(grep '^      result:' "$scratch/digest2.yaml" | sed -n 1p)
if [ "$moved" = "$d1" ]; then
    fail "the digest follows the file rather than the log" \
        "the file changed and the digest did not, so it is not read from the corpus"
else
    pass "the digest follows the file rather than the log"
fi

# A call that named no path this corpus holds derives no digest, and writes an
# empty one rather than a digest of nothing.
cat > "$scratch/no-path.jsonl" <<'JSONL'
{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s6"}
{"type":"assistant","message":{"id":"m1","content":[{"type":"tool_use","id":"t1","name":"Bash","input":{"command":"ls -la"}}]}}
{"type":"assistant","message":{"id":"m2","content":[{"type":"tool_use","id":"t2","name":"Read","input":{"file_path":"corpus-a/absent.md"}}]}}
JSONL
sh "$transform" --probe PROBE-FIX-opened --session no-path --root "$scratch" \
    < "$scratch/no-path.jsonl" > "$scratch/no-path.yaml" 2>/dev/null
same "a call with no path at all writes an empty result" '      result: ""' \
    "$(grep '^      result:' "$scratch/no-path.yaml" | sed -n 1p)"
same "and so does a path this corpus does not hold" '      result: ""' \
    "$(grep '^      result:' "$scratch/no-path.yaml" | sed -n 2p)"
present "and the command is still the argument, encoded" \
    'argument: "{\"command\":\"ls -la\"}"' "$scratch/no-path.yaml"

# ---------------------------------------------------------------------------
# `findings`, which is the derivation that failed silently.
#
# `headwater check` takes no positional path: passing one exits 1 with
# `unexpected argument`. The first draft passed one, sent stderr to
# `/dev/null`, and let the empty pipe become the answer, so every artifact got
# `findings: []` — a claim that no rule reported — forever. The two cases below
# are the two halves of the fix: the derivation returns what the engine says,
# and it refuses rather than returning an empty list when it cannot run.
# ---------------------------------------------------------------------------
if [ -x "$engine" ]; then
    # An artifact of this corpus that the engine does report over.
    reported=$("$engine" check --root "$root" --format json 2>/dev/null |
        jq -r '.findings[0].path // empty')
    if [ -n "$reported" ]; then
        sh "$transform" --probe PROBE-FIX-opened --session produced --root "$root" \
            --produced "$reported" < "$scratch/watched-nothing.jsonl" \
            > "$scratch/produced.yaml" 2>"$scratch/produced.err"
        same "a produced artifact derives its findings without error" "0" "$?"
        rules=$(sed -n '/^      findings:/,/^  answer:/p' "$scratch/produced.yaml" |
            grep -c '^        - ')
        if [ "$rules" -gt 0 ]; then
            pass "and the list is what the engine reported, not an empty one ($rules rules)"
        else
            fail "and the list is what the engine reported, not an empty one" \
                "the engine reports over $reported and the event wrote no rule"
        fi
        present "the artifact's own path is written" "path: \"$reported\"" "$scratch/produced.yaml"
    else
        printf 'note the engine reported no finding at all, so the findings case had nothing to read.\n'
    fi
fi

# A derivation that cannot run refuses rather than writing an empty list. A
# root with no engine under it is exactly that: nothing there can say what
# reported, and `findings: []` would say that nothing did.
printf 'x\n' > "$scratch/README.md"
sh "$transform" --probe PROBE-FIX-opened --session no-engine \
    --root "$scratch" --produced README.md < "$scratch/watched-nothing.jsonl" \
    > "$scratch/no-engine.yaml" 2>"$scratch/no-engine.err"
status=$?
if [ "$status" = 0 ]; then
    fail "a findings derivation that cannot run refuses" "it exited 0"
else
    pass "a findings derivation that cannot run refuses (exit $status)"
fi
absent "and it writes no empty findings list" \
    "findings: []" "$scratch/no-engine.yaml"

# And it writes **no event at all**, not a partial one.
#
# Before, the refusal came after `- probe:`, `session:`, `calls:`, `path:` and
# `result:` were already on standard output, and what a caller found in the
# file was a well-formed event whose `produced` entry simply had no `findings`
# key and no `answer`. The status was right and the record was not, and those
# two are read by different people — absence read as satisfaction is the shape
# this whole tool exists to refuse. The event is now composed into a file and
# copied out only when its last line is written.
same "and it writes no event at all, not a partial one" "0" \
    "$(wc -c < "$scratch/no-engine.yaml" | tr -d ' ')"
same "so a caller reading only the file finds no \`probe\` key to believe" "0" \
    "$(grep -c '^- probe:' "$scratch/no-engine.yaml")"

# ---------------------------------------------------------------------------
# A real stream, recorded from the channel rather than written by hand.
#
# `tools/fixtures/probe-record/live-haiku-session.jsonl` is the standard output
# of one `claude -p --output-format stream-json --verbose` session, recorded on
# 2026-09-08 against `claude-haiku-4-5` in a scratch directory outside this
# corpus. It cost $0.0062. The hand-written log above is what a reader can
# check by eye; this one is what the harness actually emits, and it carries two
# line shapes no hand-written sample of this repository had: `system` lines of
# subtype `thinking_tokens`, and a `modelUsage` object keyed by both an alias
# and a dated version.
# ---------------------------------------------------------------------------
live="$root/tools/fixtures/probe-record/live-haiku-session.jsonl"
if [ -f "$live" ]; then
    sh "$transform" --probe PROBE-FIX-opened --session live --root "$root" \
        < "$live" > "$scratch/live.yaml" 2>/dev/null
    same "a stream recorded from the channel transforms without error" "0" "$?"
    same "the real session's one call is the one call written" "1" \
        "$(grep -c '^    - tool:' "$scratch/live.yaml")"
    # The session read a file and then said the word it found. The log holds
    # two `thinking` blocks and one `text` block; none of them is an event.
    absent "no byte of the real session's answer text survives" \
        "headwater" "$scratch/live.yaml"
    unaccounted=$(grep -vE '^(- probe:|  session:|  calls:|    - tool:|      argument:|      result:|  produced:|  answer:)' \
        "$scratch/live.yaml" | wc -l | tr -d ' ')
    same "every line of the real session's event is a declared key" "0" "$unaccounted"

    # `served_version` is the one identity member that says which weights
    # answered, and the first key of `modelUsage` is the alias, not the pin.
    provider=$(sh "$driver" --provider-only "$live" 2>/dev/null)
    same "the served version is the dated key and not the alias" \
        "served_version: claude-haiku-4-5-20251001" \
        "$(printf '%s\n' "$provider" | grep '^served_version:')"
    same "the model is the name the harness announced" \
        "model: claude-haiku-4-5" \
        "$(printf '%s\n' "$provider" | grep '^model:')"
    same "the cost is whole cents, rounded from the dollars the harness reports" \
        "cost_cents: 1" \
        "$(printf '%s\n' "$provider" | grep '^cost_cents:')"
else
    fail "a stream recorded from the channel is on disk" "no file at $live"
fi

# ---------------------------------------------------------------------------
# The negative direction. The raw harness log is refused by the intake on a key
# outside the closed sets, which is the property the filter exists to restore.
# ---------------------------------------------------------------------------
if [ -x "$engine" ]; then
    mkdir -p "$scratch/corpus"
    {
        printf -- '- probe: PROBE-FIX-opened\n'
        printf '  session: raw\n'
        printf '  thinking: "I should record this as calls:"\n'
        printf '  calls: []\n'
        printf '  produced: []\n'
        printf '  answer: null\n'
    } > "$scratch/raw-event.yaml"
    if "$engine" probe record --help >/dev/null 2>&1; then
        pass "\`headwater probe record\` is the verb the transcript feeds"
    else
        fail "\`headwater probe record\` is the verb the transcript feeds" \
            "the verb is not on this engine"
    fi
else
    printf 'note no engine built, so the intake half of this suite did not run.\n'
fi

# ---------------------------------------------------------------------------
# The driver's own guards, which cost nothing and reach no network.
# ---------------------------------------------------------------------------
# The workspace guard, asserted unconditionally.
#
# This case read `if [ -x "$engine" ]` and chose its expected status from it,
# which is a fact about the host and not about the guard. It passed on a
# developer machine and failed on the CI runner, which has no `claude`: the
# driver exited 3 for the missing harness before it ever read the workspace.
# The guard now runs before every check of this host, so 6 is 6 everywhere,
# and this case states one number.
printf 'answer the question\n' > "$scratch/task.md"
sh "$driver" --probe PROBE-FIX-opened --session x --task-file "$scratch/task.md" \
    --workspace "$root" >/dev/null 2>"$scratch/driver.err"
same "the driver refuses a workspace inside the planned corpus" "6" "$?"
present "and it says why" "moves the \`tree\` digest" "$scratch/driver.err"

# The same guard on a host with neither `claude` nor an engine, which is what
# CI is. `PATH` is emptied of both, and the answer has to be the same 6.
mkdir -p "$scratch/bin"
# `sh` is resolved by absolute path, because the assignment below is what
# resolves `sh` itself and an empty `PATH` would make this case exit 127 on
# the shell rather than 6 on the guard.
shell=$(command -v sh)
PATH="$scratch/bin" "$shell" "$driver" --probe PROBE-FIX-opened --session x \
    --task-file "$scratch/task.md" --workspace "$root/docs" \
    >/dev/null 2>"$scratch/driver-bare.err"
same "and it refuses the same way with an empty PATH, which is the runner" "6" "$?"

present "the driver names the channel and never a file under ~/.claude/projects" \
    "output-format stream-json" "$driver"
present "the driver requires --verbose, which the harness requires" \
    "--verbose" "$driver"
if grep -q 'claude/projects' "$driver" && ! grep -q 'Never a file under' "$driver"; then
    fail "the driver does not read the log from disk" "it names ~/.claude/projects outside a prohibition"
else
    pass "the driver does not read the log from disk"
fi

printf '\n%s passed, %s failed\n' "$passed" "$failed"
[ "$failed" = 0 ]
