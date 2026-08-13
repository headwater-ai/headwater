#!/bin/sh
# What holds the skills and the maintainer agent of this repository.
#
# A skill is prose that an agent reads. Nothing makes a model follow it, and
# spec 5 says so rather than pretending otherwise. What a suite can hold is the
# half that is a claim about this engine: a verb that a skill tells an agent to
# run, a refusal a skill says the verb makes, a rule a skill names, a path a
# skill cites, and a sentence a skill has to keep saying for its fixture to
# still be about anything.
#
# So there are two kinds of case below.
#
#   derived   read out of the skill files themselves and held against what the
#             engine reports. Nothing here lists what to check, so a skill that
#             names a rule, a verb or a path that stops existing fails with no
#             edit to this file.
#   claimed   a sentence that must be in a named skill, and a command whose
#             result must be what that sentence says. Both halves, so the prose
#             and the engine cannot drift apart without one of them failing.
#
# Run it from anywhere:
#     sh .claude/skills/fixtures.sh
#
# It needs a built engine for the cases that call one, and it says which cases
# it skipped when there is none. It writes only into a scratch copy of the
# corpus under the temporary directory, and it removes that copy on exit.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
skills="$root/.claude/skills"
agents="$root/.claude/agents"
engine="$root/engine/target/release/headwater"

passed=0
failed=0
skipped=0

pass() {
    printf 'ok   %s\n' "$1"
    passed=$((passed + 1))
}

fail() {
    printf 'FAIL %s\n  %s\n' "$1" "$2"
    failed=$((failed + 1))
}

skip() {
    printf 'skip %s (%s)\n' "$1" "$2"
    skipped=$((skipped + 1))
}

# Every SKILL.md, and the agent files beside them. The list is the tree rather
# than a constant, so a skill added with no thought about this suite still
# reaches every derived case below.
skill_files=$(find "$skills" -name SKILL.md | sort)
agent_files=$(find "$agents" -name '*.md' 2>/dev/null | sort)
instruction_files="$skill_files $agent_files"

# Every backticked span of a file, one per line, with the backticks removed.
spans() {
    grep -oE '`[^`]+`' "$1" 2>/dev/null | tr -d '`'
}

# --- structure --------------------------------------------------------------

printf '# every skill and agent declares what a harness reads\n'
for file in $instruction_files; do
    dir=$(dirname "$file")
    case $file in
        */SKILL.md) expected=$(basename "$dir") ;;
        *) expected=$(basename "$file" .md) ;;
    esac
    declared=$(sed -n 's/^name: *//p' "$file" | head -1)
    if [ "$declared" = "$expected" ]; then
        pass "$expected declares the name a harness addresses it by"
    else
        fail "$expected declares the name a harness addresses it by" \
            "front matter says \`$declared\`"
    fi

    # The description is the whole mechanism by which a skill loads, so an
    # empty one is a skill nobody can reach. The band is spec 5's length
    # measure at the one placement no check can see.
    description=$(sed -n 's/^description: *//p' "$file" | head -1)
    length=$(printf '%s' "$description" | wc -c | tr -d ' ')
    if [ "$length" -ge 80 ] && [ "$length" -le 700 ]; then
        pass "$expected carries a description inside the length band"
    else
        fail "$expected carries a description inside the length band" \
            "it is $length characters, and the band is 80 to 700"
    fi
done

# --- derived: every path a skill cites is a path this tree holds -------------

printf '\n# every repository path a skill cites exists\n'
missing=''
counted=0
for file in $instruction_files; do
    for span in $(spans "$file"); do
        case $span in
            docs/*|.claude/*|.headwater/*|packages/*|engine/*) ;;
            *) continue ;;
        esac
        # A shelf pattern and a directory both name a directory.
        path=${span%\*\*}
        path=${path%/}
        counted=$((counted + 1))
        [ -e "$root/$path" ] || missing="$missing $(basename "$file"):$span"
    done
done
if [ -z "$missing" ]; then
    pass "$counted cited paths, and every one of them is on this tree"
else
    fail 'every cited path is on this tree' "these are not:$missing"
fi

printf '\n# every link a skill writes resolves\n'
missing=''
counted=0
for file in $instruction_files; do
    dir=$(dirname "$file")
    for link in $(grep -oE '\]\([^)#]+\)' "$file" 2>/dev/null | sed 's/^](//; s/)$//'); do
        case $link in
            http*) continue ;;
        esac
        counted=$((counted + 1))
        [ -e "$dir/$link" ] || missing="$missing $(basename "$file"):$link"
    done
done
if [ -z "$missing" ]; then
    pass "$counted links, and every one of them resolves"
else
    fail 'every link resolves' "these do not:$missing"
fi

# --- derived: every rule and verb a skill names is one the engine carries ----

if [ -x "$engine" ]; then
    printf '\n# every rule a skill names is a rule this engine runs\n'
    rules=$("$engine" check --format json --root "$root" 2>/dev/null |
        tr ',' '\n' | grep -oE '"[a-z_]+\.[a-z_.]+"' | tr -d '"' | sort -u)
    missing=''
    counted=0
    for file in $instruction_files; do
        for span in $(spans "$file"); do
            # A dotted lower-case token with no slash and no file extension is
            # a rule name. Everything else in a code span is something else.
            case $span in
                */*|*' '*|*:*) continue ;;
                *.md|*.yml|*.json|*.lock|*.sh|*.rs|*.toml|*.txt|*.py) continue ;;
            esac
            printf '%s' "$span" | grep -qE '^[a-z_]+\.[a-z_.]+$' || continue
            counted=$((counted + 1))
            printf '%s\n' "$rules" | grep -qx "$span" ||
                missing="$missing $(basename "$file"):$span"
        done
    done
    if [ -z "$missing" ]; then
        pass "$counted rule names, and this engine runs every one of them"
    else
        fail 'every rule a skill names is one this engine runs' "these are not:$missing"
    fi

    printf '\n# every verb a skill tells an agent to run is a verb that ships\n'
    verbs=$("$engine" --help 2>&1 | grep -oE '^headwater [a-z]+' | awk '{print $2}' | sort -u)
    missing=''
    counted=0
    for file in $instruction_files; do
        for verb in $(grep -oE 'headwater [a-z]+' "$file" | awk '{print $2}' | sort -u); do
            counted=$((counted + 1))
            printf '%s\n' "$verbs" | grep -qx "$verb" ||
                missing="$missing $(basename "$file"):headwater $verb"
        done
    done
    if [ -z "$missing" ]; then
        pass "$counted verbs named, and every one of them ships"
    else
        fail 'every verb a skill names ships' "these do not:$missing"
    fi
else
    skip 'the derived cases that read the engine' 'no built engine'
fi

# --- claimed: a sentence in a skill, and the engine behaving as it says ------

# Assert that a file holds a sentence, then run a command and hold its exit
# status and its output against what the sentence says.
claim() {
    name=$1 file=$2 sentence=$3 status=$4 substring=$5
    shift 5
    if ! grep -qF "$sentence" "$skills/$file" 2>/dev/null; then
        fail "$name" "\`$file\` no longer says: $sentence"
        return
    fi
    out=$("$@" 2>&1)
    got=$?
    if [ "$got" -ne "$status" ]; then
        fail "$name" "expected exit $status, got $got:
$out"
        return
    fi
    case $out in
        *"$substring"*) pass "$name" ;;
        *) fail "$name" "expected the output to hold \`$substring\`, got:
$out" ;;
    esac
}

if [ -x "$engine" ]; then
    printf '\n# headwater-authoring, against the verb it calls\n'

    # A scratch copy, because three of the cases below write a document. The
    # committed corpus is never the tree a fixture writes into.
    scratch=$(mktemp -d "${TMPDIR:-/tmp}/headwater-skills-XXXXXX")
    trap 'rm -rf "$scratch"' EXIT INT TERM
    cp -r "$root/docs" "$scratch/docs"
    cp -r "$root/.headwater" "$scratch/.headwater"

    claim 'the kind list the skill prints is the kind list the verb admits' \
        headwater-authoring/SKILL.md \
        '`decision`, `decision_register`, `design_spec`, `evaluation`, `obligation_record`, `obligation_register`, `review_prompt`, `review_record` and `specification`' \
        1 'no kind is named `nonesuch`' \
        "$engine" new nonesuch --title 'A kind nobody declared' --root "$scratch"

    claim 'a kind that a relation may name and that mints nothing is refused' \
        headwater-authoring/SKILL.md \
        'the kind names no identifier scheme and a relation may name a document of it' \
        1 'names no identifier scheme' \
        "$engine" new specification --title 'A specification nobody can name' --root "$scratch"

    claim 'an edge the taxonomy does not assign to a scaffold is refused' \
        headwater-authoring/SKILL.md \
        'The verb writes an edge only where the taxonomy declares `created_by: scaffold` on the relation.' \
        1 'created_by: hook' \
        "$engine" new obligation_record --title 'An edge a hook pays for' \
        --relates traces_to=SPEC-HW-ai-integration --root "$scratch"

    # A shelf with no layout, because a layout that carries the identifier
    # sequence mints a fresh number on every run and so never collides. The
    # evaluations shelf declares none, so a second run at one title lands on the
    # file the first run wrote.
    claim 'the verb never overwrites a document that is already there' \
        headwater-authoring/SKILL.md \
        'The verb never overwrites.' \
        1 'never overwrites a document' \
        "$engine" new evaluation --title 'First contact' --root "$scratch"

    printf '\n# headwater-taxonomy, against the taxonomy it edits\n'

    claim 'the committed lock is what the sources resolve to' \
        headwater-taxonomy/SKILL.md \
        '`headwater check` reads the lock and never the sources.' \
        0 'is what the sources resolve to' \
        "$engine" taxonomy resolve --check --root "$root"

    claim 'the validator reports what it did not decide' \
        headwater-taxonomy/SKILL.md \
        'Read the `not decided` lines' \
        0 'not decided' \
        "$engine" taxonomy validate --root "$root"

    # The layout claim, asserted on the name the verb writes rather than on a
    # substring, because the number is the whole of what the declaration buys.
    sentence='`{seq:04d}-{slug}.md` reads the sequence of the identifier that the run mints'
    if ! grep -qF "$sentence" "$skills/headwater-taxonomy/SKILL.md"; then
        fail 'a numbered shelf names its file from the identifier' \
            "the skill no longer says: $sentence"
    else
        written=$("$engine" new obligation_record --title 'A fixture for the skill suite' \
            --now 2026-08-14 --root "$scratch" 2>&1 | sed -n 's/^wrote //p')
        case $written in
            docs/obligations/[0-9][0-9][0-9][0-9]-a-fixture-for-the-skill-suite.md)
                pass 'a numbered shelf names its file from the identifier' ;;
            *)
                fail 'a numbered shelf names its file from the identifier' \
                    "the verb wrote \`$written\`" ;;
        esac
    fi

    printf '\n# headwater-maintainer, against the hook it invokes\n'

    # The agent tells its reader to drive the write hook by hand, one path at a
    # time. If that invocation stops working the agent's third part is empty and
    # nothing else reports it.
    sentence='hook_event_name":"PostToolUse'
    if ! grep -qF "$sentence" "$agents/headwater-maintainer.md"; then
        fail 'the impact invocation the agent prints answers' \
            'the agent no longer names the PostToolUse position'
    else
        export HEADWATER_HOOK_ROOT="$root"
        out=$(printf '{"hook_event_name":"PostToolUse","tool_name":"Edit","tool_input":{"file_path":".claude/hooks/write.sh"}}' |
            sh "$root/.claude/hooks/write.sh" 2>&1)
        case $out in
            *docs/spec/05-ai-integration.md*)
                pass 'the impact invocation the agent prints answers' ;;
            *)
                fail 'the impact invocation the agent prints answers' \
                    "it named no governing document:
$out" ;;
        esac
    fi

    rm -rf "$scratch"
    trap - EXIT INT TERM
else
    skip 'every claimed case' 'no built engine'
fi

printf '\n%s passed, %s failed, %s skipped\n' "$passed" "$failed" "$skipped"
[ "$failed" -eq 0 ] || exit 1
[ "$skipped" -eq 0 ] || printf 'Build the engine to run the skipped cases:\n  cargo build --release -p headwater-cli --manifest-path engine/Cargo.toml\n'
exit 0
