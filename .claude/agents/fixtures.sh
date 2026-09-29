#!/bin/sh
# What holds the build-order agents, the command that dispatches them, and the
# doctrine both carry.
#
# `.claude/commands/next-run.md` names five agent definitions by
# `subagent_type`, each definition names the skills it invokes, and every one of
# those files cites rulings by identifier. None of that is read by any rule of
# the engine, and a name that stops resolving fails silently: a harness handed
# an unknown `subagent_type` falls back to a general agent with none of the
# definition's instructions, and a skill name nobody matches never loads. This
# suite is what reports the drift.
#
# Ten cases, and the ceilings are the reason two of them exist. The parent's
# context is the unit of cost ([HW-PD-0003]), so the command and the doctrine
# carry a byte ceiling declared here, once, and CLAUDE.md carries one because
# every agent pays for it on every dispatch ([HW-PD-0001]).
#
# Run it from anywhere:
#     sh .claude/agents/fixtures.sh
#
# It needs no engine. It reads the tree and the committed graph export, and it
# writes only under a temporary directory for the refusal arms.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
agents="$root/.claude/agents"
commands="$root/.claude/commands"
skills="$root/.claude/skills"

# The three ceilings, in bytes. Declared here and nowhere else.
claude_md_ceiling=9216
next_run_ceiling=8192
doctrine_ceiling=2600

passed=0
failed=0

pass() {
    printf 'ok   %s\n' "$1"
    passed=$((passed + 1))
}

fail() {
    printf 'FAIL %s\n  %s\n' "$1" "$2"
    failed=$((failed + 1))
}

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

# Every backticked span of a file, one per line, with the backticks removed.
spans() {
    grep -oE '`[^`]+`' "$1" 2>/dev/null | tr -d '`'
}

# The agent names a command file dispatches: every span that is shaped like an
# agent name. `hw-` is the build-order prefix and the two `headwater-` agents
# are the ones this repository already had.
agent_names_in() {
    spans "$1" | grep -E '^(hw-[a-z]+|headwater-(maintainer|product-owner))$' | sort -u
}

# The sentences of a file, one per line: each line split at a full stop that a
# space follows, so a path such as `next-run.md` stays whole.
sentences() {
    awk '{ n = split($0, s, /\. /); for (i = 1; i <= n; i++) print s[i] }' "$1"
}

# The build-order stages a file dispatches in plain prose, which
# `agent_names_in` cannot see. Each dispatch verb of a sentence is read on its
# own: the verb's object runs to the next dispatch verb or to a relative clause
# (`, which`), and it names `hw-build`, `hw-verify`, the builder or the verifier.
# A verb whose word before it is never, not, no or without is a refusal, and a
# verb whose word before it is which, that, who, it or hw-iterate is somebody
# else's dispatch. A negation earlier in the sentence ("If it does not answer,
# dispatch the verifier") does not count.
prose_dispatches_in() {
    sentences "$1" | awk '
        function lastword(s,    n, w, i) {
            n = split(s, w, /[^a-z-]+/)
            for (i = n; i >= 1; i--) if (w[i] != "") return w[i]
            return ""
        }
        {
            t = tolower($0)
            pos = 1
            while (pos <= length(t) && match(substr(t, pos), /(dispatch|launch|spawn)[a-z]*/)) {
                s = pos + RSTART - 1
                e = s + RLENGTH
                pos = e
                if (s > 1 && substr(t, s - 1, 1) ~ /[a-z]/) continue
                if (lastword(substr(t, 1, s - 1)) ~ /^(never|not|no|without|which|that|who|it|hw-iterate)$/) continue
                obj = substr(t, e)
                if (match(obj, /(dispatch|launch|spawn)/)) obj = substr(obj, 1, RSTART - 1)
                if (match(obj, /, (which|who|that) /)) obj = substr(obj, 1, RSTART - 1)
                if (obj ~ /(^|[^a-z-])(hw-build|builder)([^a-z-]|$)/) print "hw-build"
                if (obj ~ /(^|[^a-z-])(hw-verify|verifier)([^a-z-]|$)/) print "hw-verify"
            }
        }' | sort -u
}

# Whether a file dispatches a stage with `isolation: "worktree"`: one sentence
# in which a dispatch verb, then the stage, then the phrase come in that order,
# with no never, not, no, without, skip or omit between the stage and the
# phrase.
dispatches_isolated() {
    sentences "$1" | awk -v stage="$2" '
        {
            p = index($0, "isolation: \"worktree\"")
            if (!p) next
            pre = substr($0, 1, p - 1)
            sp = 0
            while ((i = index(substr(pre, sp + 1), stage)) > 0) sp += i
            # With no stage, sp is 0, the prefix below is empty and holds no verb.
            if (tolower(substr(pre, 1, sp - 1)) !~ /(^|[^a-z])(dispatch|launch|spawn)/) next
            if (tolower(substr(pre, sp)) ~ /(^|[^a-z])(never|not|no|without|skip|omit)([^a-z]|$)/) next
            found = 1
        }
        END { exit !found }'
}

# --- 1. every agent a command or an agent dispatches is a definition ----------

printf '# every agent a command or an agent dispatches is a definition on this tree\n'
check_dispatches() {
    file=$1
    missing=''
    for name in $(agent_names_in "$file"); do
        [ -f "$agents/$name.md" ] || missing="$missing $name"
    done
    printf '%s' "$missing"
}
for file in "$commands"/*.md "$agents"/*.md; do
    named=$(agent_names_in "$file" | wc -l | tr -d ' ')
    [ "$named" -gt 0 ] || continue
    missing=$(check_dispatches "$file")
    if [ -z "$missing" ]; then
        pass "$(basename "$file") dispatches $named agents, and every one is defined"
    else
        fail "$(basename "$file") dispatches only defined agents" "not defined:$missing"
    fi
done
# The refusal arm: a mistyped name reddens.
printf 'Dispatch `hw-adjudicate` and then `hw-adjudciate`.\n' > "$scratch/typo.md"
missing=$(check_dispatches "$scratch/typo.md")
if [ "$missing" = " hw-adjudciate" ]; then
    pass 'and a mistyped agent name is reported'
else
    fail 'a mistyped agent name is reported' "reported: \`$missing\`"
fi
# The build stage dispatches the maintainer before it opens a pull request, so
# the agent that names stale documents runs on every build and not only when a
# person asks (#954). A dropped or renamed line fails here, not in silence.
dispatches_agent() {
    agent_names_in "$1" | grep -qx "$2"
}
if dispatches_agent "$agents/hw-build.md" headwater-maintainer; then
    pass 'hw-build.md dispatches headwater-maintainer'
else
    fail 'hw-build.md dispatches headwater-maintainer' 'no `headwater-maintainer` span in .claude/agents/hw-build.md'
fi
# Its refusal arm: the same file with the name removed is reported.
sed 's/`headwater-maintainer`/the maintainer/g' "$agents/hw-build.md" > "$scratch/hw-build.md"
if dispatches_agent "$scratch/hw-build.md" headwater-maintainer; then
    fail 'a build stage without the maintainer dispatch is reported' 'the scratch copy still names it'
else
    pass 'and a build stage without the maintainer dispatch is reported'
fi

# --- 2. every skill an agent invokes is a skill on this tree ------------------

printf '\n# every skill an agent invokes is a skill on this tree\n'
for file in "$agents"/hw-*.md; do
    missing=''
    counted=0
    for name in $(grep -i 'invoke' "$file" | grep -oE '`[a-z][a-z-]*`' | tr -d '`' | sort -u); do
        counted=$((counted + 1))
        [ -f "$skills/$name/SKILL.md" ] || missing="$missing $name"
    done
    if [ "$counted" -eq 0 ]; then
        fail "$(basename "$file") invokes at least one skill" 'no invoke line names a skill'
    elif [ -z "$missing" ]; then
        pass "$(basename "$file") invokes $counted skills, and every one exists"
    else
        fail "$(basename "$file") invokes only skills that exist" "missing:$missing"
    fi
done

# --- 3. every ruling cited under .claude/ exists and is not superseded --------

printf '\n# every ruling cited under .claude/ is on the graph and not superseded\n'
# The graph export is computed and never committed (#1251), so this computes
# it from the newer of the two built engines, the way any reader of it does.
export_json="$scratch/export.json"
. "$root/tools/repo/resolve-engine.sh"
if ! engine=$(hw_resolve_engine_bin "$root"); then
    fail 'the built engine exports this repository' "$(hw_resolve_engine_missing_message "$root")"
elif ! "$engine" export --format json --root "$root" >"$export_json" 2>"$scratch/export.err"; then
    fail 'the built engine exports this repository' "$(tail -n 5 "$scratch/export.err")"
else
    missing=''
    superseded=''
    counted=0
    # Prose only. The shell suites under `.claude/` cite invented identifiers
    # on purpose, to prove a refusal, and those are not citations.
    for id in $(grep -rhoE 'HW-(DR|PD|OBL)-[0-9]{4}|HW-EVAL-[a-z][a-z0-9-]*' "$root/.claude" --include='*.md' | sort -u); do
        counted=$((counted + 1))
        if ! grep -q "\"id\": \"$id\"" "$export_json"; then
            missing="$missing $id"
        elif grep -A4 "\"id\": \"$id\"," "$export_json" | grep -q '"status": "superseded"'; then
            superseded="$superseded $id"
        fi
    done
    if [ "$counted" -eq 0 ]; then
        fail 'the harness cites at least one ruling' 'no identifier found under .claude/'
    elif [ -n "$missing" ]; then
        fail 'every cited ruling is on the graph' "not on the graph:$missing"
    elif [ -n "$superseded" ]; then
        fail 'no cited ruling is superseded' "superseded:$superseded"
    else
        pass "$counted rulings cited, every one on the graph and none superseded"
    fi
fi

# --- 4. front matter is exactly what a harness reads, and two agents cannot write

printf '\n# each build-order agent declares exactly name, description, tools, model and effort\n'
for file in "$agents"/hw-*.md; do
    name=$(basename "$file" .md)
    keys=$(sed -n '2,/^---$/p' "$file" | sed '/^---$/d' | sed 's/:.*//' | tr '\n' ' ' | sed 's/ $//')
    if [ "$keys" = "name description tools model effort" ]; then
        pass "$name declares name, description, tools, model and effort, in that order"
    else
        fail "$name declares exactly name, description, tools, model and effort" "it declares: $keys"
    fi
    declared=$(sed -n 's/^name: *//p' "$file" | head -1)
    [ "$declared" = "$name" ] || fail "$name is addressed by its file name" "front matter says \`$declared\`"
done
for name in hw-verify hw-integrate; do
    tools=$(sed -n 's/^tools: *//p' "$agents/$name.md" | head -1)
    flat=$(printf '%s' "$tools" | tr -d ' ')
    case ",$flat," in
        *,Edit,*|*,Write,*)
            fail "$name cannot edit or write" "tools: $tools" ;;
        *)
            pass "$name has neither Edit nor Write" ;;
    esac
done

# --- 5. the verification bar's headings are stated once ------------------------

printf '\n# every heading of the verification bar appears nowhere else under .claude/\n'
bar="$skills/hw-verification-bar/SKILL.md"
duplicated=''
counted=0
while IFS= read -r heading; do
    [ -n "$heading" ] || continue
    counted=$((counted + 1))
    hits=$(grep -rlxF --include='*.md' -- "$heading" "$root/.claude" | grep -v "hw-verification-bar/SKILL.md" || true)
    [ -z "$hits" ] || duplicated="$duplicated ${heading#\#\# }"
done <<EOF
$(grep '^## ' "$bar")
EOF
if [ "$counted" -lt 5 ]; then
    fail 'the verification bar carries its checks as headings' "only $counted headings"
elif [ -z "$duplicated" ]; then
    pass "$counted checks, each stated once"
else
    fail 'every check is stated once' "also stated elsewhere:$duplicated"
fi
# The refusal arm: a copy of one heading in another file is reported.
cp "$bar" "$scratch/bar.md"
grep '^## ' "$bar" | head -1 > "$scratch/other.md"
first=$(head -1 "$scratch/other.md")
if grep -rlxF -- "$first" "$scratch" | grep -q other.md; then
    pass 'and a heading copied into another file is found'
else
    fail 'a heading copied into another file is found' 'the copy was not found'
fi

# --- 6. the ceilings ---------------------------------------------------------

printf '\n# the three files the parent and every agent pay for hold their ceilings\n'
ceiling() {
    file=$1 limit=$2
    size=$(wc -c < "$root/$file" | tr -d ' ')
    if [ "$size" -le "$limit" ]; then
        pass "$file is $size bytes, under its ceiling of $limit"
    else
        fail "$file holds its ceiling of $limit bytes" "it is $size"
    fi
}
ceiling CLAUDE.md "$claude_md_ceiling"
ceiling .claude/commands/next-run.md "$next_run_ceiling"
ceiling .claude/run/doctrine.md "$doctrine_ceiling"

# --- 7. the doctrine in the command is the doctrine file, byte for byte -------

printf '\n# the doctrine block in next-run.md is byte-identical to .claude/run/doctrine.md\n'
sed -n '/^<!-- doctrine -->$/,/^<!-- \/doctrine -->$/p' "$commands/next-run.md" | sed '1d;$d' > "$scratch/block.md"
if [ ! -s "$scratch/block.md" ]; then
    fail 'next-run.md carries a doctrine block between its markers' 'no block found'
elif cmp -s "$scratch/block.md" "$root/.claude/run/doctrine.md"; then
    pass 'the block and the file are the same bytes'
else
    fail 'the block and the file are the same bytes' "$(diff "$scratch/block.md" "$root/.claude/run/doctrine.md" | head -5)"
fi
# The refusal arm: one changed byte is reported.
sed 's/^1\. /1) /' "$root/.claude/run/doctrine.md" > "$scratch/doctrine-changed.md"
if cmp -s "$scratch/block.md" "$scratch/doctrine-changed.md"; then
    fail 'a changed byte in the doctrine is reported' 'cmp read two different files as the same'
else
    pass 'and a changed byte in the doctrine is reported'
fi

# --- 8. the two commands name the same agents --------------------------------

printf '\n# next.md and next-run.md name the same build-order agents\n'
run_set=$(agent_names_in "$commands/next-run.md" | grep '^hw-')
one_set=$(agent_names_in "$commands/next.md" | grep '^hw-')
if [ -z "$run_set" ]; then
    fail 'next-run.md names the build-order agents' 'it names none'
elif [ "$run_set" = "$one_set" ]; then
    pass "both commands name the same $(printf '%s\n' "$run_set" | wc -l | tr -d ' ') agents"
else
    fail 'both commands name the same agents' "next-run: $(printf '%s' "$run_set" | tr '\n' ' ') / next: $(printf '%s' "$one_set" | tr '\n' ' ')"
fi

# --- 9. every cargo build/test an agent definition runs goes through tools/hw-cargo

# Scoped to the agent definitions themselves, and not to a skill or a command:
# a skill such as headwater-engine is the general reference and teaches the
# bare invocation on purpose, before explaining which stage wraps it and why.
# An agent definition's own command block is what a dispatch actually runs.
printf '\n# no agent definition runs a bare cargo build or cargo test\n'
bare=''
for file in "$agents"/*.md; do
    [ -f "$file" ] || continue
    hits=$(grep -nE 'cargo (build|test)\b' "$file" | grep -v 'hw-cargo' || true)
    [ -z "$hits" ] || bare="$bare
$(basename "$file"): $(printf '%s' "$hits" | tr '\n' ';')"
done
if [ -z "$bare" ]; then
    pass 'every agent definition runs cargo build/test through tools/hw-cargo'
else
    fail 'every agent definition runs cargo build/test through tools/hw-cargo' "$bare"
fi
# The refusal arm: a bare cargo build in a fresh file is reported.
printf 'Run `cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked`.\n' > "$scratch/bare-cargo.md"
hits=$(grep -nE 'cargo (build|test)\b' "$scratch/bare-cargo.md" | grep -v 'hw-cargo' || true)
if [ -n "$hits" ]; then
    pass 'and a bare cargo build in a fresh file is found'
else
    fail 'a bare cargo build in a fresh file is found' 'nothing was found'
fi

# --- 10. the parent dispatches the loop agent, and the loop agent dispatches build and verify

# The verify-and-rework loop for one issue is below the parent (#1276,
# HW-PD-0022). The parent names `hw-iterate`, and it names neither `hw-build`
# nor `hw-verify` and dispatches neither the builder nor the verifier in plain
# prose. `hw-iterate` names both, passes `isolation: "worktree"` in the sentence
# that dispatches each, with no never, not, no or without before it there, and
# can resume its builder by id, which needs `SendMessage` in its tools.
printf '\n# the parent dispatches hw-iterate, and hw-iterate dispatches hw-build and hw-verify\n'
# Prints why the pair fails, or nothing when it holds.
loop_below_parent() {
    command_file=$1 iterate_file=$2
    why=''
    names=$(agent_names_in "$command_file"; prose_dispatches_in "$command_file")
    names=$(printf '%s\n' "$names" | sort -u)
    printf '%s\n' "$names" | grep -qx hw-iterate || why="$why; $(basename "$command_file") does not dispatch hw-iterate"
    for stage in hw-build hw-verify; do
        printf '%s\n' "$names" | grep -qx "$stage" && why="$why; $(basename "$command_file") dispatches $stage itself"
    done
    if [ ! -f "$iterate_file" ]; then
        why="$why; no $(basename "$iterate_file")"
    else
        inner=$(agent_names_in "$iterate_file")
        for stage in hw-build hw-verify; do
            printf '%s\n' "$inner" | grep -qx "$stage" || why="$why; $(basename "$iterate_file") does not dispatch $stage"
        done
        for stage in hw-build hw-verify; do
            dispatches_isolated "$iterate_file" "$stage" || why="$why; $(basename "$iterate_file") passes no isolation: \"worktree\" to $stage"
        done
        tools=$(sed -n 's/^tools: *//p' "$iterate_file" | head -1 | tr -d ' ')
        case ",$tools," in
            *,SendMessage,*) ;;
            *) why="$why; $(basename "$iterate_file") cannot resume its builder: no SendMessage in tools" ;;
        esac
    fi
    printf '%s' "${why#; }"
}
why=$(loop_below_parent "$commands/next-run.md" "$agents/hw-iterate.md")
if [ -z "$why" ]; then
    pass 'next-run.md dispatches hw-iterate and names no build or verify dispatch, by name or in prose, and hw-iterate dispatches hw-build and hw-verify each in a sentence that passes isolation: "worktree"'
else
    fail 'the verify-and-rework loop is below the parent' "$why"
fi
# The refusal arms, each on a scratch copy.
cp "$commands/next-run.md" "$scratch/next-run.md"
printf 'On a build report, dispatch `hw-verify`.\n' >> "$scratch/next-run.md"
why=$(loop_below_parent "$scratch/next-run.md" "$agents/hw-iterate.md")
case "$why" in
    *'dispatches hw-verify itself'*) pass 'and a parent that dispatches hw-verify is reported' ;;
    *) fail 'a parent that dispatches hw-verify is reported' "reported: \`$why\`" ;;
esac
# A dispatch in plain prose names no backticked agent, and it is the same
# dispatch.
cp "$commands/next-run.md" "$scratch/next-run.md"
printf 'On a build report, dispatch the verifier.\n' >> "$scratch/next-run.md"
why=$(loop_below_parent "$scratch/next-run.md" "$agents/hw-iterate.md")
case "$why" in
    *'dispatches hw-verify itself'*) pass 'and a parent that dispatches the verifier in plain prose is reported' ;;
    *) fail 'a parent that dispatches the verifier in plain prose is reported' "reported: \`$why\`" ;;
esac
cp "$commands/next-run.md" "$scratch/next-run.md"
printf 'When the queue holds an issue, dispatch hw-build for it.\n' >> "$scratch/next-run.md"
why=$(loop_below_parent "$scratch/next-run.md" "$agents/hw-iterate.md")
case "$why" in
    *'dispatches hw-build itself'*) pass 'and a parent that dispatches hw-build unquoted is reported' ;;
    *) fail 'a parent that dispatches hw-build unquoted is reported' "reported: \`$why\`" ;;
esac
cp "$commands/next-run.md" "$scratch/next-run.md"
printf 'Never dispatch the verifier yourself.\n' >> "$scratch/next-run.md"
why=$(loop_below_parent "$scratch/next-run.md" "$agents/hw-iterate.md")
if [ -z "$why" ]; then
    pass 'and a parent that forbids itself the verifier in prose is not reported'
else
    fail 'a parent that forbids itself the verifier in prose is not reported' "reported: \`$why\`"
fi
# One sentence appended to the parent's command file, and the stage it must be
# reported as dispatching, or nothing where it must stay clean.
prose_arm() {
    cp "$commands/next-run.md" "$scratch/next-run.md"
    printf '%s\n' "$1" >> "$scratch/next-run.md"
    why=$(loop_below_parent "$scratch/next-run.md" "$agents/hw-iterate.md")
    if [ -n "$2" ]; then
        case "$why" in
            *"dispatches $2 itself"*) pass "and \"$1\" is reported as $2" ;;
            *) fail "\"$1\" is reported as $2" "reported: \`$why\`" ;;
        esac
    elif [ -z "$why" ]; then
        pass "and \"$1\" is not reported"
    else
        fail "\"$1\" is not reported" "reported: \`$why\`"
    fi
}
prose_arm 'If hw-iterate does not answer, dispatch the verifier yourself.' hw-verify
prose_arm 'When the queue holds no refusal, dispatch the builder for the issue.' hw-build
prose_arm 'On a build report, launch the verifier.' hw-verify
prose_arm 'Spawn hw-build for the next issue.' hw-build
prose_arm 'Do not dispatch the verifier yourself.' ''
prose_arm 'Rule each verdict without dispatching the verifier.' ''
prose_arm 'There is no dispatch of the builder in this file.' ''
prose_arm 'Dispatch hw-iterate, which launches the builder and the verifier.' ''
prose_arm 'Dispatch hw-iterate, which owns the builder and the verifier.' ''
prose_arm 'Dispatch hw-iterate and let it launch the builder.' ''
if [ -f "$agents/hw-iterate.md" ]; then
    # The phrase survives in the file, but only in a sentence that forbids it:
    # both stages would run in the shared checkout.
    sed 's/ with `isolation: "worktree"`//g' "$agents/hw-iterate.md" > "$scratch/hw-iterate.md"
    printf 'Never pass `isolation: "worktree"`.\n' >> "$scratch/hw-iterate.md"
    why=$(loop_below_parent "$commands/next-run.md" "$scratch/hw-iterate.md")
    case "$why" in
        *'passes no isolation: "worktree" to hw-build'*'passes no isolation: "worktree" to hw-verify'*) pass 'and a loop agent that names isolation only to forbid it is reported' ;;
        *) fail 'a loop agent that names isolation only to forbid it is reported' "reported: \`$why\`" ;;
    esac
    sed 's/Dispatch `hw-build` with `isolation/Dispatch `hw-build` without `isolation/' "$agents/hw-iterate.md" > "$scratch/hw-iterate.md"
    why=$(loop_below_parent "$commands/next-run.md" "$scratch/hw-iterate.md")
    case "$why" in
        *'passes no isolation: "worktree" to hw-build'*) pass 'and a loop agent that dispatches its builder without isolation is reported' ;;
        *) fail 'a loop agent that dispatches its builder without isolation is reported' "reported: \`$why\`" ;;
    esac
    # The phrase and the stage in one sentence that dispatches nothing.
    sed 's/Dispatch `hw-build` with `isolation: "worktree"`/Read `hw-build` for why `isolation: "worktree"` matters/' "$agents/hw-iterate.md" > "$scratch/hw-iterate.md"
    why=$(loop_below_parent "$commands/next-run.md" "$scratch/hw-iterate.md")
    case "$why" in
        *'passes no isolation: "worktree" to hw-build'*) pass 'and a loop agent that names isolation beside its builder without dispatching it is reported' ;;
        *) fail 'a loop agent that names isolation beside its builder without dispatching it is reported' "reported: \`$why\`" ;;
    esac
    # One edit to hw-iterate.md, and the stage it must be reported as passing
    # no isolation to, or nothing where it must stay clean.
    isolation_arm() {
        sed "$1" "$agents/hw-iterate.md" > "$scratch/hw-iterate.md"
        why=$(loop_below_parent "$commands/next-run.md" "$scratch/hw-iterate.md")
        if [ -n "$2" ]; then
            case "$why" in
                *"passes no isolation: \"worktree\" to $2"*) pass "and \`$1\` is reported for $2" ;;
                *) fail "\`$1\` is reported for $2" "reported: \`$why\`" ;;
            esac
        elif [ -z "$why" ]; then
            pass "and \`$1\` is not reported"
        else
            fail "\`$1\` is not reported" "reported: \`$why\`"
        fi
    }
    isolation_arm 's/Dispatch `hw-build` with `isolation/Dispatch `hw-build` but never with `isolation/' hw-build
    isolation_arm 's/Dispatch `hw-build` with `isolation/Dispatch `hw-build`, and do not pass `isolation/' hw-build
    isolation_arm 's/Dispatch `hw-build` with `isolation/Dispatch `hw-build` with no `isolation/' hw-build
    isolation_arm 's/Dispatch `hw-build` with `isolation/Dispatch `hw-build` and skip `isolation/' hw-build
    isolation_arm 's/Dispatch `hw-build` with `isolation/Dispatch `hw-build` and omit `isolation/' hw-build
    isolation_arm 's/Dispatch `hw-build` with `isolation: "worktree"`/Dispatch `hw-build` in its own worktree/' hw-build
    isolation_arm 's/Dispatch `hw-build` with `isolation/Dispatch `hw-build` as `hw-build.md` says, with `isolation/' ''
    isolation_arm 's/Dispatch a fresh `hw-verify` with `isolation/Launch a fresh `hw-verify` with `isolation/' ''
    isolation_arm 's/Dispatch `hw-build` with `isolation/Spawn `hw-build` with `isolation/' ''
    sed 's/isolation: "worktree"/isolation unset/g' "$agents/hw-iterate.md" > "$scratch/hw-iterate.md"
    why=$(loop_below_parent "$commands/next-run.md" "$scratch/hw-iterate.md")
    case "$why" in
        *'passes no isolation'*) pass 'and a loop agent that passes no isolation is reported' ;;
        *) fail 'a loop agent that passes no isolation is reported' "reported: \`$why\`" ;;
    esac
    sed '/^tools:/s/, *SendMessage//; /^tools:/s/SendMessage, *//' "$agents/hw-iterate.md" > "$scratch/hw-iterate.md"
    why=$(loop_below_parent "$commands/next-run.md" "$scratch/hw-iterate.md")
    case "$why" in
        *'no SendMessage'*) pass 'and a loop agent that cannot resume its builder is reported' ;;
        *) fail 'a loop agent that cannot resume its builder is reported' "reported: \`$why\`" ;;
    esac
else
    fail 'the refusal arms on hw-iterate.md run' 'no .claude/agents/hw-iterate.md to copy'
fi

printf '\n%s passed, %s failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
