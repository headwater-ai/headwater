#!/bin/sh
# The wait-time hook.
#
#   PreToolUse  Bash   A wait that blocks in the foreground is capped by the
#                      harness at ten minutes. When it reaches the cap the
#                      harness moves the loop itself into the background and
#                      answers `moved to the background (ID: ...)`, which says
#                      nothing about the thing being waited on. An agent that
#                      reads that as "not finished" waits again, and the second
#                      wait is a question already being answered.
#
# What it cost: in run `cc7cc6c6`, 47 waits reached the cap and spent 7.8
# hours, a third of every shell call in the run. So the fix is not a longer
# timeout, and no timeout this harness accepts is long enough for a cold
# workspace test.
#
# The background is not the remedy either [HW-PD-0021]. A subagent whose only
# work left is a background wait ends its turn, and each attempt then wakes its
# parent at the parent's full context. Run `b5554ef1` spent about 310 parent
# turns that way, 28% of what its parent read. The one wait that works is
# `sh tools/run/wait-for.sh '<condition>'` in the foreground, with a Bash
# `timeout` of 300000: it ends inside four minutes, under both the ten-minute
# cap and the five-minute prompt cache, and it says RE-ISSUE when the agent
# must run it again. The parent runs no wait at all: it ends its turn.
#
# So this refuses three shapes, and each refusal names that one wait. It
# refuses an unbounded wait in the foreground. It refuses any wait started with
# `run_in_background: true`, `wait-for.sh` included. It refuses `wait-for.sh`
# in the foreground with a Bash `timeout` under 300000, which the harness
# would move to the background before the attempt ends. It is the same posture
# as `write.sh`: refuse the shape that cannot work, and say what does.
#
# It refuses a second shape for a different reason. A loop that waits on
# `pgrep -f "<literal>"` matches the polling shell's own command line and can
# never exit, and backgrounding that one hides the defect rather than fixing
# it. That refusal names the two waits that do work, a pid and an exit marker.
#
# What a refusal means: the harness does not run the call, and the agent reads
# the reason and re-issues. That costs one turn and no wall clock, against ten
# minutes and a turn for the timeout it replaces.
#
# What it never refuses: a foreground `wait-for.sh` with a timeout of 300000 or
# more, a heredoc that writes a wait into a script, a long job that is not a
# wait, and anything it cannot parse. It fails open on every one. A build or a
# suite still goes in the background, and `wait-for.sh` waits in the
# foreground on its exit marker.
#
# What it does not catch, stated so that nobody reads its silence as a pass: a
# long build run straight in the foreground with no loop around it, such as
# `cargo test --workspace`. One of the 47 was exactly that. Refusing every
# foreground `cargo` call would refuse the many that finish in seconds, so the
# run policy holds that case and this hook does not.
#
# The cheap read comes first on purpose. This hook is the only one in this
# directory that matches `Bash`, and `Bash` is most of what a run does: 3,865
# calls in the run above. `hw_field` starts the engine, so a hook that read a
# field on every call would add a process to every one of them. A payload with
# no `sleep`, no `gh run watch`, no `pgrep` and no `wait-for` in it cannot be
# any shape below, and that string test answers for almost all of them without starting
# anything.

. "$(dirname "$0")/lib.sh"

input=$(cat)

# The cheap gate. Nothing below runs for a call that cannot be a long wait.
printf '%s' "$input" | grep -qE 'sleep|gh run watch|pgrep|wait-for' || exit 0

command=$(hw_field "$input" tool_input command) || exit 0
[ -n "$command" ] || exit 0

# A loop that waits on `pgrep -f "<literal>"` cannot exit, and no flag fixes
# it. The polling shell's own command line contains the literal, so `pgrep -f`
# matches the shell that is doing the asking and the condition is true forever.
# Measured directly: a loop whose pattern sits in its own argv never exits, and
# the same pattern assembled at run time exits at once.
#
# The test asks for `-f` and no quote, on purpose. An earlier cut required a
# quoted literal and so let `pgrep -f $pat` through, which self-matches just as
# surely, because the assignment that sets `$pat` sits in the same command line.
# Assembling the pattern at run time is not offered as a remedy either: it works
# only while the literal is genuinely absent from the whole line, which is not a
# property an author can hold on to. The two remedies below do not depend on it.
#
# This is checked before the background flag on purpose. Backgrounding this
# shape does not repair it, it hides it: the loop becomes an immortal
# background job. One sweep of this repository collected 40 of them, aged
# between three and fifty-two hours. In run `cc7cc6c6`, 34 waits were this
# shape and spent 3.6 hours, and 14 of the 47 that reached the cap were never
# going to return whatever they were given.
if printf '%s' "$command" | grep -qE '(^|[;&|[:space:]])(until|while)([[:space:]]|$)' &&
    printf '%s' "$command" | grep -qE 'pgrep[^;|&]*-[[:alnum:]]*f[[:alnum:]]*[[:space:]]'; then
    reason="This waits on \`pgrep -f\` with a literal pattern, and it can never exit.

The shell running this loop carries the pattern in its own command line, so \`pgrep -f\` matches that shell, the condition stays true, and the loop runs until something kills it. \`run_in_background: true\` does not fix this one. It makes it worse: the loop becomes a background job that never ends. A sweep of this repository collected 40 such loops, the oldest 52 hours old.

Wait on the thing itself rather than on a pattern that describes it:

  until ! kill -0 <pid> 2>/dev/null; do sleep 30; done

or have the job write its own exit marker and wait for that:

  <command...> > \"\$LOG\" 2>&1; echo \"EXIT:\$?\" >> \"\$LOG\"
  until tail -1 \"\$LOG\" | grep -q '^EXIT:'; do sleep 30; done

Either way, put the condition in \`sh tools/run/wait-for.sh '<condition>'\` and run it in the foreground with a Bash \`timeout\` of \`300000\`. It ends inside four minutes, so it stays under the ten-minute cap of a foreground call. On \`RE-ISSUE\`, run the identical call again. In a subagent, a background wait ends the turn and wakes the parent at each attempt (HW-PD-0021).

And end the wait when you exit. A marker wait can exit and still run forever: of 40 abandoned loops swept from this machine, 9 were exactly the marker shape above, still polling between 7 and 18 hours after the agent that started them had gone, because the job they watched had been killed and the marker was never written. Waiting on a pid or a marker fixes a loop that cannot exit. It does not fix a loop that nobody is left to end."
    quoted=$(hw_quote "$reason") || exit 0
    printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":%s}}\n' "$quoted"
    exit 0
fi

# A command that writes a script rather than running one. A heredoc that
# carries a wait loop into a file is not itself a wait, and refusing it would
# refuse the act of writing the very script this hook asks for. Replaying run
# `cc7cc6c6` caught one of these in 400 calls.
printf '%s' "$command" | grep -qE '^[[:space:]]*cat[[:space:]]*>' && exit 0

background=no
case $(hw_field "$input" tool_input run_in_background) in true | True | TRUE) background=yes ;; esac

deny() {
    quoted=$(hw_quote "$1") || exit 0
    printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":%s}}\n' "$quoted"
    exit 0
}

remedy="Run \`sh tools/run/wait-for.sh '<condition>'\` in the foreground, with a Bash \`timeout\` of \`300000\`. It polls every thirty seconds and ends inside four minutes, on the condition or on \`RE-ISSUE\`. On \`RE-ISSUE\`, run the identical call again: that attempt has ended. For CI on a commit the condition is \`sh tools/run/ci-done.sh <sha>\`."

background_reason="This starts a wait with \`run_in_background: true\`.

A subagent whose only work left is a background wait ends its turn, and every attempt then wakes its parent at the parent's full context to read a line that says nothing. Run \`b5554ef1\` spent about 310 parent turns that way, 28% of what its parent read (HW-PD-0021). The parent runs no wait at all: it ends its turn, and the report of the stage wakes it.

$remedy

A long job that is not a wait, such as a build or a suite, still goes in the background, and \`wait-for.sh\` waits in the foreground on its exit marker."

# The bounded wait itself. It is the remedy in the foreground, and it is the
# shape HW-PD-0021 forbids in the background. In the foreground it needs a
# Bash timeout past its four-minute cap: the default is 120000, and at the
# timeout the harness moves the call to the background.
# Only in command position, so a `grep` or an `echo` that names the script
# is not a wait.
if printf '%s' "$command" | grep -qE '(^|[;&|])[[:space:]]*((sh|bash|timeout[[:space:]]+[0-9]+[a-z]?)[[:space:]]+)?([^[:space:];&|]*/)?wait-for\.sh([[:space:]]|$)'; then
    [ "$background" = yes ] && deny "$background_reason"
    timeout=$(hw_field "$input" tool_input timeout)
    case $timeout in '' | *[!0-9]*) timeout=0 ;; esac
    [ "$timeout" -ge 300000 ] && exit 0
    deny "This runs \`wait-for.sh\` in the foreground with a Bash \`timeout\` under \`300000\`, and the default is \`120000\`.

One attempt runs for up to four minutes. At the timeout the harness moves the call to the background, and a background wait wakes the parent at each attempt (HW-PD-0021).

$remedy"
fi

# Any loop that sleeps, at any interval. The first cut of this rule asked for
# twenty seconds or more, on the theory that a shorter sleep polls a local file
# rather than a build. The replay refused that theory: a wait on a process that
# slept fifteen still ran to the cap, and the interval turned out to say nothing
# about how long the wait would be. What decides is the shape. A loop that waits
# is refused whatever it sleeps for: in the foreground because nothing bounds
# it, and in the background because it wakes the parent (HW-PD-0021). Either
# way the remedy is the bounded wait above.
#
# `gh run watch` blocks for as long as CI takes and is the same case without a
# loop around it.
long=no
printf '%s' "$command" | grep -qE '(^|[;&|[:space:]])(until|while)([[:space:]]|$)' &&
    printf '%s' "$command" | grep -qE '(^|[;&|[:space:]])sleep[[:space:]]' &&
    long=yes
printf '%s' "$command" | grep -qE '(^|[;&|[:space:]])gh[[:space:]]+run[[:space:]]+watch' && long=yes
[ "$long" = yes ] || exit 0
[ "$background" = yes ] && deny "$background_reason"

reason="This loop waits in the foreground with no bound, and a foreground call is capped at ten minutes.

When the cap is reached the harness moves this loop into the background and answers \`moved to the background (ID: ...)\`. That line reports the loop, not the build, so it tells you nothing about what you were waiting for, and waiting again asks a question that is already being answered. In run cc7cc6c6 that pattern spent 7.8 hours, a third of every shell call in the run.

Replace the loop with the bounded wait, which never reaches the ten-minute cap. $remedy

In a subagent, a wait that runs longer than about five minutes outlives the prompt cache, and the turn after it pays to write the whole context back rather than to read it, at roughly twelve times the cost. Run \`9ab3be93\` paid \$14.85 that way in one four-hour stretch. The four-minute cap is what prevents that.

Do not start the wait with \`run_in_background: true\` in a subagent. A subagent whose only work left is a background wait ends its turn, and every attempt then wakes the parent at its full context to read a line that says nothing. Run \`b5554ef1\` spent about 310 parent turns that way, 28% of what its parent read. The parent's cache lives an hour, not five minutes, so the parent waits by ending its turn and never runs a wait at all.

A long job that is not a wait, such as a build or a suite, still goes in the background, and \`wait-for.sh\` waits in the foreground on its exit marker."
quoted=$(hw_quote "$reason") || exit 0
printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":%s}}\n' "$quoted"
exit 0
