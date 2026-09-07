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
# Eight cases, and the ceilings are the reason two of them exist. The parent's
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
export_json="$root/.headwater/export.json"

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

# --- 1. every agent a command dispatches is a definition on this tree ---------

printf '# every agent a command dispatches is a definition on this tree\n'
check_dispatches() {
    file=$1
    missing=''
    for name in $(agent_names_in "$file"); do
        [ -f "$agents/$name.md" ] || missing="$missing $name"
    done
    printf '%s' "$missing"
}
for file in "$commands"/*.md; do
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
if [ ! -f "$export_json" ]; then
    fail 'the graph export is on this tree' "$export_json is missing"
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

printf '\n# each build-order agent declares exactly name, description, tools and model\n'
for file in "$agents"/hw-*.md; do
    name=$(basename "$file" .md)
    keys=$(sed -n '2,/^---$/p' "$file" | sed '/^---$/d' | sed 's/:.*//' | tr '\n' ' ' | sed 's/ $//')
    if [ "$keys" = "name description tools model" ]; then
        pass "$name declares name, description, tools and model, in that order"
    else
        fail "$name declares exactly name, description, tools and model" "it declares: $keys"
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

printf '\n%s passed, %s failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
