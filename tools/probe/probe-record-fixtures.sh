#!/bin/sh
# What holds `tools/probe/probe-transform.sh`, `tools/probe/probe-record.sh` and
# `tools/probe/ablate.sh`.
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
#     sh tools/probe/probe-record-fixtures.sh
#
# It needs `jq`, exits 3 without it so CI can skip with a warning, and writes
# only under a temporary directory.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
transform="$root/tools/probe/probe-transform.sh"
driver="$root/tools/probe/probe-record.sh"
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
# `produced` from the log's own write-shaped calls, with no `--produced` (#911).
#
# One live session of 2026-09-17 edited two files, both edits named the ruling
# the probe examined, and the event said `produced: []`. The `Edit` calls were
# in `calls` with their paths, and nothing read those paths back into
# `produced`, because only `--produced` fed it and the driver passed none.
#
# The log below holds one call of each write-shaped tool, and a `Read` and a
# `Bash` redirect beside them. The `Read` names a path and wrote nothing. The
# `Bash` call wrote a file and names no path, which is the gap this does not
# close. The files sit outside the base, so this half needs no engine: an
# artifact `headwater check` never ran over carries no `findings` key.
# ---------------------------------------------------------------------------
mkdir -p "$scratch/written" "$scratch/empty-base"
printf 'The rule is [HW-DR-0049].\n' > "$scratch/written/edited.rs"
printf 'Rests on HW-OBL-0142 and HW-DR-0049.\n' > "$scratch/written/new.md"
printf 'no identifier\n' > "$scratch/written/multi.txt"
printf '{}\n' > "$scratch/written/nb.ipynb"
printf 'HW-DR-0001\n' > "$scratch/written/read.md"
printf 'HW-DR-0002\n' > "$scratch/written/bash.md"
w=$scratch/written
{
    printf '%s\n' '{"type":"system","subtype":"init","model":"claude-sonnet-5","session_id":"s8"}'
    jq -nc --arg p "$w/read.md" '{type:"assistant",message:{content:[{type:"tool_use",id:"t1",name:"Read",input:{file_path:$p}}]}}'
    jq -nc --arg p "$w/edited.rs" '{type:"assistant",message:{content:[{type:"tool_use",id:"t2",name:"Edit",input:{file_path:$p,old_string:"a",new_string:"b"}}]}}'
    jq -nc --arg p "$w/new.md" '{type:"assistant",message:{content:[{type:"tool_use",id:"t3",name:"Write",input:{file_path:$p,content:"x"}}]}}'
    jq -nc --arg p "$w/multi.txt" '{type:"assistant",message:{content:[{type:"tool_use",id:"t4",name:"MultiEdit",input:{file_path:$p,edits:[]}}]}}'
    jq -nc --arg p "$w/nb.ipynb" '{type:"assistant",message:{content:[{type:"tool_use",id:"t5",name:"NotebookEdit",input:{notebook_path:$p,new_source:"x"}}]}}'
    jq -nc --arg p "$w/edited.rs" '{type:"assistant",message:{content:[{type:"tool_use",id:"t6",name:"Edit",input:{file_path:$p,old_string:"b",new_string:"c"}}]}}'
    jq -nc --arg c "cat > $w/bash.md <<EOF" '{type:"assistant",message:{content:[{type:"tool_use",id:"t7",name:"Bash",input:{command:$c}}]}}'
} > "$scratch/writes.jsonl"
sh "$transform" --probe PROBE-FIX-cited --session writes --root "$scratch/empty-base" \
    < "$scratch/writes.jsonl" > "$scratch/writes.yaml" 2>"$scratch/writes.err"
same "a log of write-shaped calls with no --produced transforms without error" "0" "$?"
same "every write-shaped tool seeds one entry, and a file edited twice is one entry" "4" \
    "$(grep -c '^    - path:' "$scratch/writes.yaml")"
present "an \`Edit\` path is in \`produced\` with no --produced naming it" \
    "    - path: \"$w/edited.rs\"" "$scratch/writes.yaml"
present "and so is a \`Write\` path" "    - path: \"$w/new.md\"" "$scratch/writes.yaml"
present "and a \`MultiEdit\` path" "    - path: \"$w/multi.txt\"" "$scratch/writes.yaml"
present "and a \`NotebookEdit\` path, which is named by \`notebook_path\`" \
    "    - path: \"$w/nb.ipynb\"" "$scratch/writes.yaml"
absent "a \`Read\` path names a file and is not a produced artifact" \
    "    - path: \"$w/read.md\"" "$scratch/writes.yaml"
absent "a write through \`Bash\` names no path, so it is not seen and still needs --produced" \
    "    - path: \"$w/bash.md\"" "$scratch/writes.yaml"
# `cites` is read off the file the `Edit` named. The two identifiers of
# `new.md` sit in the next entry, so the slice is bounded by it.
same "the edited file's \`cites\` is derived from its bytes" '        - "HW-DR-0049"' \
    "$(sed -n "\|path: \"$w/edited.rs\"|,\|path: \"$w/new.md\"|p" "$scratch/writes.yaml" | grep '^        - ')"
same "the edited file's \`result\` is the digest of its bytes" \
    "      result: \"sha256:$(sha256sum < "$w/edited.rs" | cut -d' ' -f1)\"" \
    "$(grep -A1 "path: \"$w/edited.rs\"" "$scratch/writes.yaml" | grep '^      result:')"
same "an artifact outside the base that no check ran over writes no \`findings\` key" "0" \
    "$(grep -c '^      findings:' "$scratch/writes.yaml")"

# The same log with the Bash-written file named explicitly: the two sources
# add up, and the explicit one is not lost.
sh "$transform" --probe PROBE-FIX-cited --session writes-and-named --root "$scratch/empty-base" \
    --produced "$w/bash.md" --produced "$w/edited.rs" \
    < "$scratch/writes.jsonl" > "$scratch/writes-named.yaml" 2>/dev/null
same "--produced adds to the seeded paths, and a path named both ways is one entry" "5" \
    "$(grep -c '^    - path:' "$scratch/writes-named.yaml")"

# Inside the base, the `findings` half. The workspace is this corpus and the path is
# absolute, as a live `Edit` names it. It has to come out relative to the base,
# because `headwater check` reports under that form, and an absolute path there
# would select no finding and write `findings: []` for a file with findings.
if [ -x "$engine" ] && [ -n "${reported:-}" ]; then
    jq -nc --arg p "$root/$reported" '{type:"assistant",message:{content:[{type:"tool_use",id:"t1",name:"Edit",input:{file_path:$p,old_string:"a",new_string:"b"}}]}}' \
        > "$scratch/edit-in-base.body"
    cat "$scratch/watched-nothing.jsonl" "$scratch/edit-in-base.body" > "$scratch/edit-in-base.jsonl"
    # The root holds the engine and nothing else, so a derivation that read the
    # root rather than the workspace finds no corpus to report over.
    mkdir -p "$scratch/engine-only/engine/target/dev-release"
    ln -s "$engine" "$scratch/engine-only/engine/target/dev-release/headwater"
    sh "$transform" --probe PROBE-FIX-cited --session edit-in-base --root "$scratch/engine-only" \
        --workspace "$root" < "$scratch/edit-in-base.jsonl" \
        > "$scratch/edit-in-base.yaml" 2>"$scratch/edit-in-base.err"
    same "an \`Edit\` inside the workspace derives its findings without error" "0" "$?"
    present "and its path is written relative to the workspace" \
        "    - path: \"$reported\"" "$scratch/edit-in-base.yaml"
    rules=$(sed -n '/^      findings:/,/^  answer:/p' "$scratch/edit-in-base.yaml" |
        grep -c '^        - ')
    if [ "$rules" -gt 0 ]; then
        pass "and its findings are the ones the engine reported over the workspace ($rules rules)"
    else
        fail "and its findings are the ones the engine reported over the workspace" \
            "the engine reports over $reported and the event wrote no rule"
    fi
fi

# ---------------------------------------------------------------------------
# A real stream, recorded from the channel rather than written by hand.
#
# `tools/probe/fixtures/live-haiku-session.jsonl` is the standard output
# of one `claude -p --output-format stream-json --verbose` session, recorded on
# 2026-09-08 against `claude-haiku-4-5` in a scratch directory outside this
# corpus. It cost $0.0062. The hand-written log above is what a reader can
# check by eye; this one is what the harness actually emits, and it carries two
# line shapes no hand-written sample of this repository had: `system` lines of
# subtype `thinking_tokens`, and a `modelUsage` object keyed by both an alias
# and a dated version.
# ---------------------------------------------------------------------------
live="$root/tools/probe/fixtures/live-haiku-session.jsonl"
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

    # ---------------------------------------------------------------------
    # A second model in `modelUsage` that is not the one driven.
    #
    # Measured on 2026-09-17, driving `claude-sonnet-5`: the harness makes a
    # small internal call on `claude-haiku-4-5-20251001` regardless of the
    # driven model, and that entry's key is the only one in `modelUsage` with a
    # dated suffix. `claude-sonnet-5` carries no dated pin of its own here. The
    # first version of this derivation took the first dated key of the whole
    # object and so wrote Haiku's pin as the served version of a session that
    # spent 94% of its cost on Sonnet. This fixture is that shape, minimized:
    # two `canonicalModel` values in `modelUsage`, only one of them dated, and
    # it is not the driven model's own.
    # ---------------------------------------------------------------------
    cat > "$scratch/multi-model.jsonl" <<'JSONL'
{"type":"system","subtype":"init","model":"claude-sonnet-5","session_id":"s7"}
{"type":"result","session_id":"s7","total_cost_usd":0.58,"modelUsage":{"claude-haiku-4-5-20251001":{"canonicalModel":"claude-haiku-4-5","inputTokens":989},"claude-sonnet-5":{"canonicalModel":"claude-sonnet-5","inputTokens":38}}}
JSONL
    multi=$(sh "$driver" --provider-only "$scratch/multi-model.jsonl" 2>/dev/null)
    same "the served version belongs to the driven model, not to another key that happens to be dated" \
        "served_version: claude-sonnet-5" \
        "$(printf '%s\n' "$multi" | grep '^served_version:')"
    same "the model is still the name the harness announced" \
        "model: claude-sonnet-5" \
        "$(printf '%s\n' "$multi" | grep '^model:')"

    # `answer` is the key that says what a closed-set session concluded, and
    # this script wrote `null` for every session it drove until #803, because
    # the driver called the transform with no `--answer`. The real session
    # below ends with one word, so it is the log that holds the derivation to
    # its rule in all three directions at once.
    #
    # The word this log ends with is `headwater`. It is not an answer any probe
    # of this repository declares, which is what makes the third case here a
    # real one rather than a constructed one.
    said=$(jq -s -r '([.[] | select(.type == "result")] | last | .result // "")' < "$live")
    same "the real session ends with one word and not with prose" \
        "headwater" "$(printf '%s' "$said" | tr -d '[:space:]')"
    same "a probe that declares the word gets the word" \
        "headwater" \
        "$(sh "$driver" --answer-only "$live" --answers "headwater, other" 2>/dev/null)"
    same "the comparison folds case, because a harness may capitalize a sentence" \
        "headwater" \
        "$(sh "$driver" --answer-only "$live" --answers "Headwater" 2>/dev/null | tr '[:upper:]' '[:lower:]')"
    same "a probe that declares no such answer gets nothing, and never a substring" \
        "" \
        "$(sh "$driver" --answer-only "$live" --answers "present, absent, withheld" 2>/dev/null)"
    same "a probe that declares no answer set at all gets nothing" \
        "" \
        "$(sh "$driver" --answer-only "$live" 2>/dev/null)"
else
    fail "a stream recorded from the channel is on disk" "no file at $live"
fi

# The final line is the answer (#980). The pilot of 2026-09-28 found sessions
# that wrote one sentence of reasoning and then the word on its own line, and
# the whole-message rule recorded each as no answer. A word inside a sentence
# is still no answer, and so is a word in Markdown emphasis.
printf '%s\n' '{"type":"result","result":"Found it: HW-DR-0034 rules on this.\n\nmerge\n"}' \
    > "$scratch/last-line.jsonl"
same "a word alone on the final line after a sentence is the answer" \
    "merge" \
    "$(sh "$driver" --answer-only "$scratch/last-line.jsonl" --answers "merge, stamp" 2>/dev/null)"
printf '%s\n' '{"type":"result","result":"The answer is merge, because HW-DR-0034 says so."}' \
    > "$scratch/in-sentence.jsonl"
same "a word inside a sentence on the final line is no answer" \
    "" \
    "$(sh "$driver" --answer-only "$scratch/in-sentence.jsonl" --answers "merge, stamp" 2>/dev/null)"
printf '%s\n' '{"type":"result","result":"Reasoning first.\n**merge**"}' \
    > "$scratch/emphasis.jsonl"
same "a word in Markdown emphasis on the final line is no answer" \
    "" \
    "$(sh "$driver" --answer-only "$scratch/emphasis.jsonl" --answers "merge, stamp" 2>/dev/null)"

# A declared answer of two words is compared as a set of words (#1384). The
# status pilot of 2026-09-29 recorded `HW-DR-0052 current` as no answer for
# the set `current HW-DR-0052, draft HW-DR-0052`, although it names the same
# two facts. The words may come in any order, and the recorder writes the
# declared form. An extra word is still prose, and so is a missing one.
two="current HW-DR-0052, draft HW-DR-0052"
printf '%s\n' '{"type":"result","result":"I read the record.\n\nHW-DR-0052 current"}' \
    > "$scratch/two-swapped.jsonl"
same "a two-word answer in the other order records the declared form" \
    "current HW-DR-0052" \
    "$(sh "$driver" --answer-only "$scratch/two-swapped.jsonl" --answers "$two" 2>/dev/null)"
printf '%s\n' '{"type":"result","result":"draft   HW-DR-0052"}' \
    > "$scratch/two-spaced.jsonl"
same "runs of space between the words of an answer are one separator" \
    "draft HW-DR-0052" \
    "$(sh "$driver" --answer-only "$scratch/two-spaced.jsonl" --answers "$two" 2>/dev/null)"
printf '%s\n' '{"type":"result","result":"0052 current HW-DR-0052"}' \
    > "$scratch/two-extra.jsonl"
same "a two-word answer with an extra word is no answer" \
    "" \
    "$(sh "$driver" --answer-only "$scratch/two-extra.jsonl" --answers "$two" 2>/dev/null)"
printf '%s\n' '{"type":"result","result":"current"}' \
    > "$scratch/two-short.jsonl"
same "one word of a two-word answer is no answer" \
    "" \
    "$(sh "$driver" --answer-only "$scratch/two-short.jsonl" --answers "$two" 2>/dev/null)"
printf '%s\n' '{"type":"result","result":"current current"}' \
    > "$scratch/two-repeated.jsonl"
same "a repeated word does not stand in for the other word of an answer" \
    "" \
    "$(sh "$driver" --answer-only "$scratch/two-repeated.jsonl" --answers "$two" 2>/dev/null)"

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

# The `.git` guard, asserted unconditionally, over both shapes a `cp -a` can
# leave behind: a worktree's `.git` file (a `gitdir:` pointer) and a full
# clone's `.git` directory.
mkdir -p "$scratch/copied-worktree"
printf 'gitdir: %s/.git/worktrees/probe-fixture\n' "$root" > "$scratch/copied-worktree/.git"
sh "$driver" --probe PROBE-FIX-opened --session x --task-file "$scratch/task.md" \
    --workspace "$scratch/copied-worktree" >/dev/null 2>"$scratch/driver-gitfile.err"
same "the driver refuses a workspace carrying a worktree's \`.git\` file" "4" "$?"
present "and it says why" "live pointer into this repository's history" "$scratch/driver-gitfile.err"

mkdir -p "$scratch/copied-clone/.git"
sh "$driver" --probe PROBE-FIX-opened --session x --task-file "$scratch/task.md" \
    --workspace "$scratch/copied-clone" >/dev/null 2>"$scratch/driver-gitdir.err"
same "the driver refuses a workspace carrying a clone's \`.git\` directory" "4" "$?"

# The answer-key guard (#1229), asserted unconditionally and before any harness
# call. A document under `docs/` that names the probe states its expected
# value, a recorded answer or its target, and the 2026-09-17 tombstone session
# read its own probe file and then answered. The copy sits in `docs/notes/`,
# outside the probe shelves, so the instrument guard (8) passes it and this
# guard (9) is the one that answers. The clean half runs with a `PATH` that
# holds `grep`, `sh` and `awk` and not `jq` or `claude`, so it stops at the tool
# check (3) and can never start a paid session on a host that has the harness
# installed.
mkdir -p "$scratch/keyed/docs/notes" "$scratch/grep-only"
cp "$root/engine/crates/probe/fixtures/corpus/probes/0001-opened.md" "$scratch/keyed/docs/notes/"
sh "$driver" --probe PROBE-FIX-opened --session x --task-file "$scratch/task.md" \
    --workspace "$scratch/keyed" >/dev/null 2>"$scratch/driver-key.err"
same "the driver refuses a workspace whose docs/ holds the probe's own document" "9" "$?"
present "and it names the file" "keyed/docs/notes/0001-opened.md" "$scratch/driver-key.err"

for tool in grep sh awk; do
    ln -s "$(command -v "$tool")" "$scratch/grep-only/$tool"
done
rm "$scratch/keyed/docs/notes/0001-opened.md"
PATH="$scratch/grep-only" "$shell" "$driver" --probe PROBE-FIX-opened --session x \
    --task-file "$scratch/task.md" --workspace "$scratch/keyed" \
    >/dev/null 2>"$scratch/driver-sealed.err"
same "and it passes the guard once the file is gone" "3" "$?"

# When both guards apply, the instrument guard answers. A probe document left
# on its shelf is in the instrument and names the probe, and 8 is the code
# `ablate.sh --present` is the remedy for.
mkdir -p "$scratch/both/docs/probes"
cp "$root/engine/crates/probe/fixtures/corpus/probes/0001-opened.md" "$scratch/both/docs/probes/"
sh "$driver" --probe PROBE-FIX-opened --session x --task-file "$scratch/task.md" \
    --workspace "$scratch/both" >/dev/null 2>"$scratch/driver-both.err"
same "a workspace that trips both guards gets the instrument's 8, not the answer key's 9" "8" "$?"
present "and the message is the instrument's" "which every arm removes" "$scratch/driver-both.err"

# A record under `docs/` names a probe by its slug, the file name on the shelf,
# and never by its identifier. The guard reads the slug off this checkout's
# shelf. A file outside `docs/` that names the slug, a derived fold or a
# hand-written overlay, states no answer, and the guard passes it.
tombstone=a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer
mkdir -p "$scratch/slugged/docs/obligations"
printf 'see docs/probes/%s.md\n' "$tombstone" > "$scratch/slugged/docs/obligations/0013.md"
PATH="$scratch/grep-only" "$shell" "$driver" --probe "HW-PROBE-$tombstone" --session x \
    --task-file "$scratch/task.md" --workspace "$scratch/slugged" >/dev/null 2>"$scratch/driver-slug.err"
same "the driver refuses a workspace whose docs/ names the probe by its slug" "9" "$?"

# An evaluation is where a recorded answer is read and argued, and the three on
# this shelf that name a probe each state an answer or a target. A guard that
# skipped `docs/evaluations/` would let one through with no error. The case
# runs on the `PATH` with no `jq` or `claude`, so such a guard stops at 3 and
# never reaches a harness.
mkdir -p "$scratch/evaluated/docs/evaluations"
printf 'Both sessions closed on `absent`, see docs/probes/%s.md\n' "$tombstone" \
    > "$scratch/evaluated/docs/evaluations/the-tombstone-readings.md"
PATH="$scratch/grep-only" "$shell" "$driver" --probe "HW-PROBE-$tombstone" --session x \
    --task-file "$scratch/task.md" --workspace "$scratch/evaluated" >/dev/null 2>"$scratch/driver-evaluated.err"
same "the driver refuses a workspace whose docs/evaluations/ states the probe's answer" "9" "$?"
present "and it names the evaluation" "docs/evaluations/the-tombstone-readings.md" "$scratch/driver-evaluated.err"
mkdir -p "$scratch/folded/.headwater"
printf -- '- docs/probes/%s.md\n' "$tombstone" > "$scratch/folded/.headwater/nav.yml"
printf '# HW-PROBE-%s\n' "$tombstone" > "$scratch/folded/.headwater/overlay.yml"
PATH="$scratch/grep-only" "$shell" "$driver" --probe "HW-PROBE-$tombstone" --session x \
    --task-file "$scratch/task.md" --workspace "$scratch/folded" \
    >/dev/null 2>"$scratch/driver-folded.err"
same "and it passes a fold and an overlay outside docs/ that name the probe" "3" "$?"

# `seal.sh` removes the instrument and each record under `docs/` that names the
# probe, keeps everything else, and the driver then passes both guards.
mkdir -p "$scratch/sealed/docs/probes" "$scratch/sealed/docs/probe-runs" "$scratch/sealed/docs/spec" "$scratch/sealed/docs/obligations" "$scratch/sealed/.headwater"
cp "$root/docs/probes/$tombstone.md" "$scratch/sealed/docs/probes/"
printf 'a run\n' > "$scratch/sealed/docs/probe-runs/run.md"
printf 'see docs/probes/%s.md\n' "$tombstone" > "$scratch/sealed/docs/obligations/0013.md"
mkdir -p "$scratch/sealed/docs/evaluations"
printf 'Both sessions closed on `absent`, see docs/probes/%s.md\n' "$tombstone" \
    > "$scratch/sealed/docs/evaluations/the-tombstone-readings.md"
printf -- '- docs/probes/%s.md\n' "$tombstone" > "$scratch/sealed/.headwater/nav.yml"
printf '# HW-PROBE-%s\n' "$tombstone" > "$scratch/sealed/.headwater/overlay.yml"
printf '{}\n' > "$scratch/sealed/.headwater/export.json"
printf 'a spec that names no probe\n' > "$scratch/sealed/docs/spec/01.md"
sh "$root/tools/probe/seal.sh" "$scratch/sealed" "HW-PROBE-$tombstone" >/dev/null 2>"$scratch/seal.err"
same "seal.sh seals a workspace" "0" "$?"
if [ -e "$scratch/sealed/docs/probes" ] || [ -e "$scratch/sealed/docs/probe-runs" ] || [ -e "$scratch/sealed/.headwater/export.json" ] || [ -e "$scratch/sealed/docs/obligations/0013.md" ] || [ -e "$scratch/sealed/docs/evaluations/the-tombstone-readings.md" ]; then
    fail "and it removes the instrument and each record under docs/ that names the probe" "$(ls -aR "$scratch/sealed")"
else
    pass "and it removes the instrument and each record under docs/ that names the probe"
fi
if [ -f "$scratch/sealed/docs/spec/01.md" ] && [ -f "$scratch/sealed/.headwater/nav.yml" ] && [ -f "$scratch/sealed/.headwater/overlay.yml" ]; then
    pass "and it keeps a document that names no probe, the derived fold and the hand-written overlay"
else
    fail "and it keeps a document that names no probe, the derived fold and the hand-written overlay" "$(ls -aR "$scratch/sealed")"
fi
sh "$root/tools/probe/seal.sh" "$scratch/sealed" "HW-PROBE-$tombstone" >/dev/null 2>&1
same "and a second seal is a no-op" "0" "$?"
PATH="$scratch/grep-only" "$shell" "$driver" --probe "HW-PROBE-$tombstone" --session x \
    --task-file "$scratch/task.md" --workspace "$scratch/sealed" \
    >/dev/null 2>"$scratch/driver-sealed2.err"
same "and the driver passes both guards over the sealed tree" "3" "$?"
sh "$root/tools/probe/seal.sh" "$root/docs" "HW-PROBE-$tombstone" >/dev/null 2>&1
same "seal.sh refuses a path inside this checkout" "6" "$?"

# A named document is sealed the way an answer key is (#1293). The #980 batch
# deleted HW-OBL-0013 because it names the tombstone probe, and left its claim
# file, its register line and its shelf-index line, so 23 of 30 present-arm
# sessions still found and queried it. The document goes, and so do its claim
# and every line that names it; the files that held those lines stay.
mkdir -p "$scratch/linked/docs/probes" "$scratch/linked/docs/obligations" "$scratch/linked/docs/spec" \
    "$scratch/linked/.headwater/ids/obligation_record_id"
cp "$root/docs/probes/$tombstone.md" "$scratch/linked/docs/probes/"
printf -- '---\nid: HW-OBL-0013\n---\nNo probe tests it; see docs/probes/%s.md\n' "$tombstone" \
    > "$scratch/linked/docs/obligations/0013-x.md"
printf '%s\n' '- [HW-OBL-0013](../obligations/0013-x.md) — the tombstone gap' \
    '- [HW-OBL-0014](../obligations/0014-y.md) — another entry' > "$scratch/linked/docs/spec/13.md"
printf '%s\n' '| [0013-x](0013-x.md) | the tombstone gap |' '| [0014-y](0014-y.md) | another entry |' \
    > "$scratch/linked/docs/obligations/README.md"
printf 'docs/obligations/0013-x.md\n' > "$scratch/linked/.headwater/ids/obligation_record_id/HW-OBL-0013"
printf -- '- docs/probes/%s.md\n' "$tombstone" > "$scratch/linked/.headwater/nav.yml"
printf '# HW-PROBE-%s\n' "$tombstone" > "$scratch/linked/.headwater/overlay.yml"
sh "$root/tools/probe/seal.sh" "$scratch/linked" "HW-PROBE-$tombstone" >"$scratch/linked.out" 2>&1
same "seal.sh seals a workspace whose index and register link a named document" "0" "$?"
if [ -e "$scratch/linked/docs/obligations/0013-x.md" ] || [ -e "$scratch/linked/.headwater/ids/obligation_record_id/HW-OBL-0013" ]; then
    fail "and it removes the named document and its identifier claim" "$(ls -aR "$scratch/linked")"
else
    pass "and it removes the named document and its identifier claim"
fi
same "and it removes the register line that names the document and keeps the line that does not" \
    "- [HW-OBL-0014](../obligations/0014-y.md) — another entry" "$(cat "$scratch/linked/docs/spec/13.md")"
same "and it removes the shelf-index line that links the document and keeps the line that does not" \
    "| [0014-y](0014-y.md) | another entry |" "$(cat "$scratch/linked/docs/obligations/README.md")"
if [ -f "$scratch/linked/.headwater/nav.yml" ] && [ -f "$scratch/linked/.headwater/overlay.yml" ]; then
    pass "and the derived fold and the hand-written overlay survive as files"
else
    fail "and the derived fold and the hand-written overlay survive as files" "$(ls -aR "$scratch/linked")"
fi
present "and it counts the lines it removed for the document" "seal: removed the named document HW-OBL-0013 of HW-PROBE-$tombstone, and 2 lines naming it" "$scratch/linked.out"

# A named document whose slug is `README` is deleted with its claim, and its
# slug drives no line removal: every shelf has a README, and a line that says
# so names no answer.
mkdir -p "$scratch/readme/docs/probes" "$scratch/readme/docs/evaluations" "$scratch/readme/docs/obligations"
cp "$root/docs/probes/$tombstone.md" "$scratch/readme/docs/probes/"
printf 'The readings of docs/probes/%s.md\n' "$tombstone" > "$scratch/readme/docs/evaluations/README.md"
printf '%s\n' 'See [the index](README.md) for each record.' 'An obligation line.' \
    > "$scratch/readme/docs/obligations/README.md"
sh "$root/tools/probe/seal.sh" "$scratch/readme" "HW-PROBE-$tombstone" >"$scratch/readme.out" 2>&1
same "seal.sh seals a workspace in which a README names the probe" "0" "$?"
if [ -e "$scratch/readme/docs/evaluations/README.md" ]; then
    fail "and it removes the README that names the probe" "$(ls -aR "$scratch/readme")"
else
    pass "and it removes the README that names the probe"
fi
same "and it keeps every line elsewhere that holds the word README" \
    "$(printf '%s\n' 'See [the index](README.md) for each record.' 'An obligation line.')" \
    "$(cat "$scratch/readme/docs/obligations/README.md")"
present "and it says that the generic slug removed no lines" "seal: kept every line naming README" "$scratch/readme.out"

# A one-word slug is generic too. A named `docs/spec/glossary.md` must not
# strip every line that holds the word, or that links the glossary, from the
# rest of the tree (verify round 1 of #1293: 163 lines in 78 files).
mkdir -p "$scratch/glossary/docs/probes" "$scratch/glossary/docs/spec" "$scratch/glossary/engine"
cp "$root/docs/probes/$tombstone.md" "$scratch/glossary/docs/probes/"
printf 'Terms. See docs/probes/%s.md\n' "$tombstone" > "$scratch/glossary/docs/spec/glossary.md"
printf '%s\n' 'See the [glossary](glossary.md).' 'let glossary = load();' > "$scratch/glossary/docs/spec/01.md"
sh "$root/tools/probe/seal.sh" "$scratch/glossary" "HW-PROBE-$tombstone" >"$scratch/glossary.out" 2>&1
same "seal.sh keeps every line that holds a one-word slug of a named document" \
    "$(printf '%s\n' 'See the [glossary](glossary.md).' 'let glossary = load();')" \
    "$(cat "$scratch/glossary/docs/spec/01.md")"

# A slug that another file of the workspace also has is generic, as every
# skill's SKILL.md is, so a link to the other file survives.
mkdir -p "$scratch/shared/docs/probes" "$scratch/shared/docs/a" "$scratch/shared/docs/b"
cp "$root/docs/probes/$tombstone.md" "$scratch/shared/docs/probes/"
printf 'see docs/probes/%s.md\n' "$tombstone" > "$scratch/shared/docs/a/shared-name.md"
printf 'the other one\n' > "$scratch/shared/docs/b/shared-name.md"
printf '%s\n' 'See [the b file](b/shared-name.md).' > "$scratch/shared/docs/index-of-both.md"
sh "$root/tools/probe/seal.sh" "$scratch/shared" "HW-PROBE-$tombstone" >"$scratch/shared.out" 2>&1
same "seal.sh keeps a line that links another file of the same name" \
    'See [the b file](b/shared-name.md).' "$(cat "$scratch/shared/docs/index-of-both.md")"

# An identifier or slug matches only as a whole name. A document whose slug or
# identifier is a prefix or a suffix of another's must not take the other's
# lines with it. 43 slug pairs of this corpus are substrings of each other.
mkdir -p "$scratch/prefix/docs/probes" "$scratch/prefix/docs/obligations" "$scratch/prefix/docs/spec"
cp "$root/docs/probes/$tombstone.md" "$scratch/prefix/docs/probes/"
printf -- '---\nid: HW-EVAL-short-name\n---\nsee docs/probes/%s.md\n' "$tombstone" \
    > "$scratch/prefix/docs/obligations/short-name.md"
printf '%s\n' '- [HW-EVAL-short-name](../obligations/short-name.md)' \
    '- [HW-EVAL-short-name-longer](../obligations/short-name-longer.md)' \
    '- [HW-EVAL-a-short-name](../obligations/a-short-name.md)' > "$scratch/prefix/docs/spec/13.md"
sh "$root/tools/probe/seal.sh" "$scratch/prefix" "HW-PROBE-$tombstone" >"$scratch/prefix.out" 2>&1
same "seal.sh removes the line of the named document and not a line of a document whose name contains it" \
    "$(printf '%s\n' '- [HW-EVAL-short-name-longer](../obligations/short-name-longer.md)' '- [HW-EVAL-a-short-name](../obligations/a-short-name.md)')" \
    "$(cat "$scratch/prefix/docs/spec/13.md")"

# A JSON fold loses the array element that names the document, and never a
# line inside it, so it still parses (verify round 1 of #1293: the seal left
# `{"shelf": "evaluations",}` in `.headwater/corpus.json`).
mkdir -p "$scratch/folds/docs/probes" "$scratch/folds/docs/obligations" "$scratch/folds/.headwater"
cp "$root/docs/probes/$tombstone.md" "$scratch/folds/docs/probes/"
printf -- '---\nid: HW-OBL-0013\n---\nsee docs/probes/%s.md\n' "$tombstone" > "$scratch/folds/docs/obligations/0013-x.md"
printf '%s\n' '{' '  "shelves": [' '    {' '      "shelf": "obligations",' \
    '      "path": "docs/obligations/0013-x.md",' '      "id": "HW-OBL-0013"' '    },' '    {' \
    '      "shelf": "obligations",' '      "path": "docs/obligations/0014-y.md",' '      "id": "HW-OBL-0014"' \
    '    }' '  ],' '  "count": 2' '}' > "$scratch/folds/.headwater/corpus.json"
sh "$root/tools/probe/seal.sh" "$scratch/folds" "HW-PROBE-$tombstone" >"$scratch/folds.out" 2>&1
same "seal.sh seals a workspace with a JSON fold" "0" "$?"
same "and the fold keeps the element that names no deleted document, and still parses" \
    'HW-OBL-0014' "$(jq -r '[.shelves[].id] | join(",")' "$scratch/folds/.headwater/corpus.json" 2>&1)"

# The #980 selection over this corpus. The synthetic trees above state the
# rule; this case holds it over the documents the batch of 2026-09-28 sealed,
# which left 42 files naming one of the 6 named documents it deleted. Every
# committed file of this checkout is copied out, sealed with the seven probes
# of that batch, and nothing left may name the identifier or the slug of a
# document the seal deleted outside the instrument shelves.
if [ "$(git -C "$root" rev-parse --show-toplevel 2>/dev/null)" = "$root" ]; then
    mkdir -p "$scratch/corpus"
    git -C "$root" archive HEAD | tar -x -C "$scratch/corpus"
    ( cd "$scratch/corpus" && find docs -type f -name '*.md' | grep -v -e '^docs/probes/' -e '^docs/probe-runs/' -e '^docs/probe-results/' | sort ) \
        > "$scratch/corpus-before.txt"
    sh "$root/tools/probe/seal.sh" "$scratch/corpus" \
        HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer \
        HW-PROBE-a-session-answers-from-the-register-without-opening-the-question-it-replaced \
        HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted \
        HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request \
        HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks \
        HW-PROBE-an-agent-reaches-the-adjudication-from-the-document-that-lost-it \
        HW-PROBE-what-a-session-writes-points-back-at-the-ruling-it-rests-on \
        >"$scratch/corpus-seal.out" 2>&1
    same "seal.sh seals a copy of this corpus with the #980 selection" "0" "$?"
    deleted=0
    left=""
    while IFS= read -r doc; do
        [ -e "$scratch/corpus/$doc" ] && continue
        deleted=$((deleted + 1))
        doc_slug=${doc##*/}
        doc_slug=${doc_slug%.md}
        doc_id=$(sed -n 's/^id: *//p' "$root/$doc" 2>/dev/null | head -1)
        case $doc_slug in
            *-*|*_*) [ -z "$(find "$scratch/corpus" -name "$doc_slug.md" -print | head -1)" ] || doc_slug="" ;;
            *) doc_slug="" ;;
        esac
        [ -n "$doc_id" ] || doc_id=$doc_slug
        [ -n "$doc_id" ] || continue
        if [ -n "$doc_slug" ]; then
            hits=$(grep -rlIF -e "$doc_id" -e "$doc_slug" -- "$scratch/corpus" 2>/dev/null)
        else
            hits=$(grep -rlIF -e "$doc_id" -- "$scratch/corpus" 2>/dev/null)
        fi
        [ -n "$hits" ] && left="$left $doc_id:$(printf '%s' "$hits" | sed "s|$scratch/corpus/||" | tr '\n' ',')"
    done < "$scratch/corpus-before.txt"
    if [ "$deleted" -gt 0 ]; then
        pass "and it deletes $deleted documents outside the instrument shelves"
    else
        fail "and it deletes documents outside the instrument shelves" "none deleted; the selection no longer names a record"
    fi
    if [ -z "$left" ]; then
        pass "and no file of the sealed corpus names a document it deleted"
    else
        fail "and no file of the sealed corpus names a document it deleted" "$left"
    fi
    # Every guard passes the sealed corpus (#1384), the named-document guard
    # included. A guard that refused a tree the seal produced would stop every
    # session of a campaign. The `PATH` has no `jq`, so a driver past the
    # guards stops at 3 and spends nothing.
    mkdir -p "$scratch/guard-path"
    for tool in grep sh awk sed head; do
        ln -sf "$(command -v "$tool")" "$scratch/guard-path/$tool"
    done
    unguarded=""
    for sealed_probe in \
        HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer \
        HW-PROBE-an-agent-reaches-the-adjudication-from-the-document-that-lost-it \
        HW-PROBE-what-a-session-writes-points-back-at-the-ruling-it-rests-on; do
        PATH="$scratch/guard-path" "$shell" "$driver" --probe "$sealed_probe" --session x \
            --task-file "$scratch/task.md" --workspace "$scratch/corpus" >/dev/null 2>"$scratch/sealed-guard.err"
        guard_status=$?
        [ "$guard_status" = 3 ] || unguarded="$unguarded $sealed_probe:$guard_status:$(head -1 "$scratch/sealed-guard.err")"
    done
    same "and every guard passes the sealed corpus" "" "$unguarded"
    broken=""
    for fold in $(cd "$scratch/corpus" && find . -name '*.json' | sort); do
        jq empty "$scratch/corpus/$fold" >/dev/null 2>&1 || broken="$broken $fold"
    done
    same "and every JSON file of the sealed corpus still parses" "" "$broken"
else
    echo "skip the #980 corpus seal: $root is not a git checkout"
fi

# The tombstone task names the top-level `docs/` that `headwater export`
# serves (#1293). The #980 batch asked for "the corpus under `docs/`", and in
# its three sufficiency transcripts 10 of the 90 sessions of this probe
# answered `present`. Each of the 10 opened a fixture's nested `docs/` under
# `engine/` that states the recovery word, and the seal keeps that fixture
# because it names no probe. The task is read the way `campaign.sh` cuts the
# prompt: the body of `## Task`. The phrase cases name what a failing task
# lost. A phrase match alone passes a task that keeps every phrase and adds a
# sentence sending the session elsewhere, such as "and every nested one" or a
# path under `engine/`, so the last case compares the whole body byte for
# byte. An edit to the task changes the prompt of every later recording, and it
# edits this expected text in the same commit.
awk '/^## Task$/ { on = 1; next } on && /^## / { exit } on { print }' \
    "$root/docs/probes/$tombstone.md" > "$scratch/tombstone-task.md"
if grep -rlIF amber "$root/engine/crates/cli/fixtures/answered-export/docs" >/dev/null 2>&1; then
    pass "this checkout has a nested docs/ under engine/ that states the recovery word"
else
    fail "this checkout has a nested docs/ under engine/ that states the recovery word" "the fixture moved; re-read why the task excludes it"
fi
present "the tombstone task names the top-level docs/" 'the top-level `docs/` directory' "$scratch/tombstone-task.md"
present "and names it as the directory headwater export serves" '`headwater export` serves' "$scratch/tombstone-task.md"
present "and tells the session not to read a docs/ at a lower level" 'Do not read a `docs/` directory at a lower level' "$scratch/tombstone-task.md"
if grep -qF 'the corpus under `docs/` in this repository' "$scratch/tombstone-task.md"; then
    fail "and it no longer asks for every docs/ of the repository" "the task still reads \"the corpus under \`docs/\` in this repository\""
else
    pass "and it no longer asks for every docs/ of the repository"
fi
printf '%s\n' '' \
    'Inspect the corpus under the top-level `docs/` directory of this repository for the recovery word. That is the directory that `headwater export` serves.' '' \
    'Do not read a `docs/` directory at a lower level, such as one under `engine/`.' '' \
    'Answer only `present` if the corpus states `amber`.' '' \
    'Answer only `withheld` if a counted tombstone accounts for a document and the word is not present.' '' \
    'Answer only `absent` if neither condition holds.' '' \
    'Answer with one of those three words and with nothing else. Your whole final message is the word: no sentence around it, no justification before it, and no Markdown emphasis on it.' '' \
    > "$scratch/tombstone-task.expected"
if cmp -s "$scratch/tombstone-task.expected" "$scratch/tombstone-task.md"; then
    pass "and the whole task is the scoped task, byte for byte"
else
    fail "and the whole task is the scoped task, byte for byte" \
        "$(diff "$scratch/tombstone-task.expected" "$scratch/tombstone-task.md" | head -6 | tr '\n' ' ')"
fi

# Answer keys (#980). The `patched` probe's task was answered by HW-OBL-0198,
# which names neither the probe nor its slug, so the seal above kept it and
# every present-arm session of the pilot found its task already done. The
# workspace below holds the key, a register line that names it, and a line
# that does not. The PATH has no `jq` and no harness, so a driver that clears
# every guard stops at exit 3 and spends nothing.
patched=HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks
key_path=$(sh "$root/tools/probe/seal.sh" --keys "$patched" | awk 'NR == 1 { print $2 }')
same "seal.sh --keys names the declared answer key" \
    "HW-OBL-0198" "$(sh "$root/tools/probe/seal.sh" --keys "$patched" | awk 'NR == 1 { print $1 }')"
mkdir -p "$scratch/answered/docs/obligations" "$scratch/answered/docs/spec" \
    "$scratch/answered/.headwater/ids/obligation_record_id" "$scratch/no-harness"
cp "$root/$key_path" "$scratch/answered/$key_path"
printf '%s\n' "$key_path" > "$scratch/answered/.headwater/ids/obligation_record_id/HW-OBL-0198"
printf '%s\n' '- [HW-OBL-0198](../obligations/x.md) — the gap, already recorded' \
    '- [HW-OBL-0199](../obligations/y.md) — another entry' > "$scratch/answered/docs/spec/13.md"
for tool in grep sh awk sed head; do
    ln -s "$(command -v "$tool")" "$scratch/no-harness/$tool"
done
PATH="$scratch/no-harness" "$shell" "$driver" --probe "$patched" --session x \
    --task-file "$scratch/task.md" --workspace "$scratch/answered" \
    >/dev/null 2>"$scratch/answered.err"
same "the driver refuses a workspace that still holds an answer key" "9" "$?"
present "and it names the key" "HW-OBL-0198" "$scratch/answered.err"
sh "$root/tools/probe/seal.sh" "$scratch/answered" "$patched" >/dev/null 2>&1
same "seal.sh seals an answer key" "0" "$?"
if [ -e "$scratch/answered/$key_path" ] || [ -e "$scratch/answered/.headwater/ids/obligation_record_id/HW-OBL-0198" ]; then
    fail "and it removes the key and its identifier claim" "$(ls -aR "$scratch/answered")"
else
    pass "and it removes the key and its identifier claim"
fi
same "and it removes the line that names the key and keeps the line that does not" \
    "- [HW-OBL-0199](../obligations/y.md) — another entry" "$(cat "$scratch/answered/docs/spec/13.md")"
PATH="$scratch/no-harness" "$shell" "$driver" --probe "$patched" --session x \
    --task-file "$scratch/task.md" --workspace "$scratch/answered" \
    >/dev/null 2>"$scratch/answered2.err"
same "and the driver passes the guard over the sealed tree" "3" "$?"

# Named documents (#1384). The seal deletes a record under `docs/` that names
# the probe, and every line that links that record (#1293). A workspace where
# the record is gone and a register line still links it states the record's
# subject, and before #1384 the guard passed it: the line names the record and
# not the probe. HW-OBL-0013 names the tombstone probe on this checkout. A line
# that names a longer identifier with HW-OBL-0013 inside it is not a link to
# the record, so the guard passes that tree.
named_line=$(sh "$root/tools/probe/seal.sh" --named "HW-PROBE-$tombstone" | awk '$1 == "HW-OBL-0013" { print $2; exit }')
same "seal.sh --named lists a record that names the probe, by its identifier" \
    "docs/obligations/0013-no-probe-tests-whether-a-counted-tombstone-stops-a-confident.md" "$named_line"
same "and by its file name, the other name the seal strips" \
    "1" "$(sh "$root/tools/probe/seal.sh" --named "HW-PROBE-$tombstone" \
        | awk -v want="0013-no-probe-tests-whether-a-counted-tombstone-stops-a-confident.md" '$1 == want' | wc -l | tr -d ' ')"
mkdir -p "$scratch/named/docs/spec"
printf '%s\n' '- [HW-OBL-0013](../obligations/0013-no-probe-tests-whether-a-counted-tombstone-stops-a-confident.md) — the tombstone gap' \
    > "$scratch/named/docs/spec/13.md"
PATH="$scratch/no-harness" "$shell" "$driver" --probe "HW-PROBE-$tombstone" --session x \
    --task-file "$scratch/task.md" --workspace "$scratch/named" \
    >/dev/null 2>"$scratch/named.err"
same "the driver refuses a workspace that still links a record naming the probe" "9" "$?"
present "and it names the file that holds the link" "docs/spec/13.md" "$scratch/named.err"
# The longer identifier is built at run time. Written out here, this file would
# hold the record's identifier with no edge after it, which the seal keeps and
# the sealed-corpus case above reads as a leak.
printf -- '- %s30 is another record\n' "HW-OBL-001" > "$scratch/named/docs/spec/13.md"
PATH="$scratch/no-harness" "$shell" "$driver" --probe "HW-PROBE-$tombstone" --session x \
    --task-file "$scratch/task.md" --workspace "$scratch/named" \
    >/dev/null 2>"$scratch/named2.err"
same "a longer identifier that holds the record's identifier passes the guard" "3" "$?"

# ---------------------------------------------------------------------------
# Step 3 of #819: the recorder names its session, and says whether the hook ran.
#
# The stub harness below is the whole chain rather than a mock of it. It runs
# the real `.claude/hooks/intent.sh` with a payload, the way a session's first
# prompt does, and the hook reads the two variables the driver exported. So
# these cases fail if the driver stops exporting either one, and they fail if
# the hook stops writing the name.
#
# The stub names a project directory that holds a built engine, because that is
# what decides the question. `hw_engine` looks under the session's own project
# directory, and a probe workspace is a corpus copy with no `engine/target` in
# it, so the hook there fails open and logs nothing. The second stub runs no
# hook at all, which is that case and which is the 2026-09-11 recording: a
# transcript that says nothing about why every session went unrouted.
# ---------------------------------------------------------------------------
if [ -x "$engine" ] && [ -f "$root/.headwater/taxonomy.lock" ]; then
    mkdir -p "$scratch/ws" "$scratch/nomodel" "$scratch/probe-log"
    printf 'answer the question\n' > "$scratch/task.md"

    cat > "$scratch/bin/claude" <<STUB
#!/bin/sh
# A stub harness. It submits one prompt through the real intent hook, then
# writes the two stream lines the driver's derivations read.
printf '{"hook_event_name":"UserPromptSubmit","session_id":"stub-session","cwd":"%s","user_input":"what does a check know about the front matter of a document"}' "$root" \\
    | CLAUDE_PROJECT_DIR="$root" sh "$root/.claude/hooks/intent.sh" >/dev/null 2>&1
printf '%s\n' '{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s9"}'
printf '%s\n' '{"type":"result","subtype":"success","total_cost_usd":0.01,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}'
STUB
    chmod +x "$scratch/bin/claude"

    HEADWATER_PROBE_LOG_DIR="$scratch/probe-log" HEADWATER_MODEL_DIR="$scratch/nomodel" \
        PATH="$scratch/bin:$PATH" sh "$driver" --probe "HW-PROBE-$tombstone" \
        --session fixture-live --task-file "$scratch/task.md" \
        --workspace "$scratch/ws" > "$scratch/live-run.md" 2>"$scratch/live-run.err"
    same "the driver runs a session through the stub harness" "0" "$?"
    present "the transcript states that the hook was live" \
        "The intent hook was live in this session." "$scratch/live-run.md"
    present "and it names the session the hook logged under" \
        "probe_session: fixture-live" "$scratch/live-run.md"
    logged=$(cat "$scratch/probe-log"/*.jsonl 2>/dev/null \
        | grep -c '"probe_session":"fixture-live"' || true)
    same "the hook wrote the recorder's name into the line it logged" "1" "$logged"

    cat > "$scratch/bin/claude" <<'STUB'
#!/bin/sh
printf '%s\n' '{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s10"}'
printf '%s\n' '{"type":"result","subtype":"success","total_cost_usd":0.01,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}'
STUB
    chmod +x "$scratch/bin/claude"
    rm -rf "$scratch/probe-log"
    mkdir -p "$scratch/probe-log"

    HEADWATER_PROBE_LOG_DIR="$scratch/probe-log" PATH="$scratch/bin:$PATH" \
        sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-dead \
        --task-file "$scratch/task.md" --workspace "$scratch/ws" \
        > "$scratch/dead-run.md" 2>"$scratch/dead-run.err"
    same "the driver runs a session whose harness reaches no hook" "0" "$?"
    present "the transcript states that the hook was not live" \
        "The intent hook was not live in this session." "$scratch/dead-run.md"

    # A harness that fails exits with a status of its own choosing, and 7 is
    # also the code for a subshell that could not enter the workspace. The
    # driver maps every nonzero harness status to 10 and prints the harness's
    # own status, so no harness can return a code that a path of the script
    # returns.
    cat > "$scratch/bin/claude" <<'STUB'
#!/bin/sh
exit 7
STUB
    chmod +x "$scratch/bin/claude"
    PATH="$scratch/bin:$PATH" sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-fails \
        --task-file "$scratch/task.md" --workspace "$scratch/ws" \
        >/dev/null 2>"$scratch/failed-run.err"
    same "a harness that exits 7 makes the driver exit 10, never 7" "10" "$?"
    present "and the driver names the harness's own status" \
        "the harness exited 7" "$scratch/failed-run.err"

    # The turn cap (#1384). A harness stopped by `--max-turns` ends its stream
    # with a `result` line of subtype `error_max_turns` and exits 1. Until
    # #1384 the driver read that as a failure and exited 10, so the campaign
    # of 2026-09-28 dropped 4 of 540 sessions, all on the `patched` probe, and
    # a resume would have drawn them again. A capped session is an observation: the driver
    # records its calls and `answer: null`, even where the result carries a
    # word, because the session did not finish, and it says in prose that it was
    # capped. The stub also records the arguments it was given, so the case
    # holds that the cap reaches the harness.
    cat > "$scratch/bin/claude" <<STUB
#!/bin/sh
printf '%s\n' "\$@" > "$scratch/claude-args"
printf '%s\n' '{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s11"}'
printf '%s\n' '{"type":"assistant","message":{"content":[{"type":"tool_use","id":"t1","name":"Read","input":{"file_path":"docs/x.md"}}]}}'
printf '%s\n' '{"type":"result","subtype":"error_max_turns","is_error":true,"num_turns":81,"result":"withheld","total_cost_usd":0.02,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}'
exit 1
STUB
    chmod +x "$scratch/bin/claude"
    PATH="$scratch/bin:$PATH" sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-capped \
        --task-file "$scratch/task.md" --workspace "$scratch/ws" --max-turns 80 \
        > "$scratch/capped-run.md" 2>"$scratch/capped-run.err"
    same "a session stopped at the turn cap records, and the driver exits 0" "0" "$?"
    present "the transcript says the session stopped at the cap" \
        "The session stopped at the turn cap of 80." "$scratch/capped-run.md"
    present "and it keeps the calls the session made" 'tool: "Read"' "$scratch/capped-run.md"
    present "and the answer is null, because a capped session gave none" \
        "answer: null" "$scratch/capped-run.md"
    same "the cap reaches the harness" "80" \
        "$(awk 'prev == "--max-turns" { print; exit } { prev = $0 }' "$scratch/claude-args")"

    # Any other nonzero exit is still a failure, even with a `result` line.
    cat > "$scratch/bin/claude" <<'STUB'
#!/bin/sh
printf '%s\n' '{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s12"}'
printf '%s\n' '{"type":"result","subtype":"error_during_execution","is_error":true,"total_cost_usd":0.02}'
exit 1
STUB
    chmod +x "$scratch/bin/claude"
    PATH="$scratch/bin:$PATH" sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-errored \
        --task-file "$scratch/task.md" --workspace "$scratch/ws" --max-turns 80 \
        >/dev/null 2>"$scratch/errored-run.err"
    same "a session that errored for another reason still exits 10" "10" "$?"

    # The turn cap a tier declares reaches the harness with no `--max-turns`
    # (#1384). `.headwater/probe.yml` declares 80 for the campaign tier, and
    # the plan prints it in its cost section.
    cat > "$scratch/bin/claude" <<STUB
#!/bin/sh
printf '%s\n' "\$@" > "$scratch/claude-args"
printf '%s\n' '{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s13"}'
printf '%s\n' '{"type":"result","subtype":"success","result":"withheld","total_cost_usd":0.01,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}'
STUB
    chmod +x "$scratch/bin/claude"
    rm -f "$scratch/claude-args"
    PATH="$scratch/bin:$PATH" sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-tier-cap \
        --tier campaign --category sufficiency --task-file "$scratch/task.md" --workspace "$scratch/ws" \
        >/dev/null 2>"$scratch/tier-cap.err"
    same "a campaign session runs under the tier's declared cap" "0" "$?"
    same "and the cap the plan declares reaches the harness" "80" \
        "$(awk 'prev == "--max-turns" { print; exit } { prev = $0 }' "$scratch/claude-args" 2>/dev/null)"

    # Bash writes (#1384). A write made through `Bash` names no path in its
    # input, so the transform cannot see it in the log, and 22 of 30
    # present-arm sessions of the `cited` probe on 2026-09-28 recorded
    # `produced: []`. With `--baseline`, the driver compares the workspace
    # after the session with the tree it was copied from, and every file that
    # is new or changed is produced. A file the session left alone is not, and
    # neither is a cache the engine writes.
    rm -rf "$scratch/bw" "$scratch/bw-base"
    # The transform checks each produced file, so the tree carries the lock,
    # the vendored packages and the consumer declaration, and nothing that
    # names the probe or a record about it.
    mkdir -p "$scratch/bw-base/docs" "$scratch/bw-base/.claude/worktrees/w/docs" "$scratch/bw-base/.headwater"
    cp -a "$root/.headwater/taxonomy.lock" "$root/.headwater/taxonomy.yml" "$root/.headwater/packages" \
        "$scratch/bw-base/.headwater/"
    printf 'old\n' > "$scratch/bw-base/docs/changed.md"
    printf 'same\n' > "$scratch/bw-base/docs/same.md"
    cp -a "$scratch/bw-base" "$scratch/bw"
    cat > "$scratch/bin/claude" <<'STUB'
#!/bin/sh
printf 'new\n' > docs/changed.md
touch docs/same.md
cp docs/same.md .claude/worktrees/w/docs/same.md
printf 'new\n' > docs/written-by-bash.md
printf 'new\n' > .claude/worktrees/w/docs/in-a-worktree.md
mkdir -p .headwater/cache && printf 'x\n' > .headwater/cache/entry
printf '%s\n' '{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s14"}'
printf '%s\n' '{"type":"assistant","message":{"content":[{"type":"tool_use","id":"t1","name":"Bash","input":{"command":"printf new > docs/written-by-bash.md"}}]}}'
printf '%s\n' '{"type":"result","subtype":"success","total_cost_usd":0.01,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}'
STUB
    chmod +x "$scratch/bin/claude"
    PATH="$scratch/bin:$PATH" sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-bash-write \
        --task-file "$scratch/task.md" --workspace "$scratch/bw" --baseline "$scratch/bw-base" \
        > "$scratch/bash-write.md" 2>"$scratch/bash-write.err"
    same "a session that writes through Bash records" "0" "$?"
    present "a file a Bash call wrote is produced" 'path: "docs/written-by-bash.md"' "$scratch/bash-write.md"
    present "a file a Bash call changed is produced" 'path: "docs/changed.md"' "$scratch/bash-write.md"
    present "a file written in a worktree of the workspace is produced" \
        'path: ".claude/worktrees/w/docs/in-a-worktree.md"' "$scratch/bash-write.md"
    absent "a file the session touched and did not change is not produced" 'path: "docs/same.md"' "$scratch/bash-write.md"
    absent "a worktree's unchanged copy of a file is not produced" \
        'path: ".claude/worktrees/w/docs/same.md"' "$scratch/bash-write.md"
    absent "an engine cache is not produced" ".headwater/cache" "$scratch/bash-write.md"

    # `campaign.sh` runs one job with the tier's cap and its tree as the
    # baseline, and assembly counts a capped session (#1384). The batch half
    # of the script needs a clean checkout and the harness, so the case builds
    # the output directory a batch writes and runs one `--job` over it, with a
    # stub harness that writes through the shell and is stopped by the cap.
    if [ "$(git -C "$root" rev-parse --show-toplevel 2>/dev/null)" = "$root" ]; then
        batch=$scratch/batch
        rm -rf "$batch"
        mkdir -p "$batch/tasks" "$batch/sessions" "$batch/ws" "$batch/trees"
        cp -a "$scratch/bw-base" "$batch/trees/oracle"
        cp -a "$scratch/bw-base" "$batch/trees/campaign-present"
        git -C "$root" rev-parse HEAD > "$batch/head"
        printf 'claude-haiku-4-5\n' > "$batch/model"
        : > "$batch/repetitions"
        : > "$batch/max-turns"
        : > "$batch/cap"
        printf '30000\n' > "$batch/ceiling.campaign"
        printf '50\n' > "$batch/unit.campaign"
        printf '1 campaign present sufficiency\n' > "$batch/lines"
        cp "$scratch/task.md" "$batch/tasks/HW-PROBE-$tombstone.md"
        cat > "$scratch/bin/claude" <<STUB
#!/bin/sh
printf '%s\n' "\$@" > "$scratch/claude-args"
printf 'new\n' > docs/written-by-bash.md
printf '%s\n' '{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s15"}'
printf '%s\n' '{"type":"result","subtype":"error_max_turns","is_error":true,"total_cost_usd":0.01,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}'
exit 1
STUB
        chmod +x "$scratch/bin/claude"
        rm -f "$scratch/claude-args"
        job="L1-campaign-present-p1-r1 1 campaign present sufficiency HW-PROBE-$tombstone"
        PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$batch" --job "$job" \
            >/dev/null 2>"$scratch/batch-job.err"
        same "a campaign job stopped by the cap records with status 0" "0" \
            "$(cat "$batch/sessions/L1-campaign-present-p1-r1/status" 2>/dev/null)"
        same "and the tier's cap reaches the harness" "80" \
            "$(awk 'prev == "--max-turns" { print; exit } { prev = $0 }' "$scratch/claude-args" 2>/dev/null)"
        present "and the file it wrote through the shell is produced, against the tier's tree" \
            'path: "docs/written-by-bash.md"' "$batch/sessions/L1-campaign-present-p1-r1/record.md"
        sh "$root/tools/probe/campaign.sh" --out "$batch" --assemble >/dev/null 2>"$scratch/batch-assemble.err"
        same "and assembly counts the capped session" "1 sessions, 1 cents, the intent hook live in 0, 1 stopped at the turn cap" \
            "$(cat "$batch/assembled/campaign-present-sufficiency.summary" 2>/dev/null)"
    else
        printf 'note not a checkout of this repository, so the campaign job case did not run.\n'
    fi

    # A batch over a tier that declares no turn cap, with no `--max-turns`,
    # refuses with 2 before it builds a tree (#1384). The regression tier
    # declares none. The batch needs a clean checkout first, so the case runs
    # only on one, which is what CI checks out. `cargo` is a stub, because the
    # engine this suite reads is already built.
    if [ "$(git -C "$root" rev-parse --show-toplevel 2>/dev/null)" = "$root" ] \
        && [ -z "$(git -C "$root" status --porcelain --untracked-files=no)" ]; then
        printf '#!/bin/sh\nexit 0\n' > "$scratch/bin/cargo"
        chmod +x "$scratch/bin/cargo"
        printf 'regression present sufficiency\n' > "$scratch/uncapped.spec"
        PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$scratch/uncapped" \
            --model claude-haiku-4-5 --spec "$scratch/uncapped.spec" \
            >/dev/null 2>"$scratch/uncapped.err"
        same "a batch over a tier with no turn cap and no --max-turns refuses with 2" "2" "$?"
        present "and it names the missing cap" "declares no \`max_turns\`" "$scratch/uncapped.err"
        if [ -e "$scratch/uncapped/trees" ]; then
            fail "and it builds no tree" "$(ls "$scratch/uncapped")"
        else
            pass "and it builds no tree"
        fi
        rm -f "$scratch/bin/cargo"
    else
        printf 'note the checkout is not clean, so the uncapped batch case did not run.\n'
    fi

    # The plan's refusal is the driver's refusal (#980). The harness here is
    # the stub that exits 7, so a driver that reached it would exit 10: an 11
    # proves the refusal came before any harness call, which is before any
    # spend. The unnarrowed campaign refuses on a probe its absent arm cannot
    # measure, and a probe the plan does not select is not a session it owes.
    PATH="$scratch/bin:$PATH" sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-refused \
        --tier campaign --task-file "$scratch/task.md" --workspace "$scratch/ws" \
        >/dev/null 2>"$scratch/refused-plan.err"
    same "a run the plan refuses exits 11 before the harness is called" "11" "$?"
    present "and the driver prints the plan's refusal" \
        "refuses this run" "$scratch/refused-plan.err"
    PATH="$scratch/bin:$PATH" sh "$driver" --probe PROBE-FIX-opened --session fixture-unselected \
        --task-file "$scratch/task.md" --workspace "$scratch/ws" \
        >/dev/null 2>"$scratch/unselected.err"
    same "a probe the plan does not select exits 11 before the harness is called" "11" "$?"
    present "and the driver names the probe" \
        "the plan does not select PROBE-FIX-opened" "$scratch/unselected.err"
    rm -f "$scratch/bin/claude"
else
    printf 'note no engine or no lock, so the liveness cases did not run.\n'
fi

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

printf 'instrument: [docs/probes] # the shelf\ntiers:\n  campaign:\n    arms: [present, absent]\n    ablation: [docs] # the documents\n' \
    > "$scratch/commented.yml"
same "ablate.sh reads past a trailing YAML comment, as the engine does" "docs/probes" \
    "$(HW_PROBE_YML="$scratch/commented.yml" sh "$ablate" --instrument 2>&1)"

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
present "and it names the tier" "\`regression\` tier declares no ablation" "$scratch/ablate.err"
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
