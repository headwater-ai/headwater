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
# Fifteen cases, and this list is the one place that names them all:
#
#     1   every agent a command or an agent dispatches is a definition
#     2   every skill an agent invokes is a skill on this tree
#     3   every ruling cited under .claude/ exists and is not superseded
#     4   front matter is what a harness reads, and two agents cannot write
#     5   the verification bar's headings are stated once
#     6   the byte ceilings of CLAUDE.md, next-run.md and the doctrine
#     7   the doctrine in the command is the doctrine file, byte for byte
#     8   the two commands name the same agents
#     9   every cargo build or test an agent runs goes through tools/hw-cargo
#     10  the parent dispatches the loop agent, and it dispatches build and verify
#     11  the rule against filing an issue covers the parent, by its marker
#     12  the Record ruling checks the intake line against the tree
#     13  each throughput rule of 2026-10-01 has one owner
#     14  every file under .claude is shipped or named as internal
#     15  the first session hands the run to supervise.sh
#
# The ceilings are the reason two of them exist. The parent's
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

# --- 11. the rule against filing an issue covers the parent, by its marker ----

# The parent is an agent of the run too, and a lead that names only a stage
# reads as not covering it (#1275). The parent loads hw-run-policy at the start
# of every session, so the skill's no-filing rule is where the words reach it.
# The rule is the first bullet whose bold lead says an agent "files an issue" or
# "files no issue", and the lead is the text up to its first `**`, so
# single-asterisk emphasis inside it does not end it. The lead must name the
# parent. Whether it covers the parent is read from a marker the bullet carries,
# not from the prose, because no word list holds every way to exempt the parent
# in English: an earlier form of this case passed "the parent aside" and
# "unless it is the parent". The marker is one HTML comment on the bullet,
#     <!-- hw-filing-rule covers=<actor>,... filer=<agent> -->
# and it holds when `covers` names `parent` and `filer` names
# `headwater-product-owner`, the one agent of a run that files an issue.
printf '\n# the rule against filing an issue in hw-run-policy covers the parent, by its marker\n'
# Prints the whole bullet line of the no-filing rule in the file given.
no_filing_bullet() {
    awk 'index($0, "- **") == 1 {
        lead = substr($0, 5); i = index(lead, "**"); if (i == 0) next
        lead = substr(lead, 1, i - 1)
        if (lead ~ /files (an|no) issue/) { print; exit }
    }' "$1"
}
# Prints why a bullet fails to cover the parent, or nothing when it covers it.
bullet_misses_parent() {
    case "$1" in
        '') printf 'no bullet has a bold lead that says an agent files an issue or files no issue' ; return ;;
    esac
    lead=${1#- \*\*}
    lead=${lead%%\*\**}
    case "$lead" in
        *parent*) ;;
        *) printf 'the lead does not name the parent: %s' "$lead" ; return ;;
    esac
    marker=$(printf '%s\n' "$1" | grep -oE '<!-- hw-filing-rule [^>]*-->' | head -n 1)
    if [ -z "$marker" ]; then
        printf 'the rule carries no <!-- hw-filing-rule covers=... filer=... --> marker'
        return
    fi
    covers=$(printf '%s\n' "$marker" | sed -n 's/.* covers=\([^ ]*\).*/\1/p')
    filer=$(printf '%s\n' "$marker" | sed -n 's/.* filer=\([^ ]*\).*/\1/p')
    printf '%s\n' "$covers" | tr ',' '\n' | grep -qx parent || printf ' the marker does not cover the parent: %s' "$marker"
    [ "$filer" = headwater-product-owner ] || printf ' the marker does not name headwater-product-owner as the filer: %s' "$marker"
}
bullet=$(no_filing_bullet "$skills/hw-run-policy/SKILL.md")
why=$(bullet_misses_parent "$bullet")
if [ -z "$why" ]; then
    lead=${bullet#- \*\*}
    pass "the no-filing rule covers the parent: ${lead%%\*\**}"
else
    fail 'the no-filing rule covers the parent' "$why"
fi
# The arms, each a one-line rule in a scratch file.
marker='<!-- hw-filing-rule covers=every-agent,parent filer=headwater-product-owner -->'
filing_arm() {
    printf -- '- **%s** The rest of the rule.%s\n' "$1" "$2" > "$scratch/no-filing.md"
    bullet_misses_parent "$(no_filing_bullet "$scratch/no-filing.md")"
}
why=$(filing_arm 'No stage files an issue inside a run.' " $marker")
case "$why" in
    *'does not name the parent'*) pass 'and a bold lead that names only a stage is reported' ;;
    *) fail 'a bold lead that names only a stage is reported' "reported: \`$why\`" ;;
esac
# Four leads that exempt the parent in words, with no marker. The last three
# passed the word list this case held before.
for exempt in 'No stage files an issue inside a run, apart from the parent.' \
    'No agent files an issue inside a run; the parent may.' \
    'No agent files an issue inside a run, unless it is the parent.' \
    'No agent files an issue inside a run, the parent aside.'; do
    why=$(filing_arm "$exempt" '')
    case "$why" in
        *'no <!-- hw-filing-rule'*) pass "and a lead with no marker is reported: $exempt" ;;
        *) fail "a lead with no marker is reported: $exempt" "reported: \`$why\`" ;;
    esac
done
why=$(filing_arm 'No agent of a run files an issue, the parent included.' ' <!-- hw-filing-rule covers=every-stage filer=headwater-product-owner -->')
case "$why" in
    *'does not cover the parent'*) pass 'and a marker that does not cover the parent is reported' ;;
    *) fail 'a marker that does not cover the parent is reported' "reported: \`$why\`" ;;
esac
why=$(filing_arm 'No agent of a run files an issue, the parent included.' ' <!-- hw-filing-rule covers=every-agent,parents filer=headwater-product-owner -->')
case "$why" in
    *'does not cover the parent'*) pass 'and a marker that names a word containing parent is reported' ;;
    *) fail 'a marker that names a word containing parent is reported' "reported: \`$why\`" ;;
esac
why=$(filing_arm 'No agent of a run files an issue, the parent included.' ' <!-- hw-filing-rule covers=every-agent,parent filer=parent -->')
case "$why" in
    *'does not name headwater-product-owner'*) pass 'and a marker that names another filer is reported' ;;
    *) fail 'a marker that names another filer is reported' "reported: \`$why\`" ;;
esac
why=$(filing_arm 'An agent of a run, the *parent* included, files no issue.' " $marker")
if [ -z "$why" ]; then
    pass 'and a reworded lead with emphasis inside it and the marker holds'
else
    fail 'a reworded lead with emphasis inside it and the marker holds' "$why"
fi

# --- 12. the Record ruling checks the intake line against the tree -----------

# An intake line becomes an obligation record under the product owner's Record
# ruling, and three records filed that way were false on the day they were
# written, because nothing opened the file the line named (#1486). The ruling
# must say two things: the product owner confirms the finding on the current
# tree, and the record names the file and the symbol it checked. The case reads
# the item under `## Intake` whose bold lead is `Record.`, and it matches on the
# words `tree`, `file` and `symbol` as whole words, so a rewording that keeps the
# requirement still holds and `filed` does not count as `file`.
printf '\n# the Record ruling of headwater-product-owner checks the line against the tree\n'
# Prints the item under `## Intake` whose bold lead is `Record.`.
record_ruling() {
    awk '/^## / { inside = ($0 == "## Intake") ; next }
        inside && index($0, "**Record.**") { print; exit }' "$1"
}
# Prints why a Record ruling fails the requirement, or nothing when it holds.
record_misses_check() {
    if [ -z "$1" ]; then
        printf 'no item under ## Intake has the bold lead Record.' ; return
    fi
    # Each test reads one clause, split at a sentence end, a colon, a semicolon
    # or ", and". One clause must both name the tree and say to open, read,
    # confirm, check or verify: a ruling that meets the tree only where it
    # withdraws a line has not said to look at it. One clause must name both the
    # file and the symbol: a ruling that says "open the file" in one clause and
    # "name the symbol" in another has not asked the record to name the file.
    clauses=$(printf '%s\n' "$1" | sed 's/\. /.\n/g; s/: /:\n/g; s/; /;\n/g; s/, and /,\nand /g')
    if ! printf '%s\n' "$clauses" | grep -iw 'tree' | grep -qiwE 'opens?|reads?|confirms?|checks?|verify|verifies'; then
        printf 'the ruling does not check the line against the tree: %s' "$1" ; return
    fi
    if ! printf '%s\n' "$clauses" | grep -iw 'file' | grep -qiw 'symbol'; then
        printf 'the ruling does not require naming the file and symbol checked: %s' "$1"
    fi
}
ruling=$(record_ruling "$agents/headwater-product-owner.md")
why=$(record_misses_check "$ruling")
if [ -z "$why" ]; then
    pass 'the Record ruling checks the line against the tree and names the file and symbol'
else
    fail 'the Record ruling checks the line against the tree and names the file and symbol' "$why"
fi
# The arms, each an Intake section in a scratch file: the ruling as it stood
# before #1486 is reported, a ruling that meets the tree only where it withdraws
# a line is reported, a ruling that names no symbol or no file in the clause
# that asks the record to name them is reported, a file with no Record item
# under Intake is reported, and two reworded rulings hold.
record_arm() {
    printf '## Intake\n\n1. **Fold.** An open issue covers it.\n%s\n\n## Report\n' "$1" > "$scratch/record.md"
    record_misses_check "$(record_ruling "$scratch/record.md")"
}
why=$(record_arm '2. **Record.** The line names no reader outside this repository. It goes to 13 as an obligation record. `headwater new` writes the record at `draft`, and 13 records filed at `draft` had to move by hand.')
case "$why" in
    *'against the tree'*) pass 'and the ruling as it stood before #1486 is reported' ;;
    *) fail 'the ruling as it stood before #1486 is reported' "reported: \`$why\`" ;;
esac
why=$(record_arm '2. **Record.** Name in the Context the file and the symbol. A line that does not hold on the tree is withdrawn.')
case "$why" in
    *'against the tree'*) pass 'and a ruling that meets the tree only where it withdraws a line is reported' ;;
    *) fail 'a ruling that meets the tree only where it withdraws a line is reported' "reported: \`$why\`" ;;
esac
why=$(record_arm '2. **Record.** Open the file the line names on the current tree and confirm it, then write the record.')
case "$why" in
    *'file and symbol'*) pass 'and a ruling that checks the tree but names no symbol is reported' ;;
    *) fail 'a ruling that checks the tree but names no symbol is reported' "reported: \`$why\`" ;;
esac
why=$(record_arm '2. **Record.** Open the file the line names on the current tree and confirm it, and name in the Context the symbol you checked.')
case "$why" in
    *'file and symbol'*) pass 'and a ruling that opens the file but asks the record to name only the symbol is reported' ;;
    *) fail 'a ruling that opens the file but asks the record to name only the symbol is reported' "reported: \`$why\`" ;;
esac
why=$(record_arm '2. **Record.** Check the line on the current tree, and name in the Context the symbol you read.')
case "$why" in
    *'file and symbol'*) pass 'and a ruling that checks the tree and names a symbol but no file is reported' ;;
    *) fail 'a ruling that checks the tree and names a symbol but no file is reported' "reported: \`$why\`" ;;
esac
why=$(record_arm '2. **Backlog.** It has a reader.')
case "$why" in
    *'no item'*) pass 'and an Intake section with no Record item is reported' ;;
    *) fail 'an Intake section with no Record item is reported' "reported: \`$why\`" ;;
esac
# A Record item that would hold, under a heading other than Intake: the reader
# must not find it.
printf '## Intake\n\n1. **Fold.** An open issue covers it.\n\n## Report\n\n2. **Record.** Open the file on the current tree, and name the file and the symbol.\n' > "$scratch/record.md"
why=$(record_misses_check "$(record_ruling "$scratch/record.md")")
case "$why" in
    *'no item'*) pass 'and a Record item outside ## Intake is not read' ;;
    *) fail 'a Record item outside ## Intake is not read' "reported: \`$why\`" ;;
esac
why=$(record_arm '2. **Record.** Verify the finding on the current tree, and name in the Context the file and the symbol.')
if [ -z "$why" ]; then
    pass 'and a reworded ruling that says verify holds'
else
    fail 'a reworded ruling that says verify holds' "$why"
fi
why=$(record_arm '2. **Record.** Before you record it, confirm the finding on the current *tree*: open the File it names. The Context names that file and the Symbol you read, never a line number.')
if [ -z "$why" ]; then
    pass 'and a reworded ruling with both requirements holds'
else
    fail 'a reworded ruling with both requirements holds' "$why"
fi

# --- 13. each throughput rule of 2026-10-01 has one owner ----------------------

# The owner ruled five throughput rules in run 20260930-1333, and each one is
# stated once, in the definition of the stage that obeys it ([HW-PD-0001]). A
# rule that is dropped from its owner, or copied into a second file, is a rule
# that a stage reads in a form its owner no longer states. Each row is the
# owner's path under .claude/ and a phrase of its defining sentence. A phrase
# carries the operative words of its rule: the condition, the stop, the source
# of a script, the exception. So an edit that inverts or weakens a rule inside
# its sentence breaks the phrase. An edit that adds a new sentence after it does
# not, and nothing here reads meaning. The case reports a phrase that its owner
# lost, a phrase that another Markdown file under .claude/agents,
# .claude/commands or .claude/skills states too in any case of letters, and a
# corpus fold on the FOOTPRINT paragraph of hw-adjudicate, which the parent
# claims.
printf '\n# the operative words of each throughput rule of 2026-10-01 are in one definition\n'
throughput_rules='agents/hw-build.md|When you open the pull request, and each time you update it, write one line `- [x] #N.<k> <evidence>` in its body for each clause of `sh tools/run/gh-issue.sh clauses <N>` that the branch meets, folds included.
agents/hw-build.md|When the branch does not meet a clause, make the pull request `Refs #N` in its title and in its body.
agents/hw-build.md|**Run the verifier'"'"'s attacks before you report.**
agents/hw-build.md|Run each one on your own branch: apply each mutant you can, revert it, and list in `build.md` each one that survived.
agents/hw-build.md|Hold a survivor with a test before you report, where you can.
agents/hw-verify.md|Take the script from `main`, not from the branch: after `git fetch origin`, run `git show origin/main:tools/run/gh-issue.sh > <scratch>/gh-issue-main.sh`
agents/hw-verify.md|A `PASS` requires exit 0. When the title says
agents/hw-verify.md|When the title says `Closes #N` and the body says `Refs #N`, or the reverse, that is a `FAIL` of the body and not of the code.
agents/hw-verify.md|Before your first build makes a fresh `target-verify-<N>`, run `sh tools/run/disk-free.sh`.
agents/hw-verify.md|When the pool'"'"'s volume has less than 40G free, stop and report `FAIL` with `disk: <the reading>` as the first line of `FIRED`
agents/hw-iterate.md|**Verify.** Before each verify, run `sh tools/run/disk-free.sh`
agents/hw-iterate.md|The floor that `hw-verify` states is on the pool'"'"'s volume.
skills/hw-run-policy/SKILL.md|`hw-iterate` runs `sh tools/run/disk-free.sh` before it dispatches a verify
agents/hw-iterate.md|name them on the builder'"'"'s `attacks:` line as well, so that the builder runs them first.
agents/hw-adjudicate.md|`FOOTPRINT` is what the parent claims: the paths the change edits by hand, and the recorded fixtures it moves
agents/hw-adjudicate.md|So an overlap on a derived fold alone is not a `WAITS-ON`, and an overlap on a hand-edited path still is.
agents/hw-adjudicate.md|The cause decides the line: a fixture that the corpus moves is `DERIVED`, and one that the change'"'"'s own engine code moves is on `FOOTPRINT`.
agents/headwater-product-owner.md|**A fold never adds a clause to an issue that an open pull request says `Closes`.** Write the clause as a follow-up instead
skills/hw-run-policy/SKILL.md|A claim takes the `FOOTPRINT` line of an adjudication and never its `DERIVED` line'
throughput_count=$(printf '%s\n' "$throughput_rules" | wc -l | tr -d ' ')
# Prints why the tree under the given .claude directory fails, or nothing.
throughput_owners() {
    dot=$1
    why=''
    list=$(find "$dot/agents" "$dot/commands" "$dot/skills" -name '*.md' -type f | sort)
    printf '%s\n' "$throughput_rules" | {
        while IFS='|' read -r owner phrase; do
            grep -qF -- "$phrase" "$dot/$owner" 2>/dev/null || why="$why; $owner no longer states: $phrase"
            for f in $list; do
                [ "$f" = "$dot/$owner" ] && continue
                grep -qiF -- "$phrase" "$f" && why="$why; ${f#"$dot"/} states it too: $phrase"
            done
        done
        for fold in .headwater/corpus.json .headwater/nav.yml 'README.md` index'; do
            grep -F -- '`FOOTPRINT` is what the parent claims' "$dot/agents/hw-adjudicate.md" 2>/dev/null | grep -qF -- "$fold" &&
                why="$why; hw-adjudicate.md names a corpus fold on FOOTPRINT: $fold"
        done
        printf '%s' "${why#; }"
    }
}
why=$(throughput_owners "$root/.claude")
if [ -z "$why" ]; then
    pass "the $throughput_count operative phrases of the throughput rules are each in their owner and in no other file"
else
    fail 'the operative words of each throughput rule are in its owner alone' "$why"
fi
# The refusal arms, on a scratch copy of the Markdown under .claude/.
mkdir -p "$scratch/dot"
(cd "$root/.claude" && find agents commands skills -name '*.md' -type f) | while read -r f; do
    mkdir -p "$scratch/dot/$(dirname "$f")"
    cp "$root/.claude/$f" "$scratch/dot/$f"
done
grep -vF 'A `PASS` requires exit 0' "$root/.claude/agents/hw-verify.md" > "$scratch/dot/agents/hw-verify.md"
why=$(throughput_owners "$scratch/dot")
case "$why" in
    *'hw-verify.md no longer states'*) pass 'and a rule dropped from its owner is reported' ;;
    *) fail 'a rule dropped from its owner is reported' "reported: \`$why\`" ;;
esac
sed 's/has less than 40G free, stop and report `FAIL`/has less than 40G free, note it and build anyway, or report `FAIL`/' "$root/.claude/agents/hw-verify.md" > "$scratch/dot/agents/hw-verify.md"
why=$(throughput_owners "$scratch/dot")
case "$why" in
    *"hw-verify.md no longer states: When the pool's volume has less than 40G"*) pass 'and a rule weakened inside its own sentence is reported' ;;
    *) fail 'a rule weakened inside its own sentence is reported' "reported: \`$why\`" ;;
esac
cp "$root/.claude/agents/hw-verify.md" "$scratch/dot/agents/hw-verify.md"
printf 'Note: so an overlap on a derived fold alone is not a `WAITS-ON`, and an overlap on a hand-edited path still is.\n' >> "$scratch/dot/skills/hw-run-policy/SKILL.md"
why=$(throughput_owners "$scratch/dot")
case "$why" in
    *'skills/hw-run-policy/SKILL.md states it too'*) pass 'and a rule copied into a second file is reported' ;;
    *) fail 'a rule copied into a second file is reported' "reported: \`$why\`" ;;
esac
cp "$root/.claude/skills/hw-run-policy/SKILL.md" "$scratch/dot/skills/hw-run-policy/SKILL.md"
sed 's/`docs\/probe-results\/`, and the `package.yml`/`docs\/probe-results\/`, `.headwater\/corpus.json`, and the `package.yml`/' "$root/.claude/agents/hw-adjudicate.md" > "$scratch/dot/agents/hw-adjudicate.md"
why=$(throughput_owners "$scratch/dot")
case "$why" in
    *'names a corpus fold on FOOTPRINT: .headwater/corpus.json'*) pass 'and a corpus fold put back on FOOTPRINT is reported' ;;
    *) fail 'a corpus fold put back on FOOTPRINT is reported' "reported: \`$why\`" ;;
esac

# --- 14. every file under .claude is shipped or named as internal --------------

# `headwater init --harness` writes the shipped set from text compiled into the
# binary, and `engine/crates/cli/harness/` is that text (#1578). Everything
# else under `.claude/agents`, `.claude/skills` and `.claude/commands` is this
# repository's own build order and must not reach an adopter. So each name is
# in exactly one of two lists: the compiled set, read from the tree, and the
# internal list below. A new file in neither fails, and so does an internal
# name placed in the compiled set.
printf '\n# every file under .claude is in the shipped set or named as internal\n'
# One `case` list, so `hw-*` is a pattern and never a glob over the current
# directory. This list is the internal list of the case.
is_internal() {
    case $1 in
        hw-* | headwater-engine | headwater-product-owner | repo-cleanup | ste-editor | next | next-run | product-owner) return 0 ;;
    esac
    return 1
}
# The names of the compiled set under a harness source directory.
shipped_names() {
    for file in "$1"/skills/*/SKILL.md "$1"/agents/*.md; do
        [ -f "$file" ] || continue
        case $file in
            */SKILL.md) basename "$(dirname "$file")" ;;
            *) basename "$file" .md ;;
        esac
    done
}
# Every reason the boundary does not hold, for a harness source and a .claude.
boundary() {
    harness=$1 dot=$2
    shipped=$(shipped_names "$harness")
    [ -n "$shipped" ] || printf ' the compiled set under %s is empty' "$harness"
    for name in $shipped; do
        is_internal "$name" && printf ' %s is internal and in the compiled set' "$name"
    done
    for file in "$dot"/agents/*.md "$dot"/commands/*.md "$dot"/skills/*/SKILL.md; do
        [ -f "$file" ] || continue
        case $file in
            */SKILL.md) name=$(basename "$(dirname "$file")") ;;
            *) name=$(basename "$file" .md) ;;
        esac
        printf '%s\n' "$shipped" | grep -qx "$name" && continue
        is_internal "$name" || printf ' %s is neither shipped nor named as internal' "$name"
    done
}
why=$(boundary "$root/engine/crates/cli/harness" "$root/.claude")
if [ -z "$why" ]; then
    pass "every name under .claude is shipped ($(shipped_names "$root/engine/crates/cli/harness" | wc -l | tr -d ' ')) or internal"
else
    fail 'every name under .claude is shipped or internal' "$why"
fi
# The two refusal arms: an internal agent in the compiled set, and a new file
# in neither list.
mkdir -p "$scratch/harness/agents" "$scratch/harness/skills/headwater-orient"
cp "$root/engine/crates/cli/harness/skills/headwater-orient/SKILL.md" "$scratch/harness/skills/headwater-orient/SKILL.md"
cp "$root/.claude/agents/hw-build.md" "$scratch/harness/agents/hw-build.md"
case $(boundary "$scratch/harness" "$root/.claude") in
    *'hw-build is internal and in the compiled set'*) pass 'and an internal agent placed in the compiled set is reported' ;;
    *) fail 'an internal agent placed in the compiled set is reported' 'the boundary passed it' ;;
esac
mkdir -p "$scratch/newdot/agents"
printf -- '---\nname: unsorted\n---\n' > "$scratch/newdot/agents/unsorted.md"
case $(boundary "$root/engine/crates/cli/harness" "$scratch/newdot") in
    *'unsorted is neither shipped nor named as internal'*) pass 'and a file in neither list is reported' ;;
    *) fail 'a file in neither list is reported' 'the boundary passed it' ;;
esac
# The same arm in the other two directories the boundary reads.
mkdir -p "$scratch/cmddot/commands" "$scratch/skilldot/skills/adopter-tips"
printf -- '---\ndescription: a command in neither list\n---\n' > "$scratch/cmddot/commands/ship.md"
printf -- '---\nname: adopter-tips\n---\n' > "$scratch/skilldot/skills/adopter-tips/SKILL.md"
case $(boundary "$root/engine/crates/cli/harness" "$scratch/cmddot") in
    *'ship is neither shipped nor named as internal'*) pass 'and a command in neither list is reported' ;;
    *) fail 'a command in neither list is reported' 'the boundary passed it' ;;
esac
case $(boundary "$root/engine/crates/cli/harness" "$scratch/skilldot") in
    *'adopter-tips is neither shipped nor named as internal'*) pass 'and a skill in neither list is reported' ;;
    *) fail 'a skill in neither list is reported' 'the boundary passed it' ;;
esac
# A name that is part of a shipped name is not that name.
mkdir -p "$scratch/partdot/skills/orient"
printf -- '---\nname: orient\n---\n' > "$scratch/partdot/skills/orient/SKILL.md"
case $(boundary "$root/engine/crates/cli/harness" "$scratch/partdot") in
    *'orient is neither shipped nor named as internal'*) pass 'and a skill named for part of a shipped name is reported' ;;
    *) fail 'a skill named for part of a shipped name is reported' 'the boundary passed it' ;;
esac

# --- 15. the first session hands the run to supervise.sh ---------------------

# `tools/run/supervise.sh` restarts the parent every few merges, and until
# #1275 no sentence told anybody to start it: the policy said it "runs each
# session after the first" and named no starter, so every run stayed one
# unsupervised session, the failure the script exists to prevent. The parent
# loads hw-run-policy at the start of every session, so the handover is a
# sub-bullet of the restart rule there, found by `first session` in its bold
# lead. It must name the script, the first session as the one that starts it,
# `HW_SUPERVISE_TREE`, because the script refuses a main checkout, and the drain
# to zero before it starts, because two live parents on one run each fill
# slots. It must also hold the steps: a bold lead that says the first session
# starts the loop, `run_in_background: true` and no foreground, a first session
# that fills no slot, and a wake on the `STOPPED:` line. This case holds the
# text alone. Only a supervised run shows that the
# loop runs.
printf '\n# the first session of a run hands the run to supervise.sh in hw-run-policy\n'
# Prints the handover sub-bullet of the restart rule in the file given: the
# indented bullet whose bold lead names the first session, under the top-level
# bullet whose bold lead says a parent restarts.
handover_bullet() {
    awk 'index($0, "- **") == 1 {
            lead = substr($0, 5); i = index(lead, "**")
            inrule = (i > 0 && index(substr(lead, 1, i - 1), "restarts") > 0); next
        }
        inrule && index($0, "  - **") == 1 {
            lead = substr($0, 7); i = index(lead, "**")
            if (i > 0 && index(substr(lead, 1, i - 1), "first session") > 0) { print; exit }
        }' "$1"
}
# Prints why the handover does not hold, or nothing when it holds.
handover_misses() {
    bullet=$(handover_bullet "$1")
    if [ -z "$bullet" ]; then
        printf 'the restart rule has no sub-bullet whose bold lead names the first session, so nothing starts supervise.sh'
        return
    fi
    case $bullet in *'tools/run/supervise.sh'*) ;; *) printf ' the handover does not name tools/run/supervise.sh' ;; esac
    printf '%s\n' "$bullet" | grep -qE 'first session[^.]* start' || printf ' the handover does not say the first session starts it'
    case $bullet in *HW_SUPERVISE_TREE*) ;; *) printf ' the handover does not name HW_SUPERVISE_TREE' ;; esac
    printf '%s\n' "$bullet" | grep -qE 'drains? to zero' || printf ' the handover does not drain to zero first'
    # The steps, not only the words: the lead states the starter, the loop runs
    # in the background, the first session fills no slot, and it wakes on the
    # loop's stop line. A foreground loop blocks the first session on the whole
    # run, and a first session that fills slots is a second live parent.
    lead=${bullet#  - \*\*}
    lead=${lead%%\*\**}
    case $lead in *'first session starts the loop'*) ;; *) printf ' the bold lead does not say the first session starts the loop: %s' "$lead" ;; esac
    case $bullet in *'run_in_background: true'*) ;; *) printf ' the handover does not start the loop with run_in_background: true' ;; esac
    case $bullet in *foreground*) printf ' the handover runs the loop in the foreground' ;; esac
    case $bullet in *'fills no slot'*) ;; *) printf ' the handover does not say the first session fills no slot' ;; esac
    case $bullet in *'fills slots'*) printf ' the handover has the first session fill slots' ;; esac
    case $bullet in *'STOPPED:'*) ;; *) printf ' the handover does not wake on the STOPPED: line' ;; esac
}
why=$(handover_misses "$skills/hw-run-policy/SKILL.md")
if [ -z "$why" ]; then
    pass 'the first session hands the run to tools/run/supervise.sh'
else
    fail 'the first session hands the run to tools/run/supervise.sh' "$why"
fi
# The arms, each a restart rule in a scratch file: the rule as it read before
# #1275, which names no starter, and a handover that drops the drain to zero.
handover_arm() {
    printf -- '- **A parent restarts every few merges, and drains to zero first.** `tools/run/supervise.sh` runs each session after the first.\n%s' "$1" > "$scratch/restart.md"
    handover_misses "$scratch/restart.md"
}
case $(handover_arm '') in
    *'nothing starts supervise.sh'*) pass 'and a restart rule that names no starter is reported' ;;
    *) fail 'a restart rule that names no starter is reported' 'the handover passed it' ;;
esac
case $(handover_arm '  - **The first session hands the run over.** With the rulings recorded, the first session starts `HW_SUPERVISE_TREE=<tree> sh tools/run/supervise.sh <run-id>` in the background.
') in
    *'does not drain to zero first'*) pass 'and a handover that does not drain to zero first is reported' ;;
    *) fail 'a handover that does not drain to zero first is reported' 'the handover passed it' ;;
esac
case $(handover_arm '  - **The first session hands the run over.** It drains to zero, then starts `sh tools/run/supervise.sh <run-id>` in the background.
') in
    *'does not name HW_SUPERVISE_TREE'*) pass 'and a handover that names no tree is reported' ;;
    *) fail 'a handover that names no tree is reported' 'the handover passed it' ;;
esac
case $(handover_arm '  - **The first session hands the run over.** It drains to zero, then starts `HW_SUPERVISE_TREE=<tree> sh supervise <run-id>` in the background.
') in
    *'does not name tools/run/supervise.sh'*) pass 'and a handover that does not name the script is reported' ;;
    *) fail 'a handover that does not name the script is reported' 'the handover passed it' ;;
esac
case $(handover_arm '  - **The first session waits.** A person starts `HW_SUPERVISE_TREE=<tree> sh tools/run/supervise.sh <run-id>` after the first session drains to zero.
') in
    *'does not say the first session starts it'*) pass 'and a handover that a person starts is reported' ;;
    *) fail 'a handover that a person starts is reported' 'the handover passed it' ;;
esac
# The verifier's mutants of round 1: a lead that reverses the step, a loop in
# the foreground, and a first session that fills slots, each beside a body that
# otherwise holds; and a handover that never wakes on the stop line.
handover_good='It drains to zero, then starts `HW_SUPERVISE_TREE=<tree> sh tools/run/supervise.sh <run-id>` with `run_in_background: true`. Then the first session starts nothing else: it fills no slot, ends its turn, and wakes on the `STOPPED: <reason>` line.'
case $(handover_arm "  - **The first session starts the loop, and nothing else does.** $handover_good
") in
    '') pass 'and a handover that holds every step passes' ;;
    *) fail 'a handover that holds every step passes' "reported: \`$(handover_arm "  - **The first session starts the loop, and nothing else does.** $handover_good
")\`" ;;
esac
case $(handover_arm "  - **The first session may skip the loop.** $handover_good
") in
    *'bold lead does not say'*) pass 'and a lead that lets the first session skip the loop is reported' ;;
    *) fail 'a lead that lets the first session skip the loop is reported' 'the handover passed it' ;;
esac
case $(handover_arm "  - **The first session starts the loop, and nothing else does.** $(printf '%s' "$handover_good" | sed 's/with `run_in_background: true`/in the foreground/')
") in
    *'in the foreground'*) pass 'and a loop started in the foreground is reported' ;;
    *) fail 'a loop started in the foreground is reported' 'the handover passed it' ;;
esac
case $(handover_arm "  - **The first session starts the loop, and nothing else does.** $(printf '%s' "$handover_good" | sed 's/it fills no slot/it fills slots itself/')
") in
    *'fill slots'*) pass 'and a first session that fills slots is reported' ;;
    *) fail 'a first session that fills slots is reported' 'the handover passed it' ;;
esac
case $(handover_arm "  - **The first session starts the loop, and nothing else does.** $(printf '%s' "$handover_good" | sed 's/, and wakes on the `STOPPED: <reason>` line//')
") in
    *'STOPPED: line'*) pass 'and a handover that never wakes on the stop line is reported' ;;
    *) fail 'a handover that never wakes on the stop line is reported' 'the handover passed it' ;;
esac
# A handover sub-bullet under another rule is not the restart rule's.
printf -- '- **A parent restarts every few merges, and drains to zero first.** `tools/run/supervise.sh` runs each session after the first.\n- **Another rule.** Its text.\n  - **The first session hands the run over.** It drains to zero, then starts `HW_SUPERVISE_TREE=<tree> sh tools/run/supervise.sh <run-id>`.\n' > "$scratch/restart-other.md"
case $(handover_misses "$scratch/restart-other.md") in
    *'nothing starts supervise.sh'*) pass 'and a handover under another rule is reported' ;;
    *) fail 'a handover under another rule is reported' 'the handover passed it' ;;
esac

printf '\n%s passed, %s failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
