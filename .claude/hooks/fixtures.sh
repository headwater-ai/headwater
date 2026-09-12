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
expect 'a Copilot-shaped edit of a document that already exists passes' \
    write.sh 0 '' \
    '{"hook_event_name":"PreToolUse","tool_name":"Edit","tool_input":{"path":"docs/spec/05-ai-integration.md","old_str":"a","new_str":"b"}}'

printf '\n# write.sh, on PreToolUse: an apply_patch command in place of a file_path\n'
# Codex names its edit tool `apply_patch` and passes the patch text under
# `tool_input.command` rather than a bare path, spec 16's C4 row for that
# column. `hw_patch_path` in lib.sh is the one place that reads it, and these
# hold it to the same refusals the `file_path` shape above already holds.
expect 'an apply_patch add of a new document under the corpus root is refused the same way' \
    write.sh 0 'headwater new' \
    '{"hook_event_name":"PreToolUse","tool_name":"apply_patch","tool_input":{"command":"*** Begin Patch\n*** Add File: docs/obligations/9999-a-record-nobody-scaffolded.md\n+placeholder\n*** End Patch"}}'
expect 'an apply_patch update of a document that already exists passes' \
    write.sh 0 '' \
    '{"hook_event_name":"PreToolUse","tool_name":"apply_patch","tool_input":{"command":"*** Begin Patch\n*** Update File: docs/spec/05-ai-integration.md\n@@\n-old\n+new\n*** End Patch"}}'
expect 'a tool_input with neither a file_path nor an apply_patch command is silent' \
    write.sh 0 '' \
    '{"hook_event_name":"PreToolUse","tool_name":"some_other_tool","tool_input":{"argument":"nothing this hook reads"}}'

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
        '{"hook_event_name":"PostToolUse","tool_name":"Edit","tool_input":{"file_path":"engine/crates/query/src/unrelated.rs"}}'
    # HW-OBL-0104. A `governs` edge reaches the path it names and no path
    # under it, so this fixture records the silence rather than asserting the
    # containment that a reader of spec 5 expects.
    expect 'a path under a governed directory is silent, which HW-OBL-0104 holds' \
        write.sh 0 '' \
        '{"hook_event_name":"PostToolUse","tool_name":"Edit","tool_input":{"file_path":".claude/hooks/nothing-governs-this.sh"}}'

    # An `interface_contract` over a crate. This is the same position reaching a
    # document whose subject is the code being edited rather than a document
    # that happens to name the file.
    expect 'an edit to a crate a contract governs names the contract' \
        write.sh 0 'docs/interfaces/headwater-check.md' \
        '{"hook_event_name":"PostToolUse","tool_name":"Edit","tool_input":{"file_path":"engine/crates/check/src/lib.rs"}}'

    # The parenthesized span, and not the bare words. The contract's summary
    # holds `headwater check` in its own prose, so a case asserting that alone
    # would pass whether the name was read or not. Only `Pointer::render` writes
    # the parentheses, and what it puts inside them is the facet in the `name`
    # role.
    expect 'the contract is named by the command it describes' \
        write.sh 0 '(headwater check)' \
        '{"hook_event_name":"PostToolUse","tool_name":"Edit","tool_input":{"file_path":"engine/crates/check/src/lib.rs"}}'

    # Two contracts govern `main.rs`, because that file holds the flag parsing
    # and the exit statuses of every verb. Both are named. The fan-in is a fact
    # about the file rather than about the relation, and naming one of the two
    # would be a rule this position does not have.
    expect 'a file two contracts govern names both of them' \
        write.sh 0 'docs/interfaces/headwater-sweep.md' \
        '{"hook_event_name":"PostToolUse","tool_name":"Edit","tool_input":{"file_path":"engine/crates/cli/src/main.rs"}}'

    # HW-OBL-0104 over a crate. `runner.rs` sits in the directory of a file a
    # contract governs and no edge reaches it, so it answers nothing. This is
    # the same silence the case above records, at the place a reader is most
    # likely to expect containment.
    expect 'a file beside a governed crate file is silent, which HW-OBL-0104 holds' \
        write.sh 0 '' \
        '{"hook_event_name":"PostToolUse","tool_name":"Edit","tool_input":{"file_path":"engine/crates/check/src/runner.rs"}}'
else
    skip 'write.sh PostToolUse cases' 'no built engine'
fi

printf '\n# write.sh, on PostToolUse: it fails open, and the control says so\n'
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
    open_payload='{"hook_event_name":"PostToolUse","tool_name":"Edit","tool_input":{"file_path":"engine/crates/check/src/lib.rs"}}'
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

printf '\n%s passed, %s failed, %s skipped\n' "$passed" "$failed" "$skipped"
[ "$failed" -eq 0 ] || exit 1
[ "$skipped" -eq 0 ] || printf 'Build the engine to run the skipped cases:\n  cargo build --release -p headwater-cli --manifest-path engine/Cargo.toml --locked\n'
exit 0
