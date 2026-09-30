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

# Either profile builds the engine whose behavior the skills are checked
# against, and the newer answers. A worktree with only the cheap binary skipped
# every case that calls one, and said so in a line nothing reads.
release_engine="$root/engine/target/release/headwater"
dev_release_engine="$root/engine/target/dev-release/headwater"
engine=$release_engine
if [ -x "$dev_release_engine" ] && { [ ! -x "$engine" ] || [ "$dev_release_engine" -nt "$engine" ]; }; then
    engine=$dev_release_engine
fi

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
            docs/*|.claude/*|.headwater/*|engine/*) ;;
            *) continue ;;
        esac
        # A shelf pattern and a directory both name a directory.
        path=${span%\*\*}
        path=${path%/}
        counted=$((counted + 1))
        [ -e "$root/$path" ] && continue
        # A path the tree ignores, such as `.claude/worktrees/`, is made at
        # run time and is on no clean checkout, CI's included. A skill may
        # cite it, and nothing here can check it. The trailing slash lets a
        # directory pattern match a directory that is not there.
        git -C "$root" check-ignore -q --no-index "$path" 2>/dev/null && continue
        git -C "$root" check-ignore -q --no-index "$path/" 2>/dev/null && continue
        missing="$missing $(basename "$file"):$span"
    done
done
if [ -z "$missing" ]; then
    pass "$counted cited paths, and every one of them is on this tree or ignored by it"
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

# --- derived: every verb a skill names is a verb the binary dispatches -------

# `docs/interfaces/README.md` is the `verb_index` projection: one row per entry
# of `headwater_verbs::VERBS`, which is the table `main` resolves a first word
# against before it enters any arm. A name in the first column is therefore a
# name this binary dispatches, and `headwater generate --check` holds the whole
# file against that table on every pull request, so a row written by hand and a
# verb added without regenerating both fail there.
#
# The set used to come from `headwater --help`, which made it a function of the
# rendered help text. Any left-aligned `headwater …` line added to the synopsis
# widened it, and a name nothing dispatches was then accepted as shipping, with
# no suite, hook or CI job reporting it. That is
# [#309](https://github.com/headwater-ai/headwater/issues/309), and
# `engine/crates/cli/tests/verbs.rs` closes the other half of it by holding the
# synopsis against the table in both directions.
#
# The old pipeline also read `2>&1`. This binary writes artifacts to standard
# output and run statistics to standard error, and merging the two on anything
# measured has already cost this repository a wrong entry in durable memory. The
# replacement opens a file and starts no process, so the question does not
# arise.
#
# Reading a committed artifact rather than a built binary is also what lets this
# case run in a clone that has never compiled the engine, which is why it sits
# above the block below rather than inside it.
index="$root/docs/interfaces/README.md"
verbs=$(sed -n 's/^| `\([a-z][a-z]*\)` |.*/\1/p' "$index" | sort -u)

printf '\n# the verb set this suite accepts comes from the generated verb index\n'
if ! grep -q '^<!-- headwater:generated verb_index\.' "$index"; then
    fail 'the verb set comes from the generated verb index' \
        "docs/interfaces/README.md no longer opens with the \`verb_index\` marker"
elif [ -z "$verbs" ]; then
    fail 'the verb set comes from the generated verb index' \
        'docs/interfaces/README.md holds no row this suite can read'
else
    pass "$(printf '%s\n' "$verbs" | wc -l | tr -d ' ') verbs, read from the verb index"
fi

printf '\n# every verb a skill tells an agent to run is a verb that ships\n'
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

# --- claimed: what git reports about a merged branch, and what the skill says -

# repo-cleanup decides merged by content because ancestry answers a different
# question. These cases are about git rather than about this engine, so they
# run with no built binary. Each one builds a scratch repository, reproduces
# one of the three causes the skill names, and holds the result against a
# sentence the skill has to keep saying.

printf '\n# repo-cleanup, against the git behavior it describes\n'

cleanup_skill="$skills/repo-cleanup/SKILL.md"

# Assert a sentence is still in the skill, then run a scenario builder and hold
# its verdict. Both halves, so the prose and git cannot drift apart.
git_claim() {
    name=$1 sentence=$2 expected=$3 builder=$4
    if ! grep -qF "$sentence" "$cleanup_skill" 2>/dev/null; then
        fail "$name" "repo-cleanup no longer says: $sentence"
        return
    fi
    got=$($builder 2>&1 | tail -1)
    if [ "$got" = "$expected" ]; then
        pass "$name"
    else
        fail "$name" "expected \`$expected\`, got \`$got\`"
    fi
}

# A throwaway repository with one commit on main, and the identity set locally
# so the suite never reads the ambient git config.
new_repo() {
    d=$(mktemp -d "${TMPDIR:-/tmp}/headwater-cleanup-XXXXXX")
    git -C "$d" init -q -b main
    git -C "$d" config user.email fixture@example.invalid
    git -C "$d" config user.name Fixture
    git -C "$d" config commit.gpgsign false
    echo base > "$d/file.txt"
    git -C "$d" add file.txt
    git -C "$d" commit -qm 'Base'
    printf '%s' "$d"
}

# Squash merge: the work of the branch is on main, and no commit of the branch
# is an ancestor of main.
case_squash() {
    d=$(new_repo)
    git -C "$d" checkout -qb topic
    echo work > "$d/file.txt"
    git -C "$d" commit -qam 'Do the work'
    git -C "$d" checkout -q main
    git -C "$d" merge -q --squash topic
    git -C "$d" commit -qm 'Do the work (#1)'
    refused=no
    git -C "$d" branch -d topic >/dev/null 2>&1 || refused=yes
    identical=no
    [ -z "$(git -C "$d" diff main topic)" ] && identical=yes
    rm -rf "$d"
    echo "refused=$refused identical=$identical"
}

# A history rewritten after the merge. Main carries the same tree and the
# branch is no longer reachable from it.
case_rewrite() {
    d=$(new_repo)
    git -C "$d" checkout -qb topic
    echo work > "$d/file.txt"
    git -C "$d" commit -qam 'Do the work'
    git -C "$d" checkout -q main
    git -C "$d" merge -q --ff-only topic
    git -C "$d" commit -q --amend -m 'Do the work, rewritten'
    refused=no
    git -C "$d" branch -d topic >/dev/null 2>&1 || refused=yes
    identical=no
    [ -z "$(git -C "$d" diff main topic)" ] && identical=yes
    rm -rf "$d"
    echo "refused=$refused identical=$identical"
}

# The branch is merged into main by ancestry, and the session sits on another
# branch. -d measures against HEAD and refuses anyway.
case_other_head() {
    d=$(new_repo)
    git -C "$d" checkout -qb topic
    echo work > "$d/file.txt"
    git -C "$d" commit -qam 'Do the work'
    git -C "$d" checkout -q main
    git -C "$d" merge -q --ff-only topic
    git -C "$d" checkout -q -b other main~1
    refused=no
    git -C "$d" branch -d topic >/dev/null 2>&1 || refused=yes
    ancestor=no
    git -C "$d" merge-base --is-ancestor topic main && ancestor=yes
    rm -rf "$d"
    echo "refused=$refused ancestor=$ancestor"
}

if ! command -v git >/dev/null 2>&1; then
    skip 'the repo-cleanup git cases' 'no git on the path'
else
    git_claim 'a squash-merged branch is identical to main and -d refuses it' \
        'Under a squash merge the forge writes one new commit on `origin/main`, and no commit of the branch is an ancestor of it.' \
        'refused=yes identical=yes' case_squash

    git_claim 'a rewritten history leaves a merged branch that -d refuses' \
        'A history rewritten after the merge leaves the branch unreachable from `origin/main` although its work landed.' \
        'refused=yes identical=yes' case_rewrite

    git_claim '-d refuses a branch merged into main while HEAD is elsewhere' \
        'It never measures against `origin/main` unless `origin/main` is the branch checked out.' \
        'refused=yes ancestor=yes' case_other_head
fi

# --- derived: every rule a skill names is one this engine runs ---------------

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

# --- headwater-authoring, the conventions an obligation record keeps ----------

# A sentence claim with no engine behavior behind it: no rule reads a line
# reference or a gap count in a record, and a check that did would be a new rule
# with its own issue (#1486). So this half holds only that the skill still says
# each convention, and it runs no command.
printf '\n# headwater-authoring, the conventions an obligation record keeps\n'
for sentence in \
    'The Discharge section states the closing condition, what must become true, and not a description of the present state.' \
    'A record names a function, a job key or another stable symbol, never a line number or a count' \
    'A record that bundles gaps is split when it passes five gaps.'
do
    name="the authoring skill states an obligation-record convention: $sentence"
    if grep -qF "$sentence" "$skills/headwater-authoring/SKILL.md"; then
        pass "$name"
    else
        fail "$name" "the skill no longer says: $sentence"
    fi
done

if [ -x "$engine" ]; then
    printf '\n# headwater-authoring, against the verb it calls\n'

    # A scratch copy, because three of the cases below write a document. The
    # committed corpus is never the tree a fixture writes into.
    scratch=$(mktemp -d "${TMPDIR:-/tmp}/headwater-skills-XXXXXX")
    trap 'rm -rf "$scratch"' EXIT INT TERM
    cp -r "$root/docs" "$scratch/docs"
    cp -r "$root/.headwater" "$scratch/.headwater"

    # The kind list, read out of both sides rather than carried here.
    #
    # This case was a `claim` with the list written into this file, so it held
    # two texts together and neither of them against the engine. Both were the
    # same wrong list: the taxonomy declared `probe`, `probe_result` and
    # `probe_transcript`, and the skill named none of the three, and the case
    # passed on every run. That is the defect #211 is about, in this suite.
    #
    # The refusal message is the only surface that prints the admitted kinds, so
    # it is the derivation. `sort` on both sides, because neither the verb nor
    # the sentence promises an order.
    refused=$("$engine" new nonesuch --title 'A kind nobody declared' --root "$scratch" 2>&1)
    if [ $? -eq 1 ] && [ "${refused#*no kind is named}" != "$refused" ]; then
        pass 'an unknown kind is refused, and the refusal names the kinds there are'
    else
        fail 'an unknown kind is refused, and the refusal names the kinds there are' \
            "$refused"
    fi

    admits=$(printf '%s\n' "$refused" |
        sed -n 's/.*This taxonomy declares //p' |
        tr -d '`' | tr ',' '\n' | sed 's/^ *//; s/ *$//' | sort)
    names=$(sed -n 's/.*The concrete kinds are \(.*\)\./\1/p' \
        "$skills/headwater-authoring/SKILL.md" |
        sed 's/ and /, /' | tr -d '`' | tr ',' '\n' | sed 's/^ *//; s/ *$//' | sort)
    if [ -n "$admits" ] && [ "$admits" = "$names" ]; then
        pass 'the kind list the skill prints is the kind list the verb admits'
    else
        fail 'the kind list the skill prints is the kind list the verb admits' \
            "the verb admits:
$admits
the skill names:
$names"
    fi

    claim 'a kind that a relation may name and that mints nothing is refused' \
        headwater-authoring/SKILL.md \
        'the kind names no identifier scheme and a relation may name a document of it' \
        1 'names no identifier scheme' \
        "$engine" new specification --title 'A specification nobody can name' --root "$scratch"

    claim 'an edge the taxonomy does not assign to a scaffold is refused' \
        headwater-authoring/SKILL.md \
        'The verb writes an edge only where the taxonomy declares `created_by: scaffold` on the relation.' \
        1 'created_by: agent' \
        "$engine" new obligation_record --title 'An edge an agent proposes' \
        --facet waiting_on=build \
        --relates traces_to=HW-SPEC-ai-integration --root "$scratch"

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
        '`headwater check` takes its declarations from the lock and never from the sources.' \
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
            --facet waiting_on=build \
            --now 2026-08-14 --root "$scratch" 2>&1 | sed -n 's/^wrote //p')
        case $written in
            docs/obligations/[0-9][0-9][0-9][0-9]-a-fixture-for-the-skill-suite.md)
                pass 'a numbered shelf names its file from the identifier' ;;
            *)
                fail 'a numbered shelf names its file from the identifier' \
                    "the verb wrote \`$written\`" ;;
        esac
    fi

    printf '\n# headwater-authoring, against the store the verb writes\n'

    # A second scratch with no store at all, because this case counts readings
    # and the committed store already holds some. The grain of the assertions is
    # the numerator and the direction of the denominator, never the corpus size:
    # a fixture that pinned the document count would move on every document this
    # repository adds, for a reason that has nothing to do with the claim.
    sentence='A document you write by any other route carries no reading at all'
    name='a document written by another route lowers the reach and never raises it'
    if ! grep -qF "$sentence" "$skills/headwater-authoring/SKILL.md"; then
        fail "$name" "the skill no longer says: $sentence"
    else
        clean=$(mktemp -d "${TMPDIR:-/tmp}/headwater-capture-XXXXXX")
        cp -r "$root/docs" "$clean/docs"
        cp -r "$root/.headwater" "$clean/.headwater"
        rm -f "$clean/.headwater/capture-cost.jsonl"
        rm -rf "$clean/.headwater/cache"

        "$engine" new obligation_record --title 'A record the store watched' \
            --facet waiting_on=build \
            --now 2026-08-14 --root "$clean" >/dev/null 2>&1
        scaffolded=$("$engine" capture --root "$clean" |
            sed -n 's/^  \([0-9]*\) of \([0-9]*\) classified documents carry a reading$/\1 \2/p')

        # The same document shape, written by no verb at all.
        printf -- '---\nid: HW-OBL-9998\ntitle: "A record no verb wrote"\nstatus: current\nstatus_since: 2026-08-14\nwaiting_on: build\nlast_verified: 2026-08-14\nsummary: "Written by no verb, to hold the reach figure against a route the store does not watch."\n---\n\n# A record no verb wrote\n\n## Context\n\nNone.\n\n## Obligation\n\nNone.\n\n## Discharge\n\nNone.\n' \
            > "$clean/docs/obligations/9998-a-record-no-verb-wrote.md"
        by_hand=$("$engine" capture --root "$clean" |
            sed -n 's/^  \([0-9]*\) of \([0-9]*\) classified documents carry a reading$/\1 \2/p')
        rm -rf "$clean"

        set -- $scaffolded
        was_reached=${1:-0} was_total=${2:-0}
        set -- $by_hand
        now_reached=${1:-0} now_total=${2:-0}
        if [ "$was_reached" = "1" ] && [ "$now_reached" = "1" ] &&
            [ "$now_total" -eq $((was_total + 1)) ]; then
            pass "$name"
        else
            fail "$name" \
                "scaffolded read $was_reached of $was_total, and after a hand-written document \
it read $now_reached of $now_total. The numerator must hold at 1 and the denominator must rise by 1"
        fi
    fi

    printf '\n# headwater-engine, against the flag that makes a verb runnable from anywhere\n'

    # The skill tells an agent to pass `--root` rather than to put a `cd` in
    # front of every command. That advice is worth nothing unless the same verb
    # fails without the flag, so both halves run from a directory that holds no
    # corpus, and neither half runs from this tree.
    sentence='**Every verb takes `--root`.**'
    name='a verb answers from another directory with --root, and refuses without it'
    if ! grep -qF "$sentence" "$skills/headwater-engine/SKILL.md"; then
        fail "$name" "the skill no longer says: $sentence"
    else
        elsewhere=$(mktemp -d "${TMPDIR:-/tmp}/headwater-elsewhere-XXXXXX")
        (cd "$elsewhere" && "$engine" explain docs/spec/05-ai-integration.md \
            --root "$root" >/dev/null 2>&1)
        rooted=$?
        (cd "$elsewhere" && "$engine" explain docs/spec/05-ai-integration.md \
            >/dev/null 2>&1)
        bare=$?
        rmdir "$elsewhere"
        if [ "$rooted" -eq 0 ] && [ "$bare" -ne 0 ]; then
            pass "$name"
        else
            fail "$name" \
                "with --root it exited $rooted and without it $bare, so the flag \
is not what reaches the corpus"
        fi
    fi

    printf '\n# the status probe, against the route and the recorder it meets\n'

    # The status of a settled decision in its pull request (#1294). Route on
    # the probe's own task, in a tree without the probe shelves that every arm
    # removes, offers the decision that rules it at the default budget. The
    # task is read out of the probe the way `tools/probe/campaign.sh` reads it,
    # so a reworded task is the task this case routes. The path is matched as
    # an offered pointer, at the start of a line, and not as a word in the
    # summary of some other document.
    name='route on the status probe offers the ruling at the default budget'
    status_probe="$root/docs/probes/a-session-names-the-status-a-settled-decision-carries-in-its-pull-request.md"
    status_task=$(awk '/^## Task$/ { on = 1; next } on && /^## / { exit } on { print }' "$status_probe")
    mkdir -p "$scratch/arm"
    cp -r "$root/docs" "$scratch/arm/docs"
    cp -r "$root/.headwater" "$scratch/arm/.headwater"
    rm -rf "$scratch/arm/docs/probes" "$scratch/arm/docs/probe-runs" \
        "$scratch/arm/docs/probe-results" "$scratch/arm/.headwater/export.json"
    routed=$("$engine" route --root "$scratch/arm" "$status_task" 2>/dev/null)
    got=$?
    if [ "$got" -ne 0 ]; then
        fail "$name" "route exited $got"
    elif printf '%s\n' "$routed" | grep -q '^  docs/decisions/0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it\.md '; then
        pass "$name"
    else
        fail "$name" "route offered no pointer to HW-DR-0052:
$routed"
    fi

    # The same probe, answered from a prior alone. On 2026-09-28 the two absent
    # arms answered the value `current` in 30 of 30 and 28 of 30 sessions,
    # none of them with a tool call, so a closed set
    # that the value alone satisfies measured the model and not the documents.
    # The set the plan prints must record the bare value as no answer, and the
    # value with the ruling's identifier as that line.
    name='the status probe records the value alone as no answer, and the value with the ruling as itself'
    status_answers=$("$engine" probe plan --root "$root" --tier documentation \
        --category sufficiency --repetitions 1 2>/dev/null \
        | awk '/^- HW-PROBE-a-session-names-the-status-a-settled-decision/ { on = 1; next }
               /^- / { on = 0 }
               on && /^    answers: / { sub(/^    answers: /, ""); print; exit }')
    printf '%s\n' '{"type":"result","result":"current"}' > "$scratch/bare.jsonl"
    printf '%s\n' '{"type":"result","result":"current HW-DR-0052"}' > "$scratch/ruled.jsonl"
    bare=$(sh "$root/tools/probe/probe-record.sh" --answer-only "$scratch/bare.jsonl" --answers "$status_answers")
    ruled=$(sh "$root/tools/probe/probe-record.sh" --answer-only "$scratch/ruled.jsonl" --answers "$status_answers")
    if [ -z "$status_answers" ]; then
        fail "$name" 'the plan printed no answers for the status probe'
    elif ! printf '%s' "$status_task" | grep -qF 'the identifier in its `id` field'; then
        # A set that wants the identifier, under a task that never asks for
        # one, fails every session whatever the documents say.
        fail "$name" 'the task no longer asks for the identifier that the answer set requires'
    elif [ -n "$bare" ]; then
        fail "$name" "the bare value is recorded as \`$bare\` against the set \`$status_answers\`"
    elif [ "$ruled" != 'current HW-DR-0052' ]; then
        fail "$name" "the value with the ruling is recorded as \`$ruled\` against the set \`$status_answers\`"
    else
        pass "$name"
    fi

    printf '\n# headwater-orient, against the two verbs it sends an agent to\n'

    # `explain` is offered as the thing that answers without reading the
    # document, and the two lines that claim rests on are the summary and the
    # edges. A verb that stopped printing either would leave the skill
    # recommending a command that no longer orients anybody.
    sentence='the **summary** says what the document is for, in its author'
    name='explain prints the summary and the edges that orientation reads'
    if ! grep -qF "$sentence" "$skills/headwater-orient/SKILL.md"; then
        fail "$name" "the skill no longer says: $sentence"
    else
        out=$("$engine" explain docs/spec/05-ai-integration.md --root "$root" 2>&1)
        case $out in
            *'  summary '*)
                case $out in
                    *' cited_by '*) pass "$name" ;;
                    *) fail "$name" 'it printed a summary and no edge into the document' ;;
                esac ;;
            *) fail "$name" "it printed no summary line:
$out" ;;
        esac
    fi

    # Silence is a result rather than a failure, and that is the sentence which
    # licenses a search when the route matched nothing.
    claim 'a route that matches nothing says so, and still exits 0' \
        headwater-orient/SKILL.md \
        'It reports honestly when nothing matched.' \
        0 'no declared purpose answers this task' \
        "$engine" route zzzqqqwww --root "$root"

    printf '\n# headwater-authoring, its description against the ruling it cites\n'

    # The status of a settled decision in its pull request (#1294). A skill
    # reaches a session through its description, which is always loaded, and
    # the body loads only once a session invokes the skill. So the ruling is
    # stated in the description, and the identifier it cites is the ruling.
    #
    # Every read is of the `description` key of the front matter alone:
    # `claim` searches the whole file, and the body already states the ruling,
    # so a sentence moved into the body would pass it. The front matter is the
    # block between the first two `---` lines, and its first `description:`
    # key is the one a harness loads. Leading blanks and a YAML quote are
    # stripped, as YAML strips them, before the test for Draft.
    #
    # The sentence is matched whole, from its subject to the identifier, and
    # after a sentence boundary, so a changed subject or an inserted `never`
    # fails it. The identifier is the one inside the sentence, and the ruling
    # is the first line `explain` prints, which is the path the identifier
    # resolves to. A match anywhere in the output passes any decision whose
    # edges reach 0052.
    name='the authoring description states the status ruling and does not open with Draft'
    description=$(awk 'NR == 1 && $0 == "---" { on = 1; next }
                       on && $0 == "---" { exit }
                       on && /^description:/ { sub(/^description:[ \t]*/, ""); sub(/^["'\'']/, ""); print; exit }' \
        "$skills/headwater-authoring/SKILL.md")
    sentence='. A settled document goes into its pull request at `status: current`, not `draft` ('
    ruling='docs/decisions/0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md'
    case $description in
        [Dd][Rr][Aa][Ff][Tt]*)
            fail "$name" "the description opens with Draft: $description" ;;
        *"$sentence"*)
            cited=${description#*"$sentence"}
            cited=${cited%%)*}
            resolved=$("$engine" explain "$cited" --root "$root" 2>/dev/null | head -n 1)
            if [ "$resolved" = "$ruling" ]; then
                pass "$name"
            else
                fail "$name" "the description cites \`$cited\`, which resolves to \`$resolved\` and not to $ruling"
            fi ;;
        *) fail "$name" "the description no longer says: $sentence<the ruling>): $description" ;;
    esac

    printf '\n# headwater-maintainer, against the hook it invokes\n'

    # The agent tells its reader to drive the write hook by hand, one path at a
    # time. If that invocation stops working the agent's third part is empty and
    # nothing else reports it.
    sentence='hook_event_name":"PreToolUse'
    if ! grep -qF "$sentence" "$agents/headwater-maintainer.md"; then
        fail 'the impact invocation the agent prints answers' \
            'the agent no longer names the PreToolUse position'
    else
        export HEADWATER_HOOK_ROOT="$root"
        out=$(printf '{"hook_event_name":"PreToolUse","tool_name":"Edit","tool_input":{"file_path":".claude/hooks/write.sh"}}' |
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

    # The same position, over a crate rather than over a script. This is the
    # half of the agent's first part that had nothing to report until an
    # `interface_contract` declared `governs` onto a crate: an edit to the code
    # a command is described by now names the description.
    #
    # Three assertions rather than one, and the second and third are why this
    # case is here at all.
    #
    #   the contract   the document is named. A case that stopped here would
    #                  pass on the path alone, and the path was already being
    #                  printed for every governed file before contracts existed.
    #   the verb       `(headwater check)` is the rendering of the facet in the
    #                  `name` role, and nothing else in this output writes a
    #                  parenthesized span. The bare substring `headwater check`
    #                  would not do: the contract's own summary holds it, so a
    #                  case asserting that would pass whether the name was read
    #                  or not.
    #   not silence    the empty output is the ambient result of an unbuilt
    #                  engine, an unreadable input and a path nothing governs,
    #                  so a case that could pass on it is a case that tests
    #                  none of the above. Both assertions above fail on it, and
    #                  this one says so where a reader of a failure will see it.
    sentence='answers `docs/interfaces/headwater-check.md (headwater check)`'
    name='an edit to a governed crate names the contract and the command it describes'
    if ! grep -qF "$sentence" "$agents/headwater-maintainer.md"; then
        fail "$name" 'the agent no longer says what a governed crate answers'
    else
        export HEADWATER_HOOK_ROOT="$root"
        out=$(printf '{"hook_event_name":"PreToolUse","tool_name":"Edit","tool_input":{"file_path":"engine/crates/check/src/lib.rs"}}' |
            sh "$root/.claude/hooks/write.sh" 2>/dev/null)
        if [ -z "$out" ]; then
            fail "$name" 'the hook said nothing at all'
        else
            case $out in
                *docs/interfaces/headwater-check.md*)
                    case $out in
                        *'(headwater check)'*) pass "$name" ;;
                        *) fail "$name" "it named the contract and not the command:
$out" ;;
                    esac ;;
                *)
                    fail "$name" "it named no contract:
$out" ;;
            esac
        fi
    fi

    printf '\n# headwater-sweep, against the promise that nothing gates on it\n'

    # The sweep is the one mechanism here whose output no engine produces, and
    # the promise it rests on is that no exit status ever reads that output.
    # Four cases hold it, one per thing that enforces it in spec 12. Three of
    # them read no skill sentence, because what they assert is a property of
    # this tree rather than a claim a skill makes.

    # A file the intake cannot use at all. It still exits 0: a non-zero status
    # here would be a build that a model's output can fail.
    printf 'this is not a mapping\n' > "$scratch/refused.yml"
    claim 'a return file this engine cannot use still exits 0' \
        headwater-sweep/SKILL.md \
        'exits 0 whatever it finds' \
        0 'reported nothing' \
        "$engine" sweep report "$scratch/refused.yml" --root "$root"

    # And a well-formed one that carries a finding, which exits with the same
    # status. The quotation is read out of the document rather than listed here,
    # so ordinary prose edits never rewrite this case.
    lock=$("$engine" sweep plan --under docs/spec --root "$root" |
        sed -n 's/^taxonomy: //p' | head -1)
    quote=$(sed -n 's/^# //p' "$root/docs/spec/12-check-layer.md" | head -1)
    printf 'taxonomy: %s\nslice: docs/spec\nfindings:\n  - class: undefined_concept\n    documents:\n      - docs/spec/12-check-layer.md\n    evidence:\n      - path: docs/spec/12-check-layer.md\n        quote: "%s"\n    message: A term of this document is defined nowhere in the slice.\n' \
        "$lock" "$quote" > "$scratch/carried.yml"
    claim 'a return file that carries a finding exits with the status of one that carries none' \
        headwater-sweep/SKILL.md \
        'Nothing gates on your output' \
        0 '1 carried, 0 refused' \
        "$engine" sweep report "$scratch/carried.yml" --root "$root"

    # The absence of a caller. A sweep runs when a person or a schedule asks,
    # and a grep is the whole of what says so.
    name='no gate and no CI job names the sweep'
    callers=''
    for file in "$root/.githooks/pre-commit" "$root/.github/workflows/ci.yml" \
        "$root/.claude/hooks/review.sh" "$root/.claude/hooks/write.sh" \
        "$root/.claude/hooks/intent.sh"; do
        grep -q 'headwater sweep\|sweep report\|sweep plan' "$file" 2>/dev/null &&
            callers="$callers $(basename "$file")"
    done
    if [ -z "$callers" ]; then
        pass "$name"
    else
        fail "$name" "these run it:$callers"
    fi

    # No socket. The middle part of a sweep needs a model and no crate the sweep
    # reaches can reach one, so an unreachable model is an absent file rather
    # than a failed build. `headwater-fetch` is the one crate with a client, for
    # `taxonomy vendor <location>`, and only the binary links it (HW-DR-0075,
    # held crate by crate in `engine/crates/cli/tests/network_boundary.rs`).
    name='no crate but headwater-fetch depends on a network client'
    reached=$(grep -lE '^(reqwest|hyper|ureq|curl|isahc|surf|attohttpc|tungstenite|native-tls|rustls|openssl) *=' \
        "$root"/engine/crates/*/Cargo.toml 2>/dev/null | grep -v '/engine/crates/fetch/Cargo.toml$')
    if [ -z "$reached" ]; then
        pass "$name"
    else
        fail "$name" "these manifests do: $reached"
    fi

    # The plan is deterministic, which is the only reproducibility a sweep
    # claims for itself. Two runs, over the tree as it stands.
    name='the briefing is the same bytes twice'
    "$engine" sweep plan --under docs/obligations --root "$root" > "$scratch/plan-a.txt" 2>/dev/null
    "$engine" sweep plan --under docs/obligations --root "$root" > "$scratch/plan-b.txt" 2>/dev/null
    if cmp -s "$scratch/plan-a.txt" "$scratch/plan-b.txt"; then
        pass "$name"
    else
        fail "$name" 'two runs over one tree wrote different bytes'
    fi

    printf '\n# headwater probe, against the same promise and one more\n'

    # The probe harness sits on the sweep's side of the line that keeps a model
    # out of every gate, and the four enforcements are the same four. Two of
    # them are already held above for the whole workspace — the grep for a
    # network client covers every manifest, and the crate cycle is the
    # compiler's. What is left is the caller and the exit status, plus the one
    # promise a sweep does not make: that no verb here writes a transcript.
    #
    # The third enforcement reads differently here than it does for the sweep,
    # and the assertion below says which half of it holds. No gate runs a probe.
    # One gate does reach the grader: a probe result is a projection over a
    # committed transcript, so `generate --check` grades in continuous
    # integration. What that gate compares is bytes against a derivation of
    # committed inputs, so a run in which the model answered every question
    # wrongly passes it. No exit status carries a model's behavior either way.

    name='no gate and no CI job runs a probe'
    callers=''
    for file in "$root/.githooks/pre-commit" "$root/.github/workflows/ci.yml" \
        "$root/.claude/hooks/review.sh" "$root/.claude/hooks/write.sh" \
        "$root/.claude/hooks/intent.sh"; do
        grep -q 'headwater probe\|probe plan\|probe record\|probe grade' "$file" 2>/dev/null &&
            callers="$callers $(basename "$file")"
    done
    if [ -z "$callers" ]; then
        pass "$name"
    else
        fail "$name" "these run it:$callers"
    fi

    # The transcript in this corpus exercises the probe-result projection.
    name='the probe-result projection accepts the recorded transcript'
    "$engine" generate --check --root "$root" > "$scratch/generate.txt" 2>&1
    if grep -q 'probe_result' "$scratch/generate.txt" &&
        grep -q 'regression-probe-transcript-for-2026-09-09' "$scratch/generate.txt"; then
        pass "$name"
    else
        fail "$name" 'the run does not name the transcript it generated'
    fi

    # A run this repository refuses to pay for. It exits 0, because an exit
    # status that carried a budget verdict would be a build a price list moves.
    claim 'a run over its ceiling is refused and still exits 0' \
        ../../.headwater/probe.yml \
        'is the cheaper error' \
        0 'does not start' \
        "$engine" probe plan --tier campaign --root "$root"

    # A transcript with a key outside the closed set. Spec 5 says the transcript
    # holds no model prose and that the omission is the enforcement, so the
    # failing arm is a file that carries some.
    printf '# A run\n\n## Run identity\n\n```yaml\nmodel: a\nreasoning: I read the governing document first.\n```\n\n## Events\n\n```yaml\n- probe: HW-PROBE-nothing\n```\n' \
        > "$scratch/prose.md"
    claim 'a transcript that carries model prose is refused, and the refusal exits 0' \
        ../../docs/spec/05-ai-integration.md \
        'The transcript holds no model prose' \
        0 'not one of its keys' \
        "$engine" probe record "$scratch/prose.md" --root "$root"

    # No verb writes a transcript. A recorder observes a session from outside
    # it, and a file this engine wrote would be a self-report with the engine's
    # name on it.
    name='no verb of this engine writes a transcript'
    before=$(ls "$root/docs/probe-runs" 2>/dev/null | wc -l)
    "$engine" probe plan --root "$root" > /dev/null 2>&1
    "$engine" probe record "$scratch/prose.md" --root "$root" > /dev/null 2>&1
    "$engine" probe grade "$scratch/prose.md" --root "$root" > /dev/null 2>&1
    after=$(ls "$root/docs/probe-runs" 2>/dev/null | wc -l)
    if [ "$before" = "$after" ]; then
        pass "$name"
    else
        fail "$name" "the shelf held $before documents and now holds $after"
    fi

    # A transcript this engine refused reaches no grader. The input contract of
    # the one component that returns a verdict, asserted on the verb rather than
    # described in a doc comment.
    claim 'a refused transcript reaches no grader' \
        ../../docs/spec/05-ai-integration.md \
        'It evaluates no expectation' \
        0 'reached no grader' \
        "$engine" probe grade "$scratch/prose.md" --root "$root"

    # The plan is deterministic, and so is the grade. That is what "a probe
    # result is a function of the transcript, the expectations and the grader
    # version" is testable as, and it is necessary rather than sufficient: the
    # recorded fixtures of the probe crate are what hold the verdicts.
    name='the grade over one transcript is the same bytes twice'
    "$engine" probe grade "$scratch/prose.md" --root "$root" > "$scratch/grade-a.txt" 2>/dev/null
    "$engine" probe grade "$scratch/prose.md" --root "$root" > "$scratch/grade-b.txt" 2>/dev/null
    if cmp -s "$scratch/grade-a.txt" "$scratch/grade-b.txt"; then
        pass "$name"
    else
        fail "$name" 'two runs over one transcript wrote different bytes'
    fi

    name='the run plan is the same bytes twice'
    "$engine" probe plan --root "$root" > "$scratch/probe-a.txt" 2>/dev/null
    "$engine" probe plan --root "$root" > "$scratch/probe-b.txt" 2>/dev/null
    if cmp -s "$scratch/probe-a.txt" "$scratch/probe-b.txt"; then
        pass "$name"
    else
        fail "$name" 'two runs over one tree wrote different bytes'
    fi

    rm -rf "$scratch"
    trap - EXIT INT TERM
else
    skip 'every claimed case' 'no built engine'
fi

printf '\n%s passed, %s failed, %s skipped\n' "$passed" "$failed" "$skipped"
[ "$failed" -eq 0 ] || exit 1
[ "$skipped" -eq 0 ] || printf 'Build the engine to run the skipped cases:\n  cargo build --release -p headwater-cli --manifest-path engine/Cargo.toml --locked\n'
exit 0
