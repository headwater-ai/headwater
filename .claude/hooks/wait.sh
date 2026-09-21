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
# hours, a third of every shell call in the run. All 47 had set the cap
# explicitly. None had set `run_in_background`, which has no cap, exits on its
# own condition, and notifies once. So the fix is not a longer timeout, and no
# timeout this harness accepts is long enough for a cold workspace test.
#
# This refuses the foreground form and names the flag. It is the same posture
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
# What it never refuses: a wait already in the background, a heredoc that writes
# a wait into a script, and anything it cannot parse. It fails open on every one.
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
# no `sleep`, no `gh run watch` and no `pgrep` in it cannot be either shape
# below, and that string test answers for almost all of them without starting
# anything.

. "$(dirname "$0")/lib.sh"

input=$(cat)

# The cheap gate. Nothing below runs for a call that cannot be a long wait.
printf '%s' "$input" | grep -qE 'sleep|gh run watch|pgrep' || exit 0

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

Either way, start the wait with \`run_in_background: true\`, because a foreground call is capped at ten minutes.

And end the wait when you exit. A marker wait can exit and still run forever: of 40 abandoned loops swept from this machine, 9 were exactly the marker shape above, still polling between 7 and 18 hours after the agent that started them had gone, because the job they watched had been killed and the marker was never written. Waiting on a pid or a marker fixes a loop that cannot exit. It does not fix a loop that nobody is left to end."
    quoted=$(hw_quote "$reason") || exit 0
    printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":%s}}\n' "$quoted"
    exit 0
fi

# Already backgrounded: this is the form the refusal asks for.
background=$(hw_field "$input" tool_input run_in_background)
case $background in true | True | TRUE) exit 0 ;; esac

# A command that writes a script rather than running one. A heredoc that
# carries a wait loop into a file is not itself a wait, and refusing it would
# refuse the act of writing the very script this hook asks for. Replaying run
# `cc7cc6c6` caught one of these in 400 calls.
printf '%s' "$command" | grep -qE '^[[:space:]]*cat[[:space:]]*>' && exit 0

# Any loop that sleeps, at any interval. The first cut of this rule asked for
# twenty seconds or more, on the theory that a shorter sleep polls a local file
# rather than a build. The replay refused that theory: a wait on a process that
# slept fifteen still ran to the cap, and the interval turned out to say nothing
# about how long the wait would be. What decides is the shape. A loop that waits
# belongs in the background whatever it sleeps for, which is what the harness's
# own guidance says for a single completion notification.
#
# `gh run watch` blocks for as long as CI takes and is the same case without a
# loop around it.
long=no
printf '%s' "$command" | grep -qE '(^|[;&|[:space:]])(until|while)([[:space:]]|$)' &&
    printf '%s' "$command" | grep -qE '(^|[;&|[:space:]])sleep[[:space:]]' &&
    long=yes
printf '%s' "$command" | grep -qE 'gh[[:space:]]+run[[:space:]]+watch' && long=yes
[ "$long" = yes ] || exit 0

reason="This waits in the foreground, and a foreground call is capped at ten minutes.

When the cap is reached the harness moves this loop into the background and answers \`moved to the background (ID: ...)\`. That line reports the loop, not the build, so it tells you nothing about what you were waiting for, and waiting again asks a question that is already being answered. In run cc7cc6c6 that pattern spent 7.8 hours, a third of every shell call in the run.

Re-issue this same command with \`run_in_background: true\`. There is no cap on it, the loop exits on its own condition however long that takes, and its completion notification is what wakes you. Do not then wait on it again.

In a subagent, a background wait that runs longer than about five minutes outlives the prompt cache, and the turn that reads its notification pays to write the whole context back rather than to read it, at roughly twelve times the cost. Run \`9ab3be93\` paid \$14.85 that way in one four-hour stretch. Wrap a wait that might run that long in \`timeout 240\`, so it exits on its own before the cache would have, and re-issue it if the condition still is not met: the wrapped loop has already ended by then, so this is not the re-issue the line above forbids, which is about a loop the harness itself only moved to the background and is still running. The parent's cache lives an hour, not five minutes, so the parent waits by ending its turn and never re-issues a bounded wait.

Every loop that waits belongs in the background, whatever it sleeps for."
quoted=$(hw_quote "$reason") || exit 0
printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":%s}}\n' "$quoted"
exit 0
