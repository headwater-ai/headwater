#!/bin/sh
# One dispatch of the Headwater build order, run as a `copilot -p` subprocess.
#
# Copilot needs far less translation than Codex does. It already reads
# `AGENTS.md` (a symlink to `CLAUDE.md`), `.claude/skills/*/SKILL.md` and
# `.claude/agents/*.md` directly — `--agent <name>` resolves a persona from
# `.claude/agents/<name>.md` with no mirror file required, confirmed live on
# copilot-cli 1.0.83 by removing `.github/agents/hw-queue.agent.md` and
# finding dispatch unchanged. So a stage here is a real invocation of the
# persona's own file, not a composed proxy: `tools/run/run-dir.sh` is used
# unmodified, same as the Codex spike.
#
#   sh tools/run/copilot-dispatch.sh dispatch <stage> <run> <tag> <dispatch-file>
#   sh tools/run/copilot-dispatch.sh veto     <run> <tag> <finding-file>
#   sh tools/run/copilot-dispatch.sh report   <run> <tag>
#   sh tools/run/copilot-dispatch.sh session  <run> <tag>
#
# <stage> is one of: product-owner maintainer queue adjudicate build verify integrate
# <tag>   names the dispatch inside the run: `issue-573-build`, `queue`, ...
# <run>   is the directory `sh tools/run/run-dir.sh start` printed.
#
# Every artifact of a dispatch lands under <run>:
#     prompts/<tag>.md      the composed prompt, exactly what the model read
#     reports/<tag>.md      the agent's last message, which is its report
#     sessions/<tag>.session  the Copilot session id, for `veto`
#
# Two things a real dispatch must get right, both confirmed live and neither
# obvious from `copilot --help`:
#
# 1. The session id is ours to assign, not the harness's to report back.
#    `--session-id <uuid>` on the first call fixes the id; `--resume=<uuid>`
#    on a later, separate process continues it with full context, confirmed
#    by a round trip through a memorable word across two independent
#    invocations. Codex has to parse a thread id out of its own JSON stream
#    because it assigns the id; Copilot does not need that here.
#
# 2. Identity does not survive a bare `--resume`. A session resumed without
#    `--agent <name>` keeps the transcript but reverts to Copilot's default
#    persona — confirmed live: asked to name whose voice it was speaking in,
#    a bare resume answered "GitHub Copilot CLI, not a specialized sub-agent"
#    while still half-recalling the prior persona's constraints from the
#    transcript, a genuinely confused answer. Passing `--agent <name>` again
#    on the resume call restores both identity and context together,
#    confirmed by the same round trip repeated with `--agent` on both calls.
#    Every call below carries `--agent`, dispatch and veto alike, because of
#    this.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)

die() { echo "copilot-dispatch: $*" >&2; exit 1; }

persona_for() {
    case $1 in
        product-owner) echo headwater-product-owner ;;
        maintainer)    echo headwater-maintainer ;;
        queue)         echo hw-queue ;;
        adjudicate)    echo hw-adjudicate ;;
        build)         echo hw-build ;;
        verify)        echo hw-verify ;;
        integrate)     echo hw-integrate ;;
        *) die "unknown stage \`$1\`. One of: product-owner maintainer queue adjudicate build verify integrate" ;;
    esac
}

new_session_id() {
    if [ -r /proc/sys/kernel/random/uuid ]; then
        cat /proc/sys/kernel/random/uuid
    elif command -v uuidgen >/dev/null 2>&1; then
        uuidgen
    else
        die "no uuid source: need /proc/sys/kernel/random/uuid or uuidgen"
    fi
}

# Copilot reads the persona and the skills directly from the checkout, so the
# preamble here is four gaps, not the Codex translation of every mechanism:
# `EnterWorktree` does not exist, the scratch path needs stating in this
# harness's own terms, an in-process second opinion is a real capability
# (Copilot's own `task` tool) but is not exercised by this dispatcher yet, and
# the two rules this harness adds are the same two the Codex spike adds.
preamble() {
    scratch=$1
    cat <<'PRE'
You are one stage of a Headwater build-order run, dispatched as your own
Copilot session by `tools/run/copilot-dispatch.sh`, running as the named agent
your `--agent` flag selected. Your instructions are your own persona file,
read directly — nothing below repeats them. Three things this harness does
differently from the one they were written against:

- **`EnterWorktree` does not exist here.** Where your instructions say to make
  a worktree by hand with `git worktree add`, that is exactly what you do, and
  it is the only path.
- **`$CLAUDE_JOB_DIR/tmp/issue-<N>/` means the `scratch:` path in the dispatch
  below.** It already exists. Write your notes there and nowhere else.
- **You have a native `task` tool that can dispatch a named agent as an
  in-process sub-task**, which is a real second opinion where your
  instructions ask for one from another model. It is not required: if you use
  it, name the agent and model you dispatched in your report; if you do not,
  report that you did not take one rather than faking it. `hw-verification-bar`
  already tells you which section asks for this.

Two rules this harness adds:

1. **Your last message is your report.** It is captured verbatim to a file the
   parent reads, so it is under 400 tokens and it ends in the fixed block your
   instructions name. Say nothing after that block.
2. **The turn gate is live.** `.github/hooks/hooks.json` runs
   `.githooks/pre-commit` at the end of your turn, so a finding it reports is
   yours to fix before you can finish. That is the same gate a commit runs.
PRE
    printf '\nYour scratch directory is `%s`.\n' "$scratch"
}

compose() {
    stage=$1 dispatch=$2 out=$3 scratch=$4
    [ -f "$dispatch" ] || die "no dispatch file at $dispatch"

    preamble "$scratch" > "$out"

    printf '\n\n## THE VALUE RULE\n\nBefore any work starts, name the reader who is not this repository. If the only party better off is Headwater'"'"'s own corpus, the work is not eligible for an iteration and not eligible for the tracker: it is scaffolded as an obligation record under `docs/spec/13-open-obligations.md` and left there. `adopter-blocking` means work an outside adopter cannot proceed without, and it sorts above everything else.\n' >> "$out"

    printf '\n\n## YOUR DISPATCH\n\n' >> "$out"
    cat "$dispatch" >> "$out"
}

dispatch() {
    [ $# -eq 4 ] || die "usage: dispatch <stage> <run> <tag> <dispatch-file>"
    stage=$1 run=$2 tag=$3 dfile=$4
    [ -d "$run" ] || die "$run is not a run directory"
    agent=$(persona_for "$stage")

    mkdir -p "$run/prompts" "$run/reports" "$run/sessions"
    scratch="$run/tmp/$tag"
    mkdir -p "$scratch"

    prompt="$run/prompts/$tag.md"
    report="$run/reports/$tag.md"
    sid=$(new_session_id)

    compose "$stage" "$dfile" "$prompt" "$scratch"

    # `-p` takes the prompt as a literal argument, not stdin: `-p -` was
    # tried and the model received the two-character string `-`, confirmed
    # live. So the composed prompt goes on the command line, which is why
    # `compose` keeps it to the dispatch file's own content plus a short
    # preamble rather than Codex's full persona-and-skills inlining — a
    # build dispatch here stays well under ARG_MAX. `--silent` prints only
    # the agent's final message, which is the report — Copilot has no
    # `-o <file>` flag the way Codex does, so this is stdout redirection
    # rather than a captured file argument. `--allow-all-tools` is required
    # for non-interactive mode at all; full access because every stage runs
    # `gh`, `cargo` and `git push`.
    (
        cd "$root" || exit 1
        copilot -C "$root" --agent "$agent" --session-id "$sid" \
            --allow-all-tools --silent \
            -p "$(cat "$prompt")" > "$report" 2>"$run/reports/$tag.err"
    )
    status=$?

    printf '%s\n' "$sid" > "$run/sessions/$tag.session"

    printf 'STAGE: %s\nAGENT: %s\nTAG: %s\nSESSION: %s\nEXIT: %s\nREPORT: %s\n' \
        "$stage" "$agent" "$tag" "$sid" "$status" "$report"
    [ -s "$report" ] && { printf -- '--- report ---\n'; cat "$report"; }
    return $status
}

# The veto. `--resume=<session-id> --agent <name>` continues that exact
# session with its context intact and its identity restored — the `--agent`
# is not optional here, confirmed live: a resume without it answers in
# Copilot's default voice, half-recalling the persona rather than being it.
veto() {
    [ $# -eq 3 ] || [ $# -eq 4 ] || die "usage: veto <run> <tag> <finding-file> [stage]"
    run=$1 tag=$2 finding=$3
    sfile="$run/sessions/$tag.session"
    [ -f "$sfile" ] || die "no recorded session for \`$tag\`; the veto cannot resume a session it never saw"
    [ -f "$finding" ] || die "no finding file at $finding"
    sid=$(cat "$sfile")

    # The stage the session was dispatched as. `<tag>` is `issue-<N>-<stage>`
    # by convention; if that convention was not followed, pass the stage
    # explicitly as a fourth argument.
    stage=${4:-${tag##*-}}
    agent=$(persona_for "$stage")

    report="$run/reports/$tag-veto-$(date -u +%H%M%S).md"

    (
        cd "$root" || exit 1
        copilot -C "$root" --agent "$agent" --resume="$sid" \
            --allow-all-tools --silent \
            -p "$(cat "$finding")" > "$report" 2>&1
    )
    status=$?

    printf 'RESUMED: %s\nAGENT: %s\nSESSION: %s\nEXIT: %s\nREPORT: %s\n' "$tag" "$agent" "$sid" "$status" "$report"
    [ -s "$report" ] && { printf -- '--- report ---\n'; cat "$report"; }
    return $status
}

case ${1:-} in
    dispatch) shift; dispatch "$@" ;;
    veto)     shift; veto "$@" ;;
    report)   [ $# -eq 3 ] || die "usage: report <run> <tag>"; cat "$2/reports/$3.md" ;;
    session)  [ $# -eq 3 ] || die "usage: session <run> <tag>"; cat "$2/sessions/$3.session" ;;
    *) sed -n '/^#   sh tools/,/^#$/p' "$0" | sed 's/^# \{0,2\}//' >&2; exit 2 ;;
esac
