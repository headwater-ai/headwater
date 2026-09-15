#!/bin/sh
# The output-bounding hook. It registers on `PreToolUse` with a `Bash` matcher
# and rewrites the command so that what a shell prints into an agent's context
# is bounded, while the whole of it stays on disk.
#
# Why this position and no other. A harness gives three moments around a tool
# call, and only one of them can change what the model reads. `PreToolUse` is
# the one: its `updatedInput` replaces the arguments before the tool runs.
# `PostToolUse` fires after the result is already in the transcript and carries
# no field that alters it, and `SubagentStart` carries none either. So a hook
# that wants a smaller result has to ask for a smaller result in advance, which
# is what this file does.
#
# What it is worth, measured over 28 `hw-build` dispatches on 2026-09-12. A
# construction agent's context runs from 22K tokens to 178K over 134 calls, and
# 80% of what it is billed for is that growth rather than the preamble. Of the
# growth, 59% is tool output and harness reminders and 41% is the agent's own
# thinking and tool arguments, which nothing here can touch. `Bash` is 80% of
# the part that can be touched. The median `Bash` result is 233 tokens and the
# harness already truncates near 10K, so the cost is not a fat tail to clip: it
# is the middle of the distribution, and only a low cap reaches it. At the
# default below the saving is about 15% of that stage's bill.
#
# What it hands the agent. The head, the tail, a count of the lines between
# them, the exit status, and the path of the whole log. The last two are the
# point. A bounded result that loses the exit status turns a failure into a
# silence, and a bounded result that loses the log turns a truncation into a
# dead end. Both are the shape this repository keeps meeting, where a check
# that cannot run reads as a check that passed.
#
# What it does not touch. A command short enough to fit under the cap is passed
# through whole, so the common case reads exactly as it did. A backgrounded
# call is left alone: its output is collected elsewhere and the wrapper would
# hold the shell open. A command this hook has already wrapped is left alone,
# which is what makes it safe against a harness that replays an input.
#
# Two properties of the generated command decide whether it is safe, and both
# are held by `.claude/hooks/fixtures.sh`. It runs inside a brace group and not a
# subshell, so a `cd` still moves the shell the next call inherits. An `EXIT`
# trap prints the capped output, so a command that ends in `exit` prints its
# result instead of killing the wrapper before it reports anything.
#
# It fails open, silently, on every path it cannot decide. No engine, no
# command, an input that will not parse: the hook prints nothing and the
# harness runs what the agent wrote.

. "$(dirname "$0")/lib.sh"

# Where this position looks for a binary, which is not always the checkout it
# was handed. A fresh worktree has no `engine/target` at all, and an agent is
# dispatched into one before it builds anything. This hook would then fail open
# for the early part of every dispatch, which is the part whose output it most
# needs to bound.
#
# It may reach past the worktree because of what it asks the engine for.
# `headwater json` is the one verb of this binary that reads no corpus, which
# [HW-DR-0055](../../docs/decisions/0055-a-hook-reads-a-wire-format-through-the-engine-and-not-through-an-interpreter.md)
# states when it puts the verb outside the term that forbids a hook verb. So
# which build answers changes nothing: a binary months behind `main` parses a
# payload exactly as a fresh one does, and the staleness that makes a shared
# binary wrong for `check` cannot reach a reader of standard input. A position
# that called a corpus verb could not do this, and this file calls none.
if ! hw_engine >/dev/null 2>&1; then
    hw_common=$(git -C "$hw_root" rev-parse --git-common-dir 2>/dev/null) || exit 0
    case $hw_common in
    /*) ;;
    *) hw_common=$hw_root/$hw_common ;;
    esac
    hw_root=$(dirname "$hw_common")
fi

# How much of a long result reaches the context, in lines from each end. The
# default pair is about a thousand tokens, which is the measured knee: it
# reaches 17% of calls and takes 15% off the stage. A higher pair reaches
# almost nothing, and a lower one starts hiding the middle of a test run.
cap_head=${HW_CAP_HEAD:-60}
cap_tail=${HW_CAP_TAIL:-40}

# Where the whole log goes. It is outside the checkout on purpose: these are a
# byproduct of a run and not a part of the tree, and the temporary directory is
# swept by the host rather than by this repository.
cap_logs=${HW_CAP_LOGDIR:-${TMPDIR:-/tmp}/headwater-bashlogs}

input=$(cat)

# The command, or nothing this hook can act on.
command=$(hw_field "$input" tool_input command) || exit 0
[ -n "$command" ] || exit 0

# A backgrounded call is not wrapped. `hw_field` exits non-zero when the field
# is absent, which is the ordinary case, so the test reads the value and not
# the status.
background=$(hw_field "$input" tool_input run_in_background 2>/dev/null)
[ "$background" = "true" ] && exit 0

# A command this hook already wrapped carries the marker. Wrapping it twice
# would nest the traps and report the inner status as the outer one.
case $command in
*__hwcap*) exit 0 ;;
esac

# The wrapper. `__hwdump` is defined before the trap that calls it, the brace
# group keeps a `cd` in the caller's shell, and the redirection is scoped to
# the group so that the summary itself reaches standard output.
wrapped=$(
    printf '%s' \
        "__hwcap=\$(mkdir -p '$cap_logs' && mktemp '$cap_logs'/cmd.XXXXXX); " \
        "__hwdump() { __hwn=\$(wc -l <\"\$__hwcap\"); " \
        "if [ \"\$__hwn\" -le $((cap_head + cap_tail)) ]; then cat \"\$__hwcap\"; " \
        "else head -n $cap_head \"\$__hwcap\"; " \
        "echo \"...[\$(( __hwn - $cap_head - $cap_tail )) lines omitted; whole log: \$__hwcap]...\"; " \
        "tail -n $cap_tail \"\$__hwcap\"; fi; " \
        "echo \"[exit=\$1 log=\$__hwcap]\"; }; " \
        "trap '__hwdump \$?' EXIT; " \
        "{ $command ; } >\"\$__hwcap\" 2>&1; __hwrc=\$?; " \
        "trap - EXIT; __hwdump \$__hwrc; ( exit \$__hwrc )"
)

# The engine writes the string literal, because a command holds every character
# that breaks a hand-built one. A quote this hook cannot get back is a hook
# that hands the harness a broken object, so it says nothing instead.
quoted=$(hw_quote "$wrapped") || exit 0
[ -n "$quoted" ] || exit 0

printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow","updatedInput":{"command":%s}}}\n' "$quoted"
