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
# Eleven cases, and the ceilings are the reason two of them exist. The parent's
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
# HW-PD-0022). The parent names `hw-iterate` and never `hw-build` or
# `hw-verify`; `hw-iterate` names both, gives each its own worktree, and can
# resume its builder by id, which needs `SendMessage` in its tools.
printf '\n# the parent dispatches hw-iterate, and hw-iterate dispatches hw-build and hw-verify\n'
# Prints why the pair fails, or nothing when it holds.
loop_below_parent() {
    command_file=$1 iterate_file=$2
    why=''
    names=$(agent_names_in "$command_file")
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
        grep -qF 'isolation: "worktree"' "$iterate_file" || why="$why; $(basename "$iterate_file") passes no isolation: \"worktree\""
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
    pass 'next-run.md dispatches hw-iterate alone, and hw-iterate dispatches hw-build and hw-verify in worktrees'
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
if [ -f "$agents/hw-iterate.md" ]; then
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

# --- 11. the rule against filing an issue covers the parent in its bold lead --

# The parent is an agent of the run too, and a lead that names only a stage
# reads as not covering it (#1275). The parent loads hw-run-policy at the start
# of every session, so the skill's bold lead is where the words reach it. The
# case reads the bold lead alone: the rest of the line already ends "and the
# parent rules on it", which is about ruling and not about filing. The lead is
# the text of a bullet up to its first `**`, so single-asterisk emphasis
# inside it does not end it, and the rule is the first bullet whose lead says an agent "files an
# issue" or "files no issue", so a rewording in either form is still found.
printf '\n# the rule against filing an issue in hw-run-policy covers the parent in its bold lead\n'
# Prints the bold lead of the no-filing rule in the file given.
no_filing_lead() {
    awk 'index($0, "- **") == 1 {
        lead = substr($0, 5); i = index(lead, "**"); if (i == 0) next
        lead = substr(lead, 1, i - 1)
        if (lead ~ /files (an|no) issue/) { print lead; exit }
    }' "$1"
}
# Prints why a lead fails to cover the parent, or nothing when it covers it.
lead_misses_parent() {
    case "$1" in
        '') printf 'no bullet has a bold lead that says an agent files an issue or files no issue' ; return ;;
        *parent*) ;;
        *) printf 'the lead does not name the parent: %s' "$1" ; return ;;
    esac
    if printf '%s\n' "$1" | grep -qiE '(except|apart from|other than|but|besides|excluding|save|not) (for )?the parent'; then
        printf 'the lead names the parent as an exception: %s' "$1"
    fi
}
lead=$(no_filing_lead "$skills/hw-run-policy/SKILL.md")
why=$(lead_misses_parent "$lead")
if [ -z "$why" ]; then
    pass "the bold lead covers the parent: $lead"
else
    fail 'the bold lead of the no-filing rule covers the parent' "$why"
fi
# The arms, each a one-line rule in a scratch file: two leads that do not cover
# the parent are reported, and a rewording with emphasis inside the lead holds.
lead_arm() {
    printf -- '- **%s** The rest of the rule.\n' "$1" > "$scratch/no-filing.md"
    lead_misses_parent "$(no_filing_lead "$scratch/no-filing.md")"
}
why=$(lead_arm 'No stage files an issue inside a run.')
case "$why" in
    *'does not name the parent'*) pass 'and a bold lead that names only a stage is reported' ;;
    *) fail 'a bold lead that names only a stage is reported' "reported: \`$why\`" ;;
esac
why=$(lead_arm 'No stage files an issue inside a run, apart from the parent.')
case "$why" in
    *'as an exception'*) pass 'and a bold lead that exempts the parent is reported' ;;
    *) fail 'a bold lead that exempts the parent is reported' "reported: \`$why\`" ;;
esac
why=$(lead_arm 'An agent of a run, the *parent* included, files no issue.')
if [ -z "$why" ]; then
    pass 'and a reworded lead with emphasis inside it holds'
else
    fail 'a reworded lead with emphasis inside it holds' "$why"
fi

printf '\n%s passed, %s failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
