#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# The share of input that the cache served, over the recorded sessions of one
# or more batch directories of a probe campaign (#1875).
#
#     sh tools/probe/cached-share.sh <batch>...
#
# A recorded session is a directory `<batch>/sessions/<id>/` whose `status` is
# 0. `tools/probe/campaign.sh` writes each session's `tokens` file beside its
# `cost`, from the `usage` of the `result` line of its `raw.jsonl`, through
# `probe-record.sh --provider-only`:
#
#     cache_read_tokens: <n>
#     cache_write_tokens: <n>
#     input_tokens: <n>
#
# The harness counts the three apart: `input_tokens` is the input the cache did
# not touch, `cache_write_tokens` is input written to the cache, and
# `cache_read_tokens` is input read from it. So the input of a session is the
# sum of the three, and the cached share is the cache reads over that sum. It
# prints one line:
#
#     cached share: <read> of <input> input tokens read from the cache over <n> recorded sessions, <p>%
#
# with `<p>` to one decimal, and adds `, <m> recorded sessions without a token
# record` when a recorded session has no `tokens` file, such as one recorded
# before #1875. With no recorded session that has a token record, it prints
# `cached share: no recorded sessions`. A batch directory that does not exist,
# or has no `sessions/`, adds nothing. `campaign.sh --assemble` prints the line
# for its batch, and the dry run of an additive spec prints it for the batch
# that `HEADWATER_CAMPAIGN_RECORDED` names. It always exits 0, because it is a
# report and not a gate.

read_n=0
input_n=0
counted=0
without=0
for batch in "$@"; do
    for dir in "$batch"/sessions/*/; do
        [ -d "$dir" ] || continue
        [ "$(cat "$dir/status" 2>/dev/null)" = 0 ] || continue
        if [ ! -s "$dir/tokens" ]; then
            without=$((without + 1))
            continue
        fi
        r=$(sed -n 's/^cache_read_tokens: \([0-9][0-9]*\)$/\1/p' "$dir/tokens" | head -1)
        w=$(sed -n 's/^cache_write_tokens: \([0-9][0-9]*\)$/\1/p' "$dir/tokens" | head -1)
        i=$(sed -n 's/^input_tokens: \([0-9][0-9]*\)$/\1/p' "$dir/tokens" | head -1)
        if [ -z "$r" ] || [ -z "$w" ] || [ -z "$i" ]; then
            without=$((without + 1))
            continue
        fi
        read_n=$((read_n + r))
        input_n=$((input_n + r + w + i))
        counted=$((counted + 1))
    done
done
tail=
[ "$without" -gt 0 ] && tail=", $without recorded sessions without a token record"
if [ "$counted" = 0 ]; then
    printf 'cached share: no recorded sessions%s\n' "$tail"
    exit 0
fi
share=$(awk -v r="$read_n" -v t="$input_n" 'BEGIN { if (t == 0) print "0.0"; else printf "%.1f", 100 * r / t }')
printf 'cached share: %s of %s input tokens read from the cache over %s recorded sessions, %s%%%s\n' \
    "$read_n" "$input_n" "$counted" "$share" "$tail"
