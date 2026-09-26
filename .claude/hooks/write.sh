#!/bin/sh
# The write-time hook. Spec 5 gives this moment two mechanisms, and they want
# opposite postures, so this file registers on two events and dispatches on the
# one the harness names.
#
#   PreToolUse  Write|Edit   backfill, then impact detection, in that order.
#
#                            Backfill: a new document written straight to disk
#                            invents its front matter, its identifier, its
#                            placement and its sections. `headwater new` derives
#                            all four from the committed lock and refuses
#                            eighteen ways before it writes a byte. So this
#                            refuses the raw write and names the verb.
#
#                            Impact detection, for every call the refusal does
#                            not deny: a document may declare that it governs a
#                            code path, written as a pattern or as a list of
#                            them (HW-DR-0074), and an edit to any path a
#                            pattern admits raises an advisory prompt that names
#                            the documents at risk. It comes before the edit,
#                            so the agent reads the governing set while the
#                            change is still a plan (#953). Spec 5 makes it
#                            advisory on purpose: a gate here trains an author
#                            to answer "no doc impact" by reflex, and that
#                            destroys the signal.
#
#                            A path that the governed scope admits and that no
#                            document governs gets the engine's account instead
#                            of silence: the scope fact and the front-matter
#                            lines that would declare the edge (#953). The hook
#                            writes nothing, and a path outside the scope stays
#                            silent. `headwater route` decides both; this file
#                            holds no scope pattern.
#
#                            The reverse direction, in the same call and the
#                            same JSON object: an edit to a document that
#                            governs code paths, or that other documents
#                            declare an edge onto, names those paths and those
#                            documents (#1008). A document edited as if it
#                            stood alone is how a governing decision drifts
#                            from the code it rules. Every fact of this part
#                            is read from `headwater explain --json`.
#
#   PostToolUse Write|Edit   one line, or nothing. The line names each
#                            governing edge of the edited path that the edit
#                            left suspect: the edge records a
#                            `verified_revision`, and the revision its target
#                            has now differs (#953). It says that `headwater
#                            check` reports the edge. The engine decides which
#                            edges are suspect, with the comparison the check
#                            makes, and `headwater route --json` carries them
#                            on each anchored pointer. The pointers themselves
#                            are not printed again: the advisory said them
#                            before the edit.
#
# What it passes to the engine: one path. What it gets back: for the refusal,
# the classification `headwater explain` reports on standard error for a path
# with no document behind it — [#319](https://github.com/headwater-ai/headwater/issues/319)
# is what put that answer inside the engine rather than in this file. The
# question is answered by `explain`, an existing verb, and not by a new
# `headwater hook write` verb: spec 5's hook contract states "no hook
# introduces a verb", because "two entry points to one answer are two answers
# as soon as one drifts". For the advisory, the pointers `headwater route`
# resolves from the anchor, and the `related` entries `headwater explain
# --json` reports for a document.
#
# What a refusal means: the harness does not run the tool call, and the agent
# reads the reason. What happens when the harness ignores it: the write lands.
# A `Bash` call that writes the same file matches no matcher here and this hook
# never sees it. That bypass is one tool call away and it leaves no trace. The
# commit hook and the CI job are what hold the result, and this position holds
# nothing on its own.
#
# It fails open, silently, on every path it cannot decide.
#
# This is the one script spec 16 registers under three names: `.claude/`,
# `.codex/hooks.json` and `.github/hooks/*.json` all point a `PreToolUse` and
# a `PostToolUse` position at this file, the second silent for now, because a binding calls a verb that
# ships and carries no rule of its own. `hook_event_name` and `tool_name`
# agree across all three, confirmed live, and the one path a call names is the
# one field that does not: Claude Code passes `tool_input.file_path`, Copilot
# passes `tool_input.path` on the same `Write`/`Edit` tool names, and Codex's
# edit tool is `apply_patch`, which passes the patch text under
# `tool_input.command` rather than a bare path at all. `hw_patch_path` in
# `lib.sh` is the one place that reads that last shape.

. "$(dirname "$0")/lib.sh"

input=$(cat)
event=$(hw_field "$input" hook_event_name) || exit 0
path=$(hw_field "$input" tool_input file_path) \
    || path=$(hw_field "$input" tool_input path) \
    || path=$(hw_patch_path "$input") \
    || exit 0
[ -n "$path" ] || exit 0

# Corrected from the payload's own `cwd` now that it is readable. Everything
# below this line treats `hw_root` as the repository: the prefix comparison
# right after it, and the `explain`/`route` calls each branch makes with
# `--root "$hw_root"`. `event` and `path` above are payload reads, not
# checkout reads, so they were worth taking through the pre-correction root
# first.
hw_root=$(hw_resolve_root "$input")

# The path as the corpus names it: relative to the repository root.
case $path in
    "$hw_root"/*) rel=${path#"$hw_root"/} ;;
    /*) exit 0 ;;
    *) rel=$path ;;
esac

# The impact advisory, on standard output, or nothing. It is one JSON object
# with up to two parts, worded for an edit that has not happened yet. The
# forward part names the documents that govern the path. The reverse part,
# for a path that is itself a document, names the code paths it governs and
# the documents that declare an edge onto it (#1008). Either part can be
# absent, and a call with neither prints nothing.
advise() {
    advisory=
    named=
    if pointers=$(hw_governing_pointers "$rel"); then
        advisory="Headwater impact detection: a document in this corpus declares that it governs \`$rel\`, which you are about to change.

$pointers"
        named=1
    elif ungoverned=$(hw_ungoverned_in_scope "$rel"); then
        # #953: the path is one the taxonomy expects a `governs` edge to reach,
        # and none does. A term route over such a path reached a document for
        # 3 of 213 in-scope ungoverned entries on 2026-09-26, so the advisory
        # speaks whether or not it has a document to name. It proposes the
        # edge and writes nothing.
        advisory="Headwater impact detection: \`$rel\`, which you are about to change, is a path this taxonomy expects a document to govern, and no document governs it.

$ungoverned"
        # The documents a term route reached, where it reached any, are still
        # documents to read.
        case $ungoverned in *'The route reached these documents'*) named=1 ;; esac
    fi
    if reverse=$(hw_governed_by_document "$rel"); then
        [ -n "$advisory" ] && advisory="$advisory

"
        advisory="${advisory}Headwater impact detection: \`$rel\`, which you are about to change, is a document that other files depend on.

$reverse"
        named=1
    fi
    [ -n "$advisory" ] || exit 0
    # "Read each one" only where the advisory names a document to read. An
    # ungoverned path that no route reached names none (#953, verify finding 3).
    if [ -n "$named" ]; then
        advisory="$advisory

This is advisory. Read each one before the edit, and say whether the change invalidates it. Nothing here blocks the edit."
    else
        advisory="$advisory

This is advisory. Nothing here blocks the edit, and nothing here writes the edge. Declare it only in a document that rules this path."
    fi
    quoted=$(hw_quote "$advisory") || exit 0
    printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","additionalContext":%s}}\n' "$quoted"
    exit 0
}

case $event in
PreToolUse)
    # The refusal is for a file that does not exist yet. An edit to a document
    # that already has front matter is what `check --fix` and the commit hook
    # hold, and refusing it here would refuse every edit this repository is
    # made of. Every call the refusal does not deny gets the advisory instead.
    [ -e "$hw_root/$rel" ] && advise
    case $rel in *.md) ;; *) advise ;; esac

    # Whether the corpus claims this path is a question `headwater explain`
    # now answers for a path with no file behind it — the same
    # `headwater_meta::pattern::Pattern` the walk itself matches an existing
    # file against, reached through one verb rather than reimplemented here.
    # #319: this used to read the generated corpus descriptor and match the
    # exclusion patterns with a Python glob matcher, a second implementation
    # nothing compared against the engine's own.
    engine=$(hw_engine) || exit 0
    account=$("$engine" explain --root "$hw_root" "$rel" 2>&1 >/dev/null)
    case $account in
        *'is a path of this corpus, with no document written there yet'*) ;;
        *) advise ;;
    esac

    reason="\`$rel\` is a new document under this corpus, and a raw write invents what the taxonomy already decides.

Run \`headwater new <kind> --title \"<title>\"\` instead. It reads .headwater/taxonomy.lock and derives the shelf that fixes the path, the front matter the kind requires, an identifier under the kind's scheme, and the sections the contract requires. It refuses rather than guessing, and it never overwrites a document.

Then edit the file it wrote. This refusal is for creation only, and every later edit passes.

If the file is genuinely not a document of any kind this taxonomy declares, it does not belong under this corpus."
    quoted=$(hw_quote "$reason") || exit 0
    printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":%s}}\n' "$quoted"
    exit 0
    ;;
PostToolUse)
    # One line where the edit left a governing edge suspect, and nothing
    # otherwise. The header says why the pointers are not printed again.
    line=$(hw_suspect_edges "$rel") || exit 0
    quoted=$(hw_quote "$line") || exit 0
    printf '{"hookSpecificOutput":{"hookEventName":"PostToolUse","additionalContext":%s}}\n' "$quoted"
    exit 0
    ;;
*)
    exit 0
    ;;
esac
