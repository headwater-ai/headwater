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
            --now 2026-08-14 --root "$clean" >/dev/null 2>&1
        scaffolded=$("$engine" capture --root "$clean" |
            sed -n 's/^  \([0-9]*\) of \([0-9]*\) classified documents carry a reading$/\1 \2/p')

        # The same document shape, written by no verb at all.
        printf -- '---\nid: OBL-repo-9998\ntitle: "A record no verb wrote"\nstatus: current\nstatus_since: 2026-08-14\nlast_verified: 2026-08-14\nsummary: "Written by no verb, to hold the reach figure against a route the store does not watch."\n---\n\n# A record no verb wrote\n\n## Context\n\nNone.\n\n## Obligation\n\nNone.\n\n## Discharge\n\nNone.\n' \
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

    # No socket. The middle part of a sweep needs a model and no crate of this
    # engine can reach one, so an unreachable model is an absent file rather
    # than a failed build.
    name='no crate of this engine depends on a network client'
    reached=$(grep -lE '^(reqwest|hyper|ureq|curl|isahc|surf|attohttpc|tungstenite|native-tls|rustls|openssl) *=' \
        "$root"/engine/crates/*/Cargo.toml 2>/dev/null)
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

    # The one gate that reaches the grader, and what it says over a corpus that
    # holds no transcript. The declaration is not passed over and the run is not
    # silent about it: the reason names the missing input, which is where an
    # empty arm belongs rather than in a register a reader has to look up.
    name='the probe-result projection states the input this corpus does not hold'
    "$engine" generate --check --root "$root" > "$scratch/generate.txt" 2>&1
    if grep -q 'probe_result' "$scratch/generate.txt" &&
        grep -q 'holds no `probe_transcript` document' "$scratch/generate.txt"; then
        pass "$name"
    else
        fail "$name" 'the run does not name the transcript it wants'
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
    printf '# A run\n\n## Run identity\n\n```yaml\nmodel: a\nreasoning: I read the governing document first.\n```\n\n## Events\n\n```yaml\n- probe: PROBE-HW-nothing\n```\n' \
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
[ "$skipped" -eq 0 ] || printf 'Build the engine to run the skipped cases:\n  cargo build --release -p headwater-cli --manifest-path engine/Cargo.toml\n'
exit 0
