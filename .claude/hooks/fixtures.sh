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
export HEADWATER_HOOK_ROOT="$root"

# The engine this suite runs and stages into its scratch roots, resolved the way
# `hw_engine` resolves it: either profile counts and the newer answers. `lib.sh`
# states that rule and the cases below hold it, so this reads it from there
# rather than writing a second copy that could disagree with the one under test.
#
# It named the `release` path alone, which mattered once this repository started
# telling a session to build `--profile dev-release`. A worktree with only that
# binary skipped every case here and exited 0.
. "$hooks/lib.sh"
engine=$(hw_engine) || engine="$root/engine/target/release/headwater"

passed=0
failed=0
skipped=0
skipped_engine=0

# The same, in the negative: run a hook and assert its output does NOT contain
# a substring. `expect` holds what a refusal says; this holds what it must not
# say. `wait.sh` refuses two shapes for two reasons and offers two different
# remedies, and one of those remedies makes the other shape worse, so a case
# that pins them apart is worth as much as either case that pins them down.
refute() {
    name=$1 script=$2 substring=$3 payload=$4
    out=$(printf '%s' "$payload" | sh "$hooks/$script" 2>&1)
    case $out in
        *"$substring"*)
            printf 'FAIL %s\n  output should not contain %s, and does:\n%s\n' \
                "$name" "$substring" "$out"
            failed=$((failed + 1))
            ;;
        *)
            printf 'ok   %s\n' "$name"
            passed=$((passed + 1))
            ;;
    esac
}

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
    # Counted apart, because the two reasons a case skips are not the same
    # fact. A missing engine is a caller's mistake and CI asserts against it
    # below. A missing embedding model is expected on a hosted runner, which
    # fetches no model, and it must not be read as one.
    [ "$2" = 'no built engine' ] && skipped_engine=$((skipped_engine + 1))
    return 0
}

# A stand-in for the engine, writing one document of this runner's choosing.
#
# `intent.sh` decides on a member of what `headwater route --json` writes, and
# two of the states that member can be in are states no corpus produces on
# demand: a document that will not parse, and a pointer set whose report says
# something this runner can recognize. A stand-in is the only way to state
# either. It writes the argument verbatim, so the case above reads as the bytes
# the hook meets.
#
# It stands in for `route` and for nothing else. `json` is passed through to the
# real binary, because the hook reads the harness payload and the routing
# document through the same verb, and a stand-in that answered that verb with a
# route document would be testing the stand-in. The wire-format reader is held
# by `engine/crates/yaml/src/json.rs` and by the cases below that name it.
#
# It needs `$open_root`, so it is only callable inside the fail-open block.
stub_route() {
    printf '%s\n' "$1" > "$open_root/document.json"
    printf '#!/bin/sh\ncase ${1:-} in json) exec "%s" "$@" ;; esac\ncat "%s"\n' \
        "$engine" "$open_root/document.json" \
        > "$open_root/engine/target/release/headwater"
    chmod u+x "$open_root/engine/target/release/headwater"
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
# Two prompts nobody wrote. A harness running agents in parallel submits one
# each time an agent finishes, carrying that agent's whole closing report, and
# another when a peer session sends a message. The route has nothing to say
# about either, and the pointers it offered over them were chosen against text
# no person submitted as an intent.
#
# The hook tests the opening characters rather than searching the body, so a
# question that merely mentions a task notification still routes. The first case
# in this file is that control: it carries ordinary prose and expects pointers.
expect 'a completion notification is not a task, and routes to nothing' \
    intent.sh 0 '' \
    '{"hook_event_name":"UserPromptSubmit","user_input":"<task-notification>\n<task-id>a1</task-id>\n<result>I shipped the check and the fixtures pass.</result>\n</task-notification>"}'
expect 'a message from another session is not a task either' \
    intent.sh 0 '' \
    '{"hook_event_name":"UserPromptSubmit","user_input":"<cross-session-message from=\"peer\">the branch is green</cross-session-message>"}'

expect 'an input with no prompt in it is silent rather than an error' \
    intent.sh 0 '' \
    '{"hook_event_name":"UserPromptSubmit"}'
expect 'an input that will not parse is silent rather than an error' \
    intent.sh 0 '' \
    'not json at all'

printf '\n# lib.sh hw_engine: two profiles build an engine, and the newer one answers\n'
# Every position above and below reaches the engine through this one function,
# and both profiles produce the same binary, so no hook's output can say which
# of the two answered. These cases read the function instead. It is the only
# block here that sources `lib.sh` rather than driving a hook, and the exception
# is the point: the whole of the rule is which path comes back, and nothing
# downstream can observe it.
#
# The binaries are empty and executable. `hw_engine` runs none of them, it tests
# for a file anyone may execute, and a case that copied a real engine in would
# spend a second of I/O to assert a path.
engine_root=$(mktemp -d "${TMPDIR:-/tmp}/headwater-hwengine-XXXXXX")
mkdir -p "$engine_root/engine/target/release" "$engine_root/engine/target/dev-release"

# The function's answer for the tree as it now stands, or `(none)`. It runs in a
# subshell so that sourcing `lib.sh` cannot leak `hw_root` into the cases after
# this block, all of which run against the real root.
engine_pick() {
    (
        HEADWATER_HOOK_ROOT="$engine_root"
        export HEADWATER_HOOK_ROOT
        . "$hooks/lib.sh"
        hw_engine || printf '(none)'
    )
}

engine_case() {
    got=$(engine_pick)
    if [ "$got" = "$2" ]; then
        printf 'ok   %s\n' "$1"
        passed=$((passed + 1))
    else
        printf 'FAIL %s\n  expected %s\n  got      %s\n' "$1" "$2" "$got"
        failed=$((failed + 1))
    fi
}

engine_case 'neither profile built, and the function reports none' '(none)'

: > "$engine_root/engine/target/release/headwater"
chmod u+x "$engine_root/engine/target/release/headwater"
engine_case 'a release binary alone is the engine' \
    "$engine_root/engine/target/release/headwater"

# The case this change exists for. A tree that built only the cheap profile used
# to report no engine at all, so every hook went silent and the commit gate
# passed the commit through unchecked.
rm -f "$engine_root/engine/target/release/headwater"
: > "$engine_root/engine/target/dev-release/headwater"
chmod u+x "$engine_root/engine/target/dev-release/headwater"
engine_case 'a dev-release binary alone is the engine' \
    "$engine_root/engine/target/dev-release/headwater"

# Both built, and the file system decides. A rule that ranked the two profiles
# by name would hand a hook the older binary in one of these two, and the hook
# would then read a change through an engine that predates it.
#
# The two timestamps are stated rather than taken from the order these lines
# run in. Two files written one statement apart can carry one mtime on a file
# system that keeps whole seconds, and the case that wants the second file to be
# the newer one would then report the first. That is a case whose verdict is a
# property of the runner's disk.
: > "$engine_root/engine/target/release/headwater"
chmod u+x "$engine_root/engine/target/release/headwater"
touch -t 202601010000 "$engine_root/engine/target/dev-release/headwater"
touch -t 202601010001 "$engine_root/engine/target/release/headwater"
engine_case 'both built and release is the newer, so release answers' \
    "$engine_root/engine/target/release/headwater"

touch -t 202601010002 "$engine_root/engine/target/dev-release/headwater"
engine_case 'both built and dev-release is the newer, so dev-release answers' \
    "$engine_root/engine/target/dev-release/headwater"

# An exact tie, which the two cases above are shaped to avoid and which this one
# is shaped to provoke. Nothing rests on the answer, because two binaries with
# one mtime are equally current, but the function returns one of them and a
# reader should not have to derive which from the `-nt` operator.
touch -t 202601010003 "$engine_root/engine/target/release/headwater"
touch -t 202601010003 "$engine_root/engine/target/dev-release/headwater"
engine_case 'two binaries of the same age, and release answers' \
    "$engine_root/engine/target/release/headwater"

# A file that exists and that nobody may execute is not an engine, at either
# path. The gate and all three positions test for execution rather than for
# presence, and this states it for the profile that has never carried a case.
chmod a-x "$engine_root/engine/target/dev-release/headwater"
engine_case 'a dev-release binary nobody may execute falls back to release' \
    "$engine_root/engine/target/release/headwater"
chmod a-x "$engine_root/engine/target/release/headwater"
engine_case 'neither binary executable, and the function reports none' '(none)'

rm -rf "$engine_root"

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
# #1008: an edit to spec 5 now carries the reverse advisory, because other
# documents declare an edge onto it. "Passes" is the absence of a refusal.
refute 'an edit to a document that already exists passes' \
    write.sh 'permissionDecision' \
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

printf '\n# write.sh, on PreToolUse: one matcher decides an exclusion\n'
# #319: this hook used to read `.headwater/corpus.json` and test each
# exclusion with Python's `fnmatch`. `headwater_meta::pattern::Pattern` is
# what the census walk matches an existing file against, and nothing compared
# the two. They agree on the plain `prefix/**` exclusion this repository
# declares for real, so the drift never showed up here — it takes a
# mid-pattern `*` to see it: `fnmatch` treats `*` as any run of characters,
# including `/`, so it would call a path four segments deep excluded.
# `Pattern` keeps `*` inside one segment, so it does not, and the write is
# corpus content. This scratch corpus declares exactly that pattern, over
# this repository's own maintained taxonomy source, so a second matcher
# reintroduced anywhere on the path from this hook to the walk answers the
# first case below wrong.
if [ -x "$engine" ]; then
    classify_root=$(mktemp -d "${TMPDIR:-/tmp}/headwater-classify-XXXXXX")
    mkdir -p "$classify_root/.headwater/packages" "$classify_root/docs" "$classify_root/.headwater" \
        "$classify_root/engine/target/release"
    cp -r "$root/taxonomy-source/headwater-standard" "$classify_root/.headwater/packages/headwater-standard"
    # `contents.bundles` is relative to the package directory, and #792 put
    # that directory one level deeper, so the scratch copy is repointed. It is
    # the one scalar `taxonomy publish` rewrites on every artifact it writes.
    sed -i 's|bundles: \.\./\.\./docs/taxonomies|bundles: ../../../docs/taxonomies|' \
        "$classify_root/.headwater/packages/headwater-standard/package.yml"
    cp -r "$root/docs/taxonomies" "$classify_root/docs/taxonomies"
    cp "$root/.headwater/overlay.yml" "$classify_root/.headwater/overlay.yml"
    awk '{print} /^  exclude:$/{
        print "    - path: docs/excluded/*.md"
        print "      reason: a fixture for #319, where a `*` inside one segment and a `*` across `/` disagree."
    }' "$root/.headwater/taxonomy.yml" > "$classify_root/.headwater/taxonomy.yml"
    cp "$engine" "$classify_root/engine/target/release/headwater"

    if resolved=$("$classify_root/engine/target/release/headwater" taxonomy resolve --root "$classify_root" 2>&1); then
        HEADWATER_HOOK_ROOT="$classify_root"
        export HEADWATER_HOOK_ROOT
        expect 'a path four segments deep is corpus content under the segment-aware matcher, so the write is refused' \
            write.sh 0 '"permissionDecision":"deny"' \
            '{"hook_event_name":"PreToolUse","tool_name":"Write","tool_input":{"file_path":"docs/excluded/sub/dir.md"}}'
        expect 'a path one segment deep is excluded under either matcher, so the write passes' \
            write.sh 0 '' \
            '{"hook_event_name":"PreToolUse","tool_name":"Write","tool_input":{"file_path":"docs/excluded/dir.md"}}'
        HEADWATER_HOOK_ROOT="$root"
        export HEADWATER_HOOK_ROOT
    else
        printf 'FAIL one matcher decides an exclusion (setup)\n  the scratch corpus did not resolve:\n%s\n' "$resolved"
        failed=$((failed + 1))
    fi
    rm -rf "$classify_root"
else
    skip 'write.sh PreToolUse classify cases' 'no built engine'
fi

printf '\n# write.sh, on PreToolUse: Copilot names the same field `path`\n'
# Confirmed live: Copilot passes `Write`/`Edit` tool names like Claude Code,
# but `tool_input.path` rather than `tool_input.file_path`.
expect 'a Copilot-shaped write of a new document under the corpus root is refused the same way' \
    write.sh 0 'headwater new' \
    '{"hook_event_name":"PreToolUse","tool_name":"Write","tool_input":{"path":"docs/obligations/9999-a-record-nobody-scaffolded.md","file_text":"placeholder"}}'
refute 'a Copilot-shaped edit of a document that already exists passes' \
    write.sh 'permissionDecision' \
    '{"hook_event_name":"PreToolUse","tool_name":"Edit","tool_input":{"path":"docs/spec/05-ai-integration.md","old_str":"a","new_str":"b"}}'

printf '\n# write.sh, on PreToolUse: an apply_patch command in place of a file_path\n'
# Codex names its edit tool `apply_patch` and passes the patch text under
# `tool_input.command` rather than a bare path, spec 16's C4 row for that
# column. `hw_patch_path` in lib.sh is the one place that reads it, and these
# hold it to the same refusals the `file_path` shape above already holds.
expect 'an apply_patch add of a new document under the corpus root is refused the same way' \
    write.sh 0 'headwater new' \
    '{"hook_event_name":"PreToolUse","tool_name":"apply_patch","tool_input":{"command":"*** Begin Patch\n*** Add File: docs/obligations/9999-a-record-nobody-scaffolded.md\n+placeholder\n*** End Patch"}}'
refute 'an apply_patch update of a document that already exists passes' \
    write.sh 'permissionDecision' \
    '{"hook_event_name":"PreToolUse","tool_name":"apply_patch","tool_input":{"command":"*** Begin Patch\n*** Update File: docs/spec/05-ai-integration.md\n@@\n-old\n+new\n*** End Patch"}}'
expect 'a tool_input with neither a file_path nor an apply_patch command is silent' \
    write.sh 0 '' \
    '{"hook_event_name":"PreToolUse","tool_name":"some_other_tool","tool_input":{"argument":"nothing this hook reads"}}'

printf '\n# write.sh, on PreToolUse: impact detection, before the edit\n'
if [ -x "$engine" ]; then
    expect 'an edit to a path a document governs names that document' \
        write.sh 0 'docs/spec/05-ai-integration.md' \
        '{"hook_event_name":"PreToolUse","tool_name":"Edit","tool_input":{"file_path":".claude/hooks/write.sh"}}'
    expect 'the advisory says it blocks nothing' \
        write.sh 0 'Nothing here blocks the edit' \
        '{"hook_event_name":"PreToolUse","tool_name":"Edit","tool_input":{"file_path":".claude/hooks/intent.sh"}}'
    # #953: a path the governed scope admits and nothing governs is not a
    # silence. The engine states the fact and the front-matter lines that
    # would declare the edge, and the hook writes neither. Both paths below
    # are names with no file, so a later `governs` edge on a real file cannot
    # move either case.
    expect 'an edit to a path in the governed scope that nothing governs says so' \
        write.sh 0 'engine/crates/query/src/unrelated.rs is in the governed scope, and nothing governs it' \
        '{"hook_event_name":"PreToolUse","tool_name":"Edit","tool_input":{"file_path":"engine/crates/query/src/unrelated.rs"}}'
    expect 'the same advisory prints the front-matter lines that declare the edge' \
        write.sh 0 '      governs:\n        - engine/crates/query/src/unrelated.rs' \
        '{"hook_event_name":"PreToolUse","tool_name":"Edit","tool_input":{"file_path":"engine/crates/query/src/unrelated.rs"}}'
    expect 'an edit to a path outside the governed scope that nothing governs is silent' \
        write.sh 0 '' \
        '{"hook_event_name":"PreToolUse","tool_name":"Edit","tool_input":{"file_path":"engine/crates/query/tests/unrelated.rs"}}'
    # HW-DR-0074 discharged HW-OBL-0104: a `code_path` anchor is a pattern, and
    # spec 5 now governs `.claude/hooks/**` rather than five files by name, so
    # a new file under that directory is named too.
    expect 'a path under a governed pattern names the document, per HW-DR-0074' \
        write.sh 0 'docs/spec/05-ai-integration.md' \
        '{"hook_event_name":"PreToolUse","tool_name":"Edit","tool_input":{"file_path":".claude/hooks/nothing-governs-this.sh"}}'

    # An `interface_contract` over a crate. This is the same position reaching a
    # document whose subject is the code being edited rather than a document
    # that happens to name the file.
    expect 'an edit to a crate a contract governs names the contract' \
        write.sh 0 'docs/interfaces/headwater-check.md' \
        '{"hook_event_name":"PreToolUse","tool_name":"Edit","tool_input":{"file_path":"engine/crates/check/src/lib.rs"}}'

    # The parenthesized span, and not the bare words. The contract's summary
    # holds `headwater check` in its own prose, so a case asserting that alone
    # would pass whether the name was read or not. Only `Pointer::render` writes
    # the parentheses, and what it puts inside them is the facet in the `name`
    # role.
    expect 'the contract is named by the command it describes' \
        write.sh 0 '(headwater check)' \
        '{"hook_event_name":"PreToolUse","tool_name":"Edit","tool_input":{"file_path":"engine/crates/check/src/lib.rs"}}'

    # Two contracts govern `main.rs`, because that file holds the flag parsing
    # and the exit statuses of every verb. Both are named. The fan-in is a fact
    # about the file rather than about the relation, and naming one of the two
    # would be a rule this position does not have.
    expect 'a file two contracts govern names both of them' \
        write.sh 0 'docs/interfaces/headwater-sweep.md' \
        '{"hook_event_name":"PreToolUse","tool_name":"Edit","tool_input":{"file_path":"engine/crates/cli/src/main.rs"}}'

    # `runner.rs` sits beside a file `docs/interfaces/headwater-check.md`
    # governs by a literal, one-file anchor. HW-DR-0074 lets an author widen
    # that anchor to a pattern; this contract has not been rewritten to one,
    # so the edge still answers for no file beside the one it names.
    expect 'a file beside a governed crate file is silent, until its contract adopts a pattern' \
        write.sh 0 '' \
        '{"hook_event_name":"PreToolUse","tool_name":"Edit","tool_input":{"file_path":"engine/crates/check/src/runner.rs"}}'

    # #953: the advisory is heard before the edit, and it says so. Until then
    # this branch exited at once for a path that exists, so the case above on
    # `.claude/hooks/write.sh` printed nothing and the agent heard the governing
    # set only after it had already changed the file.
    expect 'the advisory is worded for an edit that has not happened yet' \
        write.sh 0 'which you are about to change' \
        '{"hook_event_name":"PreToolUse","tool_name":"Edit","tool_input":{"file_path":".claude/hooks/write.sh"}}'
    expect 'the pre-edit advisory adds context and never decides the call' \
        write.sh 0 '"additionalContext"' \
        '{"hook_event_name":"PreToolUse","tool_name":"Edit","tool_input":{"file_path":".claude/hooks/write.sh"}}'
    refute 'the pre-edit advisory carries no permission decision' \
        write.sh 'permissionDecision' \
        '{"hook_event_name":"PreToolUse","tool_name":"Edit","tool_input":{"file_path":".claude/hooks/write.sh"}}'

    # The refusal keeps its precedence. A new document under the corpus is
    # denied, and the advisory never takes the place of the denial.
    refute 'a refused raw write carries the refusal and not the advisory' \
        write.sh 'additionalContext' \
        '{"hook_event_name":"PreToolUse","tool_name":"Write","tool_input":{"file_path":"docs/obligations/9999-a-record-nobody-scaffolded.md"}}'
else
    skip 'write.sh PreToolUse impact cases' 'no built engine'
fi

printf '\n# write.sh, on PreToolUse: the reverse advisory, for an edit to a governing document\n'
# #1008: PR #1004 edited HW-PD-0007 and heard nothing about the three files
# that decision governs, because the advisory answered one direction only.
# Every fact of the reverse part is read from `headwater explain --json`.
if [ -x "$engine" ]; then
    pd7='{"hook_event_name":"PreToolUse","tool_name":"Edit","tool_input":{"file_path":"docs/process/decisions/0007-a-background-wait-caps-below-the-cache-lifetime-and-re-issues-itself.md"}}'
    expect 'an edit to a governing decision names the first path it governs' \
        write.sh 0 '.claude/hooks/wait.sh' "$pd7"
    expect 'an edit to a governing decision names the second path it governs' \
        write.sh 0 'tools/run/run-census.sh' "$pd7"
    expect 'an edit to a governing decision names the third path it governs' \
        write.sh 0 '.claude/skills/hw-run-policy/SKILL.md' "$pd7"
    expect 'an edit to a governing decision names the document that traces to it' \
        write.sh 0 'docs/spec/17-orchestration-architecture.md (traces_to of)' "$pd7"
    expect 'the reverse advisory is worded for an edit that has not happened yet' \
        write.sh 0 'which you are about to change, is a document that other files depend on' "$pd7"
    expect 'the reverse advisory says it blocks nothing' \
        write.sh 0 'Nothing here blocks the edit' "$pd7"
    refute 'the reverse advisory carries no permission decision' \
        write.sh 'permissionDecision' "$pd7"

    # Independence: the forward advisory on a governed code path is unchanged,
    # and a code path is no document, so the reverse part is absent.
    wait_edit='{"hook_event_name":"PreToolUse","tool_name":"Edit","tool_input":{"file_path":".claude/hooks/wait.sh"}}'
    expect 'a governed code path still hears the decision that governs it' \
        write.sh 0 'docs/process/decisions/0007-a-background-wait-caps-below-the-cache-lifetime-and-re-issues-itself.md' "$wait_edit"
    refute 'a code path carries no reverse advisory' \
        write.sh 'is a document that other files depend on' "$wait_edit"

    # A scratch corpus, so the inbound half, both halves at once, and the
    # silent document are each provoked on purpose. Alpha governs a path, Beta
    # traces to Alpha, Gamma governs Alpha's own path, and Delta has no edge.
    reverse_root=$(mktemp -d "${TMPDIR:-/tmp}/headwater-reverse-XXXXXX")
    mkdir -p "$reverse_root/.headwater/packages" "$reverse_root/docs/process/decisions" \
        "$reverse_root/engine/target/release" "$reverse_root/tools"
    cp -r "$root/taxonomy-source/headwater-standard" "$reverse_root/.headwater/packages/headwater-standard"
    sed -i 's|bundles: \.\./\.\./docs/taxonomies|bundles: ../../../docs/taxonomies|' \
        "$reverse_root/.headwater/packages/headwater-standard/package.yml"
    cp -r "$root/docs/taxonomies" "$reverse_root/docs/taxonomies"
    cp "$root/.headwater/overlay.yml" "$root/.headwater/taxonomy.yml" "$reverse_root/.headwater/"
    cp "$engine" "$reverse_root/engine/target/release/headwater"
    : > "$reverse_root/tools/alpha.sh"
    reverse_doc() {
        printf -- '---\nid: %s\nstatus: current\nstatus_since: 2026-09-24\nsummary: "%s"\nlast_verified: 2026-09-24\n%b---\n\n# %s\n\n## Context\n\nA fixture.\n\n## Decision\n\nA fixture.\n\n## Consequences\n\nA fixture.\n' \
            "$2" "$3" "$4" "$3" > "$reverse_root/docs/process/decisions/$1"
    }
    reverse_doc 0001-alpha.md HW-PD-0001 'Alpha is the edited document' 'relations:\n  governs:\n    - tools/alpha.sh\n'
    reverse_doc 0002-beta.md HW-PD-0002 'Beta traces to alpha' 'relations:\n  traces_to:\n    - HW-PD-0001\n'
    reverse_doc 0003-gamma.md HW-PD-0003 'Gamma governs alpha' 'relations:\n  governs:\n    - docs/process/decisions/0001-alpha.md\n'
    reverse_doc 0004-delta.md HW-PD-0004 'Delta has no edge' ''

    if resolved=$("$reverse_root/engine/target/release/headwater" taxonomy resolve --root "$reverse_root" 2>&1); then
        HEADWATER_HOOK_ROOT="$reverse_root"
        export HEADWATER_HOOK_ROOT
        alpha='{"hook_event_name":"PreToolUse","tool_name":"Edit","tool_input":{"file_path":"docs/process/decisions/0001-alpha.md"}}'
        expect 'an edit to a document names the document that declares an edge onto it' \
            write.sh 0 'docs/process/decisions/0002-beta.md (traces_to of)' "$alpha"
        expect 'a document both governed and governing hears the forward part' \
            write.sh 0 'docs/process/decisions/0003-gamma.md' "$alpha"
        expect 'a document both governed and governing hears the reverse part' \
            write.sh 0 'tools/alpha.sh' "$alpha"
        # One call, one object: the harness reads the first JSON object a hook
        # writes, so two objects would lose one part.
        out=$(printf '%s' "$alpha" | sh "$hooks/write.sh" 2>/dev/null)
        objects=$(printf '%s\n' "$out" | grep -c 'hookSpecificOutput')
        if [ "$objects" -eq 1 ] && printf '%s' "$out" | "$engine" json field hookSpecificOutput additionalContext >/dev/null 2>&1; then
            printf 'ok   %s\n' 'both parts arrive in one JSON object'
            passed=$((passed + 1))
        else
            printf 'FAIL %s\n  expected one parseable object, got:\n%s\n' 'both parts arrive in one JSON object' "$out"
            failed=$((failed + 1))
        fi
        expect 'a document with no edge in either direction is silent' \
            write.sh 0 '' \
            '{"hook_event_name":"PreToolUse","tool_name":"Edit","tool_input":{"file_path":"docs/process/decisions/0004-delta.md"}}'
        expect 'a document that does not exist yet gets the refusal alone' \
            write.sh 0 '"permissionDecision":"deny"' \
            '{"hook_event_name":"PreToolUse","tool_name":"Write","tool_input":{"file_path":"docs/process/decisions/0005-epsilon.md"}}'
        expect 'the reverse advisory is silent after the edit' \
            write.sh 0 '' \
            '{"hook_event_name":"PostToolUse","tool_name":"Edit","tool_input":{"file_path":"docs/process/decisions/0001-alpha.md"}}'
        HEADWATER_HOOK_ROOT="$root"
        export HEADWATER_HOOK_ROOT
    else
        printf 'FAIL the reverse advisory (setup)\n  the scratch corpus did not resolve:\n%s\n' "$resolved"
        failed=$((failed + 1))
    fi
    rm -rf "$reverse_root"
else
    skip 'write.sh PreToolUse reverse advisory cases' 'no built engine'
fi

printf '\n# write.sh, on PostToolUse: silent, and write.sh says why #952 left it so\n'
# The advisory moved to PreToolUse, so the post-edit position says nothing and
# no edit prints the same pointers twice. These are the payloads that printed
# the advisory before #953.
if [ -x "$engine" ]; then
    expect 'an edit to a path a document governs is silent after the edit' \
        write.sh 0 '' \
        '{"hook_event_name":"PostToolUse","tool_name":"Edit","tool_input":{"file_path":".claude/hooks/write.sh"}}'
    expect 'an edit to a crate a contract governs is silent after the edit' \
        write.sh 0 '' \
        '{"hook_event_name":"PostToolUse","tool_name":"Edit","tool_input":{"file_path":"engine/crates/check/src/lib.rs"}}'
else
    skip 'write.sh PostToolUse cases' 'no built engine'
fi

printf '\n# read.sh, on PreToolUse: the governing set before a read\n'
# #953: a session heard the governing set only after an edit, and never when it
# opened a file. This position routes the one path a read names and adds the
# pointers as context. It never decides the call, so a read always proceeds.
if [ -x "$engine" ]; then
    expect 'a read of a crate a contract governs names the contract' \
        read.sh 0 'docs/interfaces/headwater-check.md' \
        '{"hook_event_name":"PreToolUse","tool_name":"Read","tool_input":{"file_path":"engine/crates/check/src/lib.rs"}}'
    expect 'the read advisory arrives as added context' \
        read.sh 0 '"additionalContext"' \
        '{"hook_event_name":"PreToolUse","tool_name":"Read","tool_input":{"file_path":"engine/crates/check/src/lib.rs"}}'
    refute 'the read advisory carries no permission decision' \
        read.sh 'permissionDecision' \
        '{"hook_event_name":"PreToolUse","tool_name":"Read","tool_input":{"file_path":"engine/crates/check/src/lib.rs"}}'
    expect 'a read of a path nothing governs is silent' \
        read.sh 0 '' \
        '{"hook_event_name":"PreToolUse","tool_name":"Read","tool_input":{"file_path":"engine/crates/query/src/unrelated.rs"}}'
    expect 'a read of an absolute path inside the root names what governs it' \
        read.sh 0 'docs/spec/05-ai-integration.md' \
        "{\"hook_event_name\":\"PreToolUse\",\"tool_name\":\"Read\",\"tool_input\":{\"file_path\":\"$root/.claude/hooks/write.sh\"}}"
    expect 'a read of a path outside the root is silent' \
        read.sh 0 '' \
        '{"hook_event_name":"PreToolUse","tool_name":"Read","tool_input":{"file_path":"/etc/hostname"}}'
    expect 'a Copilot-shaped read, with the path under `path`, is read the same way' \
        read.sh 0 'docs/interfaces/headwater-check.md' \
        '{"hook_event_name":"PreToolUse","tool_name":"view","tool_input":{"path":"engine/crates/check/src/lib.rs"}}'
    expect 'a read position on any other event is silent' \
        read.sh 0 '' \
        '{"hook_event_name":"PostToolUse","tool_name":"Read","tool_input":{"file_path":"engine/crates/check/src/lib.rs"}}'
    expect 'a read payload with no path is silent' \
        read.sh 0 '' \
        '{"hook_event_name":"PreToolUse","tool_name":"Read","tool_input":{}}'
else
    skip 'read.sh PreToolUse cases' 'no built engine'
fi

printf '\n# write.sh and read.sh, on PreToolUse: they fail open, and the control says so\n'
# Three sabotages, one per thing a hook cannot assume it has. Each one must let
# the edit proceed: exit 0, and not one byte written.
#
# The trap this block is built around is that *silence is the ambient outcome*.
# An edit proceeds when the hook is broken, when the hook is absent, when the
# payload is malformed and when nothing governs the path, so a case that only
# asserts silence passes without testing anything at all. Every sabotage below
# therefore runs twice over one scratch root and one payload: once intact, where
# the hook must name the contract, and once sabotaged, where it must say
# nothing. The control is what makes the silence mean something, and a control
# that goes quiet is reported as a failure of the case rather than of the hook.
if [ -x "$engine" ]; then
    # A root the sabotage can act on. The corpus is shared with the real tree by
    # symlink, because a copy of it would be a second corpus that drifts, and
    # `engine/crates` is shared too because a `code_path` anchor binds on the
    # file existing. What is not shared is `engine/target/release`, which is the
    # one directory a sabotage has to own.
    open_root=$(mktemp -d "${TMPDIR:-/tmp}/headwater-failopen-XXXXXX")
    for shared in docs packages .headwater .githooks .claude; do
        [ -e "$root/$shared" ] && ln -s "$root/$shared" "$open_root/$shared"
    done
    mkdir -p "$open_root/engine/target/release" "$open_root/bin"
    ln -s "$root/engine/crates" "$open_root/engine/crates"
    open_payload='{"hook_event_name":"PreToolUse","tool_name":"Edit","tool_input":{"file_path":"engine/crates/check/src/lib.rs"}}'
    read_payload='{"hook_event_name":"PreToolUse","tool_name":"Read","tool_input":{"file_path":"engine/crates/check/src/lib.rs"}}'
    deny_payload='{"hook_event_name":"PreToolUse","tool_name":"Write","tool_input":{"file_path":"docs/obligations/9999-a-record-nobody-scaffolded.md"}}'

    # Every case below runs against the scratch root and with the scratch `bin`
    # ahead of the real one. Both are restored at the end of the block. The
    # assignment is made here rather than in front of each call, because a
    # variable assignment in front of a shell function persists in some shells
    # and not in others, and a suite that leaked either one would run the cases
    # after it against a root nobody chose.
    real_path=$PATH
    HEADWATER_HOOK_ROOT="$open_root"
    PATH="$open_root/bin:$real_path"
    export HEADWATER_HOOK_ROOT PATH

    # One sabotage, twice: the control, then the broken run. The second argument
    # runs between them and the third puts the root back. The arguments are read
    # positionally rather than into names, because `expect` assigns to `name`
    # and nothing here is local to a function.
    fails_open() {
        expect "the control for: $1" write.sh 0 '(headwater check)' "$open_payload"
        eval "$2"
        expect "$1" write.sh 0 '' "$open_payload"
        eval "$3"
        # The read position meets the same three sabotages, over the same root,
        # and a read has to proceed through each of them the way an edit does.
        expect "the control for, at the read position: $1" read.sh 0 '(headwater check)' "$read_payload"
        eval "$2"
        expect "at the read position: $1" read.sh 0 '' "$read_payload"
        eval "$3"
    }

    cp "$engine" "$open_root/engine/target/release/headwater"

    fails_open 'no built engine at all, and the edit proceeds in silence' \
        'rm -f "$open_root/engine/target/release/headwater"' \
        'cp "$engine" "$open_root/engine/target/release/headwater"'

    fails_open 'a built engine nobody may execute, and the edit proceeds in silence' \
        'chmod a-x "$open_root/engine/target/release/headwater"' \
        'chmod u+x "$open_root/engine/target/release/headwater"'

    # The refusal position on the same terms, and this is the case that would
    # make a clean clone unusable if it were ever inverted: a `PreToolUse` that
    # cannot read its input has to let the write land rather than deny it. The
    # control is the deny, over the same root and the same payload.
    expect 'the control for: a PreToolUse that cannot read its input' \
        write.sh 0 '"permissionDecision":"deny"' "$deny_payload"
    rm -f "$open_root/engine/target/release/headwater"
    expect 'a PreToolUse that cannot read its input denies nothing' \
        write.sh 0 '' "$deny_payload"
    cp "$engine" "$open_root/engine/target/release/headwater"

    # #1008: the reverse advisory on the same terms, with its control first.
    reverse_payload='{"hook_event_name":"PreToolUse","tool_name":"Edit","tool_input":{"file_path":"docs/process/decisions/0007-a-background-wait-caps-below-the-cache-lifetime-and-re-issues-itself.md"}}'
    expect 'the control for: the reverse advisory with no engine' \
        write.sh 0 'tools/run/run-census.sh' "$reverse_payload"
    rm -f "$open_root/engine/target/release/headwater"
    expect 'the reverse advisory with no engine is silent, and the edit proceeds' \
        write.sh 0 '' "$reverse_payload"
    cp "$engine" "$open_root/engine/target/release/headwater"

    # No position runs an interpreter, which is HW-DR-0055 and the discharge of
    # HW-OBL-0146. A sabotage cannot state this the way the three above state
    # theirs: an interpreter that answers non-zero changes nothing now, and a
    # case asserting that nothing changed would pass on a tree where the
    # interpreter was never installed either.
    #
    # So this one is stated by the call rather than by the outcome. The stand-in
    # records every invocation, all three positions are driven through it at
    # full strength, and the file it would have written is the assertion. The
    # `PATH` here already puts `$open_root/bin` first, so a hook that reached
    # for the name would reach this.
    printf '#!/bin/sh\nprintf "%%s\\n" "$@" >> "%s/interpreter-was-called"\nexit 1\n' \
        "$open_root" > "$open_root/bin/python3"
    chmod u+x "$open_root/bin/python3"
    cp "$open_root/bin/python3" "$open_root/bin/python"
    rm -f "$open_root/interpreter-was-called"

    expect 'the write position runs at full strength with no interpreter behind it' \
        write.sh 0 '(headwater check)' "$open_payload"
    expect 'the read position runs at full strength with no interpreter behind it' \
        read.sh 0 '(headwater check)' "$read_payload"
    expect 'the refusal position runs at full strength with no interpreter behind it' \
        write.sh 0 '"permissionDecision":"deny"' "$deny_payload"
    expect 'the intent position runs at full strength with no interpreter behind it' \
        intent.sh 0 'docs/spec/' \
        '{"hook_event_name":"UserPromptSubmit","user_input":"what does a check know about the front matter of a document"}'
    expect 'the review position reads its re-entry guard with no interpreter behind it' \
        review.sh 0 '' '{"hook_event_name":"Stop","stop_hook_active":true}'

    if [ -e "$open_root/interpreter-was-called" ]; then
        printf 'FAIL a hook called an interpreter, with:\n%s\n' \
            "$(cat "$open_root/interpreter-was-called")"
        failed=$((failed + 1))
    else
        printf 'ok   %s\n' 'no position called an interpreter at all'
        passed=$((passed + 1))
    fi
    rm -f "$open_root/bin/python3" "$open_root/bin/python"

    # Three sabotages at the intent position, over the same root. The first two
    # are the two the write position meets above, and the third is a document
    # that only a stand-in can produce. A
    # prompt proceeds whatever happens here, so silence is the ambient outcome
    # at this position too and only a control makes it mean anything. The
    # control is a task this corpus answers, which reaches the agent as
    # pointers.
    intent_payload='{"hook_event_name":"UserPromptSubmit","user_input":"what does a check know about the front matter of a document"}'
    intent_fails_open() {
        expect "the control for: $1" intent.sh 0 'docs/spec/' "$intent_payload"
        eval "$2"
        expect "$1" intent.sh 0 '' "$intent_payload"
        eval "$3"
    }

    intent_fails_open 'no built engine at all, and the prompt proceeds in silence' \
        'rm -f "$open_root/engine/target/release/headwater"' \
        'cp "$engine" "$open_root/engine/target/release/headwater"'

    intent_fails_open 'a built engine nobody may execute, and the prompt proceeds in silence' \
        'chmod a-x "$open_root/engine/target/release/headwater"' \
        'chmod u+x "$open_root/engine/target/release/headwater"'

    # A route document this hook cannot parse. `headwater route --json` is the
    # one thing this position reads, so a stand-in that writes something else is
    # the only way to state the case: no corpus produces it and no payload can
    # ask for it.
    intent_fails_open 'a route document that will not parse, and the prompt proceeds in silence' \
        'stub_route "not a document at all"' \
        'cp "$engine" "$open_root/engine/target/release/headwater"'

    # The pointer set is what this position decides on, and these two state it
    # directly rather than through whatever this corpus answers for a task.
    #
    # The second is also the proof that the hook renders no line of its own. The
    # marker it asserts is in the document's `text` member and in none of its
    # pointers, so a hook that composed a pointer line out of the members could
    # not print it. That is the property #321 asks for: one rendering, in the
    # engine, read by the terminal and by an agent alike.
    stub_route '{"version":"1.0","task":"t","pointers":[],"text":"route \"t\"\n  no declared purpose answers this task\n"}'
    expect 'an empty pointer set is a silence, whatever the report beside it says' \
        intent.sh 0 '' "$intent_payload"

    stub_route '{"version":"1.0","task":"t","pointers":[{"path":"docs/a.md","kind":"decision","unwarranted":false}],"text":"only-the-rendered-report-carries-this\n"}'
    expect 'a pointer set with something in it hands back the report the engine rendered' \
        intent.sh 0 'only-the-rendered-report-carries-this' "$intent_payload"

    cp "$engine" "$open_root/engine/target/release/headwater"

    PATH=$real_path
    HEADWATER_HOOK_ROOT="$root"
    export HEADWATER_HOOK_ROOT PATH
    rm -rf "$open_root"
else
    skip 'write.sh and intent.sh fail-open cases' 'no built engine'
fi

printf '\n# intent.sh: the shadow-mode routing log HW-DR-0064 adds beside it\n'
# Step 1 of #819's build order: the deterministic half of the log, in the hook
# alone. Two things earn a case here that no case above already covers. One is
# the "harness output unchanged" bar, which nothing above would catch because
# nothing above runs the hook twice with a session id present — every case
# before this block predates the log and carries no `session_id`, so none of
# them ever reaches the write at all. The other is the actual risk of this
# change: a write failure that is not as silent as the routing decision it
# rides beside. `intent.sh`'s own header names the shell gotcha this guards
# against, and this is the fixture that would catch a regression back into it.
#
# Every case here writes to a scratch directory named through
# `HEADWATER_SHADOW_LOG_DIR`, and never to the log a session keeps. Until #917
# this block set `shadow_dir` to `<git common dir>/headwater-shadow-log`, the
# real one, and removed it at the start and the end. One run of this suite at
# 08:53Z on 2026-09-17 removed every line the hook had written since #893
# landed the day before, which read afterwards as a hook that never wrote. The
# first case below plants a file in the real directory and fails if the suite
# removes it.
if [ -x "$engine" ]; then
    common=$(git -C "$root" rev-parse --git-common-dir 2>/dev/null)
    case $common in
        /*) ;;
        *) common="$root/$common" ;;
    esac
    real_shadow_dir="$common/headwater-shadow-log"
    shadow_dir=$(mktemp -d "${TMPDIR:-/tmp}/headwater-shadow-fixtures.XXXXXX")
    HEADWATER_SHADOW_LOG_DIR=$shadow_dir
    export HEADWATER_SHADOW_LOG_DIR
    # Every case runs without a model unless it names one, so none of them runs
    # inference or depends on whether this host fetched the pinned files.
    no_model_dir="$shadow_dir.no-model"
    mkdir -p "$no_model_dir"
    HEADWATER_MODEL_DIR=$no_model_dir
    export HEADWATER_MODEL_DIR
    # The sentinel is not a `.jsonl` file, so a reader that counts session
    # files never counts it, and the trap removes it on an interrupt.
    mkdir -p "$real_shadow_dir"
    sentinel="$real_shadow_dir/fixtures-sentinel-$$.keep"
    : > "$sentinel"
    trap 'chmod u+w "$shadow_dir" 2>/dev/null; rm -rf "$shadow_dir" "$no_model_dir"; rm -f "$sentinel"' EXIT INT TERM

    # The cheaper regression: the same payload the very first case in this
    # file already answers with `docs/spec/`, run twice with a session id and
    # the log therefore live, still answers with that identical substring
    # both times. A hook that let logging change what it prints would fail
    # this before it failed anything more specific.
    routed_session="fixture-session-shadow-$$"
    routed_file="$shadow_dir/$routed_session.jsonl"
    routed_payload="{\"hook_event_name\":\"UserPromptSubmit\",\"session_id\":\"$routed_session\",\"user_input\":\"what does a check know about the front matter of a document\"}"
    expect 'a routed task with a session id hands back the report unchanged, once' \
        intent.sh 0 'docs/spec/' "$routed_payload"
    expect 'a routed task with a session id hands back the report unchanged, twice' \
        intent.sh 0 'docs/spec/' "$routed_payload"

    if [ -s "$routed_file" ] && [ "$(wc -l < "$routed_file")" -eq 2 ]; then
        printf 'ok   %s\n' 'the two calls above each left their own shadow-log line, in the one file for their session'
        passed=$((passed + 1))
    else
        printf 'FAIL %s\n' 'the two routed calls above did not leave two shadow-log lines'
        failed=$((failed + 1))
    fi

    # `route` invoked directly is not this hook, and Done-when says it writes
    # no entry. The file the two calls above just wrote to is the proof: a
    # direct call between them would have to land in the same file, since a
    # session id is not something a direct call carries or could invent one
    # of its own to collide with.
    before_lines=$(wc -l < "$routed_file")
    "$engine" route --root "$root" --json "what does a check know about the front matter of a document" > /dev/null 2>&1
    after_lines=$(wc -l < "$routed_file")
    if [ "$before_lines" = "$after_lines" ]; then
        printf 'ok   %s\n' 'headwater route invoked directly writes no shadow-log line'
        passed=$((passed + 1))
    else
        printf 'FAIL %s\n' 'headwater route invoked directly changed the shadow-log file it never should have reached'
        failed=$((failed + 1))
    fi

    # The decisive fixture. The control shows the mechanism is live: a fresh,
    # still-writable directory gains a line for a task the deterministic route
    # is already silent about — routing's own silence is not what is under
    # test, it is silent with or without a log. The sabotage removes write
    # permission from that same directory before its session's first prompt
    # ever reaches it, so the write has to create a file rather than append to
    # one already there, and a directory with no write permission refuses
    # exactly that create. What is under test is whether the refusal stays as
    # silent, on both streams, as the routing decision already was.
    rm -rf "$shadow_dir"
    mkdir -p "$shadow_dir"
    silent_payload_of() {
        printf '{"hook_event_name":"UserPromptSubmit","session_id":"%s","user_input":"xyzzy plugh frobnicate quuxbar"}' "$1"
    }

    control_session="fixture-session-shadow-control-$$"
    control_file="$shadow_dir/$control_session.jsonl"
    expect 'the control for: an unwritable shadow-log path, and the prompt proceeds in silence' \
        intent.sh 0 '' "$(silent_payload_of "$control_session")"
    if [ -s "$control_file" ]; then
        printf 'ok   %s\n' 'the control above left a shadow-log line while the directory was still writable'
        passed=$((passed + 1))
    else
        printf 'FAIL %s\n' 'the control above left no shadow-log line, so the sabotage below would prove nothing'
        failed=$((failed + 1))
    fi

    chmod a-w "$shadow_dir"
    sabotage_session="fixture-session-shadow-sabotage-$$"
    sabotage_file="$shadow_dir/$sabotage_session.jsonl"
    expect 'an unwritable shadow-log path, and the prompt proceeds in silence' \
        intent.sh 0 '' "$(silent_payload_of "$sabotage_session")"
    chmod u+w "$shadow_dir"
    if [ -e "$sabotage_file" ]; then
        printf 'FAIL %s\n' 'the sabotage above wrote a shadow-log line through a directory with no write permission'
        failed=$((failed + 1))
    else
        printf 'ok   %s\n' 'the sabotage above left no shadow-log line and no complaint on either stream'
        passed=$((passed + 1))
    fi

    # A prompt the harness submits on a schedule reaches this hook with the
    # same payload a typed one does. Measured on 2026-09-17 under
    # `claude -p` 2.1.272: the payload carries `session_id`, `transcript_path`,
    # `cwd`, `prompt_id`, `permission_mode`, `hook_event_name` and `prompt`, and
    # the transcript does not yet hold the prompt's own line when the hook
    # runs. The line that later says `"origin":{"kind":"human"}` is written
    # after. So the hook records `prompt_id`, and a count joins it against the
    # transcript. This case holds the join key.
    joined_session="fixture-session-shadow-joined-$$"
    joined_file="$shadow_dir/$joined_session.jsonl"
    expect 'a prompt with a prompt id is routed as before' \
        intent.sh 0 'docs/spec/' \
        "{\"hook_event_name\":\"UserPromptSubmit\",\"session_id\":\"$joined_session\",\"prompt_id\":\"fixture-prompt-$$\",\"user_input\":\"what does a check know about the front matter of a document\"}"
    if [ -s "$joined_file" ] && [ "$("$engine" json field prompt_id < "$joined_file" 2>/dev/null)" = "fixture-prompt-$$" ]; then
        printf 'ok   %s\n' 'the shadow-log line carries the prompt id a count joins against the transcript'
        passed=$((passed + 1))
    else
        printf 'FAIL %s\n' 'the shadow-log line does not carry the prompt id of the payload'
        failed=$((failed + 1))
    fi

    # The harness moves a session's working directory whenever a `cd`
    # persists, so a prompt typed during engine work arrives with `cwd` set to
    # `engine/` or deeper. `HEADWATER_HOOK_ROOT` turns off the payload read, so
    # this case unsets it and names the checkout the way the harness does.
    sub_session="fixture-session-shadow-subdir-$$"
    sub_file="$shadow_dir/$sub_session.jsonl"
    sub_payload="{\"hook_event_name\":\"UserPromptSubmit\",\"session_id\":\"$sub_session\",\"cwd\":\"$root/engine/crates\",\"user_input\":\"what does a check know about the front matter of a document\"}"
    sub_out=$(printf '%s' "$sub_payload" | env -u HEADWATER_HOOK_ROOT CLAUDE_PROJECT_DIR="$root" sh "$hooks/intent.sh" 2>&1)
    case $sub_out in
        *docs/spec/*)
            printf 'ok   %s\n' 'a prompt whose cwd is a subdirectory of the checkout routes over the repository root'
            passed=$((passed + 1))
            ;;
        *)
            printf 'FAIL %s\n  expected output to hold docs/spec/, got:\n%s\n' 'a prompt whose cwd is a subdirectory of the checkout routes over the repository root' "$sub_out"
            failed=$((failed + 1))
            ;;
    esac
    if [ -s "$sub_file" ] && [ "$(tail -n 1 "$sub_file" | "$engine" json field corpus_root 2>/dev/null)" = "$root" ]; then
        printf 'ok   %s\n' 'a prompt from a subdirectory leaves a line that names the repository root'
        passed=$((passed + 1))
    else
        printf 'FAIL %s\n' 'a prompt from a subdirectory left no line naming the repository root'
        failed=$((failed + 1))
    fi

    # Step 3 of #819: the recorder's name for its session, which
    # `tools/probe/probe-record.sh` exports before it starts the harness. A
    # person's prompt carries none, and a count subtracts the lines that do.
    probe_named_session="fixture-session-shadow-probe-$$"
    probe_named_file="$shadow_dir/$probe_named_session.jsonl"
    HEADWATER_PROBE_SESSION=fixture-probe-run
    export HEADWATER_PROBE_SESSION
    expect 'a prompt submitted under a recorder is routed as before' \
        intent.sh 0 'docs/spec/' \
        "{\"hook_event_name\":\"UserPromptSubmit\",\"session_id\":\"$probe_named_session\",\"user_input\":\"what does a check know about the front matter of a document\"}"
    unset HEADWATER_PROBE_SESSION
    if [ "$(tail -n 1 "$probe_named_file" 2>/dev/null | "$engine" json field probe_session 2>/dev/null)" = "fixture-probe-run" ]; then
        printf 'ok   %s\n' 'a line written under a recorder carries the name the recorder exported'
        passed=$((passed + 1))
    else
        printf 'FAIL %s\n' 'a line written under a recorder does not carry the recorder name'
        failed=$((failed + 1))
    fi
    if [ "$(tail -n 1 "$control_file" 2>/dev/null | "$engine" json field probe_session 2>/dev/null)" = "" ]; then
        printf 'ok   %s\n' "a person's line carries an empty recorder name rather than no member"
        passed=$((passed + 1))
    else
        printf 'FAIL %s\n' "a person's line does not carry an empty recorder name"
        failed=$((failed + 1))
    fi

    # Step 2 of #819: the embedding column. With no loadable model the line
    # still lands, and says the ranking is missing rather than omitting it.
    if [ -s "$control_file" ] && tail -n 1 "$control_file" | grep -q '"neighbors":null}$'; then
        printf 'ok   %s\n' 'a line written with no loadable model carries "neighbors":null'
        passed=$((passed + 1))
    else
        printf 'FAIL %s\n' 'a line written with no loadable model does not end in "neighbors":null'
        failed=$((failed + 1))
    fi

    # With the pinned files present, the hook's output is the same report and
    # the line gains the three members. The files are 90MB and fetched by
    # `tools/embed/fetch-model.sh`, so a host without them skips, and says so.
    shared_models="$common/headwater-models"
    if [ -f "$shared_models/model.onnx" ] && [ -f "$shared_models/vocab.txt" ]; then
        embed_session="fixture-session-shadow-embed-$$"
        embed_file="$shadow_dir/$embed_session.jsonl"
        embed_payload="{\"hook_event_name\":\"UserPromptSubmit\",\"session_id\":\"$embed_session\",\"user_input\":\"what does a check know about the front matter of a document\"}"
        HEADWATER_MODEL_DIR=$shared_models
        expect 'a routed task with the model present hands back the report unchanged' \
            intent.sh 0 'docs/spec/' "$embed_payload"
        HEADWATER_MODEL_DIR=$no_model_dir
        embed_model=$(tail -n 1 "$embed_file" 2>/dev/null | "$engine" json field model_digest 2>/dev/null)
        embed_tree=$(tail -n 1 "$embed_file" 2>/dev/null | "$engine" json field tree_digest 2>/dev/null)
        embed_count=$(tail -n 1 "$embed_file" 2>/dev/null | "$engine" json field neighbors 2>/dev/null | "$engine" json count neighbors 2>/dev/null)
        case "$embed_model/$embed_tree/$embed_count" in
            sha256:*/sha256:*/10)
                printf 'ok   %s\n' 'the line carries the model digest, the tree digest and ten neighbors'
                passed=$((passed + 1))
                ;;
            *)
                printf 'FAIL %s\n  got model=%s tree=%s neighbors=%s\n' 'the line does not carry the embedding column' "$embed_model" "$embed_tree" "$embed_count"
                failed=$((failed + 1))
                ;;
        esac
    else
        skip 'the embedding column with the pinned model present' "no model files at $shared_models; run tools/embed/fetch-model.sh"
    fi

    if [ -e "$sentinel" ]; then
        printf 'ok   %s\n' 'the suite left the shadow log a session keeps as it found it'
        passed=$((passed + 1))
    else
        printf 'FAIL %s\n' 'the suite removed a file from the shadow log a session keeps'
        failed=$((failed + 1))
    fi

    rm -rf "$shadow_dir" "$no_model_dir"
    rm -f "$sentinel"
    rmdir "$real_shadow_dir" 2>/dev/null
    unset HEADWATER_SHADOW_LOG_DIR HEADWATER_MODEL_DIR
    trap - EXIT INT TERM
else
    skip 'intent.sh shadow-log cases' 'no built engine'
fi

printf '\n# touch.sh, on PreToolUse: the marker review.sh reads back\n'
if [ -x "$engine" ]; then
    common=$(git -C "$root" rev-parse --git-common-dir 2>/dev/null)
    case $common in
        /*) ;;
        *) common="$root/$common" ;;
    esac
    fixture_session="fixture-session-touch-$$"
    marker="$common/headwater-session/$fixture_session/touched"
    trap 'rm -rf "$common/headwater-session/$fixture_session"' EXIT INT TERM

    expect 'a Write call is silent' \
        touch.sh 0 '' \
        "{\"hook_event_name\":\"PreToolUse\",\"tool_name\":\"Write\",\"session_id\":\"$fixture_session\"}"
    if [ -f "$marker" ]; then
        printf 'ok   %s\n' 'a Write call leaves review.sh a marker for this session'
        passed=$((passed + 1))
    else
        printf 'FAIL %s\n' 'a Write call left no marker for review.sh to read back'
        failed=$((failed + 1))
    fi

    expect 'a second call with the same session id is silent and safe' \
        touch.sh 0 '' \
        "{\"hook_event_name\":\"PreToolUse\",\"tool_name\":\"Edit\",\"session_id\":\"$fixture_session\"}"

    expect 'touch.sh fails open with no session id at all' \
        touch.sh 0 '' \
        '{"hook_event_name":"PreToolUse","tool_name":"Write"}'

    rm -rf "$common/headwater-session/$fixture_session"
    trap - EXIT INT TERM
else
    skip 'touch.sh cases' 'no built engine'
fi

printf '\n# touch.sh collect: a session the harness marked ended is removed, a live one is not\n'
if [ -x "$engine" ]; then
    common=$(git -C "$root" rev-parse --git-common-dir 2>/dev/null)
    case $common in
        /*) ;;
        *) common="$root/$common" ;;
    esac
    ended_session="fixture-session-ended-$$"
    live_session="fixture-session-live-$$"
    blocked_session="fixture-session-blocked-$$"
    unknown_session="fixture-session-unknown-$$"
    multi_session="fixture-session-multi-$$"
    fixture_jobs=$(mktemp -d)
    cleanup_collect() {
        rm -rf "$common/headwater-session/$ended_session" \
            "$common/headwater-session/$live_session" \
            "$common/headwater-session/$blocked_session" \
            "$common/headwater-session/$unknown_session" \
            "$common/headwater-session/$multi_session" \
            "$fixture_jobs"
    }
    trap cleanup_collect EXIT INT TERM

    for s in "$ended_session" "$live_session" "$blocked_session" "$unknown_session" "$multi_session"; do
        mkdir -p "$common/headwater-session/$s" && : > "$common/headwater-session/$s/touched"
    done

    # A job the harness has finished: no clock read anywhere, only the
    # `state` field `state.json` already carries.
    mkdir -p "$fixture_jobs/job-ended"
    printf '{"sessionId":"%s","state":"done"}' "$ended_session" > "$fixture_jobs/job-ended/state.json"
    # A job still running has no `state.json` at all -- this is the harness's
    # own signal for "live", and this case never writes one.
    mkdir -p "$fixture_jobs/job-live"
    # A job the harness paused rather than ended. It reads as still alive.
    mkdir -p "$fixture_jobs/job-blocked"
    printf '{"sessionId":"%s","state":"blocked"}' "$blocked_session" > "$fixture_jobs/job-blocked/state.json"
    # $unknown_session names no job at all: a marker this cannot tell about
    # is kept, never removed.

    # The regression `hw-verify` found: one session id, two job directories.
    # `job-multi-1-done` sorts before `job-multi-2-blocked` in the glob, and a
    # scan that stopped at the first match would read this session as ended
    # on `job-multi-1-done`'s word alone, never reaching the second job that
    # says otherwise. Both are named here so a reader can see the sort order
    # is deliberate, not incidental.
    mkdir -p "$fixture_jobs/job-multi-1-done" "$fixture_jobs/job-multi-2-blocked"
    printf '{"sessionId":"%s","state":"done"}' "$multi_session" > "$fixture_jobs/job-multi-1-done/state.json"
    printf '{"sessionId":"%s","state":"blocked"}' "$multi_session" > "$fixture_jobs/job-multi-2-blocked/state.json"

    out=$(HEADWATER_JOBS_ROOT="$fixture_jobs" sh "$hooks/touch.sh" collect 2>&1)
    if [ -d "$common/headwater-session/$ended_session" ]; then
        printf 'FAIL %s\n  the marker is still there\n' 'collect removes a marker the harness reports done'
        failed=$((failed + 1))
    else
        printf 'ok   %s\n' 'collect removes a marker the harness reports done'
        passed=$((passed + 1))
    fi
    for s in "$live_session" "$blocked_session" "$unknown_session" "$multi_session"; do
        if [ -d "$common/headwater-session/$s" ]; then
            printf 'ok   %s\n' "collect keeps $s"
            passed=$((passed + 1))
        else
            printf 'FAIL %s\n  %s was removed and should not have been\n' 'collect keeps a marker it has no ended evidence for' "$s"
            failed=$((failed + 1))
        fi
    done
    case $out in
        *"COLLECTED: $ended_session"*)
            printf 'ok   %s\n' '  and it names the session it collected'
            passed=$((passed + 1))
            ;;
        *)
            printf 'FAIL %s\n  got:\n%s\n' '  and it names the session it collected' "$out"
            failed=$((failed + 1))
            ;;
    esac

    cleanup_collect
    trap - EXIT INT TERM
else
    skip 'touch.sh collect cases' 'no built engine'
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

    # The session-marker exemption, over the same planted failure. A session
    # with no marker under it could not have caused the finding above, so the
    # gate skips it; the same session id with a marker present is gated the
    # same as one with no session id at all.
    common=$(git -C "$root" rev-parse --git-common-dir 2>/dev/null)
    case $common in
        /*) ;;
        *) common="$root/$common" ;;
    esac
    fixture_session="fixture-session-review-$$"
    session_dir="$common/headwater-session/$fixture_session"
    trap 'rm -f "$planted"; rm -rf "$session_dir"' EXIT INT TERM

    expect 'a session with no touched marker is let through the same failing tree' \
        review.sh 0 '' \
        "{\"hook_event_name\":\"Stop\",\"stop_hook_active\":false,\"session_id\":\"$fixture_session\"}"

    mkdir -p "$session_dir" && : > "$session_dir/touched"
    expect 'the same session marked touched is gated as before' \
        review.sh 2 '9999-a-fixture-that-this-runner-removes.md' \
        "{\"hook_event_name\":\"Stop\",\"stop_hook_active\":false,\"session_id\":\"$fixture_session\"}"

    rm -rf "$session_dir"
    trap 'rm -f "$planted"' EXIT INT TERM

    # Confirmed live: Copilot's `Stop` does not honor exit 2 the way Claude
    # Code and Codex do. An exit-2 hook there is logged and the turn ends
    # anyway, and the block instead reads a `{"decision":"block",...}` object
    # on standard output at exit 0. `COPILOT_CLI` is the one environment
    # variable that told the two mechanisms apart in that test.
    COPILOT_CLI=1 expect 'on Copilot the same refusal is an exit-0 decision, not exit 2' \
        review.sh 0 '"decision":"block"' \
        '{"hook_event_name":"Stop","stop_hook_active":false}'

    # The loop guard, and it is asserted here rather than over a clean tree on
    # purpose. Over a clean tree the gate exits 0 anyway, so the case passes
    # whether the guard runs or not, and a fixture that cannot fail proves
    # nothing. The planted document makes the guard the only way out.
    expect 'a stop the hook already blocked is let through, so no turn loops' \
        review.sh 0 '' \
        '{"hook_event_name":"Stop","stop_hook_active":true}'

    # The same pair with no interpreter behind the guard, which is what
    # HW-OBL-0146 asks for and what the interpreter cost until HW-DR-0055. The
    # planted document is still on the tree, so the gate refuses this tree and
    # the guard is the only way out of the loop it would otherwise start. A
    # `python3` that answers non-zero stands first on `PATH` and records every
    # call, so the pair states the outcome and the file states the cause.
    guard_bin=$(mktemp -d "${TMPDIR:-/tmp}/headwater-guard-XXXXXX")
    printf '#!/bin/sh\nprintf "%%s\\n" "$@" >> "%s/called"\nexit 1\n' \
        "$guard_bin" > "$guard_bin/python3"
    chmod u+x "$guard_bin/python3"
    cp "$guard_bin/python3" "$guard_bin/python"
    guard_path=$PATH
    PATH="$guard_bin:$guard_path"
    export PATH

    expect 'a second stop is let through with no interpreter behind the guard' \
        review.sh 0 '' \
        '{"hook_event_name":"Stop","stop_hook_active":true}'
    expect 'a first stop still stops the turn with no interpreter behind the guard' \
        review.sh 2 '9999-a-fixture-that-this-runner-removes.md' \
        '{"hook_event_name":"Stop","stop_hook_active":false}'

    PATH=$guard_path
    export PATH
    if [ -e "$guard_bin/called" ]; then
        printf 'FAIL the review position called an interpreter, with:\n%s\n' \
            "$(cat "$guard_bin/called")"
        failed=$((failed + 1))
    else
        printf 'ok   %s\n' 'the review position called no interpreter for either stop'
        passed=$((passed + 1))
    fi
    rm -rf "$guard_bin"

    # No engine, and therefore no read of the guard. The position ends the turn
    # rather than running a gate it could not stop twice, over the same tree the
    # two cases above refuse. The engine moves rather than the tree, so the
    # control for this case is every case above it.
    #
    # Both profiles move, because `hw_engine` accepts either one. A version of
    # this case that hid the `release` binary alone passed on a tree that had
    # never built the other and reported nothing on a tree that had, which is a
    # case whose verdict is a property of the runner's machine.
    moved="$root/engine/target/release/headwater.moved-by-fixtures"
    dev_engine="$root/engine/target/dev-release/headwater"
    dev_moved="$root/engine/target/dev-release/headwater.moved-by-fixtures"
    trap 'rm -f "$planted"; [ -e "$moved" ] && mv "$moved" "$engine"; [ -e "$dev_moved" ] && mv "$dev_moved" "$dev_engine"' EXIT INT TERM
    mv "$engine" "$moved"
    [ -e "$dev_engine" ] && mv "$dev_engine" "$dev_moved"
    expect 'a stop with no engine of either profile to read the guard ends the turn' \
        review.sh 0 '' \
        '{"hook_event_name":"Stop","stop_hook_active":false}'
    mv "$moved" "$engine"
    [ -e "$dev_moved" ] && mv "$dev_moved" "$dev_engine"

    rm -f "$planted"
    trap - EXIT INT TERM

    expect 'the tree is clean again once the fixture is gone' \
        review.sh 0 '' \
        '{"hook_event_name":"Stop","stop_hook_active":false}'
else
    skip 'review.sh cases that call the engine' 'no built engine'
fi

printf '\n# wait.sh, on PreToolUse: a foreground wait\n'
if [ -x "$engine" ]; then
    expect 'an until loop that sleeps is refused, and the refusal names the flag' \
        wait.sh 0 'run_in_background' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"until ! kill -0 1234 2>/dev/null; do sleep 30; done"}}'
    expect 'the refusal is a deny decision the harness can act on' \
        wait.sh 0 '"permissionDecision":"deny"' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"until [ -f /tmp/x.status ]; do sleep 30; done"}}'
    expect 'a while loop is refused the same way' \
        wait.sh 0 '"permissionDecision":"deny"' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"while kill -0 1234 2>/dev/null; do sleep 20; done; echo done"}}'
    expect 'a loop that sleeps fifteen is refused too, because the interval never said how long the wait would be' \
        wait.sh 0 '"permissionDecision":"deny"' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"until [ ! -d /proc/3472820 ]; do sleep 15; done"}}'
    expect 'a run watch is refused with no loop around it' \
        wait.sh 0 '"permissionDecision":"deny"' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"gh run watch 35480000000 --exit-status"}}'
    expect 'the foreground refusal states the cache-lifetime bound a background wait must not outlive' \
        wait.sh 0 'outlives the prompt cache' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"until [ -f /tmp/x.status ]; do sleep 30; done"}}'
    expect '  and names the remedy, a bounded wait that re-issues itself' \
        wait.sh 0 'timeout 240' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"until [ -f /tmp/x.status ]; do sleep 30; done"}}'
    expect '  and names the script that is that wait, and the condition for CI' \
        wait.sh 0 'tools/run/ci-done.sh' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"until [ -f /tmp/x.status ]; do sleep 30; done"}}'
    refute 'a run watch named inside a quoted argument is not a wait' \
        wait.sh '"permissionDecision":"deny"' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"grep -n '"'"'gh run watch'"'"' .claude/agents/hw-build.md"}}'
    expect '  and exempts the parent, whose cache lives an hour' \
        wait.sh 0 'the parent waits by ending its turn' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"until [ -f /tmp/x.status ]; do sleep 30; done"}}'
    refute 'the pgrep refusal does not also state the cache-lifetime bound, which belongs to the other remedy' \
        wait.sh 'outlives the prompt cache' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"until ! pgrep -f \"cargo test\" >/dev/null; do sleep 30; done"}}'

    expect 'a loop that waits on a pgrep literal is refused for a different reason' \
        wait.sh 0 'can never exit' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"until ! pgrep -f \"cargo test --workspace\" >/dev/null; do sleep 30; done"}}'
    expect 'that refusal names the two waits that do work' \
        wait.sh 0 'kill -0' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"until ! pgrep -af \"hw-cargo build\" >/dev/null; do sleep 20; done"}}'
    expect 'a pgrep loop already in the background is still refused, because backgrounding hides it' \
        wait.sh 0 '"permissionDecision":"deny"' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"until ! pgrep -f \"cargo test\" >/dev/null; do sleep 30; done","run_in_background":true}}'
    expect 'a pgrep outside a loop is not a wait and passes' \
        wait.sh 0 '' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"pgrep -af cargo | head -5"}}'
    expect 'the positive form, a while loop that runs WHILE pgrep finds it, is refused too' \
        wait.sh 0 'can never exit' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"while pgrep -f \"cargo build --profile\" >/dev/null; do sleep 10; done"}}'
    expect 'a pgrep nested in a command substitution is refused, though it looks nothing like the plain form' \
        wait.sh 0 'can never exit' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"until [ ! -d /proc/$(pgrep -f '"'"'hw-cargo test'"'"' | head -1) ]; do sleep 30; done"}}'
    expect 'the recommended kill -0 remedy is still refused when its pid comes from a pgrep literal' \
        wait.sh 0 'can never exit' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"until ! kill -0 $(pgrep -f \"cargo.*headwater-cli\" | head -1) 2>/dev/null; do sleep 30; done"}}'
    expect 'an unquoted pgrep pattern is refused, because the assignment is in the same command line' \
        wait.sh 0 'can never exit' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"pat=cargo-build; until ! pgrep -f $pat >/dev/null; do sleep 30; done"}}'
    expect 'the refusal says a marker wait must also end with the agent that started it' \
        wait.sh 0 'nobody is left to end' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"until ! pgrep -f \"cargo test\" >/dev/null; do sleep 30; done"}}'
    expect 'run_in_background absent entirely is refused, not only run_in_background false' \
        wait.sh 0 '"permissionDecision":"deny"' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"until [ -f /tmp/x.status ]; do sleep 30; done","timeout":600000}}'
    expect 'run_in_background false is refused the same as absent' \
        wait.sh 0 '"permissionDecision":"deny"' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"until [ -f /tmp/x.status ]; do sleep 30; done","run_in_background":false}}'
    expect 'a sleep with a unit suffix is refused, because the rule is the shape and not the number' \
        wait.sh 0 '"permissionDecision":"deny"' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"until [ -f /tmp/x.status ]; do sleep 2m; done"}}'
    expect 'a one-second poll of a local file is refused too, and this case is here to say that was chosen' \
        wait.sh 0 '"permissionDecision":"deny"' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"until [ -f /tmp/appears-in-200ms ]; do sleep 1; done"}}'
    refute 'the pgrep refusal does not also offer the cap remedy, which would make this shape immortal' \
        wait.sh 'capped at ten minutes.

When the cap' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"until ! pgrep -f \"cargo test\" >/dev/null; do sleep 30; done"}}'
    refute 'the foreground refusal does not claim the wait can never exit' \
        wait.sh 'can never exit' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"until [ -f /tmp/x.status ]; do sleep 30; done"}}'
    expect 'the heredoc silence is not vacuous: the same wait without the cat is refused' \
        wait.sh 0 '"permissionDecision":"deny"' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"while true; do sleep 30; done"}}'
    expect 'a run watch already in the background passes' \
        wait.sh 0 '' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"gh run watch 35480000000 --exit-status","run_in_background":true}}'
    expect 'a bare sleep with no loop around it is not a wait and passes' \
        wait.sh 0 '' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"sleep 30; echo awake"}}'
    expect 'an empty command string is silent' \
        wait.sh 0 '' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":""}}'
    expect 'the same wait already in the background passes' \
        wait.sh 0 '' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"until ! kill -0 1234 2>/dev/null; do sleep 30; done","run_in_background":true}}'
    expect 'a heredoc that writes a wait into a script is not itself a wait' \
        wait.sh 0 '' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"cat > /tmp/wait-ci.sh <<SCRIPT\nwhile true; do sleep 30; done\nSCRIPT"}}'
    expect 'a tool_input with no command at all is silent' \
        wait.sh 0 '' \
        '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"timeout":600000}}'
else
    skip 'wait.sh cases that call the engine' 'no built engine'
fi

# The cheap gate and the parse failure answer without an engine, so they run
# whether or not one was built. A payload with no `sleep` and no run watch in it
# never reaches `hw_field`, which is the whole point of reading the raw input
# first: this hook is the only one that matches `Bash`, and `Bash` is most of
# what a run does.
expect 'a command with neither in it never starts the engine' \
    wait.sh 0 '' \
    '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"ls -la /tmp"}}'
expect 'a build run in the foreground with no loop around it is not caught, and says nothing' \
    wait.sh 0 '' \
    '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"cargo test --workspace --manifest-path engine/Cargo.toml"}}'
expect 'an input that will not parse is silent rather than an error' \
    wait.sh 0 '' \
    'not json at all, but it does contain the word sleep'

printf '\n# .claude/settings.json: a hook declaration guards its own script (#971)\n'

# `CLAUDE_PROJECT_DIR` freezes at session start and a worktree can be retired,
# rebased past, or started outside a commit that holds a script the tree it
# reads from does not. `wait.sh` first existed at `578e5114`, and any session
# whose configuration named it from before that commit met the failure this
# suite now provokes on purpose: the harness passes each declaration's
# `command` string to `sh -c`, `/bin/sh` here is dash, and dash exits 2 when
# it cannot open a script that is not there — the one exit code every event
# below reads as a deliberate refusal rather than a failure.
#
# The guard's `else` branch exits 1 rather than falling through to an
# implicit 0. Confirmed live against https://code.claude.com/docs/en/hooks
# (fetched 2026-09-22): "Stderr from a hook that exits 0 goes to the debug
# log only, never the transcript, and Claude never sees it," while "any
# other exit code doesn't block on its own for most hook events," and empty
# stdout with a non-2 exit "shows the transcript a `<hook name>` hook error
# notice followed by the first line of stderr." Exit 2 is still the only
# code documented to block any of these six events (`UserPromptSubmit`'s own
# decision-control section names only `decision: block` or exit 2, no other
# code). So exit 1 keeps every declaration failing open exactly as exit 0
# would, and additionally makes the absence visible where exit 0 would not
# — closer to the issue's own ELI5, which asks for "a visible note," than
# the literal "exits 0" of its Done-when clause 1, whose intent this reads
# as "fails open" rather than as a constraint on which non-blocking code.
#
# `CLAUDE_PROJECT_DIR` is what every declaration's own command string reads,
# and it is not the variable `HEADWATER_HOOK_ROOT` this suite already exports
# at the top; a case below that ran a declaration's command with it unset
# would be testing a hook that resolved its own script against `/`, not the
# guard this issue writes.
CLAUDE_PROJECT_DIR=$root
export CLAUDE_PROJECT_DIR

# The `command` string of the Nth hook declaration in `.claude/settings.json`,
# in file order, unescaped to the text `sh -c` actually receives once the
# harness has parsed the surrounding JSON. Each declaration's `command` sits
# on its own line by convention, which the standing case just below checks
# stays true, so a line-oriented extraction reads the same text a JSON parser
# would without adding the interpreter dependency HW-DR-0055 refused for a
# hook body — `hw_patch_path` in lib.sh already scrapes a different wire
# shape the same way. The guard this issue writes into every declaration
# carries no backslash of its own, so the only JSON escape left to undo is
# `\"`.
hw_settings_command() {
    sed -n 's/^[[:space:]]*"command": "\(.*\)",\{0,1\}$/\1/p' "$root/.claude/settings.json" |
        sed -n "${1}p" |
        sed 's/\\"/"/g'
}

# Like `expect`, but holds standard output and standard error apart, for a
# clause whose contract is which stream a line lands on. `expect` cannot make
# this distinction: it reads both through one `2>&1` (line 66), which is
# right for a case that only asks whether a hook spoke and wrong here, where
# `UserPromptSubmit` folds a hook's stdout into the model's context, so a
# diagnostic landing on stdout would leak into the conversation rather than
# staying a log line.
expect_streams() {
    name=$1 status=$2 err_substring=$3
    shift 3
    _out=$(mktemp) _err=$(mktemp)
    "$@" >"$_out" 2>"$_err" </dev/null
    got=$?
    _stdout=$(cat "$_out") _stderr=$(cat "$_err")
    rm -f "$_out" "$_err"
    if [ "$got" -ne "$status" ]; then
        printf 'FAIL %s\n  expected exit %s, got %s\n' "$name" "$status" "$got"
        failed=$((failed + 1))
        return
    fi
    if [ -n "$_stdout" ]; then
        printf 'FAIL %s\n  expected no standard output, got:\n%s\n' "$name" "$_stdout"
        failed=$((failed + 1))
        return
    fi
    case $_stderr in
        *"$err_substring"*) ;;
        *)
            printf 'FAIL %s\n  expected standard error to hold %s, got:\n%s\n' "$name" "$err_substring" "$_stderr"
            failed=$((failed + 1))
            return
            ;;
    esac
    printf 'ok   %s\n' "$name"
    passed=$((passed + 1))
}

# The standing half of clause 5: no declaration may fall back to the bare
# `sh "$CLAUDE_PROJECT_DIR/.claude/hooks/<name>.sh"` this issue found, with no
# guard around it. This reads `.claude/settings.json` itself rather than a
# count of declarations, so an eighth one added later in this exact shape is
# caught here rather than shipped, whether or not this file's other seven cases
# below are ever updated to know about it.
bare=$(grep -nE '"command": *"sh \\"\$CLAUDE_PROJECT_DIR/\.claude/hooks/[A-Za-z_.]+\.sh\\""' "$root/.claude/settings.json") || true
if [ -n "$bare" ]; then
    printf 'FAIL %s\n  found a hook declaration with no guard around its script:\n%s\n' \
        'no declaration in .claude/settings.json is a bare, unguarded script path' "$bare"
    failed=$((failed + 1))
else
    printf 'ok   %s\n' 'no declaration in .claude/settings.json is a bare, unguarded script path'
    passed=$((passed + 1))
fi

# The two directions of the seven declarations themselves, run through their
# own command string exactly as `.claude/settings.json` holds it — not a
# paraphrase of it — with the script it names moved aside and then replaced
# by a stub that refuses on purpose. Mirrors the shape `review.sh`'s own
# engine-absence case already uses above: move the dependency aside, prove
# the fixture cleans up even on interrupt, restore it.
#
# The ordinal of each declaration is its position in `.claude/settings.json`
# file order (`intent.sh`, `write.sh` PreToolUse, `touch.sh`, `wait.sh`,
# `read.sh`, `write.sh` PostToolUse, `review.sh`); the bare-pattern scan just above is
# the one of the two checks that does not depend on this list staying
# up to date with that order.
hw_settings_case() {
    ordinal=$1 script=$2
    command=$(hw_settings_command "$ordinal")
    real="$hooks/$script"

    # Direction one: the script file is absent. Every declaration must fail
    # open — exit 1, not the harness's one blocking code, 2 — say one line
    # on standard error naming the path, and stay silent on standard
    # output, in every affected mode a session meets: a prompt submits
    # (`intent.sh`), `Bash` runs (`wait.sh`), `Write`/`Edit` run (`write.sh`,
    # `touch.sh`), and the session can `Stop` (`review.sh`).
    moved="$real.moved-by-fixtures-971"
    trap 'mv -f "$moved" "$real" 2>/dev/null' EXIT INT TERM
    mv "$real" "$moved"
    expect_streams "$script (declaration $ordinal): script absent exits 1 (fails open, visibly) with one stderr line naming the path" \
        1 "hook script not found: $real" \
        sh -c "$command"
    mv "$moved" "$real"
    trap - EXIT INT TERM

    # Direction two: the script is present and refuses on purpose. A guard
    # written as `[ -f "$P" ] && sh "$P" || exit 0` would swallow this,
    # because its `||` arm catches a deliberate refusal along with a missing
    # file, turning every gate in the table into a no-op. The `if`/`else`
    # form this issue writes must preserve the script's own exit code
    # exactly.
    stub="$real.stub-by-fixtures-971"
    cp "$real" "$stub"
    trap 'mv -f "$stub" "$real" 2>/dev/null' EXIT INT TERM
    printf '#!/bin/sh\nexit 2\n' >"$real"
    out=$(sh -c "$command" 2>&1 </dev/null)
    got=$?
    if [ "$got" -eq 2 ]; then
        printf 'ok   %s\n' "$script (declaration $ordinal): present and exiting 2 on purpose still blocks, exit code preserved"
        passed=$((passed + 1))
    else
        printf 'FAIL %s\n  expected exit 2 preserved through the guard, got %s, with:\n%s\n' \
            "$script (declaration $ordinal): present and exiting 2 on purpose still blocks" "$got" "$out"
        failed=$((failed + 1))
    fi
    mv -f "$stub" "$real"
    trap - EXIT INT TERM
}

hw_settings_case 1 intent.sh
hw_settings_case 2 write.sh
hw_settings_case 3 touch.sh
hw_settings_case 4 wait.sh
hw_settings_case 5 read.sh
hw_settings_case 6 write.sh
hw_settings_case 7 review.sh

printf '\n%s passed, %s failed, %s skipped\n' "$passed" "$failed" "$skipped"
# A caller that knows an engine should be there says so, and this answers
# before the failure count below, because it explains that count rather than
# competing with it.
#
# What a missing engine actually does here, measured rather than assumed: the
# suite reports 36 passed, 5 failed and 9 skipped, and exits 1. It does not go
# green. Five `write.sh` cases assert a refusal and get silence from a hook
# that fails open, so they fail outright. The remaining engine cases sit behind
# an `[ -x "$engine" ]` guard and skip. Anyone reading that output cold sees
# five broken refusals and starts debugging a hook, which is what happened to
# the session that first ran this suite without an engine.
#
# So this line is a diagnosis, not a gate that a green run depends on. It names
# the cause above the symptom. It is also the thing that keeps the gate honest
# if those five cases are ever moved behind the same guard the other nine sit
# behind, because then nothing would fail and the suite would exit 0 having
# tested almost nothing.
#
# It counts the skips whose reason is a missing engine and no others. A hosted
# runner fetches no embedding model, so the shadow-log case skips there every
# time and always will; the first cut of this guard counted that one too and
# failed an otherwise green CI run on it.
if [ -n "${HEADWATER_FIXTURES_REQUIRE_ENGINE:-}" ] && [ "$skipped_engine" -ne 0 ]; then
    printf 'FAIL %s case(s) skipped for want of an engine while HEADWATER_FIXTURES_REQUIRE_ENGINE is set\n' "$skipped_engine"
    printf '  an engine was expected at this point and none was found;\n'
    printf '  any failures above are most likely that and not a broken hook\n'
    exit 1
fi

[ "$failed" -eq 0 ] || exit 1
# Named against the engine count, not the total: a skip for want of an
# embedding model is not answered by building the engine, and saying so sent
# at least one reader to the wrong remedy.
[ "$skipped_engine" -eq 0 ] || printf 'Build the engine to run the skipped cases:\n  cargo build --release -p headwater-cli --manifest-path engine/Cargo.toml --locked\n'
exit 0
