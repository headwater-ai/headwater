#!/bin/sh
# The write-time hook. It registers on two events and acts on the one the
# harness names.
#
#   PreToolUse  Write|Edit   A refusal, then an advisory.
#
#                            The refusal: a new document written straight to
#                            disk invents its front matter, its identifier,
#                            its place and its sections. `headwater new`
#                            derives all four from the lock, and refuses
#                            rather than guesses. So this hook refuses the raw
#                            write of a new document and names the verb. An
#                            edit to a file that exists always passes.
#
#                            The advisory, for every call the refusal does not
#                            deny, in one JSON object of up to three parts:
#                            - the documents that declare they govern the path,
#                              or, for a path the governed scope admits and no
#                              document governs, the account `headwater route`
#                              gives of it;
#                            - for a path that is itself a document, the code
#                              paths it governs and the documents that declare
#                              an edge onto it;
#                            - for a path that a producer writes, its record
#                              and the command that rebuilds it, and whether a
#                              merge left it in conflict.
#                            It comes before the edit, so the agent reads the
#                            documents while the change is still a plan. It is
#                            advisory on purpose: a gate here teaches an author
#                            to answer "no impact" by reflex.
#
#   PostToolUse Write|Edit   One line, or nothing. The line names each
#                            governing edge of the path that the edit left
#                            suspect.
#
# Every fact comes from an existing verb: `explain`, `route` and `derived`.
# The hook adds no verb and holds no rule of its own.
#
# A refusal stops the tool call, and the agent reads the reason. A write
# through a shell command reaches no hook, so the commit and the pull request
# are what hold the result. The hook fails open, silently, on every path it
# cannot decide.
#
# Claude Code names the path `tool_input.file_path`, Copilot names it
# `tool_input.path`, and Codex passes a patch under `tool_input.command`.
# `hw_patch_path` in `lib.sh` reads the last one.

. "${0%/*}/lib.sh"

input=$(cat)
event=$(hw_field "$input" hook_event_name) || exit 0
path=$(hw_field "$input" tool_input file_path) \
    || path=$(hw_field "$input" tool_input path) \
    || path=$(hw_patch_path "$input") \
    || exit 0
[ -n "$path" ] || exit 0

# Every read below treats `hw_root` as the repository. `event` and `path` are
# fields of the payload, so they are read before this correction.
hw_root=$(hw_resolve_root "$input")

# The path as the corpus names it: relative to the repository root.
case $path in
    "$hw_root"/*) rel=${path#"$hw_root"/} ;;
    /*) exit 0 ;;
    *) rel=$path ;;
esac

# The advisory, on standard output, or nothing. Any part can be absent, and a
# call with no part prints nothing.
advise() {
    advisory=
    named=
    unrouted=
    if pointers=$(hw_governing_pointers "$rel"); then
        advisory="Headwater impact detection: a document in this corpus declares that it governs \`$rel\`, which you are about to change.

$pointers"
        named=1
    elif ungoverned=$(hw_ungoverned_in_scope "$rel"); then
        # The taxonomy expects a `governs` edge to reach this path, and none
        # does. The advisory proposes the edge and writes nothing.
        advisory="Headwater impact detection: \`$rel\`, which you are about to change, is a path this taxonomy expects a document to govern, and no document governs it.

$ungoverned"
        # The documents a term route reached, where it reached any, are still
        # documents to read.
        case $ungoverned in *'The route reached these documents'*) named=1 ;; *) unrouted=1 ;; esac
    fi
    if reverse=$(hw_governed_by_document "$rel"); then
        [ -n "$advisory" ] && advisory="$advisory

"
        advisory="${advisory}Headwater impact detection: \`$rel\`, which you are about to change, is a document that other files depend on.

$reverse"
        named=1
    fi
    # The path is one a producer writes. The row, the treatment and the
    # command come from `headwater derived`.
    if derived=$(hw_derived_report "$rel"); then
        heading=$(printf '%s\n' "$derived" | sed -n 1p)
        rebuild=$(printf '%s\n' "$derived" | sed -n 2p)
        [ -n "$advisory" ] && advisory="$advisory

"
        conflicted=
        if state=$(hw_merge_state); then
            hw_unmerged_paths | grep -Fqx -- "$rel" && conflicted=$state
        fi
        if [ -n "$conflicted" ]; then
            advisory="${advisory}Headwater: \`$rel\`, which you are about to change, is a derived artifact, and this $conflicted left it in conflict. Its record is: $heading."
            if [ -n "$rebuild" ]; then
                advisory="$advisory

Do not resolve it by hand. Resolve the sources, then run \`$rebuild\` on the merged tree and stage what it writes."
            fi
        else
            advisory="${advisory}Headwater: \`$rel\`, which you are about to change, is a derived artifact. Its record is: $heading."
            if [ -n "$rebuild" ]; then
                advisory="$advisory

Rebuild it with \`$rebuild\` rather than editing it by hand. An edit by hand is lost at the next run of that command."
            fi
        fi
    fi
    [ -n "$advisory" ] || exit 0
    # "Read each one" only where the advisory names a document to read.
    if [ -n "$named" ]; then
        advisory="$advisory

This is advisory. Read each one before the edit, and say whether the change invalidates it. Nothing here blocks the edit."
    elif [ -n "$unrouted" ]; then
        advisory="$advisory

This is advisory. Nothing here blocks the edit, and nothing here writes the edge. Declare it only in a document that rules this path."
    else
        advisory="$advisory

This is advisory. Nothing here blocks the edit."
    fi
    quoted=$(hw_quote "$advisory") || exit 0
    printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","additionalContext":%s}}\n' "$quoted"
    exit 0
}

case $event in
PreToolUse)
    # The refusal is for a file that does not exist yet. Every call it does
    # not deny gets the advisory instead.
    [ -e "$hw_root/$rel" ] && advise
    case $rel in *.md) ;; *) advise ;; esac

    # Whether the corpus claims this path is a question `headwater explain`
    # answers for a path with no file behind it, with the matcher the walk
    # itself uses.
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
    # otherwise. The advisory named the pointers before the edit.
    line=$(hw_suspect_edges "$rel") || exit 0
    quoted=$(hw_quote "$line") || exit 0
    printf '{"hookSpecificOutput":{"hookEventName":"PostToolUse","additionalContext":%s}}\n' "$quoted"
    exit 0
    ;;
*)
    exit 0
    ;;
esac
