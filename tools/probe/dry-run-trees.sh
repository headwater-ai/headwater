# SPDX-License-Identifier: Apache-2.0
#
# The trees of a dry run, built once for each set of inputs they depend on.
# `campaign-dry-run.sh` and `additive-dry-run.sh` source this file. It reads
# their `root`, `head`, `engine`, `server`, `declaration`, `work` and
# `all_probes`. It is not run on its own.
#
# A dry run has two halves. The first builds trees: the present tree, each
# arm's or scenario's tree, the `ablate.sh --diff` of each against its base,
# and the MCP tools a server lists in a tree. The second runs the leak check
# over those trees. Only the second reads the `leaks:` and `leaks_kept:`
# blocks of the declaration. `seal.sh` reads the answer keys and never
# `leaks:`, and `ablate.sh` reads neither block.
#
# Without `HW_CAMPAIGN_TREE_CACHE`, the trees live under the run's own `work`
# directory and are removed with it, and a scenario tree is removed once
# checked. That is what every batch and every dry run a person starts does.
#
# `HW_CAMPAIGN_TREE_CACHE` names a directory where the first half outlives
# the run. It is for the fixtures alone. Their dry runs differ in their leak
# strings and almost never in their trees, and each rebuilt the same trees
# from a 90 MB `git archive`, which was most of the time of the suite in CI.
# Each set of trees lives under a key. The key is the sha256 of every input
# the first half reads: the checkout's path and `HEAD`, the probes the spec
# selects, the declaration with `leaks:` and `leaks_kept:` taken out by
# `declared.py trees`, the engine and the server, and the scripts that build
# and check the trees. A run whose key is new removes every other key's
# trees first, so the cache holds one set at a time. The text the intent hook
# injects for each probe, which the leak check reads in a scenario that holds
# the hook, is kept beside the trees, as `tools/probe/leak.py` says.
#
#     trees_open           sets `trees` and `trees_kept`, or returns 3
#     trees_present        builds `$trees/present` once, or returns 3
#     trees_memo <name> <command>...
#                          runs a command of the first half once, and prints
#                          its output and returns its status every time
#     trees_drop <name>... removes trees and their memos when nothing keeps them

trees_open() {
    if [ -z "${HW_CAMPAIGN_TREE_CACHE:-}" ]; then
        trees=$work/trees
        trees_kept=0
        unset HW_LEAK_HOOK_CACHE
        mkdir -p "$trees" || return 3
        return 0
    fi
    trees_key=$(
        {
            printf '%s\n%s\n%s\n' "$root" "$head" "$all_probes"
            python3 "$root/tools/probe/declared.py" "$declaration" trees || exit 3
            for trees_input in "$engine" "$server" "$root/tools/probe/ablate.sh" "$root/tools/probe/seal.sh" \
                "$root/tools/probe/campaign-dry-run.sh" "$root/tools/probe/additive-dry-run.sh" \
                "$root/tools/probe/dry-run-trees.sh"; do
                sha256sum < "$trees_input" || exit 3
            done
        } | sha256sum | cut -c1-16
    ) || return 3
    [ -n "$trees_key" ] || return 3
    mkdir -p "$HW_CAMPAIGN_TREE_CACHE" || return 3
    for trees_other in "$HW_CAMPAIGN_TREE_CACHE"/*; do
        [ -d "$trees_other" ] || continue
        [ "${trees_other##*/}" = "$trees_key" ] || rm -rf "$trees_other"
    done
    trees=$HW_CAMPAIGN_TREE_CACHE/$trees_key
    trees_kept=1
    mkdir -p "$trees" || return 3
    # The text the intent hook injects for each probe's task depends on the
    # tree and the task alone, and `tools/probe/leak.py` keeps it here.
    HW_LEAK_HOOK_CACHE=$trees/hook-text
    export HW_LEAK_HOOK_CACHE
}

# The present tree: the `git archive` of `HEAD`, this checkout's engine copied
# in, `ablate.sh --present`, and `seal.sh` over every probe the spec selects.
# It is built under another name and moved, so a build that stopped half way
# is never read as a tree.
trees_present() {
    [ -d "$trees/present" ] && return 0
    rm -rf "$trees/present.building"
    mkdir -p "$trees/present.building/engine/target/dev-release" || return 3
    git -C "$root" archive "$head" | tar -x -C "$trees/present.building" || return 3
    cp "$engine" "$trees/present.building/engine/target/dev-release/headwater" || return 3
    sh "$root/tools/probe/ablate.sh" --present "$trees/present.building" > /dev/null || return 3
    # shellcheck disable=SC2086
    sh "$root/tools/probe/seal.sh" "$trees/present.building" $all_probes > /dev/null || return 3
    mv "$trees/present.building" "$trees/present" || return 3
}

# A command whose output depends on the trees alone. Its standard output and
# standard error are kept together, as every caller read them.
trees_memo() {
    trees_name=$trees/$1
    shift
    if [ ! -f "$trees_name.status" ]; then
        "$@" > "$trees_name.out" 2>&1
        printf '%s\n' "$?" > "$trees_name.status"
    fi
    cat "$trees_name.out"
    return "$(cat "$trees_name.status")"
}

trees_drop() {
    [ "$trees_kept" = 1 ] && return 0
    for trees_name in "$@"; do
        rm -rf "${trees:?}/$trees_name" "$trees/$trees_name".*
    done
}
