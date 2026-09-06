#!/bin/sh
# The intent-time hook: `headwater route`, on the task before the agent reads a
# file. Registered on `UserPromptSubmit`, which takes no matcher, so it sees
# every prompt, including the ones no person wrote. It answers only the ones a
# person did: the envelope test below is the whole of that distinction.
#
# What it passes to the engine: the prompt, as text. Nothing else. The engine
# reads the checkout itself.
#
# What it hands back: what `route` wrote, verbatim, on standard output, which
# this harness adds to the agent's context. It hands back nothing at all when
# the route offered no pointer, because spec 5 makes silence a result and a
# wrong pointer costs more than a missing one.
#
# What a refusal means here: there is none. `route` exits 0 whether or not it
# offers a pointer, and this hook never blocks a prompt. An intent-time hook
# that refused would refuse the work rather than the mistake.
#
# It fails open and silently. Every other position in this repository prints one
# line when it cannot run, and this one does not, because it runs on every
# prompt and a line per prompt is a cost the reader pays forever. The commit
# hook says out loud when the engine is missing, once, at the moment it matters.
#
# # What it reads, and what it used to read
#
# `route --json` writes one document, and this hook decides on its `pointers`
# member: written on every run, and written empty where the route had nothing to
# offer. Until #321 there was no machine form of this verb at all, so the
# decision was a grep for an em-dash separator over the rendered report, with
# an em dash inside a document summary standing in for "the corpus has
# something for this". A summary written without one, or a line of prose that
# happened to carry one, moved the decision. That is clig.dev's "human-readable
# output breaking machine-readable output", and it was live in this repository.
#
# The same document carries the rendered report as `text`, so what reaches the
# agent is still the engine's own rendering, byte for byte. This hook composes
# no line of its own: a pointer line assembled here would be a second copy of
# `Pointer::render`, and a terminal and an agent that read different text about
# one document are reading about different corpora as far as either can tell.

. "$(dirname "$0")/lib.sh"

input=$(cat)
engine=$(hw_engine) || exit 0

task=$(hw_field "$input" user_input) || task=$(hw_field "$input" prompt) || exit 0
[ -n "$task" ] || exit 0

# Not every prompt was written by a person. This position takes no matcher, and
# the header above says it therefore sees every prompt — written when a prompt
# meant somebody typing. A harness that runs agents in parallel submits a prompt
# each time one of them finishes, and the text of that prompt is the agent's own
# closing report inside an envelope. Routing over it is a category error twice
# over: the report is not a task, and it is already in the reader's context, so
# every pointer the route offers was chosen against text nobody wrote as an
# intent.
#
# Measured on run 22, on 2026-09-06: 39 of this hook's 46 firings were a
# `<task-notification>`, and none of the 39 had a task in it. One of them routed
# over a finished agent's report, emitted 45.4KB, and reported that the budget
# had withheld 279 more pointers.
#
# So the envelope is the signal, and the test is the first characters rather
# than a search: a person may well write the words `task-notification` in a
# question about this repository, and that question deserves its pointers.
case $task in
    '<task-notification>'* | '<cross-session-message'* ) exit 0 ;;
esac

route=$("$engine" route --root "$hw_root" --json "$task" 2>/dev/null) || exit 0

# Two reads of one document, and the same engine that wrote it reads it back.
# `pointers` is written on every run and written empty where the route had
# nothing to offer, so the count is the decision and the emptiness is a result
# rather than a failure. A route that offered something then hands back the
# rendering it wrote. Every other way this can fail — a document that will not
# parse, a member that is not there, no built engine — leaves the substitution
# empty or its status non-zero, and both of those end the hook with the prompt
# untouched.
pointers=$(hw_count "$route" pointers) || exit 0
[ "$pointers" -gt 0 ] 2>/dev/null || exit 0

report=$(hw_field "$route" text) || exit 0
[ -n "$report" ] || exit 0

printf 'Headwater routed this task to the documents that govern it, before you open a file.\n'
printf 'These are pointers, not content. Open the ones that bear on the task.\n\n'
printf '%s\n' "$report"
exit 0
