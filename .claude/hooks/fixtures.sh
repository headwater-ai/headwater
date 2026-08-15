#!/bin/sh
# What holds the three harness hooks: one fixture per outcome each hook can
# reach, driven through the same standard input the harness writes.
#
# Spec 12 refuses a check that ships with no failing fixture. A hook is not a
# check, and the reason still applies: a hook that has never refused anything is
# a hook nobody has seen work. So every refusal below is provoked on purpose,
# and every silence is provoked on purpose too, because a hook that says nothing
# when it should have spoken looks exactly like a hook that had nothing to say.
#
# Run it from anywhere:
#     sh .claude/hooks/fixtures.sh
#
# It needs a built engine for the cases that call one, and it says which cases
# it skipped when there is none. It writes one file under `docs/obligations/`
# for the review case and removes it again, and it removes it on an interrupt.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
hooks="$root/.claude/hooks"
engine="$root/engine/target/release/headwater"
export HEADWATER_HOOK_ROOT="$root"

passed=0
failed=0
skipped=0

# Run a hook with a JSON object on standard input, and hold the result against
# an expected exit status and an expected substring of the output. An empty
# expectation asserts that the hook wrote nothing at all.
expect() {
    name=$1 script=$2 status=$3 substring=$4 payload=$5
    out=$(printf '%s' "$payload" | sh "$hooks/$script" 2>&1)
    got=$?
    if [ "$got" -ne "$status" ]; then
        printf 'FAIL %s\n  expected exit %s, got %s\n' "$name" "$status" "$got"
        failed=$((failed + 1))
        return
    fi
    case $substring in
        "")
            if [ -n "$out" ]; then
                printf 'FAIL %s\n  expected no output, got:\n%s\n' "$name" "$out"
                failed=$((failed + 1))
                return
            fi
            ;;
        *)
            case $out in
                *"$substring"*) ;;
                *)
                    printf 'FAIL %s\n  expected output to hold %s, got:\n%s\n' "$name" "$substring" "$out"
                    failed=$((failed + 1))
                    return
                    ;;
            esac
            ;;
    esac
    printf 'ok   %s\n' "$name"
    passed=$((passed + 1))
}

skip() {
    printf 'skip %s (%s)\n' "$1" "$2"
    skipped=$((skipped + 1))
}

printf '# intent.sh, on UserPromptSubmit\n'
if [ -x "$engine" ]; then
    expect 'a task the corpus answers reaches the agent as pointers' \
        intent.sh 0 'docs/spec/' \
        '{"hook_event_name":"UserPromptSubmit","user_input":"what does a check know about the front matter of a document"}'
    expect 'a task no purpose answers is silent, and never a wrong pointer' \
        intent.sh 0 '' \
        '{"hook_event_name":"UserPromptSubmit","user_input":"xyzzy plugh frobnicate quuxbar"}'
    expect 'the older field name for the prompt is read too' \
        intent.sh 0 'route' \
        '{"hook_event_name":"UserPromptSubmit","prompt":"what does a check know about the front matter of a document"}'
else
    skip 'intent.sh cases that call the engine' 'no built engine'
fi
expect 'an input with no prompt in it is silent rather than an error' \
    intent.sh 0 '' \
    '{"hook_event_name":"UserPromptSubmit"}'
expect 'an input that will not parse is silent rather than an error' \
    intent.sh 0 '' \
    'not json at all'

printf '\n# write.sh, on PreToolUse: backfill\n'
expect 'a new document under the corpus root is refused, and the refusal names the verb' \
    write.sh 0 'headwater new' \
    '{"hook_event_name":"PreToolUse","tool_name":"Write","tool_input":{"file_path":"docs/obligations/9999-a-record-nobody-scaffolded.md"}}'
expect 'the refusal is a deny decision the harness can act on' \
    write.sh 0 '"permissionDecision":"deny"' \
    '{"hook_event_name":"PreToolUse","tool_name":"Write","tool_input":{"file_path":"docs/decisions/9999-a-decision-nobody-scaffolded.md"}}'
expect 'an absolute path inside the repository is refused the same way' \
    write.sh 0 '"permissionDecision":"deny"' \
    "{\"hook_event_name\":\"PreToolUse\",\"tool_name\":\"Write\",\"tool_input\":{\"file_path\":\"$root/docs/obligations/9999-absolute.md\"}}"
expect 'an edit to a document that already exists passes' \
    write.sh 0 '' \
    '{"hook_event_name":"PreToolUse","tool_name":"Edit","tool_input":{"file_path":"docs/spec/05-ai-integration.md"}}'
expect 'a file outside the corpus root passes' \
    write.sh 0 '' \
    '{"hook_event_name":"PreToolUse","tool_name":"Write","tool_input":{"file_path":"engine/crates/query/src/nothing.rs"}}'
expect 'a new Markdown file outside the corpus root passes' \
    write.sh 0 '' \
    '{"hook_event_name":"PreToolUse","tool_name":"Write","tool_input":{"file_path":"engine/crates/query/NOTES.md"}}'
expect 'a path the corpus descriptor excludes passes' \
    write.sh 0 '' \
    '{"hook_event_name":"PreToolUse","tool_name":"Write","tool_input":{"file_path":"docs/taxonomies/decision-record/fixtures/corpus/docs/decisions/0007-a-fixture.md"}}'
expect 'a file that is not Markdown passes' \
    write.sh 0 '' \
    '{"hook_event_name":"PreToolUse","tool_name":"Write","tool_input":{"file_path":"docs/obligations/notes.txt"}}'

printf '\n# write.sh, on PostToolUse: impact detection\n'
if [ -x "$engine" ]; then
    expect 'an edit to a path a document governs names that document' \
        write.sh 0 'docs/spec/05-ai-integration.md' \
        '{"hook_event_name":"PostToolUse","tool_name":"Edit","tool_input":{"file_path":".claude/hooks/write.sh"}}'
    expect 'the advisory says it blocks nothing' \
        write.sh 0 'Nothing here blocks the edit' \
        '{"hook_event_name":"PostToolUse","tool_name":"Edit","tool_input":{"file_path":".claude/hooks/intent.sh"}}'
    expect 'an edit to a path nothing governs is silent' \
        write.sh 0 '' \
        '{"hook_event_name":"PostToolUse","tool_name":"Edit","tool_input":{"file_path":"engine/crates/query/src/lib.rs"}}'
    # HW-OBL-0104. A `governs` edge reaches the path it names and no path
    # under it, so this fixture records the silence rather than asserting the
    # containment that a reader of spec 5 expects.
    expect 'a path under a governed directory is silent, which HW-OBL-0104 holds' \
        write.sh 0 '' \
        '{"hook_event_name":"PostToolUse","tool_name":"Edit","tool_input":{"file_path":".claude/hooks/nothing-governs-this.sh"}}'
else
    skip 'write.sh PostToolUse cases' 'no built engine'
fi

printf '\n# review.sh, on Stop\n'
if [ -x "$engine" ]; then
    expect 'a tree the commit gate passes lets the turn end' \
        review.sh 0 '' \
        '{"hook_event_name":"Stop","stop_hook_active":false}'

    # The failing case. A document with a British spelling in it is an error
    # under `language.controlled.not_met`, which is the fixability bar, and the
    # commit gate refuses it. The hook has to refuse the turn for the same
    # reason and name the same file.
    planted="$root/docs/obligations/9999-a-fixture-that-this-runner-removes.md"
    trap 'rm -f "$planted"' EXIT INT TERM
    {
        echo '---'
        echo 'id: HW-OBL-9999'
        echo 'title: "A fixture that this runner removes"'
        echo 'status: current'
        echo 'status_since: 2026-08-14'
        echo 'last_verified: 2026-08-14'
        echo 'summary: "A planted document that holds one British spelling, so the review hook has a refusal to make."'
        echo 'provenance:'
        echo '  warrant: asserted'
        echo '  agency: machine'
        echo '  drafted_by: hook-fixtures'
        echo '  activity: draft'
        echo '  evidence_basis: unevidenced'
        echo '---'
        echo ''
        echo '# A fixture that this runner removes'
        echo ''
        echo '## Context'
        echo ''
        echo 'This document states the behaviour of nothing, and the runner that wrote it removes it again.'
        echo ''
        echo '## Obligation'
        echo ''
        echo 'None. The file exists for the length of one assertion.'
        echo ''
        echo '## Discharge'
        echo ''
        echo 'The runner deletes it.'
    } > "$planted"

    expect 'a tree the commit gate refuses stops the turn and names the file' \
        review.sh 2 '9999-a-fixture-that-this-runner-removes.md' \
        '{"hook_event_name":"Stop","stop_hook_active":false}'
    expect 'the refusal carries the remediation the commit gate prints' \
        review.sh 2 'headwater check' \
        '{"hook_event_name":"Stop","stop_hook_active":false}'

    # The loop guard, and it is asserted here rather than over a clean tree on
    # purpose. Over a clean tree the gate exits 0 anyway, so the case passes
    # whether the guard runs or not, and a fixture that cannot fail proves
    # nothing. The planted document makes the guard the only way out.
    expect 'a stop the hook already blocked is let through, so no turn loops' \
        review.sh 0 '' \
        '{"hook_event_name":"Stop","stop_hook_active":true}'

    rm -f "$planted"
    trap - EXIT INT TERM

    expect 'the tree is clean again once the fixture is gone' \
        review.sh 0 '' \
        '{"hook_event_name":"Stop","stop_hook_active":false}'
else
    skip 'review.sh cases that call the engine' 'no built engine'
fi

printf '\n%s passed, %s failed, %s skipped\n' "$passed" "$failed" "$skipped"
[ "$failed" -eq 0 ] || exit 1
[ "$skipped" -eq 0 ] || printf 'Build the engine to run the skipped cases:\n  cargo build --release -p headwater-cli --manifest-path engine/Cargo.toml\n'
exit 0
