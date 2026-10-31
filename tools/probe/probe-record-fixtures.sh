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
# The newer of the two profiles answers, by the rule in
# `tools/repo/resolve-engine.sh` (#990). With neither built, `engine` names a
# path that is not executable, and each case that needs one skips on that.
. "$root/tools/repo/resolve-engine.sh"
engine=$(hw_resolve_engine_bin "$root") || engine="$root/engine/target/release/headwater"

if ! command -v jq >/dev/null 2>&1; then
    echo "no \`jq\` on the path, and the tool under test reads JSON with it." >&2
    exit 3
fi

scratch=$(mktemp -d) || exit 1
# The confinement cases keep their workspace outside `/tmp`, where a batch
# keeps its own, because the session's `/tmp` is a tmpfs of its own (#1659).
outer=$(mktemp -d /var/tmp/hw-probe-fixtures.XXXXXX) || exit 1
trap 'rm -rf "$scratch" "$outer"' EXIT HUP INT TERM

passed=0
failed=0
pass() { printf 'ok   %s\n' "$1"; passed=$((passed + 1)); }
fail() { printf 'FAIL %s\n  %s\n' "$1" "$2"; failed=$((failed + 1)); }
# Each helper takes exactly three arguments. Two calls joined on one line pass
# the second as extra arguments, and the second case would never run, so an
# extra argument is a failure of its own (verify of #1467).
arity() {
    [ "$2" = 3 ] && return 0
    fail "$1" "the case helper received $2 arguments, not 3: two cases joined on one line?"
    return 1
}
same() {
    arity "$1" $# || return 0
    if [ "$2" = "$3" ]; then pass "$1"; else fail "$1" "expected \`$2\`, got \`$3\`"; fi
}
absent() {
    # $1 name, $2 needle, $3 file
    arity "$1" $# || return 0
    if grep -qF -- "$2" "$3"; then
        fail "$1" "the output holds \`$2\`, which came from a block the filter must drop"
    else
        pass "$1"
    fi
}
present() {
    arity "$1" $# || return 0
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

# Auto-memory is not a produced artifact (HW-OBL-0237, #1775). A session that
# writes a memory file under its configuration directory named a path that
# carries no `findings` key, and the grader could not grade it: one session
# of the 2026-10-03 campaign was drawn again for this. With `--config-dir`,
# a path under that directory is not in `produced`, and a path beside it is.
mkdir -p "$scratch/cfg-dir/projects/ws/memory" "$scratch/cfg-dir-sibling"
printf 'remember HW-DR-0049\n' > "$scratch/cfg-dir/projects/ws/memory/MEMORY.md"
printf 'HW-DR-0049\n' > "$scratch/cfg-dir-sibling/kept.md"
{
    printf '%s\n' '{"type":"system","subtype":"init","model":"claude-sonnet-5","session_id":"s9"}'
    jq -nc --arg p "$scratch/cfg-dir/projects/ws/memory/MEMORY.md" '{type:"assistant",message:{content:[{type:"tool_use",id:"t1",name:"Write",input:{file_path:$p,content:"x"}}]}}'
    jq -nc --arg p "$scratch/cfg-dir-sibling/kept.md" '{type:"assistant",message:{content:[{type:"tool_use",id:"t2",name:"Write",input:{file_path:$p,content:"x"}}]}}'
} > "$scratch/memory.jsonl"
sh "$transform" --probe PROBE-FIX-cited --session memory --root "$scratch/empty-base" \
    --config-dir "$scratch/cfg-dir" < "$scratch/memory.jsonl" > "$scratch/memory.yaml" 2>"$scratch/memory.err"
same "a log that writes auto-memory transforms without error" "0" "$?"
present "the memory write is still a call of the log" \
    "      argument: \"$scratch/cfg-dir/projects/ws/memory/MEMORY.md\"" "$scratch/memory.yaml"
absent "a memory file under the configuration directory is not produced" \
    "    - path: \"$scratch/cfg-dir/projects/ws/memory/MEMORY.md\"" "$scratch/memory.yaml"
present "and a file beside that directory, whose name only starts the same, still is" \
    "    - path: \"$scratch/cfg-dir-sibling/kept.md\"" "$scratch/memory.yaml"
sh "$transform" --probe PROBE-FIX-cited --session memory-no-config --root "$scratch/empty-base" \
    < "$scratch/memory.jsonl" > "$scratch/memory-none.yaml" 2>/dev/null
present "with no --config-dir the memory file is produced, as before" \
    "    - path: \"$scratch/cfg-dir/projects/ws/memory/MEMORY.md\"" "$scratch/memory-none.yaml"
present "the recorder gives the transform its session's configuration directory" \
    '--config-dir "$config"' "$driver"

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
# `--oracle-tree` checks the session's whole patch, not one artifact (#1472).
#
# A campaign derives `findings` in a copy of the sealed present-arm tree. The
# first version wrote one artifact into each copy, so a change task whose two
# files must agree could not pass: each half alone reports
# `relation.reciprocity.missing`. The probe that asks for this change is
# `a-session-records-which-obligation-an-evaluation-discharged`, and this case
# runs its two sessions through the transform a campaign calls. The oracle rule
# is read from the probe, so a probe that names a rule its sound session still
# carries goes red here.
# ---------------------------------------------------------------------------
oprobe=$root/docs/probes/a-session-records-which-obligation-an-evaluation-discharged.md
oeval=docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md
oobl=docs/obligations/0104-a-governs-edge-reaches-the-path-it-names-and-nothing.md
if [ -x "$engine" ] && [ -f "$oprobe" ]; then
    orule=$(sed -n 's/^oracle: *//p' "$oprobe" | tr -d "\"'" | sed -n 1p)
    oeid=$(sed -n 's/^id: *//p' "$root/$oeval" | sed -n 1p)
    rm -rf "$scratch/otree" "$scratch/ows-one" "$scratch/ows-both"
    mkdir -p "$scratch/otree" "$scratch/engine-only/engine/target/dev-release"
    [ -e "$scratch/engine-only/engine/target/dev-release/headwater" ] ||
        ln -s "$engine" "$scratch/engine-only/engine/target/dev-release/headwater"
    cp -a "$root/docs" "$root/.headwater" "$root/CLAUDE.md" "$scratch/otree/"
    cp -a "$scratch/otree" "$scratch/ows-one"
    awk '{print} /^relations:$/ && !x {print "  discharges:"; print "    - HW-OBL-0104"; x=1}' \
        "$root/$oeval" > "$scratch/ows-one/$oeval"
    cp -a "$scratch/ows-one" "$scratch/ows-both"
    awk -v id="$oeid" '{print} /^relations:$/ && !x {print "  discharged_by:"; print "    - " id; x=1}' \
        "$root/$oobl" > "$scratch/ows-both/$oobl"
    "$engine" check --root "$scratch/otree" --format json > "$scratch/otree.json" 2>/dev/null
    same "the probe's oracle is not reported over the asked file before any patch" "0" \
        "$(jq -r --arg p "$oeval" --arg r "$orule" '[.findings[] | select(.path == $p and .rule == $r)] | length' < "$scratch/otree.json")"
    sh "$transform" --probe HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged \
        --session one-half --root "$scratch/engine-only" --workspace "$scratch/ows-one" \
        --oracle-tree "$scratch/otree" --produced "$oeval" \
        < "$scratch/watched-nothing.jsonl" > "$scratch/ows-one.yaml" 2>"$scratch/ows-one.err"
    same "a session that patched one half derives its findings through the oracle tree" "0" "$?"
    present "and the oracle reports over the half that stands alone" \
        "        - \"$orule\"" "$scratch/ows-one.yaml"
    sh "$transform" --probe HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged \
        --session both-halves --root "$scratch/engine-only" --workspace "$scratch/ows-both" \
        --oracle-tree "$scratch/otree" --produced "$oeval" --produced "$oobl" \
        < "$scratch/watched-nothing.jsonl" > "$scratch/ows-both.yaml" 2>"$scratch/ows-both.err"
    same "a session that patched both halves derives its findings through the oracle tree" "0" "$?"
    same "and both halves are in \`produced\`" "2" "$(grep -c '^    - path:' "$scratch/ows-both.yaml")"
    absent "and the oracle reports over neither, because the copy holds the whole patch" \
        "        - \"$orule\"" "$scratch/ows-both.yaml"

    # A write the harness refused landed nothing, so it is not an artifact
    # (#1659). Four sessions of the 2026-10-03 campaign met the workspace's
    # write hook: each `Write` named a path, the hook refused it, and the old
    # transform kept that path in `produced`. The oracle branch then exited 5
    # on the `cp` of a file that was never there, and the non-oracle branch
    # wrote `findings: []` for it, which the grader scores as a clean pass.
    # Spec 15 gives `path` as "where the artifact landed", so the path leaves.
    refused='docs/obligations/9999-a-write-the-hook-refused.md'
    {
        printf '%s\n' '{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s-refused"}'
        printf '{"type":"assistant","message":{"id":"m1","content":[{"type":"tool_use","id":"t1","name":"Write","input":{"file_path":"%s","content":"x"}}]}}\n' "$refused"
        printf '%s\n' '{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t1","is_error":true,"content":"PreToolUse:Write hook error: run headwater new"}]}}'
        printf '%s\n' '{"type":"result","subtype":"success","total_cost_usd":0.001}'
    } > "$scratch/refused-write.jsonl"
    sh "$transform" --probe HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged \
        --session refused-only --root "$scratch/engine-only" --workspace "$scratch/ows-one" \
        --oracle-tree "$scratch/otree" \
        < "$scratch/refused-write.jsonl" > "$scratch/refused-only.yaml" 2>"$scratch/refused-only.err"
    same "a refused write with no other artifact records through the oracle tree" "0" "$?"
    same "and its event says \`produced: []\`, the scored failure" "  produced: []" \
        "$(grep '^  produced:' "$scratch/refused-only.yaml")"
    sh "$transform" --probe HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged \
        --session refused-then-landed --root "$scratch/engine-only" --workspace "$scratch/ows-one" \
        --oracle-tree "$scratch/otree" --produced "$oeval" \
        < "$scratch/refused-write.jsonl" > "$scratch/refused-then-landed.yaml" 2>"$scratch/refused-then-landed.err"
    same "a refused write then an artifact at another path records through the oracle tree" "0" "$?"
    same "and \`produced\` names the path that landed and nothing else" \
        "    - path: \"$oeval\"" "$(grep '^    - path:' "$scratch/refused-then-landed.yaml")"
    present "and that artifact carries a \`findings\` key" \
        "      findings:" "$scratch/refused-then-landed.yaml"
    sh "$transform" --probe HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged \
        --session refused-no-oracle --root "$scratch/engine-only" --workspace "$scratch/ows-one" \
        < "$scratch/refused-write.jsonl" > "$scratch/refused-no-oracle.yaml" 2>"$scratch/refused-no-oracle.err"
    same "a refused write records without an oracle tree too" "0" "$?"
    same "and it says \`produced: []\`, not an entry with \`findings: []\` that grades as clean" \
        "  produced: []" "$(grep '^  produced:' "$scratch/refused-no-oracle.yaml")"
    # An empty file that landed is an artifact: the test is that a file is
    # there, not that it holds bytes.
    rm -rf "$scratch/ows-empty"
    cp -a "$scratch/ows-one" "$scratch/ows-empty"
    : > "$scratch/ows-empty/$refused"
    sh "$transform" --probe HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged \
        --session empty-landed --root "$scratch/engine-only" --workspace "$scratch/ows-empty" \
        < "$scratch/refused-write.jsonl" > "$scratch/empty-landed.yaml" 2>"$scratch/empty-landed.err"
    same "an empty file that landed is in \`produced\`" "    - path: \"$refused\"" \
        "$(grep '^    - path:' "$scratch/empty-landed.yaml")"
fi

# ---------------------------------------------------------------------------
# A command oracle fails every artifact of a session whose command failed
# (#1834).
#
# `.headwater/probe.yml` declares a command under `oracles`, and the transform
# runs it in its copy of the oracle tree. The grader is satisfied by any one
# artifact whose findings lack the oracle, so the decisive case is a session
# that wrote code that does not compile and also a README: before #1834 the
# README carried `findings: []`, the code carried no `test:` finding at all,
# and the session graded as a pass. The stub stands in for `cargo test`,
# because what the case tests is the parser and not cargo.
# ---------------------------------------------------------------------------
if [ -x "$engine" ]; then
    cot=$scratch/cmd-oracle
    rm -rf "$cot" "$scratch/cmd-ws" "$scratch/cmd-ran"
    mkdir -p "$cot/src" "$cot/site"
    cp -a "$root/.headwater" "$root/CLAUDE.md" "$cot/"
    # The fixture's two oracles stand in for the block the declaration
    # carries, whatever that block holds (#1835 declared the first ones).
    awk 'skip && /^[^ ]/ { skip = 0 }
    skip { next }
    /^oracles:/ {
        skip = 1
        print "oracles:"
        print "  fixture-oracle:"
        print "    paths: [src/**]"
        print "    command: sh oracle-stub.sh"
        print "    parser: cargo-test"
        print "    holds: [tests/hidden.sh]"
        print "  fixture-exit:"
        print "    paths: [site/**]"
        print "    command: sh exit-stub.sh"
        print "    parser: exit"
        next
    } { print }' "$root/.headwater/probe.yml" > "$cot/.headwater/probe.yml"
    cat > "$cot/oracle-stub.sh" <<STUB
#!/bin/sh
: > "$scratch/cmd-ran"
: > ran-here
case \$(cat stub-mode) in
    compile) printf 'error[E0425]: cannot find value\nerror: could not compile \`fixture\` (lib) due to 1 previous error\n'; exit 101 ;;
    pass) printf 'running 1 test\ntest a::b ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored\n'; exit 0 ;;
    failcase) printf 'running 2 tests\ntest a::b ... FAILED\ntest a::c ... ok\n\ntest result: FAILED. 1 passed; 1 failed; 0 ignored\n'; exit 101 ;;
    zero) printf 'running 0 tests\n\ntest result: ok. 0 passed; 0 failed; 0 ignored\n'; exit 0 ;;
    held) sh tests/hidden.sh; exit \$? ;;
    env) printf '%s\n' "\$HEADWATER_ORACLE_TRANSCRIPT" "\$HEADWATER_ORACLE_HARNESS_LOG" > "$scratch/cmd-env"; cp "\$HEADWATER_ORACLE_TRANSCRIPT" "$scratch/cmd-env-stream"; printf 'test result: ok. 1 passed; 0 failed\n'; exit 0 ;;
    exitonly) printf 'running 1 test\ntest a::b ... ok\n\ntest result: ok. 1 passed; 0 failed\nerror: test failed, to rerun pass \`--doc\`\n'; exit 101 ;;
esac
exit 99
STUB
    printf 'exit "$(cat exit-code)"\n' > "$cot/exit-stub.sh"
    printf 'pub fn pinned() {}\n' > "$cot/src/lib.rs"
    printf '<p>pinned</p>\n' > "$cot/site/index.html"
    printf '0\n' > "$cot/exit-code"
    mkdir -p "$cot/tests"
    printf "printf 'test hidden::case ... FAILED\\\\ntest result: FAILED. 0 passed; 1 failed\\\\n'; exit 101\n" \
        > "$cot/tests/hidden.sh"
    cp -a "$cot" "$scratch/cmd-ws"
    printf 'pub fn changed( {}\n' > "$scratch/cmd-ws/src/lib.rs"
    printf '# changed\n' > "$scratch/cmd-ws/README.md"
    printf '<p>changed</p>\n' > "$scratch/cmd-ws/site/index.html"
    # cmd_run <name> <mode> <produced...>: one transform run over the stub.
    cmd_run() {
        c_name=$1 c_mode=$2
        shift 2
        printf '%s\n' "$c_mode" > "$cot/stub-mode"
        rm -f "$scratch/cmd-ran"
        c_args=
        for c_path in "$@"; do c_args="$c_args --produced $c_path"; done
        # shellcheck disable=SC2086
        sh "$transform" --probe HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged \
            --session "$c_name" --root "$scratch/engine-only" --workspace "$scratch/cmd-ws" \
            --oracle-tree "$cot" $c_args \
            < "$scratch/watched-nothing.jsonl" > "$scratch/cmd-$c_name.yaml" 2>"$scratch/cmd-$c_name.err"
    }
    # cmd_count <needle> <name>: how many artifacts of one event carry a finding.
    cmd_count() {
        grep -cxF -- "        - \"$1\"" "$scratch/cmd-$2.yaml"
    }

    cmd_run compile compile src/lib.rs README.md
    same "a session whose code does not compile records through a command oracle" "0" "$?"
    same "and both artifacts carry the whole-oracle finding, the README too" "2" \
        "$(cmd_count test:fixture-oracle compile)"
    same "and both carry \`test:build\`, the diagnostic that splits a failed build from a wrong one" "2" \
        "$(cmd_count test:build compile)"
    if [ -f "$scratch/cmd-ran" ]; then
        pass "and the command ran, because an artifact matched its pattern"
    else
        fail "and the command ran, because an artifact matched its pattern" "$(cat "$scratch/cmd-compile.err")"
    fi
    if [ -e "$cot/ran-here" ] || [ -e "$scratch/cmd-ws/ran-here" ]; then
        fail "and it ran in a copy, never in the oracle tree or the workspace" "$(ls -a "$cot" "$scratch/cmd-ws")"
    else
        pass "and it ran in a copy, never in the oracle tree or the workspace"
    fi

    cmd_run pass pass src/lib.rs README.md
    same "a passing command records" "0" "$?"
    same "and no artifact carries a \`test:\` finding" "0" "$(grep -c -- '- "test:' "$scratch/cmd-pass.yaml")"

    cmd_run failcase failcase src/lib.rs README.md
    same "a failed case gives that case on both artifacts" "2" "$(cmd_count test:a::b failcase)"
    same "and the whole-oracle finding on both" "2" "$(cmd_count test:fixture-oracle failcase)"
    same "and no finding for the case that passed" "0" "$(cmd_count test:a::c failcase)"

    cmd_run exitonly exitonly src/lib.rs README.md
    same "a command that exits non-zero after a passing case fails the oracle on both artifacts" "2" \
        "$(cmd_count test:fixture-oracle exitonly)"

    cmd_run zero zero src/lib.rs README.md
    same "a run in which no case ran gives the whole-oracle finding on both artifacts" "2" \
        "$(cmd_count test:fixture-oracle zero)"
    same "and not \`test:build\`, because it compiled" "0" "$(cmd_count test:build zero)"

    cmd_run untouched pass README.md
    same "a session with no artifact under the pattern records" "0" "$?"
    same "and the whole-oracle finding is on its artifact" "1" "$(cmd_count test:fixture-oracle untouched)"
    if [ -e "$scratch/cmd-ran" ]; then
        fail "and the command did not run" "the stub wrote its marker"
    else
        pass "and the command did not run"
    fi

    printf '0\n' > "$cot/exit-code"
    cmd_run exit0 pass site/index.html README.md
    same "an \`exit\` oracle that exits 0 gives no \`command:\` finding" "0" \
        "$(cmd_count command:fixture-exit exit0)"
    printf '3\n' > "$cot/exit-code"
    cmd_run exit3 pass site/index.html README.md
    same "an \`exit\` oracle that exits 3 gives \`command:fixture-exit\` on both artifacts" "2" \
        "$(cmd_count command:fixture-exit exit3)"
    printf '0\n' > "$cot/exit-code"
    same "and a site-only session fails the code oracle it did not attempt" "2" \
        "$(cmd_count test:fixture-oracle exit3)"

    # The declaration is the oracle tree's, never the workspace's: a session
    # that rewrites the fold cannot swap the command for `true`.
    sed -e 's/command: sh oracle-stub.sh/command: "true"/' \
        -e '/command: "true"/{n;s/parser: cargo-test/parser: exit/;}' "$cot/.headwater/probe.yml" \
        > "$scratch/cmd-ws/.headwater/probe.yml"
    cmd_run swapped compile src/lib.rs README.md .headwater/probe.yml
    same "a session that rewrote the declaration still fails on the oracle tree's command" "3" \
        "$(cmd_count test:fixture-oracle swapped)"
    cp "$cot/.headwater/probe.yml" "$scratch/cmd-ws/.headwater/probe.yml"

    # A held file is the oracle tree's. A session that writes its own hidden
    # test, one that passes, still grades against the oracle tree's test.
    printf "printf 'test result: ok. 1 passed; 0 failed\\\\n'\n" > "$scratch/cmd-ws/tests/hidden.sh"
    cmd_run held held src/lib.rs tests/hidden.sh
    same "a session that rewrote a held test records" "0" "$?"
    same "and it fails on the oracle tree's test, not its own" "2" "$(cmd_count test:hidden::case held)"
    same "and the whole oracle fails on both artifacts" "2" "$(cmd_count test:fixture-oracle held)"
    cp "$cot/tests/hidden.sh" "$scratch/cmd-ws/tests/hidden.sh"

    # The command reads the session's logs through two variables (#1875): the
    # stream the transform read, and the `projects` directory of the
    # configuration the harness ran under. A value in the caller's
    # environment never reaches it, and a relative `--config-dir` reaches it
    # as an absolute path, because the command runs in a copy.
    printf 'env\n' > "$cot/stub-mode"
    rm -f "$scratch/cmd-env" "$scratch/cmd-env-stream"
    mkdir -p "$scratch/cmd-config/projects"
    (cd "$scratch" && HEADWATER_ORACLE_TRANSCRIPT=/nowhere HEADWATER_ORACLE_HARNESS_LOG=/nowhere \
        sh "$transform" --probe HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged \
        --session env --root "$scratch/engine-only" --workspace "$scratch/cmd-ws" \
        --oracle-tree "$cot" --config-dir cmd-config --produced src/lib.rs \
        < "$scratch/watched-nothing.jsonl" > "$scratch/cmd-env.yaml" 2>"$scratch/cmd-env.err")
    same "a command oracle run with a configuration directory records" "0" "$?"
    if cmp -s "$scratch/watched-nothing.jsonl" "$scratch/cmd-env-stream"; then
        pass "and HEADWATER_ORACLE_TRANSCRIPT names the stream the transform read"
    else
        fail "and HEADWATER_ORACLE_TRANSCRIPT names the stream the transform read" "$(cat "$scratch/cmd-env" "$scratch/cmd-env.err" 2>&1)"
    fi
    same "and HEADWATER_ORACLE_HARNESS_LOG names the projects directory of the configuration, as an absolute path" \
        "$scratch/cmd-config/projects" "$(sed -n 2p "$scratch/cmd-env" 2>/dev/null)"
    printf 'env\n' > "$cot/stub-mode"
    rm -f "$scratch/cmd-env"
    HEADWATER_ORACLE_HARNESS_LOG=/nowhere sh "$transform" --probe HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged \
        --session env2 --root "$scratch/engine-only" --workspace "$scratch/cmd-ws" \
        --oracle-tree "$cot" --produced src/lib.rs \
        < "$scratch/watched-nothing.jsonl" > "$scratch/cmd-env2.yaml" 2>"$scratch/cmd-env2.err"
    same "and with no configuration directory HEADWATER_ORACLE_HARNESS_LOG is empty, not the caller's" \
        "" "$(sed -n 2p "$scratch/cmd-env" 2>/dev/null)"

    # A command that cannot start found no failure, so nothing is written.
    cp "$cot/.headwater/probe.yml" "$scratch/cmd-probe.yml"
    sed 's/command: sh oracle-stub.sh/command: no-such-oracle-command-1834/' "$scratch/cmd-probe.yml" \
        > "$cot/.headwater/probe.yml"
    cmd_run missing pass src/lib.rs
    same "a command that cannot start refuses with exit 5" "5" "$?"
    same "and writes no event" "0" "$(wc -c < "$scratch/cmd-missing.yaml" | tr -d ' ')"
    present "and says the command could not start" "could not start" "$scratch/cmd-missing.err"
    rm -f "$scratch/rec-missing.tsv"
    printf 'pass\n' > "$cot/stub-mode"
    sh "$transform" --probe HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged \
        --session missing2 --root "$scratch/engine-only" --workspace "$scratch/cmd-ws" \
        --oracle-tree "$cot" --produced src/lib.rs --oracle-record "$scratch/rec-missing.tsv" \
        < "$scratch/watched-nothing.jsonl" > /dev/null 2>&1
    same "and a refused transform asked for the record exits 5" "5" "$?"
    same "and writes no oracle record" "absent" "$([ -e "$scratch/rec-missing.tsv" ] && echo present || echo absent)"
    cp "$scratch/cmd-probe.yml" "$cot/.headwater/probe.yml"

    # The oracle record (#1875, clause 4): one line for each declared oracle,
    # `<name>\t<verdict>\t<evidence>`, sorted by name. The verdict restates
    # the findings: `fail` or `not-run` exactly where the oracle's finding is
    # on the artifacts. The evidence of an `exit` oracle is the first line of
    # its standard output alone, and never its standard error.
    rot=$scratch/rec-oracle
    rm -rf "$rot" "$scratch/rec-ws"
    mkdir -p "$rot/src"
    cp -a "$root/.headwater" "$root/CLAUDE.md" "$rot/"
    awk 'skip && /^[^ ]/ { skip = 0 }
    skip { next }
    /^oracles:/ {
        skip = 1
        print "oracles:"
        print "  b:"
        print "    paths: [src/**]"
        print "    command: sh b.sh"
        print "    parser: exit"
        print "  a:"
        print "    paths: [src/**]"
        print "    command: sh a.sh"
        print "    parser: exit"
        print "  c:"
        print "    paths: [nowhere/**]"
        print "    command: sh b.sh"
        print "    parser: exit"
        print "  d:"
        print "    paths: [src/**]"
        print "    command: sh d.sh"
        print "    parser: exit"
        print "  e:"
        print "    paths: [src/**]"
        print "    command: sh e.sh"
        print "    parser: exit"
        next
    } { print }' "$root/.headwater/probe.yml" > "$rot/.headwater/probe.yml"
    printf "printf 'a: fail: x\\\\nsecond line\\\\n'; printf 'noise\\\\n' >&2; exit 1\n" > "$rot/a.sh"
    printf "printf 'b: pass: y\\\\n'\n" > "$rot/b.sh"
    printf "printf 'd:\\\\tz\\\\r\\\\n'\n" > "$rot/d.sh"
    printf "printf 'e: diagnostics only\\\\n' >&2\n" > "$rot/e.sh"
    printf 'pub fn pinned() {}\n' > "$rot/src/lib.rs"
    cp -a "$rot" "$scratch/rec-ws"
    printf 'pub fn changed() {}\n' > "$scratch/rec-ws/src/lib.rs"
    printf '# changed\n' > "$scratch/rec-ws/README.md"
    tab=$(printf '\t')
    # The carriage return of `d` becomes a space, so its evidence ends in one.
    sp=' '
    # rec_run <name> <record or -> <produced...>: one transform run over the
    # record's oracle tree.
    rec_run() {
        r_name=$1 r_record=$2
        shift 2
        r_args=
        for r_path in "$@"; do r_args="$r_args --produced $r_path"; done
        [ "$r_record" = - ] || r_args="$r_args --oracle-record $r_record"
        # shellcheck disable=SC2086
        sh "$transform" --probe HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged \
            --session "$r_name" --root "$scratch/engine-only" --workspace "$scratch/rec-ws" \
            --oracle-tree "$rot" $r_args \
            < "$scratch/watched-nothing.jsonl" > "$scratch/rec-$r_name.yaml" 2>"$scratch/rec-$r_name.err"
    }
    rm -f "$scratch/rec-three.tsv"
    rec_run three "$scratch/rec-three.tsv" src/lib.rs README.md
    same "a session recorded with --oracle-record records" "0" "$?"
    same "and the record holds one line for each declared oracle, sorted, with its verdict and its evidence" \
        "a${tab}fail${tab}a: fail: x
b${tab}pass${tab}b: pass: y
c${tab}not-run${tab}-
d${tab}pass${tab}d: z${sp}
e${tab}pass${tab}-" "$(cat "$scratch/rec-three.tsv" 2>/dev/null)"
    absent "and the command's standard error is in no line" "noise" "$scratch/rec-three.tsv"
    absent "and a command that wrote to standard error alone has no evidence from it" "diagnostics" "$scratch/rec-three.tsv"
    # The invariant: each verdict agrees with the findings on both artifacts.
    same "and \`a\`, which failed, is a finding on both artifacts" "2" \
        "$(grep -cxF -- '        - "command:a"' "$scratch/rec-three.yaml")"
    same "and \`b\`, which passed, is on none" "0" \
        "$(grep -cxF -- '        - "command:b"' "$scratch/rec-three.yaml")"
    same "and \`c\`, which did not run, is on both" "2" \
        "$(grep -cxF -- '        - "command:c"' "$scratch/rec-three.yaml")"
    rm -f "$scratch/rec-none.tsv"
    rec_run none - src/lib.rs README.md
    same "a session recorded with no --oracle-record records" "0" "$?"
    same "and writes no record" "absent" "$([ -e "$scratch/rec-none.tsv" ] && echo present || echo absent)"
    same "and its event is the event of the run with the record" \
        "$(sed 's/three/none/' "$scratch/rec-three.yaml")" "$(cat "$scratch/rec-none.yaml")"
    rm -f "$scratch/rec-empty.tsv"
    rec_run empty "$scratch/rec-empty.tsv"
    same "a session that produced no artifact records" "0" "$?"
    same "and records every declared oracle as not-run" \
        "a${tab}not-run${tab}-
b${tab}not-run${tab}-
c${tab}not-run${tab}-
d${tab}not-run${tab}-
e${tab}not-run${tab}-" "$(cat "$scratch/rec-empty.tsv" 2>/dev/null)"
    # A `cargo-test` oracle's evidence is its last `test result:` line.
    printf 'failcase\n' > "$cot/stub-mode"
    rm -f "$scratch/rec-cargo.tsv"
    sh "$transform" --probe HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged \
        --session cargo --root "$scratch/engine-only" --workspace "$scratch/cmd-ws" \
        --oracle-tree "$cot" --produced src/lib.rs --oracle-record "$scratch/rec-cargo.tsv" \
        < "$scratch/watched-nothing.jsonl" > "$scratch/rec-cargo.yaml" 2>"$scratch/rec-cargo.err"
    same "a cargo-test oracle records its last test result line as its evidence" \
        "fixture-exit${tab}not-run${tab}-
fixture-oracle${tab}fail${tab}test result: FAILED. 1 passed; 1 failed; 0 ignored" \
        "$(cat "$scratch/rec-cargo.tsv" 2>/dev/null)"
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
    # The token counts of the `usage` of the `result` line (#1875), which
    # `campaign.sh` writes to the session's `tokens` file beside `cost`. They
    # are not identity members, so `record.md` does not carry them.
    same "the cache-read tokens are the result's cache_read_input_tokens" \
        "cache_read_tokens: 29244" \
        "$(printf '%s\n' "$provider" | grep '^cache_read_tokens:')"
    same "the cache-write tokens are the result's cache_creation_input_tokens" \
        "cache_write_tokens: 687" \
        "$(printf '%s\n' "$provider" | grep '^cache_write_tokens:')"
    same "the input tokens are the result's uncached input_tokens" \
        "input_tokens: 18" \
        "$(printf '%s\n' "$provider" | grep '^input_tokens:')"

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
    same "a result with no usage counts 0 cache-read tokens rather than null" \
        "cache_read_tokens: 0" \
        "$(printf '%s\n' "$multi" | grep '^cache_read_tokens:')"

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
printf '%s\n' '{"type":"result","result":"merge merge"}' \
    > "$scratch/one-repeated.jsonl"
same "a one-word answer said twice is no answer" \
    "" \
    "$(sh "$driver" --answer-only "$scratch/one-repeated.jsonl" --answers "merge, stamp" 2>/dev/null)"
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
# Spec 15's account of the session, held to the driver's source (#1641).
#
# Spec 15 said the driver "clears the environment, so no token of the host
# reaches the session", while the driver set 12 variables and passed `LANG`
# and `ANTHROPIC_API_KEY` from the host. It said the file system held its
# list "and no other", while the driver also bound the target of
# `/etc/resolv.conf`. Nothing compared the two. So each set is read off the
# driver's code, with the comments stripped, and off spec 15's section, and
# the two must be equal in both directions: a variable or a bind in the
# driver that the spec does not list fails, and so does one the spec lists
# that the driver does not set.
# ---------------------------------------------------------------------------
spec15="$root/docs/spec/15-the-recorder-contract.md"
awk '/^## A session reads its workspace and nothing else of the host/ { on = 1; print; next }
     on && /^## / { exit }
     on' "$spec15" > "$scratch/spec15.section"
# One subsection of that section, from its heading to the next `### `.
spec15_sub() {
    awk -v h="$1" '$0 == h { on = 1; next } on && /^##/ { exit } on' "$scratch/spec15.section"
}
spec15_sub '### What the file system of the session holds' > "$scratch/spec15.fs"
spec15_sub '### The environment of the session' > "$scratch/spec15.env"

# The environment. A name is the first backticked word of a list item.
grep -o -- '--setenv [A-Za-z_][A-Za-z0-9_]*' "$scratch/driver.code" | awk '{ print $2 }' | LC_ALL=C sort -u > "$scratch/env.driver"
sed -n 's/^- `\([A-Za-z_][A-Za-z0-9_]*\)`.*/\1/p' "$scratch/spec15.env" | LC_ALL=C sort -u > "$scratch/env.spec"
env_unlisted=$(LC_ALL=C comm -23 "$scratch/env.driver" "$scratch/env.spec" | tr '\n' ' ')
env_unset=$(LC_ALL=C comm -13 "$scratch/env.driver" "$scratch/env.spec" | tr '\n' ' ')
if [ ! -s "$scratch/env.driver" ]; then
    fail "spec 15 lists every variable the driver sets in the session, and no other" \
        "no \`--setenv\` was read off the driver, so this case reads the wrong file"
elif [ -n "$env_unlisted" ] || [ -n "$env_unset" ]; then
    fail "spec 15 lists every variable the driver sets in the session, and no other" \
        "set by the driver and not in spec 15: ${env_unlisted:-none}; in spec 15 and not set by the driver: ${env_unset:-none}"
else
    pass "spec 15 lists every variable the driver sets in the session, and no other"
fi

# The binds. Each source the driver binds, or the destination of a symlink,
# maps to the phrase of spec 15's list that names it. A source with no row is
# a bind the spec does not list; a row whose phrase is not in the list is an
# item the spec lost; a row whose source the driver no longer binds is an
# item the spec lists and the driver does not bind.
awk '{ for (i = 1; i <= NF; i++) {
         if ($i ~ /^--(ro-|dev-)?bind(-try)?$/) print $(i + 1)
         else if ($i == "--symlink") print $NF
         else if ($i == "--proc" || $i == "--dev" || $i == "--tmpfs") print $i
     } }' "$scratch/driver.code" | LC_ALL=C sort -u > "$scratch/binds.driver"
cat > "$scratch/binds.table" <<'EOF'
/usr	`/usr` and `/etc`, read-only
/etc	`/usr` and `/etc`, read-only
"/$top"	with `/bin`, `/lib` and their siblings as the host has them
"$resolver"	The target of `/etc/resolv.conf`, read-only
--proc	A private `/proc`, `/dev` and `/tmp`
--dev	A private `/proc`, `/dev` and `/tmp`
--tmpfs	A private `/proc`, `/dev` and `/tmp`
"$here"	The workspace, read-write
"$probe_log"	The log directory of the run, read-write
"$config"	A configuration directory of the session, read-write
"$harness"	The harness binary, read-only
"$egress_proxy"	The egress proxy, read-only
"$egress/sock"	the directory of its socket, read-only
EOF
binds_unlisted=
while IFS= read -r src; do
    grep -qF -e "$src	" "$scratch/binds.table" || binds_unlisted="$binds_unlisted $src"
done < "$scratch/binds.driver"
binds_lost=
binds_unbound=
while IFS='	' read -r src phrase; do
    grep -qF -e "$phrase" "$scratch/spec15.fs" || binds_lost="$binds_lost [$phrase]"
    grep -qxF -e "$src" "$scratch/binds.driver" || binds_unbound="$binds_unbound $src"
done < "$scratch/binds.table"
if [ ! -s "$scratch/binds.driver" ]; then
    fail "spec 15 lists every bind of the session's file system, and no other" \
        "no bind was read off the driver, so this case reads the wrong file"
elif [ -n "$binds_unlisted$binds_lost$binds_unbound" ]; then
    fail "spec 15 lists every bind of the session's file system, and no other" \
        "a bind spec 15 does not list:${binds_unlisted:- none}; an item spec 15 lost:${binds_lost:- none}; a row the driver no longer binds:${binds_unbound:- none}"
else
    pass "spec 15 lists every bind of the session's file system, and no other"
fi

# The leak figure: the evaluation's three numbers, each in spec 15 and in the
# driver's comment, and each still in the evaluation they are copied from. A
# figure the evaluation no longer states is one the copies must not keep.
evaluation="$root/docs/evaluations/what-the-counterfactual-campaign-of-2026-09-30-measured-by-component.md"
leak_missing=
for pair in '118 sessions name such a path|118 of 658' \
            '58 of 658 sessions leaked|58 of 658' \
            '17 of the 58 named a copy on the host|17 of those 58'; do
    source_phrase=${pair%%|*}
    copy_phrase=${pair#*|}
    grep -qF -e "$source_phrase" "$evaluation" || leak_missing="$leak_missing [evaluation: $source_phrase]"
    grep -qF -e "$copy_phrase" "$scratch/spec15.section" || leak_missing="$leak_missing [spec 15: $copy_phrase]"
    grep -qF -e "$copy_phrase" "$driver" || leak_missing="$leak_missing [driver: $copy_phrase]"
done
for stale in '51 of 658' '16 of 658'; do
    grep -qF -e "$stale" "$scratch/spec15.section" && leak_missing="$leak_missing [spec 15 still: $stale]"
    grep -qF -e "$stale" "$driver" && leak_missing="$leak_missing [driver still: $stale]"
done
if [ -z "$leak_missing" ]; then
    pass "spec 15 and the driver state the evaluation's leak figures, 118, 58 and 17 of 58"
else
    fail "spec 15 and the driver state the evaluation's leak figures, 118, 58 and 17 of 58" \
        "missing or stale:$leak_missing"
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

# A hidden test that names its probe is sealed out of every workspace and kept
# in the grading tree (#1834). A `cargo-test` oracle can run a test the session
# never sees. The seal removes the test, because its header names the probe,
# and every arm tree is a copy of the sealed tree. The recorder grades in a
# copy of the grading tree, which `tools/probe/grading-tree.sh` builds: the
# sealed tree plus each file an oracle `holds`, and nothing else.
if [ -x "$engine" ]; then
    hidden=engine/tests/hidden-1834.rs
    rm -rf "$scratch/held" "$scratch/held-src" "$scratch/held-arm" "$scratch/held-grading"
    mkdir -p "$scratch/held/docs/probes" "$scratch/held/engine/tests" "$scratch/held/.headwater"
    cp "$root/docs/probes/$tombstone.md" "$scratch/held/docs/probes/"
    printf '// The hidden test of HW-PROBE-%s.\n#[test]\nfn hidden() {}\n' "$tombstone" \
        > "$scratch/held/$hidden"
    awk -v h="$hidden" 'skip && /^[^ ]/ { skip = 0 }
    skip { next }
    /^oracles:/ {
        skip = 1
        print "oracles:"
        print "  hidden-oracle:"
        print "    paths: [engine/**]"
        print "    command: cargo test --test hidden-1834"
        print "    parser: cargo-test"
        print "    holds: [" h "]"
        next
    } { print }' "$root/.headwater/probe.yml" > "$scratch/held/.headwater/probe.yml"
    cp -a "$scratch/held" "$scratch/held-src"
    sh "$root/tools/probe/seal.sh" "$scratch/held" "HW-PROBE-$tombstone" >/dev/null 2>"$scratch/held-seal.err"
    same "seal.sh seals a tree whose oracle holds a hidden test" "0" "$?"
    cp -a "$scratch/held" "$scratch/held-arm"
    if [ -e "$scratch/held/$hidden" ] || [ -e "$scratch/held-arm/$hidden" ]; then
        fail "and the hidden test is in neither the sealed tree nor an arm tree copied from it" \
            "$(ls -R "$scratch/held/engine")"
    else
        pass "and the hidden test is in neither the sealed tree nor an arm tree copied from it"
    fi
    same "grading-tree.sh --holds prints the held path" "$hidden" \
        "$(sh "$root/tools/probe/grading-tree.sh" --holds "$scratch/held" 2>/dev/null)"
    sh "$root/tools/probe/grading-tree.sh" "$scratch/held" "$scratch/held-src" "$scratch/held-grading" \
        >/dev/null 2>"$scratch/held-grading.err"
    same "grading-tree.sh builds the grading tree" "0" "$?"
    if cmp -s "$scratch/held-src/$hidden" "$scratch/held-grading/$hidden"; then
        pass "and the grading tree holds the hidden test"
    else
        fail "and the grading tree holds the hidden test" "$(cat "$scratch/held-grading.err")"
    fi
    if [ -e "$scratch/held-grading/docs/probes" ]; then
        fail "and it restores nothing else the seal removed" "docs/probes is back"
    else
        pass "and it restores nothing else the seal removed"
    fi
    rm "$scratch/held-src/$hidden"
    sh "$root/tools/probe/grading-tree.sh" "$scratch/held" "$scratch/held-src" "$scratch/held-grading-2" \
        >/dev/null 2>"$scratch/held-grading-2.err"
    same "a held path the source does not hold refuses with 3" "3" "$?"
    if [ -e "$scratch/held-grading-2" ] || [ -e "$scratch/held-grading-2.building" ]; then
        fail "and it leaves no grading tree" "$(ls "$scratch")"
    else
        pass "and it leaves no grading tree"
    fi
fi

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
    ( cd "$scratch/corpus" && find docs -type f -name '*.md' | grep -v -e '^docs/probes/' -e '^docs/probe-runs/' -e '^docs/probe-results/' | LC_ALL=C sort ) \
        > "$scratch/corpus-before.txt"
    # Outside `docs/`, every probe of the shelf is named by a declared fold or
    # by one of the three files that state an answer, and by nothing else
    # (#1384). The third is the case table of the harder probes (#1472), which
    # holds the shallow and the sound answer of each. Two more are the hidden
    # oracles of the code set (#1835), a test and a script, each of which
    # names its probe so that the seal removes it. The held arguments and the
    # case files of the probe oracles (#1874) each name their probe for the
    # same reason. A fold dropped from the list, or a new file that names a
    # probe, moves this set, so each entry of the list is held here. The held
    # files are every file under `tools/probe/oracle-args/` and
    # `tools/probe/oracle-cases/` of the archive, read from the two
    # directories and not listed by hand, so a new probe's files join the set
    # and each one that names no probe fails it (#1874.3).
    held_files=$(cd "$scratch/corpus" && find tools/probe/oracle-args tools/probe/oracle-cases -type f 2> /dev/null | LC_ALL=C sort | tr '\n' ' ' | sed 's/ $//')
    [ -n "$held_files" ] || fail "the archive holds the oracles' held arguments and case files" "neither tools/probe/oracle-args/ nor tools/probe/oracle-cases/ holds a file"
    # The hidden tests of the oracles are read the same way, as every
    # `tests/probe_oracle_*.rs` of a crate of the archive (#1874, slice 4b).
    hidden_tests=$(cd "$scratch/corpus" && find engine/crates -path '*/tests/probe_oracle_*.rs' -type f 2> /dev/null | LC_ALL=C sort | tr '\n' ' ' | sed 's/ $//')
    [ -n "$hidden_tests" ] || fail "the archive holds the hidden tests of the oracles" "no crate under engine/crates holds a tests/probe_oracle_*.rs"
    : > "$scratch/outside-naming.txt"
    for shelf_probe in "$root"/docs/probes/*.md; do
        shelf_id=$(awk '/^id: */ { sub(/^id: */, ""); print; exit }' "$shelf_probe")
        [ -n "$shelf_id" ] || continue
        sh "$root/tools/probe/seal.sh" --naming "$scratch/corpus" "$shelf_id" \
            | awk -v here="$scratch/corpus/" 'index($0, here) == 1 { $0 = substr($0, length(here) + 1) } $0 !~ /^docs\//' >> "$scratch/outside-naming.txt"
    done
    same "outside docs/, only the three files that state an answer, the oracles' hidden tests, the figure oracle and the oracles' held arguments and case files name a probe and are not folds" \
        "$(printf '%s\n' .claude/skills/fixtures.sh engine/crates/probe/tests/corpus.rs tools/probe/oracle-figure-added.sh tools/probe/probe-record-fixtures.sh $hidden_tests $held_files | LC_ALL=C sort -u | tr '\n' ' ' | sed 's/ $//')" \
        "$(LC_ALL=C sort -u "$scratch/outside-naming.txt" | tr '\n' ' ' | sed 's/ $//')"

    # The figure oracle of the code set (#1835), in three directions, over a
    # planted figures check: a page with the pin's keys alone, the same page
    # with one key more, and that page under a figures check that refuses.
    # The pin's keys are then held against the page, because a figure added
    # to the page and not to the pin would pass every session of the probe.
    fo=$scratch/figure-oracle
    rm -rf "$fo"
    mkdir -p "$fo/tools/site" "$fo/tools/probe" "$fo/site/proof"
    cp "$root/tools/probe/oracle-figure-added.sh" "$fo/tools/probe/"
    printf '%s\n' 'exit 0' > "$fo/tools/site/refresh-figures.sh"
    sed -n 's/^PIN_KEYS="\(.*\)"$/\1/p' "$root/tools/probe/oracle-figure-added.sh" | tr ' ' '\n' \
        | sed 's/.*/<b data-figure="&"><\/b>/' > "$fo/site/proof/index.html"
    (cd "$fo" && sh tools/probe/oracle-figure-added.sh) >/dev/null 2>&1
    same "the figure oracle fails a page that carries the pin's keys alone" "1" "$?"
    printf '%s\n' '<b data-figure="rules.quiet"></b>' >> "$fo/site/proof/index.html"
    (cd "$fo" && sh tools/probe/oracle-figure-added.sh) >/dev/null 2>&1
    same "the figure oracle passes a page that carries one key the pin's page does not" "0" "$?"
    printf '%s\n' 'exit 1' > "$fo/tools/site/refresh-figures.sh"
    (cd "$fo" && sh tools/probe/oracle-figure-added.sh) >/dev/null 2>&1
    same "the figure oracle fails that page when the figures check refuses it" "1" "$?"
    same "the figure oracle's pin keys are the keys site/proof/index.html carries" \
        "$(grep -o 'data-figure="[^"]*"' "$root/site/proof/index.html" | sed 's/^data-figure="//; s/"$//' | LC_ALL=C sort -u | tr '\n' ' ' | sed 's/ $//')" \
        "$(sed -n 's/^PIN_KEYS="\(.*\)"$/\1/p' "$root/tools/probe/oracle-figure-added.sh" | tr ' ' '\n' | LC_ALL=C sort -u | tr '\n' ' ' | sed 's/ $//')"
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
    for fold in $(cd "$scratch/corpus" && find . -name '*.json' | LC_ALL=C sort); do
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

# The answer keys are the engine's parse of the declaration (#1472, clause 8).
# The hand-written reader this script carried took the flow form alone, so a
# block sequence printed nothing and exited 0, and the seal kept the key in
# every arm. Each declaration below is the checkout's own, with the answer
# keys rewritten.
key_doc=docs/obligations/0198-nothing-states-how-much-of-a-session-s-budget-the-standing-instructions-consume-before-work-starts.md
awk -v probe="$patched" -v key="$key_doc" '
    /^answer_keys:/ { print; print "  " probe ":"; print "    - " key; skip = 1; next }
    skip && !/^ / { skip = 0 }
    !skip { print }' "$root/.headwater/probe.yml" > "$scratch/block-keys.yml"
same "seal.sh --keys reads a four-space block of answer keys, as the engine does" \
    "HW-OBL-0198 $key_doc" \
    "$(HW_PROBE_YML="$scratch/block-keys.yml" sh "$root/tools/probe/seal.sh" --keys "$patched")"
mkdir -p "$scratch/answered-block/docs/obligations" "$scratch/answered-block/.headwater/ids/obligation_record_id"
cp "$root/$key_doc" "$scratch/answered-block/$key_doc"
printf '%s\n' "$key_doc" > "$scratch/answered-block/.headwater/ids/obligation_record_id/HW-OBL-0198"
HW_PROBE_YML="$scratch/block-keys.yml" sh "$root/tools/probe/seal.sh" "$scratch/answered-block" "$patched" \
    >/dev/null 2>"$scratch/answered-block.err"
same "seal.sh seals an answer key declared as a four-space block" "0" "$?"
if [ -e "$scratch/answered-block/$key_doc" ] || [ -e "$scratch/answered-block/.headwater/ids/obligation_record_id/HW-OBL-0198" ]; then
    fail "and it removes the key and its identifier claim" "$(ls -aR "$scratch/answered-block")"
else
    pass "and it removes the key and its identifier claim"
fi

# A declaration the engine refuses is not a probe with no key. The seal runs
# `rm` on each key, so a key that leaves the tree refuses the whole file, and
# every reader of it fails and prints no path.
awk -v probe="$patched" '
    /^answer_keys:/ { print; print "  " probe ": [../outside]"; skip = 1; next }
    skip && !/^ / { skip = 0 }
    !skip { print }' "$root/.headwater/probe.yml" > "$scratch/refused-keys.yml"
HW_PROBE_YML="$scratch/refused-keys.yml" sh "$root/tools/probe/seal.sh" --keys "$patched" \
    >"$scratch/refused-keys.out" 2>"$scratch/refused-keys.err"
keys_status=$?
if [ "$keys_status" -ne 0 ]; then
    pass "seal.sh --keys fails on a declaration the engine refuses"
else
    fail "seal.sh --keys fails on a declaration the engine refuses" "exit 0"
fi
same "and prints no path" "" "$(cat "$scratch/refused-keys.out")"

# The evidence check of the oracle record (#1875, clause 4). An oracle's
# evidence line echoes identifiers and paths, so a line that names an answer
# key would carry the answer out of the sealed session. The check reads the
# answer keys alone and never `leaks:` (parent ruling on #1875, slice 1).
ev() {
    # ev <evidence>: a one-line record, checked for the patched probe.
    printf 'key-oracle\tpass\t%s\n' "$1" > "$scratch/ev-record.tsv"
    sh "$root/tools/probe/seal.sh" --evidence "$patched" "$scratch/ev-record.tsv" > "$scratch/ev.out" 2> "$scratch/ev.err"
}
ev "the gap is already HW-OBL-0198"
same "seal.sh --evidence refuses an evidence line that holds the key's identifier with exit 1" "1" "$?"
same "and prints the probe, the oracle and the key" "evidence $patched key-oracle HW-OBL-0198" "$(cat "$scratch/ev.out")"
ev "see $key_doc"
same "an evidence line that holds the key's path exits 1" "1" "$?"
ev "the record ${key_doc##*/} is there"
same "an evidence line that holds the key's <slug>.md exits 1" "1" "$?"
same "and names the key by its identifier" "evidence $patched key-oracle HW-OBL-0198" "$(cat "$scratch/ev.out")"
ev "key-oracle: pass: the claim is recorded"
same "a clean evidence line exits 0" "0" "$?"
same "and prints nothing" "" "$(cat "$scratch/ev.out")"
printf 'a-oracle\tpass\tclean\nb-oracle\tfail\tHW-OBL-0198 and %s\n' "$key_doc" > "$scratch/ev-record.tsv"
sh "$root/tools/probe/seal.sh" --evidence "$patched" "$scratch/ev-record.tsv" > "$scratch/ev.out" 2> "$scratch/ev.err"
same "a record of two lines names only the oracle whose evidence holds the key, once" \
    "evidence $patched b-oracle HW-OBL-0198" "$(cat "$scratch/ev.out")"
printf 'a-oracle\tpass\tHW-OBL-0198 is not the verdict\n' > "$scratch/ev-record.tsv"
sh "$root/tools/probe/seal.sh" --evidence HW-PROBE-$tombstone "$scratch/ev-record.tsv" > "$scratch/ev.out" 2> "$scratch/ev.err"
same "a probe that declares no answer key passes any evidence" "0" "$?"
awk -v probe="$patched" '
    /^leaks:/ { print; print "  " probe ": [HW-DR-0052]"; next }
    { print }' "$root/.headwater/probe.yml" > "$scratch/ev-leaks.yml"
printf 'key-oracle\tpass\tHW-DR-0052 decides it\n' > "$scratch/ev-record.tsv"
HW_PROBE_YML="$scratch/ev-leaks.yml" sh "$root/tools/probe/seal.sh" --evidence "$patched" "$scratch/ev-record.tsv" \
    > "$scratch/ev.out" 2> "$scratch/ev.err"
same "an evidence line that holds a leak string of the probe and no key exits 0, because leaks are not read" "0" "$?"
HW_PROBE_YML="$scratch/refused-keys.yml" sh "$root/tools/probe/seal.sh" --evidence "$patched" "$scratch/ev-record.tsv" \
    > "$scratch/ev.out" 2> "$scratch/ev.err"
same "seal.sh --evidence exits 8 on a declaration the engine refuses" "8" "$?"
same "and prints no hit" "" "$(cat "$scratch/ev.out")"
sh "$root/tools/probe/seal.sh" --evidence "$patched" "$scratch/no-such-record.tsv" > "$scratch/ev.out" 2> "$scratch/ev.err"
same "seal.sh --evidence exits 2 on a record that is not there" "2" "$?"
sh "$root/tools/probe/seal.sh" --evidence "$patched" > "$scratch/ev.out" 2> "$scratch/ev.err"
same "and 2 with no record named" "2" "$?"
# A key whose slug is generic is found by its path and its identifier, and
# never by its file name alone, which every shelf's README would share.
awk -v probe="$patched" '
    /^answer_keys:/ { print; print "  " probe ": [docs/spec/glossary.md]"; skip = 1; next }
    skip && !/^ / { skip = 0 }
    !skip { print }' "$root/.headwater/probe.yml" > "$scratch/ev-generic.yml"
printf 'key-oracle\tpass\tsee docs/spec/glossary.md\n' > "$scratch/ev-record.tsv"
HW_PROBE_YML="$scratch/ev-generic.yml" sh "$root/tools/probe/seal.sh" --evidence "$patched" "$scratch/ev-record.tsv" \
    > "$scratch/ev.out" 2> "$scratch/ev.err"
same "an evidence line that holds the path of a key with a generic slug exits 1" "1" "$?"
same "and names the key by its identifier" "evidence $patched key-oracle HW-SPEC-glossary" "$(cat "$scratch/ev.out")"
printf 'key-oracle\tpass\tthe glossary.md of another shelf\n' > "$scratch/ev-record.tsv"
HW_PROBE_YML="$scratch/ev-generic.yml" sh "$root/tools/probe/seal.sh" --evidence "$patched" "$scratch/ev-record.tsv" \
    > "$scratch/ev.out" 2> "$scratch/ev.err"
same "and an evidence line that holds only its generic file name exits 0" "0" "$?"
HW_PROBE_YML="$scratch/refused-keys.yml" sh "$root/tools/probe/seal.sh" --folds \
    >"$scratch/refused-keys.out" 2>"$scratch/refused-keys.err"
keys_status=$?
if [ "$keys_status" -ne 0 ]; then
    pass "and seal.sh --folds fails on the same declaration"
else
    fail "and seal.sh --folds fails on the same declaration" "exit 0"
fi
same "and prints no path" "" "$(cat "$scratch/refused-keys.out")"
mkdir -p "$scratch/answered-refused/docs/obligations"
cp "$root/$key_doc" "$scratch/answered-refused/$key_doc"
HW_PROBE_YML="$scratch/refused-keys.yml" sh "$root/tools/probe/seal.sh" "$scratch/answered-refused" "$patched" \
    >/dev/null 2>"$scratch/answered-refused.err"
same "and the seal stops at 8 on the same declaration" "8" "$?"

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
printf -- '- X%s is another record\n' "HW-OBL-0013" > "$scratch/named/docs/spec/13.md"
PATH="$scratch/no-harness" "$shell" "$driver" --probe "HW-PROBE-$tombstone" --session x \
    --task-file "$scratch/task.md" --workspace "$scratch/named" \
    >/dev/null 2>"$scratch/named3.err"
same "and so does one that holds it after a letter" "3" "$?"

# A file outside `docs/` that names the probe (#1384). Two committed files do
# that and state an answer: `.claude/skills/fixtures.sh` asserts the status
# probe's answer, and `probe-record-fixtures.sh`, this file, names the
# tombstone probe beside its record. Before, the seal and the guard read
# `docs/` alone, so a session could read either one. Now each reads the whole
# workspace, less the files `folds:` in `.headwater/probe.yml` declares, which
# name a probe by path or title and state no answer.
mkdir -p "$scratch/outside/.claude/skills" "$scratch/outside/.headwater" "$scratch/outside/docs/spec"
printf '# docs/probes/%s.md answers `absent`\n' "$tombstone" > "$scratch/outside/.claude/skills/x.sh"
printf -- '- docs/probes/%s.md\n' "$tombstone" > "$scratch/outside/.headwater/nav.yml"
printf 'a spec that names no probe\n' > "$scratch/outside/docs/spec/01.md"
PATH="$scratch/no-harness" "$shell" "$driver" --probe "HW-PROBE-$tombstone" --session x \
    --task-file "$scratch/task.md" --workspace "$scratch/outside" \
    >/dev/null 2>"$scratch/outside.err"
same "the driver refuses a workspace whose file outside docs/ names the probe" "9" "$?"
present "and it names that file" ".claude/skills/x.sh" "$scratch/outside.err"
sh "$root/tools/probe/seal.sh" "$scratch/outside" "HW-PROBE-$tombstone" >"$scratch/outside.out" 2>&1
same "seal.sh seals a workspace with a file outside docs/ that names the probe" "0" "$?"
if [ -e "$scratch/outside/.claude/skills/x.sh" ]; then
    fail "and it removes that file" "$(cat "$scratch/outside.out")"
else
    pass "and it removes that file"
fi
if [ -f "$scratch/outside/.headwater/nav.yml" ] && [ -f "$scratch/outside/docs/spec/01.md" ]; then
    pass "and it keeps a declared fold that names the probe, and a file that names none"
else
    fail "and it keeps a declared fold that names the probe, and a file that names none" "$(ls -aR "$scratch/outside")"
fi
PATH="$scratch/no-harness" "$shell" "$driver" --probe "HW-PROBE-$tombstone" --session x \
    --task-file "$scratch/task.md" --workspace "$scratch/outside" \
    >/dev/null 2>"$scratch/outside2.err"
same "and the driver passes the guard over the sealed tree" "3" "$?"
# A fold is declared by its path, and the same bytes at an undeclared path
# are refused: the list names files, not a kind of file.
mkdir -p "$scratch/outside/engine"
cp "$scratch/outside/.headwater/nav.yml" "$scratch/outside/engine/nav.yml"
PATH="$scratch/no-harness" "$shell" "$driver" --probe "HW-PROBE-$tombstone" --session x \
    --task-file "$scratch/task.md" --workspace "$scratch/outside" \
    >/dev/null 2>"$scratch/outside3.err"
same "the driver refuses a copy of a declared fold at a path the list does not name" "9" "$?"
present "and it names the copy" "engine/nav.yml" "$scratch/outside3.err"

# An undeclared file under `.headwater/`, the directory most folds live in,
# is not a fold by where it is (verify round 1: a filter that passed every
# path under `.headwater/` kept every case green).
mkdir -p "$scratch/dotdir/.headwater"
printf 'HW-PROBE-%s: absent\n' "$tombstone" > "$scratch/dotdir/.headwater/answers.yml"
PATH="$scratch/no-harness" "$shell" "$driver" --probe "HW-PROBE-$tombstone" --session x \
    --task-file "$scratch/task.md" --workspace "$scratch/dotdir" \
    >/dev/null 2>"$scratch/dotdir.err"
same "the driver refuses an undeclared file under .headwater/ that names the probe" "9" "$?"
sh "$root/tools/probe/seal.sh" "$scratch/dotdir" "HW-PROBE-$tombstone" >/dev/null 2>&1
if [ -e "$scratch/dotdir/.headwater/answers.yml" ]; then
    fail "and seal.sh removes it" "$(ls -a "$scratch/dotdir/.headwater")"
else
    pass "and seal.sh removes it"
fi

# A fold is its whole path from the workspace root, not a suffix of one: a
# nested `.headwater/nav.yml` is not the fold (verify round 2).
mkdir -p "$scratch/nested/engine/.headwater"
printf -- '- docs/probes/%s.md\n' "$tombstone" > "$scratch/nested/engine/.headwater/nav.yml"
PATH="$scratch/no-harness" "$shell" "$driver" --probe "HW-PROBE-$tombstone" --session x \
    --task-file "$scratch/task.md" --workspace "$scratch/nested" \
    >/dev/null 2>"$scratch/nested.err"
same "the driver refuses a nested copy whose path ends with a declared fold" "9" "$?"

# The search reads a binary file, so a file that `grep -I` would pass over
# is refused and removed rather than kept (verify round 2).
mkdir -p "$scratch/binary/assets"
printf 'a\000HW-PROBE-%s absent\n' "$tombstone" > "$scratch/binary/assets/answer.bin"
PATH="$scratch/no-harness" "$shell" "$driver" --probe "HW-PROBE-$tombstone" --session x \
    --task-file "$scratch/task.md" --workspace "$scratch/binary" \
    >/dev/null 2>"$scratch/binary.err"
same "the driver refuses a binary file that names the probe" "9" "$?"
sh "$root/tools/probe/seal.sh" "$scratch/binary" "HW-PROBE-$tombstone" >/dev/null 2>&1
if [ -e "$scratch/binary/assets/answer.bin" ]; then
    fail "and seal.sh removes it" "the binary file survived the seal"
else
    pass "and seal.sh removes it"
fi

# A file the search cannot read fails the search closed (verify round 2).
# `grep` exits 2 on it, and before, that status threw every match away: the
# guard passed a workspace whose `.claude/skills/x.sh` states the answer, and
# the seal removed nothing. A root user reads a file of mode 000, so the case
# is skipped there.
mkdir -p "$scratch/unreadable/.claude/skills" "$scratch/unreadable/notes"
printf '# docs/probes/%s.md answers `absent`\n' "$tombstone" > "$scratch/unreadable/.claude/skills/x.sh"
printf 'names no probe\n' > "$scratch/unreadable/notes/locked.txt"
chmod 000 "$scratch/unreadable/notes/locked.txt"
if [ -r "$scratch/unreadable/notes/locked.txt" ]; then
    echo "skip the unreadable-file cases: this user reads a file of mode 000"
else
    PATH="$scratch/no-harness" "$shell" "$driver" --probe "HW-PROBE-$tombstone" --session x \
        --task-file "$scratch/task.md" --workspace "$scratch/unreadable" \
        >/dev/null 2>"$scratch/unreadable.err"
    same "the driver refuses a workspace holding a file the search cannot read" "9" "$?"
    present "and it says the files that name the probe could not be read" "could not be read" "$scratch/unreadable.err"
    sh "$root/tools/probe/seal.sh" "$scratch/unreadable" "HW-PROBE-$tombstone" >/dev/null 2>"$scratch/unreadable-seal.err"
    same "seal.sh stops at 8 on a file its search cannot read" "8" "$?"
fi
chmod 644 "$scratch/unreadable/notes/locked.txt"

# A path is one name, whatever it holds: a space is not a separator, and a
# `*` is not a pattern (verify round 1: a file named `*` made the seal remove
# every file of its directory, and a split on spaces kept the file).
mkdir -p "$scratch/odd/tools/probe" "$scratch/odd/.claude"
printf 'see HW-PROBE-%s\n' "$tombstone" > "$scratch/odd/tools/probe/*"
printf 'names no probe\n' > "$scratch/odd/tools/probe/kept.sh"
printf 'see HW-PROBE-%s\n' "$tombstone" > "$scratch/odd/.claude/my notes.sh"
sh "$root/tools/probe/seal.sh" "$scratch/odd" "HW-PROBE-$tombstone" >"$scratch/odd.out" 2>&1
same "seal.sh seals a workspace whose paths hold a space and a star" "0" "$?"
if [ -e "$scratch/odd/tools/probe/*" ] || [ -e "$scratch/odd/.claude/my notes.sh" ]; then
    fail "and it removes each file that names the probe" "$(ls -aR "$scratch/odd")"
else
    pass "and it removes each file that names the probe"
fi
if [ -f "$scratch/odd/tools/probe/kept.sh" ]; then
    pass "and it keeps the file beside the star, which names no probe"
else
    fail "and it keeps the file beside the star, which names no probe" "$(cat "$scratch/odd.out")"
fi
# The same under `docs/`, and in the line removal: a record named `*`, and a
# file named `*` that links a deleted record, beside a binary file that the
# removal must not rewrite.
mkdir -p "$scratch/odddocs/docs/notes" "$scratch/odddocs/docs/spec" "$scratch/odddocs/assets"
printf -- '---\nid: HW-OBL-0013\n---\nsee docs/probes/%s.md\n' "$tombstone" > "$scratch/odddocs/docs/notes/*"
printf 'names no probe\n' > "$scratch/odddocs/docs/notes/kept.md"
printf -- '- HW-OBL-0013 the tombstone gap\n' > "$scratch/odddocs/assets/*"
printf 'a\000b\n' > "$scratch/odddocs/assets/blob.bin"
cp "$scratch/odddocs/assets/blob.bin" "$scratch/blob.expected"
sh "$root/tools/probe/seal.sh" "$scratch/odddocs" "HW-PROBE-$tombstone" >"$scratch/odddocs.out" 2>&1
same "seal.sh seals a workspace whose docs/ holds a record named with a star" "0" "$?"
if [ -f "$scratch/odddocs/docs/notes/kept.md" ] && [ ! -e "$scratch/odddocs/docs/notes/*" ]; then
    pass "and it removes that record and keeps the document beside it"
else
    fail "and it removes that record and keeps the document beside it" "$(ls -aR "$scratch/odddocs/docs")"
fi
if cmp -s "$scratch/blob.expected" "$scratch/odddocs/assets/blob.bin"; then
    pass "and the line removal leaves the binary file beside a file named with a star"
else
    fail "and the line removal leaves the binary file beside a file named with a star" "$(od -c "$scratch/odddocs/assets/blob.bin" | head -2 | tr '\n' ' ')"
fi

# The list itself. Each path it declares is a committed file of this checkout,
# so a fold that moved is found here and not in a refused campaign. The two
# files that state an answer are not on it.
folds=$(sh "$root/tools/probe/seal.sh" --folds)
same "seal.sh --folds reads the declared folds" "0" "$?"
missing=""
for fold in $folds; do
    [ -e "$root/$fold" ] || missing="$missing $fold"
done
same "and every declared fold is a file of this checkout" "" "$missing"
answering=""
for answer in .claude/skills/fixtures.sh tools/probe/probe-record-fixtures.sh; do
    if printf '%s\n' "$folds" | grep -qxF -e "$answer"; then
        answering="$answering $answer"
    fi
done
same "and no file that states a probe's answer is on it" "" "$answering"
# The engine reads a declaration whole or refuses it, so this one declares a
# tier.
printf '%s\n' 'instrument:' '  - docs/probes' '' 'tiers:' '  regression:' '    budget_cents: 1' \
    '    session_cost_cents: 1' '    repetitions: 1' '    arms: [present]' > "$scratch/no-folds.yml"
HW_PROBE_YML="$scratch/no-folds.yml" sh "$root/tools/probe/seal.sh" --folds \
    >"$scratch/no-folds.out" 2>"$scratch/no-folds.err"
same "a declaration with no folds reads" "0" "$?"
same "and it lists none, so the guard refuses every file outside docs/ that names the probe" \
    "" "$(cat "$scratch/no-folds.out")"

# The folds are the engine's parse of the declaration too (#1472, clause 8).
# The hand-written reader matched two spaces alone, so a block at four spaces
# printed nothing, the seal removed every fold, and the guard refused every
# workspace.
awk '
    /^folds:/ { on = 1; print; next }
    on && /^  - / { print "  " $0; next }
    on && !/^ / { on = 0 }
    { print }' "$root/.headwater/probe.yml" > "$scratch/indented-folds.yml"
same "seal.sh --folds reads a four-space block of folds, as the engine does" \
    "$folds" "$(HW_PROBE_YML="$scratch/indented-folds.yml" sh "$root/tools/probe/seal.sh" --folds)"
awk '
    /^folds:/ { print; print "  - ../outside"; next }
    { print }' "$root/.headwater/probe.yml" > "$scratch/refused-folds.yml"
HW_PROBE_YML="$scratch/refused-folds.yml" sh "$root/tools/probe/seal.sh" --folds \
    >"$scratch/refused-folds.out" 2>"$scratch/refused-folds.err"
folds_status=$?
if [ "$folds_status" -ne 0 ]; then
    pass "seal.sh --folds fails on a fold that leaves the tree"
else
    fail "seal.sh --folds fails on a fold that leaves the tree" "exit 0"
fi
same "and prints no path" "" "$(cat "$scratch/refused-folds.out")"

# The guard of `probe-record.sh` names no declaration, so the seal reads the
# checkout's own through `--root`, and that path refuses too. A copy of the
# script under a scratch root, with the built engine linked in and an unsafe
# declaration in place, stands for a checkout whose declaration the engine
# refuses. `HW_PROBE_YML` is unset for both runs.
seal_engine=$(hw_resolve_engine_bin "$root") || seal_engine=
if [ -n "$seal_engine" ]; then
    mkdir -p "$scratch/refusing-root/tools/probe" "$scratch/refusing-root/.headwater" \
        "$scratch/refusing-root/engine/target/dev-release"
    cp "$root/tools/probe/seal.sh" "$scratch/refusing-root/tools/probe/seal.sh"
    ln -s "$seal_engine" "$scratch/refusing-root/engine/target/dev-release/headwater"
    cp "$scratch/refused-keys.yml" "$scratch/refusing-root/.headwater/probe.yml"
    for word in --folds "--keys $patched"; do
        # shellcheck disable=SC2086
        env -u HW_PROBE_YML sh "$scratch/refusing-root/tools/probe/seal.sh" $word \
            >"$scratch/refusing-root.out" 2>"$scratch/refusing-root.err"
        refusing_status=$?
        case "$word" in
            --keys*) same "seal.sh --keys exits 8 on a checkout declaration the engine refuses" "8" "$refusing_status" ;;
            *)
                if [ "$refusing_status" -ne 0 ]; then
                    pass "seal.sh --folds fails on a checkout declaration the engine refuses"
                else
                    fail "seal.sh --folds fails on a checkout declaration the engine refuses" "exit 0"
                fi
                ;;
        esac
        same "and prints no path" "" "$(cat "$scratch/refusing-root.out")"
    done
else
    echo "skip the checkout-declaration refusal: no built engine under $root/engine/target"
fi

# With no built engine there is nothing to read the declaration with, and the
# seal refuses rather than print an empty list. A copy of the script under a
# root with no `engine/target` stands for a checkout that never built.
mkdir -p "$scratch/unbuilt-seal/tools/probe"
cp "$root/tools/probe/seal.sh" "$scratch/unbuilt-seal/tools/probe/seal.sh"
cp "$root/.headwater/probe.yml" "$scratch/unbuilt-seal.yml"
for word in --folds "--keys $patched"; do
    # shellcheck disable=SC2086
    HW_PROBE_YML="$scratch/unbuilt-seal.yml" sh "$scratch/unbuilt-seal/tools/probe/seal.sh" $word \
        >"$scratch/unbuilt-seal.out" 2>"$scratch/unbuilt-seal.err"
    unbuilt_status=$?
    if [ "$unbuilt_status" -ne 0 ]; then
        pass "seal.sh ${word%% *} with no built engine exits nonzero"
    else
        fail "seal.sh ${word%% *} with no built engine exits nonzero" "exit 0"
    fi
    same "and prints no path" "" "$(cat "$scratch/unbuilt-seal.out")"
    present "and it names the build" "Build it first" "$scratch/unbuilt-seal.err"
done

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
    # Every stub below runs inside the driver's confinement (#1467), so it can
    # write only under the workspace and the probe log directory. A stub that
    # records its arguments writes them to the log directory, which the driver
    # binds read-write and names in `HEADWATER_SHADOW_LOG_DIR`.
    HEADWATER_PROBE_LOG_DIR=$scratch/probe-log
    export HEADWATER_PROBE_LOG_DIR
    args=$scratch/probe-log/claude-args

    # The confinement hides this checkout from the session, so the live case
    # runs the hook from a workspace that carries its own engine, as
    # `campaign.sh` builds one. `campaign.sh` archives the whole tree, so its
    # workspace also holds `tools/repo/resolve-engine.sh`, which `hw_engine`
    # in `lib.sh` delegates to since #990. This one copies it too, or the hook
    # finds no engine and the case reads a silent hook.
    mkdir -p "$scratch/live-ws/.claude" "$scratch/live-ws/engine/target/dev-release" "$scratch/live-ws/tools/repo"
    cp -a "$root/.claude/hooks" "$scratch/live-ws/.claude/hooks"
    cp "$root/tools/repo/resolve-engine.sh" "$scratch/live-ws/tools/repo/resolve-engine.sh"
    cp "$engine" "$scratch/live-ws/engine/target/dev-release/headwater"
    cat > "$scratch/bin/claude" <<'STUB'
#!/bin/sh
# A stub harness. It submits one prompt through the real intent hook, then
# writes the two stream lines the driver's derivations read.
printf '{"hook_event_name":"UserPromptSubmit","session_id":"stub-session","cwd":"%s","user_input":"what does a check know about the front matter of a document"}' "$PWD" \
    | CLAUDE_PROJECT_DIR="$PWD" sh "$PWD/.claude/hooks/intent.sh" >/dev/null 2>&1
printf '%s\n' '{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s9"}'
printf '%s\n' '{"type":"result","subtype":"success","total_cost_usd":0.01,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}'
STUB
    chmod +x "$scratch/bin/claude"

    HEADWATER_MODEL_DIR="$scratch/nomodel" \
        PATH="$scratch/bin:$PATH" sh "$driver" --probe "HW-PROBE-$tombstone" \
        --session fixture-live --task-file "$scratch/task.md" \
        --workspace "$scratch/live-ws" > "$scratch/live-run.md" 2>"$scratch/live-run.err"
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
    # The stub first asks the proxy for a host off its list, as a harness
    # that needs an unlisted host does, and the driver names that refusal
    # (#1467, clause 5).
    cat > "$scratch/bin/claude" <<'STUB'
#!/bin/sh
curl -s -p -x "$HTTPS_PROXY" --max-time 3 -o /dev/null https://github.com/ 2>/dev/null
exit 7
STUB
    chmod +x "$scratch/bin/claude"
    PATH="$scratch/bin:$PATH" sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-fails \
        --task-file "$scratch/task.md" --workspace "$scratch/ws" \
        >/dev/null 2>"$scratch/failed-run.err"
    same "a harness that exits 7 makes the driver exit 10, never 7" "10" "$?"
    present "and the driver names the harness's own status" \
        "the harness exited 7" "$scratch/failed-run.err"
    present "and names the host the egress proxy refused it" \
        "the egress proxy allowed no connection, and refused 1 connection to \`github.com:443\`" "$scratch/failed-run.err"

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
printf '%s\n' "\$@" > "\$HEADWATER_SHADOW_LOG_DIR/claude-args"
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
        "$(awk 'prev == "--max-turns" { print; exit } { prev = $0 }' "$args")"

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
printf '%s\n' "\$@" > "\$HEADWATER_SHADOW_LOG_DIR/claude-args"
printf '%s\n' '{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s13"}'
printf '%s\n' '{"type":"result","subtype":"success","result":"withheld","total_cost_usd":0.01,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}'
STUB
    chmod +x "$scratch/bin/claude"
    rm -f "$args"
    # One repetition keeps the plan small, and the ceiling is not this case.
    PATH="$scratch/bin:$PATH" sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-tier-cap \
        --tier campaign --category sufficiency --repetitions 1 \
        --task-file "$scratch/task.md" --workspace "$scratch/ws" \
        >/dev/null 2>"$scratch/tier-cap.err"
    same "a campaign session runs under the tier's declared cap" "0" "$?"
    same "and the cap the plan declares reaches the harness" "80" \
        "$(awk 'prev == "--max-turns" { print; exit } { prev = $0 }' "$args" 2>/dev/null)"

    # The MCP servers and the arm's delta (#1472). Every arm runs with
    # `--strict-mcp-config`, so no server of the recording host loads. The
    # `mcp` arm's workspace holds `.mcp.json`, and the driver passes it by
    # name, because a project server does not load under `claude -p` without
    # approval. The transcript states the arm's delta and the MCP calls.
    same "every session runs with --strict-mcp-config" "1" \
        "$(grep -cx -- '--strict-mcp-config' "$args" 2>/dev/null)"
    same "and a workspace with no .mcp.json names no server" "0" \
        "$(grep -cx -- '--mcp-config' "$args" 2>/dev/null)"
    cat > "$scratch/bin/claude" <<STUB
#!/bin/sh
printf '%s\n' "\$@" > "\$HEADWATER_SHADOW_LOG_DIR/claude-args"
printf '%s\n' '{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s16"}'
printf '%s\n' '{"type":"assistant","message":{"id":"m1","content":[{"type":"tool_use","id":"t1","name":"mcp__headwater__route","input":{"task":"x"}}]}}'
printf '%s\n' '{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t1","content":"none"}]}}'
printf '%s\n' '{"type":"assistant","message":{"id":"m2","content":[{"type":"tool_use","id":"t2","name":"Read","input":{"file_path":"CLAUDE.md"}}]}}'
printf '%s\n' '{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t2","content":"none"}]}}'
printf '%s\n' '{"type":"result","subtype":"success","result":"withheld","total_cost_usd":0.01,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}'
STUB
    chmod +x "$scratch/bin/claude"
    cp "$root/tools/probe/arms/mcp/.mcp.json" "$scratch/ws/.mcp.json"
    rm -f "$args"
    PATH="$scratch/bin:$PATH" sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-mcp \
        --tier campaign --arm mcp --category sufficiency --repetitions 1 \
        --task-file "$scratch/task.md" --workspace "$scratch/ws" \
        >"$scratch/mcp-arm.md" 2>"$scratch/mcp-arm.err"
    same "a session of the mcp arm records" "0" "$?"
    rm -f "$scratch/ws/.mcp.json"
    same "and the driver passes the workspace's .mcp.json to the harness" \
        "$(cd "$scratch/ws" && pwd -P)/.mcp.json" \
        "$(awk 'prev == "--mcp-config" { print; exit } { prev = $0 }' "$args" 2>/dev/null)"
    same "and still passes --strict-mcp-config" "1" \
        "$(grep -cx -- '--strict-mcp-config' "$args" 2>/dev/null)"
    # `--mcp-config` takes one or more values, so the token after its path
    # must be a flag the driver always passes (verify round 1). An optional
    # one, such as `--model` or `--max-turns`, leaves the task text next to
    # the path whenever it is not set, and the harness reads the task as a
    # second config path.
    same "a flag the driver always passes follows the config path" "--strict-mcp-config" \
        "$(awk 'prev == "--mcp-config" { getline; print; exit } { prev = $0 }' "$args" 2>/dev/null)"
    present "the transcript counts the calls to an MCP tool, and not the Read" \
        "The session made 1 call to a tool of an MCP server." "$scratch/mcp-arm.md"
    present "and states the arm's delta from the declaration" \
        'The `mcp` arm is the present tree with `.mcp.json` added.' "$scratch/mcp-arm.md"
    PATH="$scratch/bin:$PATH" sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-no-hook \
        --tier campaign --arm no-hook --category sufficiency --repetitions 1 \
        --task-file "$scratch/task.md" --workspace "$scratch/ws" \
        >"$scratch/no-hook-arm.md" 2>"$scratch/no-hook-arm.err"
    present "a no-hook transcript states what the arm removed" \
        'The `no-hook` arm is the present tree less `.claude/hooks/intent.sh`.' "$scratch/no-hook-arm.md"
    PATH="$scratch/bin:$PATH" sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-absent-delta \
        --tier campaign --arm absent --category sufficiency --repetitions 1 \
        --task-file "$scratch/task.md" --workspace "$scratch/ws" \
        >"$scratch/absent-arm.md" 2>"$scratch/absent-arm.err"
    present "an absent transcript states the tier's ablation" \
        'The `absent` arm is the present tree less `CLAUDE.md`, `.claude`, `.githooks`, `.headwater`.' "$scratch/absent-arm.md"
    # An arm of the additive campaign is plain code with layers added (#1775),
    # so its transcript names the layers its scenario holds, from
    # `scenarios:`, and scenario 0 is plain code alone. Neither is a delta
    # from the present tree.
    for n in 3 0; do
        PATH="$scratch/bin:$PATH" sh "$driver" --probe HW-PROBE-a-session-adds-a-count-of-findings-by-severity-to-the-json-report-of-the-check \
            --session "fixture-scenario-$n" --tier additive --arm "scenario-$n" --set tuning --repetitions 18 \
            --task-file "$scratch/task.md" --workspace "$scratch/ws" \
            >"$scratch/scenario-$n-arm.md" 2>"$scratch/scenario-$n-arm.err"
    done
    present "a scenario-3 transcript names the layers its scenario adds to plain code" \
        'The `scenario-3` arm is plain code with the layers `docs`, `engine`, `skills` added: the present tree less the probe instrument and every layer it does not hold.' "$scratch/scenario-3-arm.md"
    present "a scenario-0 transcript states plain code alone" \
        'The `scenario-0` arm is plain code: the present tree less the probe instrument and the path of every layer.' "$scratch/scenario-0-arm.md"

    # A delta the declaration does not state is said to be unread, and never
    # printed as the present tree (verify round 1). The engine refuses a
    # declaration whole when a tier runs a component arm with no delta, and
    # then no instrument is read and the driver refuses before any session
    # (the case after this one). So here the tier no longer runs the arm, the
    # file reads, and the delta of the arm alone is refused.
    grep -v '^      no-hook: ' "$root/.headwater/probe.yml" \
        | sed 's/^    arms: \[present, absent, no-hook, /    arms: [present, absent, /' > "$scratch/no-delta.yml"
    HW_PROBE_YML="$scratch/no-delta.yml" PATH="$scratch/bin:$PATH" sh "$driver" --probe "HW-PROBE-$tombstone" \
        --session fixture-no-delta --tier campaign --arm no-hook --category sufficiency --repetitions 1 \
        --task-file "$scratch/task.md" --workspace "$scratch/ws" \
        >"$scratch/no-delta.md" 2>"$scratch/no-delta.err"
    present "a transcript whose arm's delta cannot be read says so" \
        'The delta of the `no-hook` arm of the `campaign` tier could not be read' "$scratch/no-delta.md"
    absent "and does not call the arm the present tree" "arm is the present tree" "$scratch/no-delta.md"

    # A declaration the engine refuses whole gives no instrument to clear, so
    # the driver fails closed with 8 before any session (#1472, clause 8).
    grep -v '^      no-hook: ' "$root/.headwater/probe.yml" > "$scratch/refused-delta.yml"
    rm -f "$scratch/harness-ran"
    HW_PROBE_YML="$scratch/refused-delta.yml" PATH="$scratch/bin:$PATH" sh "$driver" --probe "HW-PROBE-$tombstone" \
        --session fixture-refused-delta --tier campaign --arm no-hook --category sufficiency --repetitions 1 \
        --task-file "$scratch/task.md" --workspace "$scratch/ws" \
        >"$scratch/refused-delta.md" 2>"$scratch/refused-delta.err"
    same "a declaration the engine refuses stops the driver with 8" "8" "$?"
    present "and it says the instrument could not be read" \
        "the instrument of the probe declaration could not be read" "$scratch/refused-delta.err"

    # A stream that does not parse is uncounted, never "0 calls": two MCP
    # calls and a cut last line.
    cat > "$scratch/bin/claude" <<STUB
#!/bin/sh
printf '%s\n' "\$@" > "\$HEADWATER_SHADOW_LOG_DIR/claude-args"
printf '%s\n' '{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s17"}'
printf '%s\n' '{"type":"assistant","message":{"id":"m1","content":[{"type":"tool_use","id":"t1","name":"mcp__headwater__route","input":{"task":"x"}}]}}'
printf '%s\n' '{"type":"assistant","message":{"id":"m2","content":[{"type":"tool_use","id":"t2","name":"mcp__headwater__explain","input":{"path":"x"}}]}}'
printf '%s\n' '{"type":"result","subtype":"success","result":"withheld","total_cost_usd":0.01,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}'
printf '%s' '{"type":"assistant","message":{"id":"m3","content":[{"type":"tool_'
STUB
    chmod +x "$scratch/bin/claude"
    PATH="$scratch/bin:$PATH" sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-cut \
        --tier campaign --arm no-hook --category sufficiency --repetitions 1 \
        --task-file "$scratch/task.md" --workspace "$scratch/ws" \
        >"$scratch/cut.md" 2>"$scratch/cut.err"
    absent "a stream that does not parse is never counted as no MCP call" \
        "The session made 0 calls to a tool of an MCP server." "$scratch/cut.md"
    present "and is said to be uncounted" \
        "were not counted, because its stream did not parse as JSON" "$scratch/cut.md"
    absent "and is never counted as no web-tool call (#1472)" \
        "to a web tool." "$scratch/cut.md"

    # Each form of a web-tool call is counted, from the harness and from the
    # provider, and a call to another tool is not (#1472).
    cat > "$scratch/bin/claude" <<STUB
#!/bin/sh
printf '%s\n' "\$@" > "\$HEADWATER_SHADOW_LOG_DIR/claude-args"
printf '%s\n' '{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s18"}'
printf '%s\n' '{"type":"assistant","message":{"id":"m1","content":[{"type":"server_tool_use","id":"t1","name":"web_search","input":{"query":"x"}}]}}'
printf '%s\n' '{"type":"assistant","message":{"id":"m2","content":[{"type":"server_tool_use","id":"t2","name":"web_fetch","input":{"url":"https://example.org"}}]}}'
printf '%s\n' '{"type":"assistant","message":{"id":"m3","content":[{"type":"tool_use","id":"t3","name":"WebFetch","input":{"url":"https://example.org"}}]}}'
printf '%s\n' '{"type":"assistant","message":{"id":"m4","content":[{"type":"tool_use","id":"t4","name":"WebSearch","input":{"query":"x"}}]}}'
printf '%s\n' '{"type":"assistant","message":{"id":"m5","content":[{"type":"tool_use","id":"t5","name":"Grep","input":{"pattern":"web_search"}}]}}'
printf '%s\n' '{"type":"result","subtype":"success","result":"withheld","total_cost_usd":0.01,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}'
STUB
    chmod +x "$scratch/bin/claude"
    PATH="$scratch/bin:$PATH" sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-web \
        --tier campaign --arm no-hook --category sufficiency --repetitions 1 \
        --task-file "$scratch/task.md" --workspace "$scratch/ws" \
        >"$scratch/web.md" 2>"$scratch/web.err"
    present "a record counts each form of a web-tool call, and no other tool" \
        "The session made 4 calls to a web tool." "$scratch/web.md"

    # THE decisive case of #1467: a session reads only its workspace. A file
    # outside it holds a marker, and so does a sibling tree of the kind
    # `campaign.sh` keeps next to every workspace (the oracle). The stub runs
    # inside the driver's real confinement and tries each road to them: the
    # absolute path, the relative path to the sibling, a search of the whole
    # file system, a process's root and command line under `/proc`, and
    # this checkout. It writes what it got into its own workspace. With the
    # confinement removed, the marker reaches `found.txt`.
    if command -v bwrap >/dev/null 2>&1; then
        conf=$(cd "$outer" && pwd -P)
        rm -rf "$conf/outside" "$conf/trees" "$conf/conf-ws"
        mkdir -p "$conf/outside" "$conf/trees/oracle" "$conf/trees/grading" "$conf/conf-ws"
        printf 'HW-LEAK-MARKER-1467\n' > "$conf/outside/secret.md"
        printf 'HW-LEAK-MARKER-1467\n' > "$conf/trees/oracle/secret.md"
        # The grading tree (#1834) holds what the seal removed, a hidden test
        # among it, so it is held to the same confinement as the oracle tree.
        printf 'HW-LEAK-MARKER-1467\n' > "$conf/trees/grading/secret.md"
        cat > "$scratch/bin/claude" <<STUB
#!/bin/sh
{
    cat "$conf/outside/secret.md"
    cat ../trees/oracle/secret.md
    cat ../trees/grading/secret.md
    find / -name secret.md
    for p in /proc/[0-9]*/root$conf/outside/secret.md; do cat "\$p"; done
    cat /proc/[0-9]*/cmdline | tr '\\0' ' ' | grep -o 'probe-record-fixtures[.]sh'
    head -1 "$root/CLAUDE.md"
    ls /home /mnt
} > found.txt 2>/dev/null
printf '%s\n' '{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s20"}'
printf '%s\n' '{"type":"assistant","message":{"id":"m1","content":[{"type":"tool_use","id":"t1","name":"Read","input":{"file_path":"$conf/outside/secret.md"}}]}}'
printf '%s\n' '{"type":"assistant","message":{"id":"m2","content":[{"type":"tool_use","id":"t2","name":"Bash","input":{"command":"cat $conf/outside/secret.md ../trees/oracle/secret.md; find / -name secret.md 2>/dev/null; ls /home /mnt > /tmp/out.txt"}}]}}'
printf '%s\n' '{"type":"assistant","message":{"id":"m3","content":[{"type":"tool_use","id":"t3","name":"Grep","input":{"pattern":"x","path":"$conf/conf-ws/docs"}}]}}'
printf '%s\n' '{"type":"result","subtype":"success","result":"withheld","total_cost_usd":0.01,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}'
STUB
        chmod +x "$scratch/bin/claude"
        PATH="$scratch/bin:$PATH" sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-confined \
            --task-file "$scratch/task.md" --workspace "$conf/conf-ws" \
            >"$scratch/confined.md" 2>"$scratch/confined.err"
        same "a confined session records" "0" "$?"
        same "and the stub ran inside its workspace" "yes" \
            "$([ -f "$conf/conf-ws/found.txt" ] && echo yes || echo no)"
        absent "a file outside the workspace is not readable from the session" \
            "HW-LEAK-MARKER-1467" "$conf/conf-ws/found.txt"
        absent "and no search of the file system finds it, in a sibling tree either" \
            "secret.md" "$conf/conf-ws/found.txt"
        absent "and this checkout is not readable" "Headwater authoring conventions" "$conf/conf-ws/found.txt"
        same "and nothing else outside it reached the session: what it found is empty" "" \
            "$(cat "$conf/conf-ws/found.txt" 2>/dev/null)"
        present "the transcript says the session ran confined to its workspace" \
            "The session ran confined to its workspace" "$scratch/confined.md"
        present "and names the configuration directory it ran under" \
            "The session ran under the configuration directory" "$scratch/confined.md"
        # The paths are the secret's (the `Read` and the `cat` name one path),
        # `/home`, `/mnt` and the root that `find /` searches, which counts as
        # one try since #1807: 1 + 1 + 1 + 1 = 4. The `Grep` path is the
        # workspace. The session's own `/tmp` is a tmpfs of the confinement and
        # no file of the host, so the `Bash` redirect to `/tmp/out.txt` above
        # adds 0 to the 4 (#1659): #1474's L3-p2 counted 3 such redirects as
        # outside.
        present "and counts the paths outside the workspace it named, none of them readable, and not its own /tmp" \
            "The session named 4 paths outside its workspace that the confinement does not bind, so no file of the host at any of them was readable." \
            "$scratch/confined.md"
        # A workspace that is itself under `/tmp` lets the session see its own
        # path there, so a path it names under `/tmp` may be a sibling tree of
        # the host's, and the redirect counts. So does the `Grep` path, which
        # is the first workspace and not this one: 4 + 1 + 1 = 6.
        tmp_ws=$(cd "$scratch" && pwd -P)/conf-ws-tmp
        rm -rf "$tmp_ws"
        mkdir -p "$tmp_ws"
        PATH="$scratch/bin:$PATH" sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-confined-tmp \
            --task-file "$scratch/task.md" --workspace "$tmp_ws" \
            >"$scratch/confined-tmp.md" 2>"$scratch/confined-tmp.err"
        present "and a workspace under /tmp counts the session's /tmp paths as outside" \
            "The session named 6 paths outside its workspace" "$scratch/confined-tmp.md"
        rm -rf "$tmp_ws"

        # THE decisive case of #1807: the count is of paths a session could
        # try, never of the text of a pattern. In R1 of the screening round of
        # 2026-10-18, 88 of the 137 paths counted for a/L7 were a `grep -v`
        # filter or other pattern text, such as `/site/`. Each case below runs
        # a session whose stream holds one `Bash` call and no other path, and
        # states the count the transcript must give for it. The stub reads its
        # stream from its own workspace, because the confinement binds nothing
        # else of the fixture's.
        cat > "$scratch/bin/claude" <<'STUB'
#!/bin/sh
cat stream.jsonl
STUB
        chmod +x "$scratch/bin/claude"
        outside_case() {
            # $1 name, $2 the `Bash` command, $3 the count the transcript states
            rm -rf "$conf/outside-ws"
            mkdir -p "$conf/outside-ws"
            {
                printf '%s\n' '{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s21"}'
                jq -cn --arg c "$2" '{type: "assistant", message: {id: "m1", content: [{type: "tool_use", id: "t1", name: "Bash", input: {command: $c}}]}}'
                printf '%s\n' '{"type":"result","subtype":"success","result":"withheld","total_cost_usd":0.01,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}'
            } > "$conf/outside-ws/stream.jsonl"
            PATH="$scratch/bin:$PATH" sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-outside \
                --task-file "$scratch/task.md" --workspace "$conf/outside-ws" \
                >"$scratch/outside.md" 2>"$scratch/outside.err"
            if [ "$3" = 1 ]; then unit=path; else unit=paths; fi
            # The sentence is matched whole, and the count is compared on its
            # own, so a failure names the count the transcript gave.
            same "$1" "The session named $3 $unit outside its workspace that the confinement does not bind" \
                "$(grep -o '^The session named [0-9]* paths\{0,1\} outside its workspace that the confinement does not bind' "$scratch/outside.md")"
        }
        outside_case "a grep filter's pattern is not a path: \`grep -v /site/\` counts 0" \
            'grep -rn x . | grep -v /site/' 0
        outside_case "nor are a regular expression and an awk pattern: \`/^kinds:/\` and \`/\\.git/\` count 0" \
            'grep -E "/^kinds:/" a.yml; awk "/\\.git/ {print}" b; ls target/ /target/' 0
        outside_case "the egress socket's directory is bound: \`ls /opt/headwater-egress\` counts 0" \
            'ls /opt/headwater-egress' 0
        outside_case "and so is /opt, which the confinement makes: \`ls /opt\` counts 0" \
            'ls /opt' 0
        outside_case "a search of the whole file system is one try: \`find / -name x\` counts 1" \
            'find / -name x 2>/dev/null' 1
        outside_case "and a search of the root by grep is the same try: \`grep -rn x /\` counts 1" \
            'grep -rn x /' 1
        # Each program the rule reads as a walk of the root has a row of its
        # own, so dropping one from the list turns its row red.
        for walk in 'ls /' 'ls -la /' 'du -sh /' 'tree -L 1 /' 'stat /' 'rg -n x /' 'egrep -r x /'; do
            outside_case "a search of the root by \`$walk\` counts 1" "$walk" 1
        done
        outside_case "a slash that is an argument of tr is not a search of the root: \`tr / _\` counts 0" \
            'echo a/b | tr / _' 0
        outside_case "a path that opens with ~ is a try: \`cat ~/notes\` counts 1" \
            'cat ~/notes' 1
        # A `~` right after `(` or `|`, with no space, is the same try.
        outside_case "and so is one that opens a command substitution: \`out=\$(~/bin/x)\` counts 1" \
            'out=$(~/bin/x)' 1
        outside_case "and one right after a pipe: \`echo a|~/bin/y\` counts 1" \
            'echo a|~/bin/y' 1
        outside_case "a regular-expression match operator is not a path: \`[[ x =~ y ]]\` counts 0" \
            '[[ x =~ y ]] && echo HEAD~1' 0
        outside_case "a path under a top-level directory of the host still counts: \`cat /home/x /root/y\` counts 2" \
            'cat /home/x /root/y' 2
        rm -rf "$conf/outside-ws"

        # The scratch directories (#1807). The driver makes a log directory and
        # a configuration directory under `TMPDIR` when its caller names
        # neither, and until #1807 nothing removed them: `/tmp` held 11,586 of
        # them on 2026-10-18. A recording removes the ones it made, on a
        # success and on an early exit, and never one its caller named, because
        # `campaign.sh` names the configuration directory and reads it after.
        cat > "$scratch/bin/claude" <<'STUB'
#!/bin/sh
printf '%s\n' '{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s22"}'
printf '%s\n' '{"type":"result","subtype":"success","result":"withheld","total_cost_usd":0.01,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}'
STUB
        chmod +x "$scratch/bin/claude"
        rm -rf "$scratch/clean-tmp" "$conf/clean-ws"
        mkdir -p "$scratch/clean-tmp" "$conf/clean-ws"
        env -u HEADWATER_PROBE_LOG_DIR -u HEADWATER_PROBE_CONFIG_DIR TMPDIR="$scratch/clean-tmp" PATH="$scratch/bin:$PATH" \
            sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-clean \
            --task-file "$scratch/task.md" --workspace "$conf/clean-ws" \
            >"$scratch/clean.md" 2>"$scratch/clean.err"
        same "a recording with no log or configuration directory named succeeds" "0" "$?"
        same "and leaves no scratch directory of its own under TMPDIR" "" \
            "$(find "$scratch/clean-tmp" -maxdepth 1 -name 'headwater-*' 2>/dev/null)"
        rm -rf "$scratch/clean-tmp" "$scratch/kept-log" "$scratch/kept-config"
        mkdir -p "$scratch/clean-tmp"
        HEADWATER_PROBE_LOG_DIR="$scratch/kept-log" HEADWATER_PROBE_CONFIG_DIR="$scratch/kept-config" \
            TMPDIR="$scratch/clean-tmp" PATH="$scratch/bin:$PATH" \
            sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-kept \
            --task-file "$scratch/task.md" --workspace "$conf/clean-ws" \
            >"$scratch/kept.md" 2>"$scratch/kept.err"
        same "a recording whose caller names both directories succeeds" "0" "$?"
        same "and keeps the log directory its caller named" "yes" \
            "$([ -d "$scratch/kept-log" ] && echo yes || echo no)"
        same "and keeps the configuration directory its caller named, which campaign.sh reads after" "yes" \
            "$([ -d "$scratch/kept-config" ] && echo yes || echo no)"
        same "and still leaves nothing of its own under TMPDIR" "" \
            "$(find "$scratch/clean-tmp" -maxdepth 1 -name 'headwater-*' 2>/dev/null)"
        rm -rf "$scratch/clean-tmp" "$scratch/kept-log" "$scratch/kept-config" "$conf/clean-ws"

        # THE decisive case of #1467's network half (clause 5): the public
        # repository is not reachable from a session by a direct connection.
        # (The provider API stays a channel, which spec 15 names and no case
        # can measure without spending.) A server on the host's
        # loopback stands in for it, so the case needs no internet access. It
        # serves a marker file and a git repository that holds the marker. The
        # stub tries each road to it: `curl` direct, the same request through
        # the session's proxy, a `CONNECT` to it through that proxy, a socket
        # from `python3`, `git clone` and `gh api`. It also asks the proxy for
        # the provider's host, which the proxy allows and logs before any
        # upstream connection. With the network shared, `curl` reads the marker.
        rm -rf "$conf/served" "$conf/net-ws"
        mkdir -p "$conf/served" "$conf/net-ws" "$conf/served-src"
        printf 'HW-LEAK-MARKER-1467-NET\n' > "$conf/served/secret.md"
        printf 'HW-LEAK-MARKER-1467-NET\n' > "$conf/served-src/secret.md"
        git -C "$conf/served-src" init -q
        git -C "$conf/served-src" add secret.md
        git -C "$conf/served-src" -c user.name=f -c user.email=f@f commit -q -m marker
        git clone -q --bare "$conf/served-src" "$conf/served/repo.git"
        git -C "$conf/served/repo.git" update-server-info
        rm -f "$conf/port"
        python3 -c '
import functools, http.server, sys
class Quiet(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *args):
        pass
handler = functools.partial(Quiet, directory=sys.argv[1])
server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
open(sys.argv[2] + ".tmp", "w").write(str(server.server_address[1]))
__import__("os").rename(sys.argv[2] + ".tmp", sys.argv[2])
server.serve_forever()
' "$conf/served" "$conf/port" &
        served_pid=$!
        n=0
        while [ ! -s "$conf/port" ] && [ "$n" -lt 50 ]; do sleep 0.1; n=$((n + 1)); done
        port=$(cat "$conf/port" 2>/dev/null)
        same "the stand-in for the public repository serves on the host" "HW-LEAK-MARKER-1467-NET" \
            "$(curl -s --noproxy '*' --max-time 5 "http://127.0.0.1:$port/secret.md")"
        cat > "$scratch/bin/claude" <<STUB
#!/bin/sh
{
    curl -s --noproxy '*' --max-time 3 "http://127.0.0.1:$port/secret.md"
    curl -s --max-time 3 -x "\$HTTPS_PROXY" "http://127.0.0.1:$port/secret.md"
    curl -s --max-time 3 -p -x "\$HTTPS_PROXY" "http://127.0.0.1:$port/secret.md"
    python3 -c 'import socket; s = socket.create_connection(("127.0.0.1", $port), 3); s.sendall(b"GET /secret.md HTTP/1.0\r\n\r\n"); print(s.recv(4096).decode())'
    git clone -q "http://127.0.0.1:$port/repo.git" cloned && cat cloned/secret.md
    GH_TOKEN=x GH_ENTERPRISE_TOKEN=x gh api --hostname "127.0.0.1:$port" /secret.md
} > found.txt 2>/dev/null
curl -s -p -x "\$HTTPS_PROXY" --max-time 3 -o /dev/null https://api.anthropic.com/ 2>/dev/null
curl -s --max-time 5 -o /dev/null -w '%{http_code}' -H 'content-type: application/json' \
    -d '{"model":"m","max_tokens":1,"messages":[{"role":"user","content":"x"}],"tools":[{"type":"web_search_20250305","name":"web_search"}]}' \
    "\$ANTHROPIC_BASE_URL/v1/messages" > search.txt 2>/dev/null
curl -s --max-time 10 -o /dev/null -w '%{http_code}' -H 'content-type: application/json' \
    -d '{"model":"m","max_tokens":1,"messages":[{"role":"user","content":"x"}]}' \
    "\$ANTHROPIC_BASE_URL/v1/messages" > plain.txt 2>/dev/null
printf '%s\n' "\$HTTPS_PROXY" > proxy.txt
printf '%s\n' "\$ANTHROPIC_BASE_URL" > base.txt
touch /opt/headwater-egress/planted 2>/dev/null && printf '%s\n' /opt/headwater-egress >> wrote.txt
printf '%s\n' '{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s22"}'
printf '%s\n' '{"type":"result","subtype":"success","result":"withheld","total_cost_usd":0.01,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}'
STUB
        chmod +x "$scratch/bin/claude"
        PATH="$scratch/bin:$PATH" sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-no-network \
            --task-file "$scratch/task.md" --workspace "$conf/net-ws" \
            >"$scratch/no-network.md" 2>"$scratch/no-network.err"
        same "a session with no network records" "0" "$?"
        kill "$served_pid" 2>/dev/null
        wait "$served_pid" 2>/dev/null
        same "and the stub ran inside its workspace" "yes" \
            "$([ -f "$conf/net-ws/found.txt" ] && echo yes || echo no)"
        absent "no road from the session reaches a server on the host" \
            "HW-LEAK-MARKER-1467-NET" "$conf/net-ws/found.txt"
        same "and git cloned nothing from it" "" "$(cat "$conf/net-ws/cloned/secret.md" 2>/dev/null)"
        present "the session was handed its proxy" "http://127.0.0.1:" "$conf/net-ws/proxy.txt"
        same "and cannot write beside the proxy's socket, which is the host's directory" "" "$(cat "$conf/net-ws/wrote.txt" 2>/dev/null)"
        present "the transcript says the session had no network but the proxy" \
            "The session ran with no network of its own." "$scratch/no-network.md"
        present "and names the host the proxy forwards to" \
            "forwards to \`api.anthropic.com\` on port 443 a request to the provider API only after it reads the request whole" "$scratch/no-network.md"
        same "the session's calls to the provider API go in plain HTTP to its own forwarder" "http://127.0.0.1:3128" \
            "$(cat "$conf/net-ws/base.txt" 2>/dev/null)"
        same "a call the session writes itself that asks the provider for its web search is refused" "403" \
            "$(cat "$conf/net-ws/search.txt" 2>/dev/null)"
        same "and an ordinary call passes the proxy's reading" "passed" \
            "$(p=$(cat "$conf/net-ws/plain.txt" 2>/dev/null); [ -n "$p" ] && [ "$p" != 403 ] && echo passed || echo "refused: $p")"
        present "and says the proxy refused the tunnel to the provider's host" \
            "1 connection to \`api.anthropic.com:443\`" "$scratch/no-network.md"
        present "and names the host whose connection the proxy refused" \
            "\`127.0.0.1:$port\`" "$scratch/no-network.md"
        present "and says the proxy allowed the provider's host" \
            "allowed 1 connection to \`api.anthropic.com:443\`" "$scratch/no-network.md"
        present "and names the request for a server-side web tool the proxy refused" \
            "It refused 1 request for \`web_search_20250305\` of that kind, and no request whose head, body, method or path it does not forward." \
            "$scratch/no-network.md"
        absent "and no longer says the channel through the provider API is open" "One channel stays open" "$scratch/no-network.md"
        absent "and the transcript claims nothing is reachable from the session" "reachable" "$scratch/no-network.md"
        present "and claims no more than the confinement holds: direct connections refused" \
            "so a direct connection to any other host, such as \`gh api\`, \`curl\` or \`git clone\` to GitHub, was refused. The proxy allowed" "$scratch/no-network.md"
        absent "and claims no read the confinement cannot show" "could not read" "$scratch/no-network.md"
        present "and says the file system held no copy of this repository on the host, and no more" \
            "no copy of this repository on the host, no other tree of its batch and no configuration of the host was readable through the session's file system." \
            "$scratch/no-network.md"
        present "and names every bind of the confinement, the egress proxy, its socket and the resolver target included" \
            "The confinement bound the workspace read-write, \`/usr\` and \`/etc\` read-only, the harness, the egress proxy and the directory of its socket read-only, the log directory of this run, the configuration directory below, and the target of \`/etc/resolv.conf\` read-only where it links out of \`/etc\` and \`/usr\`, and nothing else of the host's file system" \
            "$scratch/no-network.md"

        # The proxy alone (#1467, clause 5). It tunnels a `CONNECT` to a
        # listed host on 443 and nothing else: a listed host on another port,
        # and a plain request for a listed host, are refused and logged before
        # any connection upstream, so these cases need no internet access.
        rm -rf "$conf/px"
        mkdir -p "$conf/px"
        python3 "$root/tools/probe/egress-proxy.py" serve "$conf/px/s" "$conf/px/log" 2>/dev/null &
        px_pid=$!
        n=0
        while [ ! -S "$conf/px/s" ] && [ "$n" -lt 50 ]; do sleep 0.1; n=$((n + 1)); done
        px_ask() {
            python3 -c '
import socket, sys
s = socket.socket(socket.AF_UNIX)
s.settimeout(5)
s.connect(sys.argv[1])
s.sendall(sys.argv[2].encode() + b"\r\nHost: x\r\n\r\n")
print(s.recv(4096).decode("latin-1").split("\r\n", 1)[0])
' "$conf/px/s" "$1" 2>/dev/null
        }
        same "the proxy refuses a CONNECT to a listed host on another port" "HTTP/1.1 403 Forbidden" \
            "$(px_ask 'CONNECT api.anthropic.com:22 HTTP/1.1')"
        same "and a plain request for a listed host" "HTTP/1.1 403 Forbidden" \
            "$(px_ask 'GET http://api.anthropic.com:443/ HTTP/1.1')"
        same "and a CONNECT to a host off the list" "HTTP/1.1 403 Forbidden" \
            "$(px_ask 'CONNECT github.com:443 HTTP/1.1')"
        same "and a host name that reads like a list entry and is not one" "HTTP/1.1 403 Forbidden" \
            "$(px_ask 'CONNECT api.anthropic.com.example.net:443 HTTP/1.1')"
        same "and logs each refusal, once, before any upstream connection" \
            "refused api.anthropic.com:22|refused api.anthropic.com:443|refused github.com:443|refused api.anthropic.com.example.net:443" \
            "$(tr '\n' '|' < "$conf/px/log" | sed 's/|$//')"
        # The provider API is reached only through the proxy's own reading of
        # each request (#1467, the owner's ruling of 2026-10-03). A tunnel to
        # the API host is opaque, so the proxy opens none, to either host the
        # harness talks to. The session sends its requests in plain HTTP to its
        # own forwarder, which `ANTHROPIC_BASE_URL` names, and the proxy reads
        # each one and refuses, before any upstream connection, a request that
        # asks the provider to reach another host for the session.
        : > "$conf/px/log"
        same "the proxy opens no tunnel to the provider API" "HTTP/1.1 403 Forbidden" \
            "$(px_ask 'CONNECT api.anthropic.com:443 HTTP/1.1')"
        same "and none to the host that refreshes a credential, which answers the Messages API too" "HTTP/1.1 403 Forbidden" \
            "$(px_ask 'CONNECT platform.claude.com:443 HTTP/1.1')"
        px_post() {
            python3 -c '
import socket, sys
s = socket.socket(socket.AF_UNIX)
s.settimeout(20)
s.connect(sys.argv[1])
body = sys.argv[3].encode()
extra = sys.argv[4].replace("\\r\\n", "\r\n").replace("\\n", "\n")
line = sys.argv[2].replace("\\n", "\n")
head = line + "\r\nHost: 127.0.0.1:3128\r\nContent-Type: application/json\r\n" + extra
if "Content-Length" not in extra:
    head += "Content-Length: %d\r\n" % len(body)
s.sendall(head.encode("latin-1") + b"\r\n" + body)
print(s.recv(4096).decode("latin-1").split("\r\n", 1)[0])
' "$conf/px/s" "$1" "$2" "${3:-}" 2>/dev/null
        }
        px_tools() {
            printf '{"model":"claude-haiku-4-5","max_tokens":1,"messages":[{"role":"user","content":"x"}],"tools":[{"name":"Bash","input_schema":{"type":"object"}},%s]}' "$1"
        }
        same "the proxy refuses a request that asks for the provider's web search" "HTTP/1.1 403 Forbidden" \
            "$(px_post 'POST /v1/messages HTTP/1.1' "$(px_tools '{"type":"web_search_20250305","name":"web_search"}')")"
        same "and one that asks for the provider's web fetch" "HTTP/1.1 403 Forbidden" \
            "$(px_post 'POST /v1/messages?beta=true HTTP/1.1' "$(px_tools '{"type":"web_fetch_20250910","name":"web_fetch"}')")"
        same "and one that names an MCP toolset" "HTTP/1.1 403 Forbidden" \
            "$(px_post 'POST /v1/messages HTTP/1.1' "$(px_tools '{"type":"mcp_toolset","mcp_server_name":"gh"}')")"
        same "and one that names an MCP server for the provider to call" "HTTP/1.1 403 Forbidden" \
            "$(px_post 'POST /v1/messages HTTP/1.1' '{"model":"m","messages":[],"mcp_servers":[{"type":"url","url":"https://example.net/","name":"x"}]}')"
        same "and one whose content asks the provider to fetch a URL, however deep" "HTTP/1.1 403 Forbidden" \
            "$(px_post 'POST /v1/messages HTTP/1.1' '{"model":"m","messages":[{"role":"user","content":[{"type":"tool_result","tool_use_id":"t","content":[{"type":"document","source":{"type":"url","url":"https://github.com/x"}}]}]}]}')"
        same "and one to a path of the API off its list" "HTTP/1.1 403 Forbidden" \
            "$(px_post 'POST /v1/files HTTP/1.1' '{}')"
        same "and one with a method off its list" "HTTP/1.1 403 Forbidden" \
            "$(px_post 'GET /v1/messages HTTP/1.1' '')"
        same "and one whose body is chunked, which it cannot read before it forwards" "HTTP/1.1 403 Forbidden" \
            "$(px_post 'POST /v1/messages HTTP/1.1' '' 'Transfer-Encoding: chunked\r\n')"
        same "and one whose body is compressed" "HTTP/1.1 403 Forbidden" \
            "$(px_post 'POST /v1/messages HTTP/1.1' '{}' 'Content-Encoding: gzip\r\n')"
        same "and one whose body is not JSON" "HTTP/1.1 403 Forbidden" \
            "$(px_post 'POST /v1/messages HTTP/1.1' 'tools: web_search_20250305')"
        same "and one whose body states a length past the cap" "HTTP/1.1 403 Forbidden" \
            "$(px_post 'POST /v1/messages HTTP/1.1' '{}' 'Content-Length: 999999999999\r\n')"
        same "and one whose body states no length it can read" "HTTP/1.1 403 Forbidden" \
            "$(px_post 'POST /v1/messages HTTP/1.1' '{}' 'Content-Length: 2\r\nContent-Length: 3\r\n')"
        same "and one whose body is JSON and not an object" "HTTP/1.1 403 Forbidden" \
            "$(px_post 'POST /v1/messages HTTP/1.1' '[{"type":"web_search_20250305"}]')"
        same "and one whose tools are not a list it can read" "HTTP/1.1 403 Forbidden" \
            "$(px_post 'POST /v1/messages HTTP/1.1' '{"tools":{"type":"web_search_20250305"}}')"
        # Verify of #1467, round 1. A key named twice reads as its last value
        # here and may read as its first upstream, a bare line feed in the
        # head can put a line of the session's choice into the forwarded
        # head, and a length in digits that are not ASCII is no length.
        same "and one that names its tools twice, the web search first" "HTTP/1.1 403 Forbidden" \
            "$(px_post 'POST /v1/messages HTTP/1.1' '{"model":"m","tools":[{"type":"web_search_20250305","name":"web_search"}],"tools":[]}')"
        same "and one that names a source twice at any depth" "HTTP/1.1 403 Forbidden" \
            "$(px_post 'POST /v1/messages HTTP/1.1' '{"messages":[{"role":"user","content":[{"type":"document","source":{"type":"url","url":"https://github.com/x"},"source":{"type":"text","data":"x"}}]}]}')"
        same "and one whose target holds a bare line feed" "HTTP/1.1 403 Forbidden" \
            "$(px_post 'POST /v1/messages?a\nTransfer-Encoding:chunked HTTP/1.1' '{}')"
        same "and one whose header holds a bare line feed" "HTTP/1.1 403 Forbidden" \
            "$(px_post 'POST /v1/messages HTTP/1.1' '{}' 'X-Note: a\nTransfer-Encoding: chunked\r\n')"
        same "and one whose length is in digits that are not ASCII" "HTTP/1.1 403 Forbidden" \
            "$(px_post 'POST /v1/messages HTTP/1.1' '{}' 'Content-Length: ²\r\n')"
        same "and one that names a host other than the session's own forwarder" "HTTP/1.1 403 Forbidden" \
            "$(px_post 'POST http://api.anthropic.com/v1/messages HTTP/1.1' "$(px_tools '{"name":"Read","input_schema":{"type":"object"}}')")"
        same "and one to another port of the session's own loopback" "HTTP/1.1 403 Forbidden" \
            "$(px_post 'POST http://127.0.0.1:8000/v1/messages HTTP/1.1' "$(px_tools '{"name":"Read","input_schema":{"type":"object"}}')")"
        same "and one to another host on the forwarder's port" "HTTP/1.1 403 Forbidden" \
            "$(px_post 'POST http://github.com:3128/v1/messages HTTP/1.1' "$(px_tools '{"name":"Read","input_schema":{"type":"object"}}')")"
        same "and reads a request in the form the harness sends it through its proxy, to the forwarder by URL" "HTTP/1.1 403 Forbidden" \
            "$(px_post 'POST http://127.0.0.1:3128/v1/messages?beta=true HTTP/1.1' "$(px_tools '{"type":"web_search_20260209","name":"web_search"}')")"
        same "and logs each refusal with what it refused, before any upstream connection" \
            "refused api.anthropic.com:443|refused platform.claude.com:443|refused-tool web_search_20250305 api.anthropic.com:443|refused-tool web_fetch_20250910 api.anthropic.com:443|refused-tool mcp_toolset api.anthropic.com:443|refused-tool mcp_servers api.anthropic.com:443|refused-tool url-source api.anthropic.com:443|refused-path /v1/files api.anthropic.com:443|refused-path GET api.anthropic.com:443|refused-body chunked api.anthropic.com:443|refused-body compressed api.anthropic.com:443|refused-body not-json api.anthropic.com:443|refused-body too-long api.anthropic.com:443|refused-body no-length api.anthropic.com:443|refused-body not-json api.anthropic.com:443|refused-tool tools api.anthropic.com:443|refused-body duplicate-key api.anthropic.com:443|refused-body duplicate-key api.anthropic.com:443|refused-head control-character api.anthropic.com:443|refused-head control-character api.anthropic.com:443|refused-body no-length api.anthropic.com:443|refused api.anthropic.com:80|refused 127.0.0.1:8000|refused github.com:3128|refused-tool web_search_20260209 api.anthropic.com:443" \
            "$(tr '\n' '|' < "$conf/px/log" | sed 's/|$//')"
        # The control: a request with the session's own client tools alone
        # passes the reading and is forwarded. With no network the forward
        # fails with 502; with one, the API answers it without a credential.
        # Either way it is not the proxy's 403.
        : > "$conf/px/log"
        px_control=$(px_post 'POST /v1/messages?beta=true HTTP/1.1' "$(px_tools '{"name":"Read","input_schema":{"type":"object"}}')")
        same "a request with client tools alone passes the reading" "passed" \
            "$([ -n "$px_control" ] && [ "$px_control" != "HTTP/1.1 403 Forbidden" ] && echo passed || echo "refused: $px_control")"
        same "and the proxy logs it allowed, once" "allowed api.anthropic.com:443" "$(cat "$conf/px/log")"
        # The forward verifies the provider's certificate (verify of #1467,
        # round 1). The proxy's own code runs with its upstream connection
        # pointed at a local TLS server whose certificate names the API host
        # and is signed by nobody the host trusts: the handshake must fail,
        # and the request must end in 502 with nothing sent to that server.
        openssl req -x509 -newkey rsa:2048 -nodes -days 1 -subj /CN=api.anthropic.com \
            -addext subjectAltName=DNS:api.anthropic.com \
            -keyout "$conf/px/key.pem" -out "$conf/px/cert.pem" >/dev/null 2>&1
        same "the forward refuses an upstream certificate that nobody the host trusts signed" "HTTP/1.1 502 Bad Gateway|received nothing" \
            "$(python3 -c '
import importlib.util, socket, ssl, sys, threading
spec = importlib.util.spec_from_file_location("egress", sys.argv[1])
egress = importlib.util.module_from_spec(spec)
spec.loader.exec_module(egress)
server = socket.socket()
server.bind(("127.0.0.1", 0))
server.listen(1)
context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
context.load_cert_chain(sys.argv[2], sys.argv[3])
got = []
def serve():
    conn, _ = server.accept()
    try:
        tls = context.wrap_socket(conn, server_side=True)
        got.append(tls.recv(65536))
        tls.sendall(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
        tls.close()
    except (OSError, ssl.SSLError):
        conn.close()
threading.Thread(target=serve, daemon=True).start()
real = socket.create_connection
egress.socket.create_connection = lambda address, timeout=None: real(server.getsockname(), timeout)
egress.Log.write = lambda *a, **k: None
client, proxy_end = socket.socketpair()
body = b"{\"model\":\"m\",\"messages\":[]}"
client.sendall(b"POST /v1/messages HTTP/1.1\r\nHost: 127.0.0.1:3128\r\nContent-Length: %d\r\n\r\n" % len(body) + body)
egress.handle(proxy_end, egress.Log("/dev/null"))
client.settimeout(5)
print(client.recv(4096).decode("latin-1").split("\r\n", 1)[0] + "|" + ("received nothing" if not any(got) else "received the request"))
' "$root/tools/probe/egress-proxy.py" "$conf/px/cert.pem" "$conf/px/key.pem" 2>/dev/null)"
        kill "$px_pid" 2>/dev/null
        wait "$px_pid" 2>/dev/null
        same "the proxy names the hosts it allows, and no variable adds one" "api.anthropic.com" \
            "$(HTTPS_PROXY=x ALLOWED=github.com HEADWATER_EGRESS_HOSTS=github.com python3 "$root/tools/probe/egress-proxy.py" hosts | tr '\n' ' ' | sed 's/ $//')"

        # No confinement, no session (#1467). A host with no `bwrap`, or one
        # that cannot create a namespace, refuses with 12 before any harness
        # call, so a refusal spends nothing. No variable turns this off.
        cat > "$scratch/bin/claude" <<'STUB'
#!/bin/sh
: > harness-ran
printf '%s\n' '{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s21"}'
printf '%s\n' '{"type":"result","subtype":"success","result":"withheld","total_cost_usd":0.01,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}'
STUB
        chmod +x "$scratch/bin/claude"
        rm -rf "$scratch/nobwrap"
        mkdir -p "$scratch/nobwrap"
        for dir in /usr/local/bin /usr/bin /bin; do
            [ -d "$dir" ] || continue
            for tool in "$dir"/*; do
                name=${tool##*/}
                [ "$name" = bwrap ] && continue
                [ -e "$scratch/nobwrap/$name" ] || [ -L "$scratch/nobwrap/$name" ] \
                    || ln -s "$tool" "$scratch/nobwrap/$name" 2>/dev/null
            done
        done
        rm -f "$conf/conf-ws/harness-ran"
        PATH="$scratch/bin:$scratch/nobwrap" sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-no-bwrap \
            --task-file "$scratch/task.md" --workspace "$conf/conf-ws" \
            >/dev/null 2>"$scratch/no-bwrap.err"
        same "a host with no bwrap refuses with 12" "12" "$?"
        present "and names what is missing" "no \`bwrap\`" "$scratch/no-bwrap.err"
        same "and the harness never ran" "no" "$([ -e "$conf/conf-ws/harness-ran" ] && echo yes || echo no)"
        mkdir -p "$scratch/bad-bwrap"
        printf '#!/bin/sh\necho "bwrap: setting up uid map: Permission denied" >&2\nexit 1\n' > "$scratch/bad-bwrap/bwrap"
        chmod +x "$scratch/bad-bwrap/bwrap"
        rm -rf "$scratch/bad-bwrap-tmp"
        mkdir -p "$scratch/bad-bwrap-tmp"
        TMPDIR="$scratch/bad-bwrap-tmp" PATH="$scratch/bad-bwrap:$scratch/bin:$PATH" sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-bad-bwrap \
            --task-file "$scratch/task.md" --workspace "$conf/conf-ws" \
            >/dev/null 2>"$scratch/bad-bwrap.err"
        same "a bwrap that cannot create a namespace refuses with 12" "12" "$?"
        present "and prints what bwrap said" "setting up uid map" "$scratch/bad-bwrap.err"
        same "and the harness never ran" "no" "$([ -e "$conf/conf-ws/harness-ran" ] && echo yes || echo no)"
        same "and no egress proxy directory is left behind" "" \
            "$(find "$scratch/bad-bwrap-tmp" -maxdepth 1 -name 'headwater-egress.*' 2>/dev/null)"
        same "and an early exit leaves no log or configuration directory behind either (#1807)" "" \
            "$(find "$scratch/bad-bwrap-tmp" -maxdepth 1 -name 'headwater-probe-*' 2>/dev/null)"

        # No route that can be stated, no session (#1467, clause 5). A host
        # with no `python3` has no proxy, and a proxy that does not start
        # leaves the session no network at all, so both refuse with 12 before
        # any harness call. The second `python3` names its hosts and then
        # dies, which is a proxy that cannot listen.
        rm -rf "$scratch/nopython"
        mkdir -p "$scratch/nopython"
        for dir in /usr/local/bin /usr/bin /bin; do
            [ -d "$dir" ] || continue
            for tool in "$dir"/*; do
                name=${tool##*/}
                case $name in python3|python3.*) continue ;; esac
                [ -e "$scratch/nopython/$name" ] || [ -L "$scratch/nopython/$name" ] \
                    || ln -s "$tool" "$scratch/nopython/$name" 2>/dev/null
            done
        done
        PATH="$scratch/bin:$scratch/nopython" sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-no-python \
            --task-file "$scratch/task.md" --workspace "$conf/conf-ws" \
            >/dev/null 2>"$scratch/no-python.err"
        same "a host with no python3 refuses with 12" "12" "$?"
        present "and names what is missing" "no \`python3\`" "$scratch/no-python.err"
        same "and the harness never ran" "no" "$([ -e "$conf/conf-ws/harness-ran" ] && echo yes || echo no)"
        mkdir -p "$scratch/dead-proxy"
        real_python=$(command -v python3)
        printf '#!/bin/sh\ncase $2 in hosts) exec %s "$@" ;; serve) echo "OSError: the fixture refuses to listen" >&2; exit 1 ;; esac\nexec %s "$@"\n' \
            "$real_python" "$real_python" > "$scratch/dead-proxy/python3"
        chmod +x "$scratch/dead-proxy/python3"
        rm -f "$conf/conf-ws/harness-ran"
        rm -rf "$scratch/dead-proxy-tmp"
        mkdir -p "$scratch/dead-proxy-tmp"
        TMPDIR="$scratch/dead-proxy-tmp" PATH="$scratch/dead-proxy:$scratch/bin:$PATH" sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-dead-proxy \
            --task-file "$scratch/task.md" --workspace "$conf/conf-ws" \
            >/dev/null 2>"$scratch/dead-proxy.err"
        same "an egress proxy that does not start refuses with 12" "12" "$?"
        present "and says the proxy did not start" "the egress proxy that is the session's one route to the network did not start" "$scratch/dead-proxy.err"
        present "and prints what the proxy said" "the fixture refuses to listen" "$scratch/dead-proxy.err"
        same "and the harness never ran" "no" "$([ -e "$conf/conf-ws/harness-ran" ] && echo yes || echo no)"
        same "and no proxy directory is left behind" "" \
            "$(find "$scratch/dead-proxy-tmp" -maxdepth 1 -name 'headwater-egress.*' 2>/dev/null)"

        # A driver killed with SIGKILL runs none of its own cleanup, so the
        # proxy must notice on its own that its parent is gone and exit. The
        # stub harness says it started and then sleeps. Once the driver is
        # killed, the proxy's socket must refuse a connection within 5 s.
        cat > "$scratch/bin/claude" <<'STUB'
#!/bin/sh
: > session-started
sleep 60
STUB
        chmod +x "$scratch/bin/claude"
        rm -rf "$scratch/killed-tmp" "$conf/killed-ws"
        mkdir -p "$scratch/killed-tmp" "$conf/killed-ws"
        TMPDIR="$scratch/killed-tmp" PATH="$scratch/bin:$PATH" sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-killed \
            --task-file "$scratch/task.md" --workspace "$conf/killed-ws" >/dev/null 2>&1 &
        killed_pid=$!
        n=0
        while [ ! -e "$conf/killed-ws/session-started" ] && [ "$n" -lt 100 ]; do sleep 0.1; n=$((n + 1)); done
        same "a session that is running has started" "yes" \
            "$([ -e "$conf/killed-ws/session-started" ] && echo yes || echo no)"
        killed_sock=$(find "$scratch/killed-tmp" -path '*/headwater-egress.*/sock/proxy.sock' 2>/dev/null | head -1)
        px_alive() {
            python3 -c '
import socket, sys
s = socket.socket(socket.AF_UNIX)
s.settimeout(2)
try:
    s.connect(sys.argv[1])
except OSError:
    sys.exit(1)
' "$1" 2>/dev/null
        }
        same "and its proxy accepts a connection while the driver lives" "yes" \
            "$([ -n "$killed_sock" ] && px_alive "$killed_sock" && echo yes || echo no)"
        kill -9 "$killed_pid" 2>/dev/null
        wait "$killed_pid" 2>/dev/null
        n=0
        while [ -n "$killed_sock" ] && px_alive "$killed_sock" && [ "$n" -lt 50 ]; do sleep 0.1; n=$((n + 1)); done
        same "a driver killed with SIGKILL leaves no proxy running" "no" \
            "$([ -n "$killed_sock" ] && px_alive "$killed_sock" && echo yes || echo no)"

        # The host's configuration reaches no session (#1467). The session
        # runs under a fresh configuration directory that holds a copy of the
        # host's credentials and nothing else, and no variable of the host's
        # environment. The stub reports what it sees into the log directory.
        mkdir -p "$conf/host-config/skills/obsidian"
        printf -- '---\nname: obsidian\ndescription: x\n---\n' > "$conf/host-config/skills/obsidian/SKILL.md"
        printf '{"fixture":"credentials"}\n' > "$conf/host-config/.credentials.json"
        printf '# the host CLAUDE.md\n' > "$conf/host-config/CLAUDE.md"
        cat > "$scratch/bin/claude" <<'STUB'
#!/bin/sh
printf '%s\n' "$@" > "$HEADWATER_SHADOW_LOG_DIR/claude-args"
{
    printf 'HOME=%s\n' "$HOME"
    printf 'CLAUDE_CONFIG_DIR=%s\n' "$CLAUDE_CONFIG_DIR"
    printf 'GH_TOKEN=%s\n' "${GH_TOKEN:-unset}"
    printf 'ANTHROPIC_API_KEY=%s\n' "${ANTHROPIC_API_KEY:-unset}"
    printf 'held=%s\n' "$(ls -A "$CLAUDE_CONFIG_DIR" | tr '\n' ' ')"
    printf 'credentials=%s\n' "$(cat "$CLAUDE_CONFIG_DIR/.credentials.json")"
    printf 'self=%s\n' "$0"
    printf 'root=%s\n' "$(ls / | tr '\n' ' ')"
} > "$HEADWATER_SHADOW_LOG_DIR/seen"
cat "$HEADWATER_SHADOW_LOG_DIR/init.jsonl"
printf '%s\n' '{"type":"result","subtype":"success","result":"withheld","total_cost_usd":0.01,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}'
STUB
        chmod +x "$scratch/bin/claude"
        rm -rf "$conf/cfg-clean"
        printf '{"type":"system","subtype":"init","model":"claude-haiku-4-5","permissionMode":"dontAsk","plugins":[{"name":"telemetry","path":"builtin"}],"mcp_servers":[],"skills":["headwater-engine","init"],"memory_paths":{"auto":"%s/cfg-clean/projects/x/memory/"},"session_id":"s22"}\n' \
            "$conf" > "$scratch/probe-log/init.jsonl"
        rm -f "$args"
        GH_TOKEN=fixture-token ANTHROPIC_API_KEY=fixture-api-key HEADWATER_PROBE_HOST_CONFIG="$conf/host-config" HEADWATER_PROBE_CONFIG_DIR="$conf/cfg-clean" \
            PATH="$scratch/bin:$PATH" sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-clean-init \
            --task-file "$scratch/task.md" --workspace "$conf/conf-ws" \
            >"$scratch/clean-init.md" 2>"$scratch/clean-init.err"
        same "a session whose init line shows no host configuration records" "0" "$?"
        present "its HOME is the configuration directory" "HOME=$conf/cfg-clean" "$scratch/probe-log/seen"
        present "and so is CLAUDE_CONFIG_DIR" "CLAUDE_CONFIG_DIR=$conf/cfg-clean" "$scratch/probe-log/seen"
        present "that directory holds the credentials and nothing else" "held=.credentials.json " "$scratch/probe-log/seen"
        present "and the credentials are the host's" 'credentials={"fixture":"credentials"}' "$scratch/probe-log/seen"
        present "no variable of the host's environment reaches the session" "GH_TOKEN=unset" "$scratch/probe-log/seen"
        present "except the provider key, which a host without OAuth credentials logs in with" \
            "ANTHROPIC_API_KEY=fixture-api-key" "$scratch/probe-log/seen"
        present "the harness runs from a path of its own, so its host path names no directory of the session" \
            "self=/opt/headwater-harness/claude" "$scratch/probe-log/seen"
        same "and the session's root holds no /home and no /mnt" "" \
            "$(sed -n 's/^root=//p' "$scratch/probe-log/seen" | tr ' ' '\n' | grep -x -e home -e mnt)"
        same "the copy of the credentials is removed when the session ends" "no" \
            "$([ -e "$conf/cfg-clean/.credentials.json" ] && echo yes || echo no)"
        present "the transcript names that configuration directory" "$conf/cfg-clean" "$scratch/clean-init.md"
        same "the session loads project and local settings only" "project,local" \
            "$(awk 'prev == "--setting-sources" { print; exit } { prev = $0 }' "$args" 2>/dev/null)"
        same "and runs in a permission mode that asks nothing and bypasses nothing" "dontAsk" \
            "$(awk 'prev == "--permission-mode" { print; exit } { prev = $0 }' "$args" 2>/dev/null)"
        same "and names the tools it may use" "1" "$(grep -cx -- '--allowedTools' "$args" 2>/dev/null)"
        same "and denies the web tools" "WebSearch,WebFetch" \
            "$(awk 'prev == "--disallowedTools" { print; exit } { prev = $0 }' "$args" 2>/dev/null)"
        same "and bypassPermissions is nowhere in its arguments" "0" "$(grep -c bypassPermissions "$args" 2>/dev/null)"
        mkdir -p "$conf/cfg-clean"
        printf 'x\n' > "$conf/cfg-clean/left-over"
        same "a configuration directory that is not empty refuses with 2 before the harness" "2" \
            "$(HEADWATER_PROBE_HOST_CONFIG="$conf/host-config" HEADWATER_PROBE_CONFIG_DIR="$conf/cfg-clean" \
                PATH="$scratch/bin:$PATH" sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-used-config \
                --task-file "$scratch/task.md" --workspace "$conf/conf-ws" >/dev/null 2>&1; echo $?)"

        # The init line says what configuration the session loaded, and the
        # driver refuses a session that loaded any of the host's (#1467,
        # clause 4). One case per kind of leak, and one with no init line.
        for refused in \
            'bypass|"permissionMode":"bypassPermissions"|bypassPermissions' \
            'plugin|"plugins":[{"name":"x","path":"/elsewhere/plugins/x"}]|plugin x' \
            'mcp|"mcp_servers":[{"name":"claude.ai Gmail","status":"connected"}]|claude.ai Gmail' \
            'host-skill|"skills":["headwater-engine","obsidian"]|skill obsidian' \
            'namespaced|"skills":["anthropic-skills:docx"]|skill anthropic-skills:docx' \
            'memory|"memory_paths":{"auto":"/elsewhere/memory/"}|/elsewhere/memory/'; do
            kind=${refused%%|*}
            rest=${refused#*|}
            member=${rest%|*}
            named=${rest##*|}
            printf '{"type":"system","subtype":"init","model":"claude-haiku-4-5",%s,"session_id":"s23"}\n' \
                "$member" > "$scratch/probe-log/init.jsonl"
            HEADWATER_PROBE_HOST_CONFIG="$conf/host-config" PATH="$scratch/bin:$PATH" sh "$driver" \
                --probe "HW-PROBE-$tombstone" --session "fixture-init-$kind" \
                --task-file "$scratch/task.md" --workspace "$conf/conf-ws" \
                >"$scratch/init-$kind.md" 2>"$scratch/init-$kind.err"
            same "an init line with host configuration ($kind) refuses with 12" "12" "$?"
            present "and names it ($kind)" "$named" "$scratch/init-$kind.err"
        done
        printf '{"type":"system","subtype":"api_retry"}\n' > "$scratch/probe-log/init.jsonl"
        HEADWATER_PROBE_HOST_CONFIG="$conf/host-config" PATH="$scratch/bin:$PATH" sh "$driver" \
            --probe "HW-PROBE-$tombstone" --session fixture-init-none \
            --task-file "$scratch/task.md" --workspace "$conf/conf-ws" \
            >/dev/null 2>"$scratch/init-none.err"
        same "a stream with no init line refuses with 12, because its configuration cannot be read" "12" "$?"
        # The mcp arm declares its server in the workspace, so that one loads.
        cp "$root/tools/probe/arms/mcp/.mcp.json" "$conf/conf-ws/.mcp.json"
        printf '{"type":"system","subtype":"init","model":"claude-haiku-4-5","mcp_servers":[{"name":"headwater","status":"connected"}],"session_id":"s24"}\n' \
            > "$scratch/probe-log/init.jsonl"
        HEADWATER_PROBE_HOST_CONFIG="$conf/host-config" PATH="$scratch/bin:$PATH" sh "$driver" \
            --probe "HW-PROBE-$tombstone" --session fixture-init-declared \
            --task-file "$scratch/task.md" --workspace "$conf/conf-ws" \
            >/dev/null 2>"$scratch/init-declared.err"
        same "a server the workspace declares in .mcp.json is not a leak" "0" "$?"
        rm -f "$conf/conf-ws/.mcp.json" "$scratch/probe-log/init.jsonl" "$scratch/probe-log/seen"
    else
        fail "the confinement cases need bwrap" "no \`bwrap\` on the path, and a recorder without one records nothing (#1467)"
    fi

    # Bash writes (#1384). A write made through `Bash` names no path in its
    # input, so the transform cannot see it in the log, and 22 of 30
    # present-arm sessions of the `cited` probe on 2026-09-28 recorded
    # `produced: []`. With `--baseline`, the driver compares the workspace
    # after the session with the tree it was copied from, and every file that
    # is new or changed is produced. A file the session left alone is not, and
    # neither is a file an engine verb writes (verify round 1 of #1384): the
    # capture-cost store `headwater new` appends to, a fold under
    # `.headwater/`, and a generated shelf index. Each one below names
    # HW-DR-0049, which is the citation the `cited` probe grades, and only the
    # present arm could write it. The probe log directory is passed over too.
    rm -rf "$scratch/bw" "$scratch/bw-base"
    # The transform checks each produced file, so the tree carries the lock,
    # the vendored packages and the consumer declaration, and nothing that
    # names the probe or a record about it.
    mkdir -p "$scratch/bw-base/docs" "$scratch/bw-base/.claude/worktrees/w/docs" "$scratch/bw-base/.headwater"
    cp -a "$root/.headwater/taxonomy.lock" "$root/.headwater/taxonomy.yml" "$root/.headwater/packages" \
        "$scratch/bw-base/.headwater/"
    printf 'old\n' > "$scratch/bw-base/docs/changed.md"
    printf 'same\n' > "$scratch/bw-base/docs/same.md"
    printf '<!-- headwater:generated shelf_index -->\n\nold\n' > "$scratch/bw-base/docs/index.md"
    printf -- '---\n"headwater:generated": "shelf_sections."\nid: HW-REG-x\n---\n\nold\n' > "$scratch/bw-base/docs/register.md"
    printf '{\n  "headwater:generated": "corpus_descriptor.",\n  "a": "old"\n}\n' > "$scratch/bw-base/docs/data.json"
    printf -- '---\nid: HW-DR-x\n---\n\nA generated page carries this member in its block:\n\n"headwater:generated": "x"\n' > "$scratch/bw-base/docs/quotes.md"
    printf '# headwater:generated site_nav.\nnav: old\n' > "$scratch/bw-base/docs/nav.yml"
    printf 'keep\n' > "$scratch/bw-base/docs/keep.md"
    cp -a "$scratch/bw-base" "$scratch/bw"
    cat > "$scratch/bin/claude" <<'STUB'
#!/bin/sh
printf 'new\n' > docs/changed.md
touch docs/same.md
cp docs/same.md .claude/worktrees/w/docs/same.md
printf 'new\n' > docs/written-by-bash.md
printf 'new\n' > .claude/worktrees/w/docs/in-a-worktree.md
mkdir -p .headwater/cache && printf 'x\n' > .headwater/cache/entry
printf '{"cites":"HW-DR-0049"}\n' >> .headwater/capture-cost.jsonl
mkdir -p .claude/worktrees/w/.headwater && printf 'nav: HW-DR-0049\n' > .claude/worktrees/w/.headwater/nav.yml
printf '<!-- headwater:generated shelf_index -->\n\nHW-DR-0049\n' > docs/index.md
printf -- '---\n"headwater:generated": "shelf_sections."\nid: HW-REG-x\n---\n\nHW-DR-0049\n' > docs/register.md
printf '{\n  "headwater:generated": "corpus_descriptor.",\n  "a": "HW-DR-0049"\n}\n' > docs/data.json
printf 'HW-DR-0049\n' >> docs/quotes.md
printf '# headwater:generated site_nav.\nnav: HW-DR-0049\n' > docs/nav.yml
printf 'kept\n' > .claude/worktrees/w/docs/keep.md
touch -r docs/keep.md .claude/worktrees/w/docs/keep.md
mkdir -p .probe-log && printf 'HW-DR-0049\n' > .probe-log/other.txt
printf '%s\n' '{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s14"}'
printf '%s\n' '{"type":"assistant","message":{"content":[{"type":"tool_use","id":"t1","name":"Bash","input":{"command":"printf new > docs/written-by-bash.md"}}]}}'
printf '%s\n' '{"type":"result","subtype":"success","total_cost_usd":0.01,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}'
STUB
    chmod +x "$scratch/bin/claude"
    rm -rf "$scratch/bw-tmp"
    mkdir -p "$scratch/bw-tmp"
    HEADWATER_PROBE_LOG_DIR="$scratch/bw/.probe-log" TMPDIR="$scratch/bw-tmp" PATH="$scratch/bin:$PATH" \
        sh "$driver" --probe "HW-PROBE-$tombstone" --session fixture-bash-write \
        --task-file "$scratch/task.md" --workspace "$scratch/bw" --baseline "$scratch/bw-base" \
        > "$scratch/bash-write.md" 2>"$scratch/bash-write.err"
    same "a session that writes through Bash records" "0" "$?"
    # The comparison with the baseline writes into a directory of its own
    # under `TMPDIR`, and the driver's one `EXIT` trap removes it with the
    # other directories the run made (#1807), so no directory is left there.
    # The stream of the session is a file, and it stays: the run prints its
    # path.
    same "and a recording compared with a baseline leaves no directory under TMPDIR" "" \
        "$(find "$scratch/bw-tmp" -mindepth 1 -maxdepth 1 -type d 2>/dev/null)"
    rm -rf "$scratch/bw-tmp"
    present "a file a Bash call wrote is produced" 'path: "docs/written-by-bash.md"' "$scratch/bash-write.md"
    present "a file a Bash call changed is produced" 'path: "docs/changed.md"' "$scratch/bash-write.md"
    present "a file written in a worktree of the workspace is produced" \
        'path: ".claude/worktrees/w/docs/in-a-worktree.md"' "$scratch/bash-write.md"
    absent "a file the session touched and did not change is not produced" 'path: "docs/same.md"' "$scratch/bash-write.md"
    absent "a worktree's unchanged copy of a file is not produced" \
        'path: ".claude/worktrees/w/docs/same.md"' "$scratch/bash-write.md"
    absent "an engine cache is not produced" ".headwater/cache" "$scratch/bash-write.md"
    absent "the capture-cost store a verb appends to is not produced" \
        ".headwater/capture-cost.jsonl" "$scratch/bash-write.md"
    absent "a fold under a worktree's .headwater/ is not produced" \
        ".claude/worktrees/w/.headwater/nav.yml" "$scratch/bash-write.md"
    absent "a file that carries the generated marker is not produced" \
        'path: "docs/index.md"' "$scratch/bash-write.md"
    absent "a YAML fold that carries the marker in its first-line comment is not produced" \
        'path: "docs/nav.yml"' "$scratch/bash-write.md"
    absent "a Markdown page that carries the marker in its front matter is not produced" \
        'path: "docs/register.md"' "$scratch/bash-write.md"
    absent "a JSON file that carries the marker as a member is not produced" \
        'path: "docs/data.json"' "$scratch/bash-write.md"
    present "a document that quotes the marker in its prose is produced" \
        'path: "docs/quotes.md"' "$scratch/bash-write.md"
    present "a worktree file with the baseline's size and time but other bytes is produced" \
        'path: ".claude/worktrees/w/docs/keep.md"' "$scratch/bash-write.md"
    absent "the probe log directory inside the workspace is not produced" \
        ".probe-log/" "$scratch/bash-write.md"

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
        cp -a "$scratch/bw-base" "$batch/trees/grading"
        cp -a "$scratch/bw-base" "$batch/trees/campaign-present"
        git -C "$root" rev-parse HEAD > "$batch/head"
        printf 'claude-haiku-4-5\n' > "$batch/model"
        # One repetition keeps the plan small, and the ceiling is not what
        # this case holds.
        printf '1\n' > "$batch/repetitions"
        : > "$batch/max-turns"
        : > "$batch/cap"
        printf '30000\n' > "$batch/ceiling.campaign"
        printf '50\n' > "$batch/unit.campaign"
        printf '1 campaign present sufficiency\n' > "$batch/lines"
        cp "$scratch/task.md" "$batch/tasks/HW-PROBE-$tombstone.md"
        cat > "$scratch/bin/claude" <<STUB
#!/bin/sh
printf '%s\n' "\$@" > "\$HEADWATER_SHADOW_LOG_DIR/claude-args"
printf 'new\n' > docs/written-by-bash.md
printf '%s\n' '{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s15"}'
printf '%s\n' '{"type":"result","subtype":"error_max_turns","is_error":true,"total_cost_usd":0.01,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}'
exit 1
STUB
        chmod +x "$scratch/bin/claude"
        rm -f "$args"
        # A resumed job finds the directory its last attempt left. The batch
        # empties it, because the driver refuses one that is not empty.
        mkdir -p "$batch/config/L1-campaign-present-p1-r1"
        printf 'x\n' > "$batch/config/L1-campaign-present-p1-r1/left-by-a-failed-attempt"
        job="L1-campaign-present-p1-r1 1 campaign present sufficiency HW-PROBE-$tombstone"
        PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$batch" --job "$job" \
            >/dev/null 2>"$scratch/batch-job.err"
        same "a campaign job stopped by the cap records with status 0" "0" \
            "$(cat "$batch/sessions/L1-campaign-present-p1-r1/status" 2>/dev/null)"
        same "and the tier's cap reaches the harness" "80" \
            "$(awk 'prev == "--max-turns" { print; exit } { prev = $0 }' "$args" 2>/dev/null)"
        present "and the file it wrote through the shell is produced, against the tier's tree" \
            'path: "docs/written-by-bash.md"' "$batch/sessions/L1-campaign-present-p1-r1/record.md"
        sh "$root/tools/probe/campaign.sh" --out "$batch" --assemble >/dev/null 2>"$scratch/batch-assemble.err"
        same "and assembly counts the capped session" "1 sessions, 1 cents, the intent hook live in 0, 1 stopped at the turn cap, 0 stopped at the session budget, 0 paths outside the workspace named, 0 sessions uncounted, 0 web-tool calls" \
            "$(cat "$batch/assembled/L1-campaign-present-sufficiency.summary" 2>/dev/null)"
        same "the job ran under a configuration directory of the batch, outside every tree (#1467)" "yes" \
            "$([ -d "$batch/config/L1-campaign-present-p1-r1" ] && echo yes || echo no)"
        present "and the record names it" "$batch/config/L1-campaign-present-p1-r1" \
            "$batch/sessions/L1-campaign-present-p1-r1/record.md"

        # A job reads the task file of its own phrasing (#1875). In a copy of
        # the batch, the probe's task has two phrasings, and the draw that
        # `HEADWATER_CAMPAIGN_DRAW` names gives its session phrasing 2, so the
        # prompt is the body of phrasing 2 alone. A draw that gives it a
        # phrasing above its count exits 5 before the harness runs.
        phr_batch=$scratch/batch-phrasing
        rm -rf "$phr_batch"
        cp -a "$batch" "$phr_batch"
        printf 'The first phrasing of the task.\n' > "$phr_batch/tasks/HW-PROBE-$tombstone.1.md"
        printf 'The second phrasing of the task.\n' > "$phr_batch/tasks/HW-PROBE-$tombstone.2.md"
        printf '2\n' > "$phr_batch/tasks/HW-PROBE-$tombstone.phrasings"
        printf 'draw: HW-PROBE-%s 2\n' "$tombstone" > "$phr_batch/draw"
        rm -f "$args"
        job="L1-campaign-present-p1-r1 1 campaign present sufficiency HW-PROBE-$tombstone"
        rm -rf "$phr_batch/sessions/L1-campaign-present-p1-r1"
        HEADWATER_CAMPAIGN_DRAW=$phr_batch/draw PATH="$scratch/bin:$PATH" \
            sh "$root/tools/probe/campaign.sh" --out "$phr_batch" --job "$job" >/dev/null 2>"$scratch/batch-phrasing.err"
        same "a job on a draw of phrasing 2 records" "0" \
            "$(cat "$phr_batch/sessions/L1-campaign-present-p1-r1/status" 2>/dev/null)"
        present "and the harness is asked the body of phrasing 2" "The second phrasing of the task." "$args"
        absent "and no other phrasing" "The first phrasing of the task." "$args"
        printf 'draw: HW-PROBE-%s 3\n' "$tombstone" > "$phr_batch/draw"
        rm -f "$args"
        rm -rf "$phr_batch/sessions/L1-campaign-present-p1-r1"
        HEADWATER_CAMPAIGN_DRAW=$phr_batch/draw PATH="$scratch/bin:$PATH" \
            sh "$root/tools/probe/campaign.sh" --out "$phr_batch" --job "$job" >/dev/null 2>"$scratch/batch-phrasing.err"
        same "a draw of phrasing 3 for a task of 2 exits 5" "5" "$?"
        same "and the harness is not asked" "no" "$([ -f "$args" ] && echo yes || echo no)"
        present "and says why" "holds 2 phrasings" "$scratch/batch-phrasing.err"
        # With no draw, repetition 2 of the job takes phrasing 2.
        rm -f "$args"
        job="L1-campaign-present-p1-r2 1 campaign present sufficiency HW-PROBE-$tombstone"
        PATH="$scratch/bin:$PATH" \
            sh "$root/tools/probe/campaign.sh" --out "$phr_batch" --job "$job" >/dev/null 2>"$scratch/batch-phrasing.err"
        present "with no draw, repetition 2 of a probe of 2 phrasings is asked phrasing 2" "The second phrasing of the task." "$args"
        rm -rf "$phr_batch"

        # The tier's ceiling reserves `--session-cents` too, not the declared
        # unit (#1659). The batch has spent 1 cent above and 50 in another
        # session of the tier, so with B = 60 a ceiling of 110 refuses the next
        # job, where the unit of 50 would let it through, and 111 admits it.
        printf '60' > "$batch/session-cents"
        mkdir -p "$batch/sessions/L1-other"
        printf 'campaign\n' > "$batch/sessions/L1-other/tier"
        printf '50\n' > "$batch/sessions/L1-other/cost"
        printf '0\n' > "$batch/sessions/L1-other/status"
        job="L1-campaign-present-p1-r2 1 campaign present sufficiency HW-PROBE-$tombstone"
        printf '110\n' > "$batch/ceiling.campaign"
        PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$batch" --job "$job" \
            >/dev/null 2>"$scratch/batch-ceiling.err"
        same "a ceiling of 110 with 51 spent and B = 60 refuses the job" "none" \
            "$(cat "$batch/sessions/L1-campaign-present-p1-r2/status" 2>/dev/null || echo none)"
        present "and says what it reserved" "at 60 each against a campaign ceiling of 110" "$scratch/batch-ceiling.err"
        printf '111\n' > "$batch/ceiling.campaign"
        # A session of another tier counts toward the cap and not toward
        # this tier's ceiling.
        mkdir -p "$batch/sessions/L2-other-tier"
        printf 'documentation\n' > "$batch/sessions/L2-other-tier/tier"
        printf '1000\n' > "$batch/sessions/L2-other-tier/cost"
        printf '0\n' > "$batch/sessions/L2-other-tier/status"
        PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$batch" --job "$job" \
            >/dev/null 2>"$scratch/batch-ceiling.err"
        rm -rf "$batch/sessions/L2-other-tier"
        same "and a ceiling of 111 admits it, whatever another tier spent" "0" \
            "$(cat "$batch/sessions/L1-campaign-present-p1-r2/status" 2>/dev/null || echo none)"
        same "and the harness received the budget of B" "0.60" \
            "$(awk 'prev == "--max-budget-usd" { print; exit } { prev = $0 }' "$args" 2>/dev/null)"
        # A job that a killed invocation left started, with no status, is not
        # in flight when it runs again: the batch has spent 52, so a cap of
        # 112 admits it at B = 60, where counting it in flight would need 172.
        printf '100000\n' > "$batch/ceiling.campaign"
        printf '112' > "$batch/cap"
        mkdir -p "$batch/sessions/L1-campaign-present-p1-r3"
        printf 'campaign\n' > "$batch/sessions/L1-campaign-present-p1-r3/tier"
        : > "$batch/sessions/L1-campaign-present-p1-r3/started"
        job="L1-campaign-present-p1-r3 1 campaign present sufficiency HW-PROBE-$tombstone"
        PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$batch" --job "$job" \
            >/dev/null 2>"$scratch/batch-killed.err"
        same "a job a killed invocation left started runs again inside the cap" "0" \
            "$(cat "$batch/sessions/L1-campaign-present-p1-r3/status" 2>/dev/null || echo none)"
        rm -rf "$batch/sessions/L1-other" "$batch/sessions/L1-campaign-present-p1-r2" \
            "$batch/sessions/L1-campaign-present-p1-r3"
        : > "$batch/session-cents"
        : > "$batch/cap"
        printf '30000\n' > "$batch/ceiling.campaign"

        # The evidence check on the oracle record (#1875, clause 4). The
        # session writes a file that an oracle of the grading tree matches,
        # and the oracle's one line of evidence names an answer key of the
        # session's probe. The batch keeps the verdict and stores the evidence
        # as `refused`, with the hit beside it. The key is declared for the
        # tombstone probe through HW_PROBE_YML, at a path no tree holds, so the
        # guard of the recorder passes the workspace.
        evb=$scratch/batch-evidence
        rm -rf "$evb"
        cp -a "$batch" "$evb"
        rm -rf "$evb/sessions" "$evb/assembled" "$evb/config"
        mkdir -p "$evb/sessions"
        awk 'skip && /^[^ ]/ { skip = 0 }
        skip { next }
        /^oracles:/ {
            skip = 1
            print "oracles:"
            print "  key-echo:"
            print "    paths: [key-echo.txt]"
            print "    command: printf \"zz-fixture-key-1875 is named\\\\n\""
            print "    parser: exit"
            next
        } { print }' "$root/.headwater/probe.yml" > "$evb/trees/grading/.headwater/probe.yml"
        awk -v probe="HW-PROBE-$tombstone" '/^answer_keys:/ { print; print "  " probe ": [docs/obligations/zz-fixture-key-1875.md]"; next } { print }' \
            "$root/.headwater/probe.yml" > "$scratch/evb-keys.yml"
        cat > "$scratch/bin/claude" <<STUB
#!/bin/sh
printf 'x\n' > key-echo.txt
printf '%s\n' '{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s31"}'
printf '%s\n' '{"type":"result","subtype":"success","result":"withheld","total_cost_usd":0.01,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}'
STUB
        chmod +x "$scratch/bin/claude"
        HW_PROBE_YML="$scratch/evb-keys.yml" PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$evb" \
            --job "L1-campaign-present-p1-r1 1 campaign present sufficiency HW-PROBE-$tombstone" \
            >/dev/null 2>"$scratch/evb.err"
        same "a session whose oracle evidence names an answer key records" "0" \
            "$(cat "$evb/sessions/L1-campaign-present-p1-r1/status" 2>/dev/null)"
        same "and its oracle record keeps the verdict and stores the evidence as refused" \
            "key-echo	pass	refused" "$(cat "$evb/sessions/L1-campaign-present-p1-r1/oracles" 2>/dev/null)"
        same "and the hit is written beside it" \
            "evidence HW-PROBE-$tombstone key-echo zz-fixture-key-1875" \
            "$(cat "$evb/sessions/L1-campaign-present-p1-r1/oracles.refused" 2>/dev/null)"
        absent "and the key is in no line of the record" "zz-fixture-key-1875" \
            "$evb/sessions/L1-campaign-present-p1-r1/oracles"
        # With no key declared, the same evidence is kept as written.
        PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$evb" \
            --job "L1-campaign-present-p1-r2 1 campaign present sufficiency HW-PROBE-$tombstone" \
            >/dev/null 2>"$scratch/evb2.err"
        same "and a probe with no answer key keeps the evidence" \
            "key-echo	pass	zz-fixture-key-1875 is named" "$(cat "$evb/sessions/L1-campaign-present-p1-r2/oracles" 2>/dev/null)"
        # When the engine refuses the declaration, `seal.sh --evidence` exits
        # 8 and nobody checked the evidence, so the batch keeps the verdict and
        # stores every evidence field as `unchecked`, with no hit file. The
        # recorder's guard reads the same declaration before the session and
        # refuses a run on it, so the declaration is the workspace's own copy,
        # which the stub harness breaks during the session, after the guard
        # and before the check. The harness can write the workspace alone. The
        # copy drops the comment lines, because one names a record that names
        # the probe, and the recorder's guard refuses a workspace that does.
        mkdir -p "$evb/trees/campaign-present/.headwater"
        awk '!/^[[:space:]]*#/' "$root/.headwater/probe.yml" > "$evb/trees/campaign-present/.headwater/probe.yml"
        cat > "$scratch/bin/claude" <<STUB
#!/bin/sh
printf 'x\n' > key-echo.txt
printf 'oracles: [\n' > .headwater/probe.yml
printf '%s\n' '{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s32"}'
printf '%s\n' '{"type":"result","subtype":"success","result":"withheld","total_cost_usd":0.01,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}'
STUB
        chmod +x "$scratch/bin/claude"
        HW_PROBE_YML="$evb/ws/L1-campaign-present-p1-r3/.headwater/probe.yml" PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$evb" \
            --job "L1-campaign-present-p1-r3 1 campaign present sufficiency HW-PROBE-$tombstone" \
            >/dev/null 2>"$scratch/evb3.err"
        same "a session whose evidence check cannot read the declaration records" "0" \
            "$(cat "$evb/sessions/L1-campaign-present-p1-r3/status" 2>/dev/null)"
        same "and its oracle record keeps the verdict and stores the evidence as unchecked" \
            "key-echo	pass	unchecked" "$(cat "$evb/sessions/L1-campaign-present-p1-r3/oracles" 2>/dev/null)"
        same "and no hit file is written for evidence nobody checked" "none" \
            "$(ls "$evb/sessions/L1-campaign-present-p1-r3/oracles.refused" "$evb/sessions/L1-campaign-present-p1-r3/oracles.hits" 2>/dev/null || echo none)"
        rm -rf "$evb"

        # THE decisive case of #1472's campaign report: assembly states the
        # paths outside the workspace that a line's sessions named and the
        # web-tool calls they made, and states a session it could not count as
        # uncounted, never as 0. One session names an unbound host path through
        # the shell, reads a bound one and calls WebSearch. The other's stream
        # does not parse past its init line.
        leak=$scratch/batch-leak
        rm -rf "$leak"
        cp -a "$batch" "$leak"
        rm -rf "$leak/sessions" "$leak/assembled" "$leak/config"
        mkdir -p "$leak/sessions"
        cat > "$scratch/bin/claude" <<STUB
#!/bin/sh
printf '%s\n' '{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s21"}'
printf '%s\n' '{"type":"assistant","message":{"id":"m1","content":[{"type":"tool_use","id":"t1","name":"Bash","input":{"command":"cat /home/someone/secret.md"}}]}}'
printf '%s\n' '{"type":"assistant","message":{"id":"m2","content":[{"type":"tool_use","id":"t2","name":"Read","input":{"file_path":"/etc/hostname"}}]}}'
printf '%s\n' '{"type":"assistant","message":{"id":"m3","content":[{"type":"server_tool_use","id":"t3","name":"WebSearch","input":{"query":"x"}}]}}'
printf '%s\n' '{"type":"result","subtype":"success","result":"withheld","total_cost_usd":0.01,"usage":{"input_tokens":10,"cache_creation_input_tokens":30,"cache_read_input_tokens":60},"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}'
STUB
        chmod +x "$scratch/bin/claude"
        PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$leak" \
            --job "L1-campaign-present-p1-r1 1 campaign present sufficiency HW-PROBE-$tombstone" \
            >/dev/null 2>"$scratch/leak-a.err"
        same "a session that names a host path records" "0" \
            "$(cat "$leak/sessions/L1-campaign-present-p1-r1/status" 2>/dev/null)"
        present "and its record counts one web-tool call" \
            "The session made 1 call to a web tool." "$leak/sessions/L1-campaign-present-p1-r1/record.md"
        # The token record beside the cost (#1875), and not in the record.
        same "the session's tokens file carries the three counts of the result's usage" \
            "cache_read_tokens: 60 cache_write_tokens: 30 input_tokens: 10" \
            "$(tr '\n' ' ' < "$leak/sessions/L1-campaign-present-p1-r1/tokens" 2>/dev/null | sed 's/ $//')"
        absent "and the record's identity block carries none of them" \
            "cache_read_tokens" "$leak/sessions/L1-campaign-present-p1-r1/record.md"
        # The oracle record beside the cost (#1875). The session produced no
        # artifact, so every oracle the grading tree declares is `not-run`.
        same "the session's oracles file holds one not-run line for each declared oracle" \
            "$("$engine" probe plan --root "$leak/trees/grading" --oracles | awk -F '\t' '!seen[$1]++' | wc -l | tr -d ' ') 0" \
            "$(awk 'END { print NR }' "$leak/sessions/L1-campaign-present-p1-r1/oracles" 2>/dev/null) $(awk -F '\t' '$2 != "not-run" || $3 != "-"' "$leak/sessions/L1-campaign-present-p1-r1/oracles" 2>/dev/null | wc -l | tr -d ' ')"
        same "and it holds no refusal" "absent" \
            "$([ -e "$leak/sessions/L1-campaign-present-p1-r1/oracles.refused" ] && echo present || echo absent)"
        cat > "$scratch/bin/claude" <<STUB
#!/bin/sh
printf '%s\n' '{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s22"}'
printf '%s\n' '{"type":"assistant","message":{"id":"m1","content":[{"type":"tool_use","id":"t1","name":"Bash","input":{"command":"cat /home/someone/other.md"}}]}}'
printf '%s\n' '{"type":"result","subtype":"success","result":"withheld","total_cost_usd":0.01,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}'
printf '%s' '{"type":"assistant","message":{"id":"m2","content":[{"type":"tool_'
STUB
        chmod +x "$scratch/bin/claude"
        # A token record left by an earlier attempt of the job, which a log
        # that does not parse must not keep (#1875).
        mkdir -p "$leak/sessions/L1-campaign-present-p1-r2"
        printf 'cache_read_tokens: 5\ncache_write_tokens: 5\ninput_tokens: 5\n' > "$leak/sessions/L1-campaign-present-p1-r2/tokens"
        printf 'stale\tpass\t-\n' > "$leak/sessions/L1-campaign-present-p1-r2/oracles"
        printf 'evidence x stale y\n' > "$leak/sessions/L1-campaign-present-p1-r2/oracles.refused"
        PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$leak" \
            --job "L1-campaign-present-p1-r2 1 campaign present sufficiency HW-PROBE-$tombstone" \
            >/dev/null 2>"$scratch/leak-b.err"
        same "a job run again drops the token record of its earlier attempt when its log does not parse" "absent" \
            "$([ -e "$leak/sessions/L1-campaign-present-p1-r2/tokens" ] && echo present || echo absent)"
        same "and the oracle record of its earlier attempt, with its refusals" "absent absent" \
            "$([ -e "$leak/sessions/L1-campaign-present-p1-r2/oracles" ] && echo present || echo absent) $([ -e "$leak/sessions/L1-campaign-present-p1-r2/oracles.refused" ] && echo present || echo absent)"
        # The transform refuses a stream that does not parse, so the session
        # spent and did not record. Nobody counted what it named.
        same "a session whose stream does not parse does not record" "4" \
            "$(cat "$leak/sessions/L1-campaign-present-p1-r2/status" 2>/dev/null)"
        present "and what it wrote says its web-tool calls were not counted" \
            "The calls this session made to a web tool were not counted" \
            "$leak/sessions/L1-campaign-present-p1-r2/record.md"
        sh "$root/tools/probe/campaign.sh" --out "$leak" --assemble >/dev/null 2>"$scratch/leak-assemble.err"
        same "assembly exits 7 for the session that did not record, as before" "7" "$?"
        same "and the line's summary states the host path, the uncounted session and the web-tool call" \
            "1 sessions, 1 cents, the intent hook live in 0, 0 stopped at the turn cap, 0 stopped at the session budget, 1 paths outside the workspace named, 1 sessions uncounted, 1 web-tool calls" \
            "$(cat "$leak/assembled/L1-campaign-present-sufficiency.summary" 2>/dev/null)"
        present "and the batch line states them over the whole batch" \
            "campaign: the batch named 1 paths outside the workspace over 2 sessions, 1 sessions uncounted, 1 web-tool calls" \
            "$scratch/leak-assemble.err"
        absent "and the uncounted session is never reported as a batch of 0" \
            "named 0 paths" "$scratch/leak-assemble.err"
        present "and the batch states the cached share of input over its one recorded session alone" \
            "campaign: cached share: 60 of 100 input tokens read from the cache over 1 recorded sessions, 60.0%" \
            "$scratch/leak-assemble.err"

        # A recorded session whose record states neither count is uncounted
        # too, never 0: the record of a stream that `jq -s` could not read,
        # or one written before the sentences existed. The session that did
        # not record is drawn again, as a resumed batch does, and passes.
        rm -rf "$leak/sessions/L1-campaign-present-p1-r2"
        cp -a "$leak/sessions/L1-campaign-present-p1-r1" "$leak/sessions/L1-campaign-present-p1-r3"
        sed -e 's/^The session named .* outside its workspace .*$/The paths outside its workspace this session named were not counted, because its stream did not parse as JSON./' \
            -e '/^The session made .* to a web tool\.$/d' \
            "$leak/sessions/L1-campaign-present-p1-r1/record.md" > "$leak/sessions/L1-campaign-present-p1-r3/record.md"
        sh "$root/tools/probe/campaign.sh" --out "$leak" --assemble >/dev/null 2>"$scratch/leak-assemble3.err"
        same "assembly of two recorded sessions exits 0, because a count is a report and not a gate" "0" "$?"
        same "and a record that states no count is uncounted, never 0" \
            "2 sessions, 2 cents, the intent hook live in 0, 0 stopped at the turn cap, 0 stopped at the session budget, 1 paths outside the workspace named, 1 sessions uncounted, 1 web-tool calls" \
            "$(cat "$leak/assembled/L1-campaign-present-sufficiency.summary" 2>/dev/null)"
        present "and the batch line says so" \
            "campaign: the batch named 1 paths outside the workspace over 2 sessions, 1 sessions uncounted, 1 web-tool calls" \
            "$scratch/leak-assemble3.err"

        # Figures that differ, so no figure can stand in for another (verify
        # round 1). A fourth session names 5 paths and makes 2 web-tool calls.
        # A fifth states its path count and not its web-tool count, so it is
        # uncounted and its 7 paths are not summed.
        cp -a "$leak/sessions/L1-campaign-present-p1-r1" "$leak/sessions/L1-campaign-present-p1-r4"
        sed -e 's/^The session named 1 path outside/The session named 5 paths outside/' \
            -e 's/^The session made 1 call to a web tool\.$/The session made 2 calls to a web tool./' \
            "$leak/sessions/L1-campaign-present-p1-r1/record.md" > "$leak/sessions/L1-campaign-present-p1-r4/record.md"
        cp -a "$leak/sessions/L1-campaign-present-p1-r1" "$leak/sessions/L1-campaign-present-p1-r5"
        sed -e 's/^The session named 1 path outside/The session named 7 paths outside/' \
            -e '/^The session made .* to a web tool\.$/d' \
            "$leak/sessions/L1-campaign-present-p1-r1/record.md" > "$leak/sessions/L1-campaign-present-p1-r5/record.md"
        sh "$root/tools/probe/campaign.sh" --out "$leak" --assemble >/dev/null 2>"$scratch/leak-assemble5.err"
        same "assembly of four recorded sessions whose figures differ exits 0" "0" "$?"
        same "and each figure of the summary is its own sum" \
            "4 sessions, 4 cents, the intent hook live in 0, 0 stopped at the turn cap, 0 stopped at the session budget, 6 paths outside the workspace named, 2 sessions uncounted, 3 web-tool calls" \
            "$(cat "$leak/assembled/L1-campaign-present-sufficiency.summary" 2>/dev/null)"
        present "and so is each figure of the batch line" \
            "campaign: the batch named 6 paths outside the workspace over 4 sessions, 2 sessions uncounted, 3 web-tool calls" \
            "$scratch/leak-assemble5.err"
        # A session recorded before #1875 has no token record, so it is named
        # and adds nothing to either sum.
        rm -f "$leak/sessions/L1-campaign-present-p1-r5/tokens"
        # A session that did not record is no recorded session, whatever
        # token record it holds.
        mkdir -p "$leak/sessions/L1-campaign-present-p1-r6"
        printf '4\n' > "$leak/sessions/L1-campaign-present-p1-r6/status"
        printf 'cache_read_tokens: 1000\ncache_write_tokens: 0\ninput_tokens: 0\n' > "$leak/sessions/L1-campaign-present-p1-r6/tokens"
        printf 'cache_read_tokens: 0\ncache_write_tokens: 100\ninput_tokens: 0\n' > "$leak/sessions/L1-campaign-present-p1-r4/tokens"
        same "the cached share sums the reads over the input of each recorded session, and names one with no token record" \
            "cached share: 120 of 300 input tokens read from the cache over 3 recorded sessions, 40.0%, 1 recorded sessions without a token record" \
            "$(sh "$root/tools/probe/cached-share.sh" "$leak")"
        same "and a batch with no recorded session says so" \
            "cached share: no recorded sessions" \
            "$(sh "$root/tools/probe/cached-share.sh" "$scratch/no-such-batch")"
    else
        printf 'note not a checkout of this repository, so the campaign job case did not run.\n'
    fi

    # A batch directory under `$HOME` is refused (#1467): the session must
    # not sit beside the host's own configuration and copies of this
    # repository. Assembly of an old batch there still runs.
    mkdir -p "$scratch/fake-home"
    HOME="$scratch/fake-home" sh "$root/tools/probe/campaign.sh" --out "$scratch/fake-home/batch" \
        --model claude-haiku-4-5 --spec "$scratch/task.md" >/dev/null 2>"$scratch/home-out.err"
    same "a batch directory under \$HOME refuses with 6" "6" "$?"
    present "and says why" "under \$HOME" "$scratch/home-out.err"

    # A batch over a tier that declares no turn cap, with no `--max-turns`,
    # refuses with 2 before it builds a tree (#1384). The regression tier
    # declares none. The batch needs a clean checkout first, so the case runs
    # only on one, which is what CI checks out. `cargo` is a stub, because the
    # engine this suite reads is already built. The batch copies the binary
    # its build wrote, at `dev-release/` under `CARGO_TARGET_DIR`, and CI
    # builds only `release`. So the stub's build is a scratch target that
    # holds the engine this suite reads, and the checkout's own target is
    # not touched.
    if [ "$(git -C "$root" rev-parse --show-toplevel 2>/dev/null)" = "$root" ] \
        && [ -z "$(git -C "$root" status --porcelain --untracked-files=no)" ]; then
        printf '#!/bin/sh\nexit 0\n' > "$scratch/bin/cargo"
        chmod +x "$scratch/bin/cargo"
        mkdir -p "$scratch/campaign-target/dev-release" "$scratch/empty-target"
        cp "$engine" "$scratch/campaign-target/dev-release/headwater"
        CARGO_TARGET_DIR=$scratch/campaign-target
        export CARGO_TARGET_DIR

        # A build that leaves no binary where cargo writes refuses with 3, and
        # never reaches the plans.
        printf 'campaign present sufficiency\n' > "$scratch/nobinary.spec"
        CARGO_TARGET_DIR=$scratch/empty-target PATH="$scratch/bin:$PATH" \
            sh "$root/tools/probe/campaign.sh" --out "$scratch/nobinary" \
            --model claude-haiku-4-5 --spec "$scratch/nobinary.spec" \
            >/dev/null 2>"$scratch/nobinary.err"
        same "a batch whose build wrote no engine under CARGO_TARGET_DIR refuses with 3" "3" "$?"
        present "and names the path" "no engine at $scratch/empty-target/dev-release/headwater" "$scratch/nobinary.err"

        printf 'regression present sufficiency\n' > "$scratch/uncapped.spec"
        PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$scratch/uncapped" \
            --model claude-haiku-4-5 --spec "$scratch/uncapped.spec" \
            >/dev/null 2>"$scratch/uncapped.err"
        same "a batch over a tier with no turn cap and no --max-turns refuses with 2" "2" "$?"
        present "and it names the missing cap" "declares no \`max_turns\`" "$scratch/uncapped.err"
        if [ -e "$scratch/uncapped/trees/oracle" ]; then
            fail "and it builds no tree" "$(ls "$scratch/uncapped/trees" | tr '\n' ' ')"
        else
            pass "and it builds no tree"
        fi
        if [ -e "$scratch/uncapped/claude-version" ]; then
            fail "and it runs no harness, not even for its version" "claude-version was written"
        else
            pass "and it runs no harness, not even for its version"
        fi

        # A staggered batch (#1472): one batch of 3 jobs run in slices against
        # one pin. `--max-sessions 2` starts 2 sessions and exits 9 with the
        # third left, and the same command again starts the third alone. A
        # harness failure that is not the turn cap halts the invocation, so no
        # later job of it starts. The stub harness logs each session it runs.
        stagger_batch() {
            rm -rf "$1"
            mkdir -p "$1/tasks" "$1/sessions" "$1/ws" "$1/trees"
            cp -a "$scratch/bw-base" "$1/trees/oracle"
            cp -a "$scratch/bw-base" "$1/trees/grading"
            cp -a "$scratch/bw-base" "$1/trees/campaign-present"
            git -C "$root" rev-parse HEAD > "$1/head"
            cp "$scratch/task.md" "$1/tasks/HW-PROBE-$tombstone.md"
            : > "$1/jobs"
            for r in 1 2 3; do
                printf 'L1-campaign-present-p1-r%s 1 campaign present sufficiency HW-PROBE-%s\n' \
                    "$r" "$tombstone" >> "$1/jobs"
            done
        }
        # $1 is `ok` or `fail-first`. The stub runs confined, so it logs to
        # the probe log directory, the one path the driver binds read-write
        # for every session, which is `$scratch/probe-log/stagger.calls`.
        stagger_stub() {
            cat > "$scratch/bin/claude" <<STUB
#!/bin/sh
case "\${1:-}" in --version) echo '9.9.9 (Claude Code)'; exit 0 ;; esac
calls=\$HEADWATER_SHADOW_LOG_DIR/stagger.calls
printf '%s %s\n' "\${CLAUDE_CONFIG_DIR:-}" "\$(pwd)" >> "\$calls"
printf '%s\n' '{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s16"}'
if [ "$1" = fail-first ] && [ "\$(wc -l < "\$calls")" -le 1 ]; then
    printf '%s\n' '{"type":"result","subtype":"error_during_execution","is_error":true,"total_cost_usd":0.01,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}'
    exit 1
fi
printf '%s\n' '{"type":"result","subtype":"success","is_error":false,"result":"done","total_cost_usd":0.01,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}'
exit 0
STUB
            chmod +x "$scratch/bin/claude"
        }
        stagger_run() {
            PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$1" \
                --model claude-haiku-4-5 --spec "$scratch/stagger.spec" --repetitions 1 \
                --max-sessions 2 >/dev/null 2>"$2"
        }
        stagger_recorded() {
            n=0
            for s in "$1"/sessions/*/status; do
                [ "$(cat "$s" 2>/dev/null)" = 0 ] && n=$((n + 1))
            done
            echo "$n"
        }
        printf 'campaign present sufficiency\n' > "$scratch/stagger.spec"
        stagger=$scratch/stagger
        stagger_batch "$stagger"
        rm -f "$scratch/probe-log/stagger.calls"
        stagger_stub ok
        stagger_run "$stagger" "$scratch/stagger-1.err"
        same "a slice of --max-sessions 2 over 3 jobs exits 9, jobs remain" "9" "$?"
        same "and records exactly 2 sessions" "2" "$(stagger_recorded "$stagger")"
        same "and runs the harness for 2 sessions" "2" "$(wc -l < "$scratch/probe-log/stagger.calls" 2>/dev/null | tr -d ' ')"
        present "and says how many remain" "2 of 3 sessions recorded, 1 remain; run the same command again" "$scratch/stagger-1.err"
        stagger_run "$stagger" "$scratch/stagger-2.err"
        same "the same command again finishes the batch with 0" "0" "$?"
        same "and all 3 jobs are recorded" "3" "$(stagger_recorded "$stagger")"
        same "and the harness ran 3 times in all, none twice" "3 3" \
            "$(LC_ALL=C sort -u "$scratch/probe-log/stagger.calls" | wc -l | tr -d ' ') $(wc -l < "$scratch/probe-log/stagger.calls" | tr -d ' ')"

        # A draw that names the probe for its first session alone leaves the
        # other two with no phrasing, so the batch exits 5 before it asks the
        # harness its version or runs a session (#1875).
        stagger_batch "$stagger"
        rm -f "$scratch/probe-log/stagger.calls"
        printf 'draw: HW-PROBE-%s 1\n' "$tombstone" > "$scratch/stagger.draw"
        HEADWATER_CAMPAIGN_DRAW=$scratch/stagger.draw PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$stagger" \
            --model claude-haiku-4-5 --spec "$scratch/stagger.spec" --repetitions 1 --max-sessions 2 >/dev/null 2>"$scratch/stagger-draw.err"
        same "a batch whose draw does not cover a job exits 5" "5" "$?"
        same "before it asks the harness its version or runs a session" "no no" \
            "$([ -e "$stagger/claude-version" ] && echo yes || echo no) $([ -e "$scratch/probe-log/stagger.calls" ] && echo yes || echo no)"
        present "and names the session the draw leaves out" "fewer than 2 times" "$scratch/stagger-draw.err"
        printf 'draw: HW-PROBE-%s 1\ndraw: HW-PROBE-%s 1\ndraw: HW-PROBE-%s 1\n' "$tombstone" "$tombstone" "$tombstone" > "$scratch/stagger.draw"
        stagger_batch "$stagger"
        HEADWATER_CAMPAIGN_DRAW=$scratch/stagger.draw PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$stagger" \
            --model claude-haiku-4-5 --spec "$scratch/stagger.spec" --repetitions 1 --max-sessions 2 >/dev/null 2>"$scratch/stagger-draw.err"
        same "a draw of 1 for each session of a task with no phrasing runs the slice" "9 2" \
            "$? $(stagger_recorded "$stagger")"
        rm -f "$scratch/probe-log/stagger.calls"

        # The bound holds across parallel workers: 3 workers, 3 jobs, 2 tokens.
        # A token taken without O_EXCL lets a third worker through in about
        # one trial of three on a host whose `mkdir` checks and then creates,
        # so the case runs 6 trials and reports the first that overran.
        trial=1
        overran=
        while [ "$trial" -le 6 ] && [ -z "$overran" ]; do
            stagger_batch "$stagger"
            rm -f "$scratch/probe-log/stagger.calls"
            PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$stagger" \
                --model claude-haiku-4-5 --spec "$scratch/stagger.spec" --repetitions 1 \
                --max-sessions 2 --parallel 3 >/dev/null 2>"$scratch/stagger-p.err"
            got="$? $(wc -l < "$scratch/probe-log/stagger.calls" 2>/dev/null | tr -d ' ')"
            [ "$got" = "9 2" ] || overran="trial $trial: $got"
            trial=$((trial + 1))
        done
        same "3 parallel workers under --max-sessions 2 start 2 sessions and exit 9, in 6 trials" "" "$overran"

        # The halt: the first session fails for a reason other than the cap.
        stagger_batch "$stagger"
        rm -f "$scratch/probe-log/stagger.calls"
        stagger_stub fail-first
        stagger_run "$stagger" "$scratch/stagger-3.err"
        same "a harness failure that is not the cap halts the slice with 9" "9" "$?"
        same "and no later job of the slice starts" "1" "$(wc -l < "$scratch/probe-log/stagger.calls" 2>/dev/null | tr -d ' ')"
        present "and the stop names the halt" "halt" "$scratch/stagger-3.err"
        rm -f "$scratch/probe-log/stagger.calls"
        stagger_stub ok
        PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$stagger" \
            --model claude-haiku-4-5 --spec "$scratch/stagger.spec" --repetitions 1 \
            >/dev/null 2>"$scratch/stagger-4.err"
        same "and a rerun with a working harness finishes all 3" "0 3" "$? $(stagger_recorded "$stagger")"

        # A spent session the recorder could not record is accepted once, by
        # a person, as a counted non-record (#1659). Four sessions of the
        # 2026-10-03 campaign exited 5 after a refused write, and a re-run of
        # the batch would have drawn each of them again and paid again.
        unrec=$scratch/unrec
        stagger_batch "$unrec"
        rm -f "$scratch/probe-log/stagger.calls"
        stagger_stub ok
        stagger_run "$unrec" "$scratch/unrec-1.err"
        # The slice recorded two jobs and left one. The one it left becomes
        # the session whose recorder exited 5, with no cost yet.
        u0= u3=
        for r in 1 2 3; do
            if [ -f "$unrec/sessions/L1-campaign-present-p1-r$r/status" ]; then
                u0=L1-campaign-present-p1-r$r
            else
                u3=L1-campaign-present-p1-r$r
            fi
        done
        mkdir -p "$unrec/sessions/$u3"
        echo 5 > "$unrec/sessions/$u3/status"
        echo campaign > "$unrec/sessions/$u3/tier"
        : > "$unrec/sessions/$u3/started"
        echo 'probe-transform: the artifact docs/x.md could not be copied into the oracle tree.' > "$unrec/sessions/$u3/record.err"
        sh "$root/tools/probe/campaign.sh" --out "$unrec" --unrecorded "$u0" >/dev/null 2>"$scratch/unrec-2.err"
        same "\`--unrecorded\` refuses a session that recorded, with 2" "2" "$?"
        present "and says why" "has status 0" "$scratch/unrec-2.err"
        sh "$root/tools/probe/campaign.sh" --out "$unrec" --unrecorded "$u3" >/dev/null 2>"$scratch/unrec-3.err"
        same "\`--unrecorded\` refuses a status-5 session that states no cost, with 2" "2" "$?"
        if [ -e "$unrec/unrecorded" ]; then
            fail "and a refusal writes no marker" "$(ls "$unrec/unrecorded" | tr '\n' ' ')"
        else
            pass "and a refusal writes no marker"
        fi
        sh "$root/tools/probe/campaign.sh" --out "$unrec" --unrecorded L9-not-a-job >/dev/null 2>"$scratch/unrec-3b.err"
        same "\`--unrecorded\` refuses a name that is not a job of the batch, with 2" "2" "$?"
        : > "$unrec/sessions/$u3/cost"
        sh "$root/tools/probe/campaign.sh" --out "$unrec" --unrecorded "$u3" >/dev/null 2>"$scratch/unrec-3c.err"
        same "\`--unrecorded\` refuses an empty cost file, with 2" "2" "$?"
        echo 'forty' > "$unrec/sessions/$u3/cost"
        sh "$root/tools/probe/campaign.sh" --out "$unrec" --unrecorded "$u3" >/dev/null 2>"$scratch/unrec-3d.err"
        same "\`--unrecorded\` refuses a cost that is not whole cents, with 2" "2" "$?"
        rm -f "$unrec/sessions/$u3/cost"
        echo 40 > "$unrec/sessions/$u3/cost"
        (cd "$unrec/sessions/$u3" && find . -type f -exec sha256sum {} + | LC_ALL=C sort; ls -la --time-style=full-iso .) > "$scratch/unrec-before.txt"
        # With no marker, assembly still fails the line on the status 5.
        sh "$root/tools/probe/campaign.sh" --out "$unrec" --assemble >/dev/null 2>"$scratch/unrec-4.err"
        same "assembly of an unmarked status 5 still exits 7" "7" "$?"
        # And the tally does not take an unmarked status 5 for one. A cap of
        # one cent refuses the job, so nothing runs and nothing spends.
        calls_before=$(wc -l < "$scratch/probe-log/stagger.calls" | tr -d ' ')
        PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$unrec" \
            --model claude-haiku-4-5 --spec "$scratch/stagger.spec" --repetitions 1 \
            --cap-cents 1 >/dev/null 2>"$scratch/unrec-4b.err"
        same "a slice over an unmarked status 5 still exits 7, and runs nothing" "7 $calls_before" \
            "$? $(wc -l < "$scratch/probe-log/stagger.calls" | tr -d ' ')"
        rm -f "$unrec/cap"
        : > "$unrec/cap"
        sh "$root/tools/probe/campaign.sh" --out "$unrec" --unrecorded "$u3" >/dev/null 2>"$scratch/unrec-5.err"
        same "\`--unrecorded\` accepts a spent status-5 session" "0" "$?"
        present "and its marker holds the recorder's last line" "could not be copied into the oracle tree" "$unrec/unrecorded/$u3"
        (cd "$unrec/sessions/$u3" && find . -type f -exec sha256sum {} + | LC_ALL=C sort; ls -la --time-style=full-iso .) > "$scratch/unrec-after.txt"
        if cmp -s "$scratch/unrec-before.txt" "$scratch/unrec-after.txt"; then
            pass "and it touches no file of the session directory"
        else
            fail "and it touches no file of the session directory" "$(diff "$scratch/unrec-before.txt" "$scratch/unrec-after.txt" | head -4 | tr '\n' ' ')"
        fi
        calls_before=$(wc -l < "$scratch/probe-log/stagger.calls" | tr -d ' ')
        stagger_run "$unrec" "$scratch/unrec-6.err"
        same "a slice over a marked job exits 0, not 7" "0" "$?"
        same "and does not draw the marked job again" "$calls_before" "$(wc -l < "$scratch/probe-log/stagger.calls" | tr -d ' ')"
        same "and leaves its status 5" "5" "$(cat "$unrec/sessions/$u3/status")"
        present "and the tally counts it as unrecorded" "1 sessions are unrecorded" "$scratch/unrec-6.err"
        sh "$root/tools/probe/campaign.sh" --out "$unrec" --assemble >/dev/null 2>"$scratch/unrec-7.err"
        same "assembly counts a marked session and exits 0" "0" "$?"
        present "and names it as unrecorded" "$u3 is unrecorded (status 5, 40 cents)" "$scratch/unrec-7.err"
        same "and counts it uncounted, with its cost in the line's" "2 sessions, 42 cents" \
            "$(sed 's/, the intent.*//' "$unrec/assembled/L1-campaign-present-sufficiency.summary")"
        present "and its summary states one session uncounted" "1 sessions uncounted" \
            "$unrec/assembled/L1-campaign-present-sufficiency.summary"

        # `--repin` moves a batch's pin only when nothing a session meets moved
        # (#1659): the engine, each line's plan identity and each task body.
        # The fixture cannot build a second engine at a second commit, so it
        # moves the recorded side instead: the batch is told it was pinned at
        # another commit, and each recorded member is altered in turn.
        # The stub `cargo` on `$scratch/bin` leaves the engine this suite
        # copied into `$CARGO_TARGET_DIR`. Without it, `--repin` ran the real
        # cargo into that directory, and on CI, whose suite engine is a
        # `--release` build, the fresh build could never hash the same.
        repin_from() {
            printf '%s\n' "$1" > "$unrec/head"
            calls_before=$(wc -l < "$scratch/probe-log/stagger.calls" | tr -d ' ')
            PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$unrec" --repin \
                >/dev/null 2>"$scratch/repin.err"
        }
        repin_target=${CARGO_TARGET_DIR:-$root/engine/target}
        case $repin_target in /*) ;; *) repin_target=$PWD/$repin_target ;; esac
        # Make the batch's engine one that no build of this tree writes, as
        # CI's `--release` engine is: a byte past the end of the binary. A
        # `--repin` that ran the real cargo would overwrite it and refuse.
        cp "$repin_target/dev-release/headwater" "$scratch/repin-binary.keep"
        printf '\n' >> "$repin_target/dev-release/headwater"
        sha256sum "$repin_target/dev-release/headwater" | awk '{ print $1 }' > "$unrec/engine.sha256"
        rm -f "$unrec/pins"
        oldpin=0000000000000000000000000000000000000001
        cp "$unrec/plans/L1" "$scratch/repin-plan.keep"
        task_file=$unrec/tasks/HW-PROBE-$tombstone.md
        cp "$task_file" "$scratch/repin-task.keep"
        cp "$unrec/engine.sha256" "$scratch/repin-engine.keep"

        sed -i 's/^tree: sha256:./tree: sha256:X/' "$unrec/plans/L1"
        repin_from "$oldpin"
        same "\`--repin\` refuses with 4 when a classified document moved the plan's tree" "4" "$?"
        present "and names the member" "the plan's \`tree\` moved" "$scratch/repin.err"
        same "and leaves the pin" "$oldpin" "$(cat "$unrec/head")"
        cp "$scratch/repin-plan.keep" "$unrec/plans/L1"
        # Each of the other five members of the identity refuses on its own.
        for member in lock selection read_set seed harness; do
            sed -i "s/^$member: .*/$member: moved/" "$unrec/plans/L1"
            repin_from "$oldpin"
            same "\`--repin\` refuses with 4 when the plan's $member moved, and names it" "4 1" \
                "$? $(grep -c "the plan's \`$member\` moved" "$scratch/repin.err")"
            cp "$scratch/repin-plan.keep" "$unrec/plans/L1"
        done
        # A dirty checkout is no pin. The suite writes nothing in the checkout
        # under test, so a `git` shim on the path reports a modified file to
        # `status` and passes every other call to the real `git`.
        mkdir -p "$scratch/dirty-bin"
        printf '#!/bin/sh\nfor a in "$@"; do [ "$a" = status ] && { echo " M LICENSE"; exit 0; }; done\nexec %s "$@"\n' \
            "$(command -v git)" > "$scratch/dirty-bin/git"
        chmod +x "$scratch/dirty-bin/git"
        printf '%s\n' "$oldpin" > "$unrec/head"
        PATH="$scratch/dirty-bin:$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$unrec" --repin \
            >/dev/null 2>"$scratch/repin.err"
        same "\`--repin\` refuses a dirty checkout with 4" "4" "$?"
        present "and says why" "uncommitted changes" "$scratch/repin.err"

        # A probe the batch draws whose recorded task is gone is a task that
        # moved, not one to skip.
        mv "$task_file" "$scratch/repin-task.gone"
        repin_from "$oldpin"
        same "\`--repin\` refuses with 4 when a recorded task file is missing" "4" "$?"
        mv "$scratch/repin-task.gone" "$task_file"

        printf 'another task\n' >> "$task_file"
        repin_from "$oldpin"
        same "\`--repin\` refuses with 4 when a probe's task moved" "4" "$?"
        present "and names the probe" "\`## Task\` of HW-PROBE-$tombstone moved" "$scratch/repin.err"
        cp "$scratch/repin-task.keep" "$task_file"

        # A batch that recorded phrasings of a task that has none now moved
        # its task too: the count of phrasings is part of it (#1875).
        printf '2\n' > "$unrec/tasks/HW-PROBE-$tombstone.phrasings"
        repin_from "$oldpin"
        same "\`--repin\` refuses with 4 when a probe's count of phrasings moved" "4" "$?"
        rm -f "$unrec/tasks/HW-PROBE-$tombstone.phrasings"

        echo 0000 > "$unrec/engine.sha256"
        repin_from "$oldpin"
        same "\`--repin\` refuses with 4 when the engine moved" "4" "$?"
        present "and names the engine" "the engine moved" "$scratch/repin.err"
        cp "$scratch/repin-engine.keep" "$unrec/engine.sha256"
        if [ -e "$unrec/pins" ]; then
            fail "and no refusal writes \`pins\`" "$(cat "$unrec/pins")"
        else
            pass "and no refusal writes \`pins\`"
        fi

        repin_from "$oldpin"
        same "\`--repin\` moves the pin when the engine, the plans and the tasks are the same" "0" "$?"
        same "and spends nothing and runs no harness" "$calls_before" "$(wc -l < "$scratch/probe-log/stagger.calls" | tr -d ' ')"
        same "and writes the new pin" "$(git -C "$root" rev-parse HEAD)" "$(cat "$unrec/head")"
        same "and appends the move to \`pins\`, with the recorded count" "$oldpin $(git -C "$root" rev-parse HEAD) 2" \
            "$(awk '{ print $1, $2, $4 }' "$unrec/pins")"
        same "and leaves the recorded plan as it was" "" "$(cmp "$scratch/repin-plan.keep" "$unrec/plans/L1" 2>&1)"
        sh "$root/tools/probe/campaign.sh" --out "$unrec" --assemble >/dev/null 2>"$scratch/unrec-8.err"
        present "assembly prints every move of the pin" "pin moved: $oldpin $(git -C "$root" rev-parse HEAD)" "$scratch/unrec-8.err"
        stagger_run "$unrec" "$scratch/unrec-9.err"
        same "and a slice at the new pin runs" "0" "$?"
        cp "$scratch/repin-binary.keep" "$repin_target/dev-release/headwater"

        # The cap is the batch's, so a resumed slice that does not give one
        # keeps the one recorded. The pin and the harness version hold too.
        stagger_batch "$stagger"
        rm -f "$scratch/probe-log/stagger.calls"
        PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$stagger" \
            --model claude-haiku-4-5 --spec "$scratch/stagger.spec" --repetitions 1 \
            --cap-cents 500 --max-sessions 1 >/dev/null 2>"$scratch/stagger-5.err"
        stagger_run "$stagger" "$scratch/stagger-6.err"
        same "a resumed slice without --cap-cents keeps the recorded cap" "500" "$(cat "$stagger/cap" 2>/dev/null)"
        same "and the first slice recorded the harness version" "9.9.9 (Claude Code)" \
            "$(cat "$stagger/claude-version" 2>/dev/null)"
        printf '1.0.0 (Claude Code)\n' > "$stagger/claude-version"
        rm -f "$scratch/probe-log/stagger.calls"
        stagger_run "$stagger" "$scratch/stagger-7.err"
        same "a slice on another harness version refuses with 4" "4" "$?"
        same "and starts no session" "0" "$(cat "$scratch/probe-log/stagger.calls" 2>/dev/null | wc -l | tr -d ' ')"
        present "and names both versions" "1.0.0 (Claude Code)" "$scratch/stagger-7.err"

        # A job the cap refuses did not record, so it is a failure (7) and
        # not a job left for the next slice (9). A cap of 49 cents is below
        # one declared session of 50, so it refuses every job.
        stagger_batch "$stagger"
        rm -f "$scratch/probe-log/stagger.calls"
        stagger_run_cap() {
            PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$stagger" \
                --model claude-haiku-4-5 --spec "$scratch/stagger.spec" --repetitions 1 \
                --max-sessions 2 --cap-cents 49 >/dev/null 2>"$scratch/stagger-8.err"
        }
        stagger_run_cap
        same "a slice the cap refuses exits 7, not 9, and records nothing" "7 0" \
            "$? $(stagger_recorded "$stagger")"

        # The hard cap (#1659). `--session-cents <B>` is the most one session
        # may spend: the driver passes it to the harness as `--max-budget-usd`,
        # and reserves B, not the declared unit, for each session in flight,
        # under one lock with the `started` marker. So the batch spends at
        # most the cap. The stub "spends" 90 cents with no budget, and
        # min(90, budget) with one, and ends at the budget subtype with exit 1
        # when the budget binds. On the old path, which reserved the declared
        # 50, a cap of 150 started a second session at 90 spent and realized
        # 180, which is the overrun of #1474.
        # $1 is `ok` or `fail-first`. The first $2 sessions (1 by default) of
        # `fail-first` fail for a reason other than the budget, after each
        # spent 50 cents.
        budget_stub() {
            cat > "$scratch/bin/claude" <<STUB
#!/bin/sh
case "\${1:-}" in --version) echo '9.9.9 (Claude Code)'; exit 0 ;; esac
calls=\$HEADWATER_SHADOW_LOG_DIR/stagger.calls
budget=
previous=
for argument in "\$@"; do
    [ "\$previous" = --max-budget-usd ] && budget=\$argument
    previous=\$argument
done
printf '%s budget=%s\n' "\$(pwd)" "\${budget:-none}" >> "\$calls"
printf '%s\n' '{"type":"system","subtype":"init","model":"claude-haiku-4-5","session_id":"s17"}'
if [ "$1" = fail-first ] && [ "\$(wc -l < "\$calls")" -le ${2:-1} ]; then
    printf '%s\n' '{"type":"result","subtype":"error_during_execution","is_error":true,"total_cost_usd":0.50,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}'
    exit 1
fi
cents=90
[ -n "\$budget" ] && cents=\$(awk -v b="\$budget" 'BEGIN { c = int(b * 100 + 0.5); print (c < 90 ? c : 90) }')
usd=\$(awk -v c="\$cents" 'BEGIN { printf "%.2f", c / 100 }')
if [ "\$cents" -lt 90 ]; then
    printf '{"type":"result","subtype":"error_max_budget_usd","is_error":true,"result":"withheld","total_cost_usd":%s,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}\n' "\$usd"
    exit 1
fi
printf '{"type":"result","subtype":"success","is_error":false,"result":"done","total_cost_usd":%s,"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":10}}}\n' "\$usd"
exit 0
STUB
            chmod +x "$scratch/bin/claude"
        }
        # What the batch spent: the cost of every session, and what an earlier
        # attempt of a job spent before it was run again.
        budget_spent() {
            n=0
            for s in "$1"/sessions/*/cost "$1"/sessions/*/spent-before; do
                [ -f "$s" ] && n=$((n + $(cat "$s")))
            done
            echo "$n"
        }
        # $1 out, $2 stderr file, then the flags of the invocation.
        budget_run() {
            run_out=$1 run_err=$2
            shift 2
            PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$run_out" \
                --model claude-haiku-4-5 --spec "$scratch/stagger.spec" --repetitions 1 \
                "$@" >/dev/null 2>"$run_err"
        }
        budget_stub ok
        # At --parallel 1 and 3, over 6 trials each. B is 60 and the cap 150,
        # so a third session would need 120 + 60 = 180: 2 sessions start, each
        # stopped at its budget of 60, and the batch spends 120.
        for workers in 1 3; do
            trial=1
            overran=
            while [ "$trial" -le 6 ] && [ -z "$overran" ]; do
                stagger_batch "$stagger"
                rm -f "$scratch/probe-log/stagger.calls"
                budget_run "$stagger" "$scratch/budget-p$workers.err" \
                    --cap-cents 150 --session-cents 60 --parallel "$workers"
                got="$? $(budget_spent "$stagger") $(wc -l < "$scratch/probe-log/stagger.calls" 2>/dev/null | tr -d ' ')"
                [ "$got" = "7 120 2" ] || overran="trial $trial: exit, cents, sessions $got"
                trial=$((trial + 1))
            done
            same "under --session-cents 60 and --cap-cents 150 at --parallel $workers, 6 trials spend 120 over 2 sessions and exit 7" \
                "" "$overran"
        done
        same "and the harness receives --max-budget-usd 0.60" "2" \
            "$(grep -c ' budget=0.60$' "$scratch/probe-log/stagger.calls" 2>/dev/null)"
        same "and the batch records the session budget" "60" "$(cat "$stagger/session-cents" 2>/dev/null)"
        same "and a session stopped at its budget records with status 0" "2" "$(stagger_recorded "$stagger")"
        PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$stagger" --assemble \
            >/dev/null 2>"$scratch/budget-assemble.err"
        present "and assembly counts the sessions the budget stopped" \
            "0 stopped at the turn cap, 2 stopped at the session budget" "$scratch/budget-assemble.err"
        present "and the transcript says the session stopped at its budget" \
            "The session stopped at its budget of \$0.60." "$(ls "$stagger"/sessions/*/record.md | head -1)"
        # The stub's budget line carries the probe's answer as its `result`,
        # and a session that did not finish states none.
        present "and it records no answer, because the session did not finish" \
            "  answer: null" "$(ls "$stagger"/sessions/*/record.md | head -1)"

        # The cap's edge, in three directions. A session starts when it fits
        # exactly, and not one cent past.
        for edge in "119 1" "120 2" "180 3"; do
            edge_cap=${edge% *}
            edge_n=${edge#* }
            stagger_batch "$stagger"
            rm -f "$scratch/probe-log/stagger.calls"
            budget_run "$stagger" "$scratch/budget-edge.err" --cap-cents "$edge_cap" --session-cents 60
            same "a cap of $edge_cap with --session-cents 60 starts $edge_n sessions" "$edge_n" \
                "$(wc -l < "$scratch/probe-log/stagger.calls" 2>/dev/null | tr -d ' ')"
        done

        # The lock. `HEADWATER_CAMPAIGN_CHECK_DELAY` holds a worker between its
        # check and its `started` marker, so 3 workers that check without one
        # lock all read 0 spent and 0 in flight, and all 3 start.
        stagger_batch "$stagger"
        rm -f "$scratch/probe-log/stagger.calls"
        HEADWATER_CAMPAIGN_CHECK_DELAY=1 budget_run "$stagger" "$scratch/budget-lock.err" \
            --cap-cents 150 --session-cents 60 --parallel 3
        same "3 workers held between check and marker still start 2 sessions under a cap of 150" "2 120" \
            "$(wc -l < "$scratch/probe-log/stagger.calls" 2>/dev/null | tr -d ' ') $(budget_spent "$stagger")"

        # A resumed invocation that gives neither flag keeps the recorded
        # budget and cap, and counts the 60 the first one spent: 60 + 60 fits
        # 150, and 120 + 60 does not.
        stagger_batch "$stagger"
        rm -f "$scratch/probe-log/stagger.calls"
        budget_run "$stagger" "$scratch/budget-r1.err" --cap-cents 150 --session-cents 60 --max-sessions 1
        same "a first slice of 1 under --session-cents 60 exits 9 at 60 cents" "9 60" "$? $(budget_spent "$stagger")"
        budget_run "$stagger" "$scratch/budget-r2.err"
        same "and a resumed slice with neither flag spends 120 in all and exits 7" "7 120" \
            "$? $(budget_spent "$stagger")"
        same "and its session also receives --max-budget-usd 0.60" "2" \
            "$(grep -c ' budget=0.60$' "$scratch/probe-log/stagger.calls" 2>/dev/null)"

        # A halted job's spend counts when it runs again. The first session
        # fails after 50 cents and halts the slice. The next invocation runs
        # it again: 50 + 60 fits 150, and 110 + 60 does not, so the batch
        # spends 110 and the harness runs twice in all. A driver that dropped
        # the 50 when it ran the job again would start a third session and
        # spend 170.
        stagger_batch "$stagger"
        rm -f "$scratch/probe-log/stagger.calls"
        budget_stub fail-first
        budget_run "$stagger" "$scratch/budget-h1.err" --cap-cents 150 --session-cents 60
        same "a first session that fails after 50 cents halts with 9" "9 50" "$? $(budget_spent "$stagger")"
        budget_stub ok
        printf 'x\n' >> "$scratch/probe-log/stagger.calls"
        budget_run "$stagger" "$scratch/budget-h2.err"
        same "and the rerun counts the 50, spends 110 in all, and exits 7" "7 110" \
            "$? $(budget_spent "$stagger")"
        same "and the harness ran twice in all" "3" \
            "$(wc -l < "$scratch/probe-log/stagger.calls" 2>/dev/null | tr -d ' ')"
        present "and the batch's total counts the 50" "110 cents spent" "$scratch/budget-h2.err"

        # Two failed attempts both count (verify of #1659). The job fails at
        # 50 twice, then runs a third time under a cap of 160 at B = 60:
        # 100 + 60 fits, the third attempt spends 60, and the batch spends
        # 160 in all. A driver that kept only the last attempt's 50 would
        # count 110 after it, 50 short of what the harness spent.
        stagger_batch "$stagger"
        rm -f "$scratch/probe-log/stagger.calls"
        budget_stub fail-first 2
        budget_run "$stagger" "$scratch/budget-t1.err" --cap-cents 160 --session-cents 60
        budget_run "$stagger" "$scratch/budget-t2.err"
        same "a job that failed twice at 50 has 100 counted" "9 100" "$? $(budget_spent "$stagger")"
        budget_run "$stagger" "$scratch/budget-t3.err"
        same "and its third attempt spends 160 in all and exits 7" "7 160" "$? $(budget_spent "$stagger")"
        same "and the harness ran 3 times in all" "3" \
            "$(wc -l < "$scratch/probe-log/stagger.calls" 2>/dev/null | tr -d ' ')"
        budget_stub ok

        # The `started` marker is written before the lock goes (verify of
        # #1659). The second seam holds the lock after the marker, so while
        # the lock is held the marker of the job that holds it exists. A
        # marker written after the release leaves a window in which another
        # worker reads the batch with this session in flight and not counted.
        stagger_batch "$stagger"
        rm -f "$scratch/probe-log/stagger.calls"
        HEADWATER_CAMPAIGN_HOLD_DELAY=3 budget_run "$stagger" "$scratch/budget-hold.err" \
            --cap-cents 150 --session-cents 60 &
        held=$!
        waited=0
        while [ ! -f "$stagger/slice/lock" ] && [ "$waited" -lt 600 ]; do
            sleep 0.1
            waited=$((waited + 1))
        done
        sleep 1
        seen="$([ -f "$stagger/slice/lock" ] && echo held || echo free) $(ls "$stagger"/sessions/*/started 2>/dev/null | wc -l | tr -d ' ')"
        wait "$held"
        same "while a worker holds the lock, its started marker exists" "held 1" "$seen"

        # A lock whose holder is gone halts the invocation (verify of #1659):
        # a worker killed inside the lock, or one that returned without
        # releasing it, would otherwise leave every other worker waiting
        # forever. Nothing starts and nothing is spent.
        stagger_batch "$stagger"
        rm -f "$scratch/probe-log/stagger.calls"
        mkdir -p "$stagger/slice/started" "$stagger/slice/refused"
        sh -c 'exit 0' &
        gone=$!
        wait "$gone"
        printf '%s\n' "$gone" > "$stagger/slice/lock"
        printf '\n' > "$stagger/model"
        : > "$stagger/repetitions"
        : > "$stagger/max-turns"
        printf '205200\n' > "$stagger/ceiling.campaign"
        printf '50\n' > "$stagger/unit.campaign"
        PATH="$scratch/bin:$PATH" timeout 60 sh "$root/tools/probe/campaign.sh" --out "$stagger" \
            --job "L1-campaign-present-p1-r1 1 campaign present sufficiency HW-PROBE-$tombstone" \
            >/dev/null 2>"$scratch/budget-gone.err"
        same "a job that finds the lock's holder gone halts and starts nothing" "yes 0" \
            "$([ -f "$stagger/slice/halt" ] && echo yes || echo no) $(cat "$scratch/probe-log/stagger.calls" 2>/dev/null | wc -l | tr -d ' ')"
        present "and says which worker holds it" "the worker $gone holds the batch lock and is gone" "$scratch/budget-gone.err"

        # A holder can release the lock and exit between the waiter's read
        # of its pid and the waiter's test of it (verify 2 of #1659). That
        # holder is gone and the lock with it, so the waiter takes the lock
        # and does not halt. The seam holds the waiter between the read and
        # the test while the lock goes.
        stagger_batch "$stagger"
        rm -f "$scratch/probe-log/stagger.calls"
        mkdir -p "$stagger/slice/started" "$stagger/slice/refused"
        printf '%s\n' "$gone" > "$stagger/slice/lock"
        printf '\n' > "$stagger/model"
        # One repetition, because the plan of the tier's 30 over the
        # thirty-four sufficiency probes since #1874.3 passes the campaign
        # ceiling, and the plan then refuses the session this case needs.
        printf '1\n' > "$stagger/repetitions"
        : > "$stagger/max-turns"
        printf '205200\n' > "$stagger/ceiling.campaign"
        printf '50\n' > "$stagger/unit.campaign"
        HEADWATER_CAMPAIGN_HOLDER_DELAY=2 PATH="$scratch/bin:$PATH" timeout 60 sh "$root/tools/probe/campaign.sh" --out "$stagger" \
            --job "L1-campaign-present-p1-r1 1 campaign present sufficiency HW-PROBE-$tombstone" \
            >/dev/null 2>"$scratch/budget-released.err" &
        waiter=$!
        sleep 1
        rm -f "$stagger/slice/lock"
        wait "$waiter"
        same "a waiter whose holder released the lock before it exited does not halt" "no 1" \
            "$([ -f "$stagger/slice/halt" ] && echo yes || echo no) $(cat "$scratch/probe-log/stagger.calls" 2>/dev/null | wc -l | tr -d ' ')"

        # A worker that finds no token left releases the lock. A slice of 1
        # over 3 jobs: the second job takes no token, and the third must
        # find the lock free, so the invocation exits 9 with no halt.
        stagger_batch "$stagger"
        rm -f "$scratch/probe-log/stagger.calls"
        budget_run "$stagger" "$scratch/budget-tok.err" --max-sessions 1 --cap-cents 500 --session-cents 60
        same "a slice of 1 over 3 jobs records 1 and exits 9 with no halt" "9 1 no" \
            "$? $(stagger_recorded "$stagger") $([ -f "$stagger/slice/halt" ] && echo yes || echo no)"

        # `--session-cents` takes a positive whole number of cents, and a
        # refused value writes nothing (verify of #1659).
        for bad in 0 1x 060 -5; do
            rm -rf "$scratch/bad-budget"
            PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$scratch/bad-budget" \
                --model claude-haiku-4-5 --spec "$scratch/stagger.spec" --session-cents "$bad" \
                >/dev/null 2>"$scratch/bad-budget.err"
            same "--session-cents $bad refuses with 2 and writes no batch" "2 no" \
                "$? $([ -e "$scratch/bad-budget" ] && echo yes || echo no)"
        done

        # Each component arm's tree is built by `ablate.sh` (#1659). The
        # slice-1 live check found no `.mcp.json` in the `mcp` arm's
        # workspace, because only the absent arm was ever ablated. A cap of
        # 1 cent refuses every job, so the batch builds its trees from this
        # checkout and starts no session.
        arms=$scratch/arms
        rm -rf "$arms"
        printf 'campaign mcp navigability\ncampaign no-hook navigability\n' > "$scratch/arms.spec"
        rm -f "$scratch/probe-log/stagger.calls"
        PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$arms" \
            --model claude-haiku-4-5 --spec "$scratch/arms.spec" --repetitions 1 \
            --cap-cents 1 >/dev/null 2>"$scratch/arms.err"
        same "a batch the cap refuses whole exits 7 and runs no harness" "7 0" \
            "$? $(cat "$scratch/probe-log/stagger.calls" 2>/dev/null | wc -l | tr -d ' ')"
        same "and the mcp arm's tree holds the server the arm adds" "yes" \
            "$([ -f "$arms/trees/campaign-mcp/.mcp.json" ] && echo yes || echo no)"
        same "and the present tree does not" "no" \
            "$([ -f "$arms/trees/oracle/.mcp.json" ] && echo yes || echo no)"
        same "and the no-hook arm's tree lost the hook's script and kept the settings" "no yes" \
            "$([ -f "$arms/trees/campaign-no-hook/.claude/hooks/intent.sh" ] && echo yes || echo no) $([ -f "$arms/trees/campaign-no-hook/.claude/settings.json" ] && echo yes || echo no)"
        same "and the present tree kept it" "yes" \
            "$([ -f "$arms/trees/oracle/.claude/hooks/intent.sh" ] && echo yes || echo no)"
        rm -rf "$arms"

        # A raise above the tier's declared count is admitted on a line only
        # up to the count the dry run prices for that line (#1659). The line
        # leaves out of discovery what each discovery line of the layer
        # campaign leaves out: the authoring-skill probe, and the relocation
        # probe that awaits spend (#1833). So k = 3 and the line prices at
        # ceil(353 / 3) = 118 from this checkout's `power:`.
        printf 'campaign present discovery HW-PROBE-the-authoring-skill-reaches-an-agent-that-is-about-to-write-a-governed-document HW-PROBE-a-session-that-moves-a-shelf-reaches-the-relocation-guide\n' \
            > "$scratch/raise.spec"
        stagger_stub ok
        for over in 119 1180; do
            raise=$scratch/raise-$over
            rm -rf "$raise"
            rm -f "$scratch/probe-log/stagger.calls"
            PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$raise" \
                --model claude-haiku-4-5 --spec "$scratch/raise.spec" --repetitions "$over" \
                --cap-cents 100 >/dev/null 2>"$raise.err"
            same "--repetitions $over above the 118 a discovery line prices refuses with 5, runs no harness and builds no tree" \
                "5 0 no" \
                "$? $(cat "$scratch/probe-log/stagger.calls" 2>/dev/null | wc -l | tr -d ' ') $([ -e "$raise/trees" ] && echo yes || echo no)"
            present "and it names the line, the count asked and the count priced" \
                "line 1 (campaign present discovery): --repetitions $over is above the 118 the dry run prices for it (ceil(353 / 3), powered)" \
                "$raise.err"
            rm -rf "$raise" "$raise.err"
        done
        # The price is read with the engine the batch plans with, the one its
        # build wrote under CARGO_TARGET_DIR, and not the checkout's own
        # `engine/target` (verify of #1659 slice 1b). The engine there is a
        # wrapper that logs each call, so the priced pass, a plan with no
        # --repetitions, must be in its log. A count above the price refuses
        # before any tree is built, so the wrapper is never copied into one.
        mkdir -p "$scratch/wrap-target/dev-release"
        printf '#!/bin/sh\nprintf "%%s\\n" "$*" >> "%s/wrap.calls"\nexec "%s" "$@"\n' "$scratch" "$engine" \
            > "$scratch/wrap-target/dev-release/headwater"
        chmod +x "$scratch/wrap-target/dev-release/headwater"
        rm -f "$scratch/wrap.calls"
        raise=$scratch/raise-wrap
        rm -rf "$raise"
        CARGO_TARGET_DIR=$scratch/wrap-target PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" \
            --out "$raise" --model claude-haiku-4-5 --spec "$scratch/raise.spec" --repetitions 119 \
            --cap-cents 100 >/dev/null 2>"$raise.err"
        same "the priced pass plans the line with the engine under CARGO_TARGET_DIR, with no --repetitions" "5 yes no" \
            "$? $(grep -q '^probe plan ' "$scratch/wrap.calls" 2>/dev/null && echo yes || echo no) $(grep -q -- '--repetitions' "$scratch/wrap.calls" 2>/dev/null && echo yes || echo no)"
        rm -rf "$raise" "$raise.err" "$scratch/wrap-target" "$scratch/wrap.calls"
        # At the priced count the batch runs: each session is planned at the
        # tier's 30, so `probe-record.sh` does not refuse it with 11, and the
        # job list holds 3 x 118 jobs.
        raise=$scratch/raise-118
        rm -rf "$raise"
        rm -f "$scratch/probe-log/stagger.calls"
        PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$raise" \
            --model claude-haiku-4-5 --spec "$scratch/raise.spec" --repetitions 118 \
            --cap-cents 100 --max-sessions 1 >/dev/null 2>"$raise.err"
        same "--repetitions 118 on that line runs one session and stops on --max-sessions with 9" "9" "$?"
        same "and the session records with status 0, not 11" "1" "$(stagger_recorded "$raise")"
        same "and the job list holds 3 probes x 118 repetitions" "354" "$(wc -l < "$raise/jobs" | tr -d ' ')"
        same "and the batch records the count asked" "118" "$(cat "$raise/repetitions")"
        # A resumed invocation that gives no --repetitions keeps the count
        # the batch recorded, and still plans each session at the tier's 30.
        PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$raise" \
            --model claude-haiku-4-5 --spec "$scratch/raise.spec" \
            --max-sessions 1 >/dev/null 2>"$raise-resume.err"
        same "a resumed invocation with no --repetitions runs one more session and exits 9" "9 2" \
            "$? $(stagger_recorded "$raise")"
        same "and keeps the recorded count and the job list" "118 354" \
            "$(cat "$raise/repetitions") $(wc -l < "$raise/jobs" | tr -d ' ')"
        # A raise on a resumed batch is checked against the price as well.
        rm -f "$scratch/probe-log/stagger.calls"
        PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$raise" \
            --model claude-haiku-4-5 --spec "$scratch/raise.spec" --repetitions 119 \
            --max-sessions 1 >/dev/null 2>"$raise-resume.err"
        same "a resumed invocation that asks 119 refuses with 5 and runs no harness" "5 0" \
            "$? $(cat "$scratch/probe-log/stagger.calls" 2>/dev/null | wc -l | tr -d ' ')"
        same "and leaves the recorded count as it was" "118" "$(cat "$raise/repetitions")"
        rm -rf "$raise" "$raise.err" "$raise-resume.err"
        # Without --repetitions nothing changes: the line runs the tier's 30.
        raise=$scratch/raise-none
        rm -rf "$raise"
        rm -f "$scratch/probe-log/stagger.calls"
        PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$raise" \
            --model claude-haiku-4-5 --spec "$scratch/raise.spec" \
            --cap-cents 1 >/dev/null 2>"$raise.err"
        same "with no --repetitions the line plans at the tier's 30: exit 7, no harness, 3 x 30 jobs" "7 0 90" \
            "$? $(cat "$scratch/probe-log/stagger.calls" 2>/dev/null | wc -l | tr -d ' ') $(wc -l < "$raise/jobs" | tr -d ' ')"
        rm -rf "$raise" "$raise.err"
        # A line not under `pooled:` prices at the tier's declared count, so
        # it admits no raise.
        printf 'campaign present sufficiency\n' > "$scratch/raise-flat.spec"
        raise=$scratch/raise-flat
        rm -rf "$raise"
        rm -f "$scratch/probe-log/stagger.calls"
        PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$raise" \
            --model claude-haiku-4-5 --spec "$scratch/raise-flat.spec" --repetitions 118 \
            --cap-cents 100 >/dev/null 2>"$raise.err"
        same "--repetitions 118 on a sufficiency line refuses with 5 and builds no tree" "5 0 no" \
            "$? $(cat "$scratch/probe-log/stagger.calls" 2>/dev/null | wc -l | tr -d ' ') $([ -e "$raise/trees" ] && echo yes || echo no)"
        present "and it names the tier's count as the price" \
            "line 1 (campaign present sufficiency): --repetitions 118 is above the 30 the dry run prices for it (the tier's 30)" \
            "$raise.err"
        # A spec that mixes the two is held line by line: the discovery line
        # admits 118 and the sufficiency line does not.
        printf 'campaign present discovery HW-PROBE-the-authoring-skill-reaches-an-agent-that-is-about-to-write-a-governed-document\ncampaign present sufficiency\n' \
            > "$scratch/raise-mixed.spec"
        rm -rf "$raise"
        PATH="$scratch/bin:$PATH" sh "$root/tools/probe/campaign.sh" --out "$raise" \
            --model claude-haiku-4-5 --spec "$scratch/raise-mixed.spec" --repetitions 118 \
            --cap-cents 100 >/dev/null 2>"$raise.err"
        same "a spec of a discovery line and a sufficiency line at 118 refuses with 5" "5" "$?"
        present "and names the sufficiency line" "line 2 (campaign present sufficiency): --repetitions 118 is above the 30" "$raise.err"
        rm -rf "$raise" "$raise.err"
        rm -rf "$stagger"
        rm -f "$scratch/bin/cargo"
        unset CARGO_TARGET_DIR
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
    unset HEADWATER_PROBE_LOG_DIR
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
    present "it prints the final round's sum" "round final: 1540 sessions, \$770.00" "$scratch/add.out"
    present "and one total" "total: 13244 sessions, \$6622.00" "$scratch/add.out"
    present "and the cached share of input, with no batch recorded, as no recorded sessions (#1875)" \
        "cached share: no recorded sessions" "$scratch/add.out"
    present "each round's count is computed from its rates" "so ceil(219 / 22) = 10 repetitions of each probe" "$scratch/add.out"
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
    present "and the final round as not approved" "round final: not approved, so the batch refuses its lines with 5" "$scratch/add.out"
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
    present "and prints the round sums all the same" "total: 13244 sessions" "$scratch/add-body.out"

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
    sed 's/^    rates: \[0\.90, 0\.80\]/    rates: [0.95, 0.80]/' "$root/.headwater/probe.yml" > "$scratch/add-rates.yml"
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
    same "and gives each its round's count, 1 repetition on a screened tuning line" "4 1 10" \
        "$(awk '$1 == 1 || $1 == 8 || $1 == 15 { print $3 }' "$scratch/add-priced.out" | tr '\n' ' ' | sed 's/ $//')"
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
        'base_ceiling_cents: 26701|together 130001, above the round'"'"'s `approved_cents` of 130000'; do
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
    same "declared.py tuning prints the base ceiling of the declaration" "base_ceiling_cents 26700" \
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
        same "the whole campaign's spec is still refused with 5, now on the round nobody approved" "5" "$?"
        present "and the refusal names the final round" "no person has approved spend on the final round" "$scratch/gate-clean.err"
        absent "and not the tuning round, which the owner approved (#1777)" "approved spend on the tuning round" "$scratch/gate-clean.err"
        absent "and not the screening round" "approved spend on the screening round" "$scratch/gate-clean.err"
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
        printf 'additive scenario-4 final\n' > "$scratch/run-final.spec"
        rm -rf "$scratch/run-final-out"
        HEADWATER_PROBE_HOST_CONFIG="$scratch/run-host" CARGO_TARGET_DIR="$scratch/gate-target" PATH="$scratch/run-bin:$PATH" \
            sh "$root/tools/probe/campaign.sh" --out "$scratch/run-final-out" --model claude-haiku-4-5 \
            --spec "$scratch/run-final.spec" > /dev/null 2> "$scratch/run-final.err"
        same "a line of the final round, which nobody approved, is refused with 5" "5" "$?"
        present "and the refusal names the round" "no person has approved spend on the final round" "$scratch/run-final.err"
        same "and it wrote no harness version, no plan, no tree and no session" "no no no no" \
            "$([ -e "$scratch/run-final-out/claude-version" ] && echo yes || echo no) $([ -e "$scratch/run-final-out/lines" ] && echo yes || echo no) $([ -e "$scratch/run-final-out/trees" ] && echo yes || echo no) $([ -e "$scratch/run-final-out/sessions" ] && echo yes || echo no)"
        same "and it ran no dry run, so it built no scenario tree" "no" \
            "$([ -e "$scratch/run-final-out/leak-gate" ] && echo yes || echo no)"
        # The owner approved the tuning round (#1877), and the layer it has
        # open is the engine on scenario 2, over its base, scenario 1. So a
        # batch of a scenario-3 tuning line, which is neither, passes the
        # round gate and is refused by the layer gate, with 5, before it
        # builds a tree or asks the harness anything. The case of a declared
        # open layer reads a scratch root, below, because the gate reads this
        # checkout's declaration and no override.
        printf 'additive scenario-3 tuning\n' > "$scratch/run-tune.spec"
        rm -rf "$scratch/run-tune-out"
        HEADWATER_PROBE_HOST_CONFIG="$scratch/run-host" CARGO_TARGET_DIR="$scratch/gate-target" PATH="$scratch/run-bin:$PATH" \
            sh "$root/tools/probe/campaign.sh" --out "$scratch/run-tune-out" --model claude-haiku-4-5 \
            --spec "$scratch/run-tune.spec" --max-sessions 1 > /dev/null 2> "$scratch/run-tune.err"
        same "a batch of a scenario-3 tuning line is refused with 5 by the layer gate" "5" "$?"
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
# the heading and without any other phrasing (spec 15).
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
same "and the final round's rates beside it" "final held-out 0.9 0.8 1 22" \
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
