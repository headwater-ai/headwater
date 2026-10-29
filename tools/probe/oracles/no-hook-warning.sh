#!/bin/sh
# Generic oracle: no hook spoke to the model in a session (#1874). Row 11 of
# the grid in HW-DR-0112, "ungoverned control": the task is done, and the
# count of hook warnings is zero.
#
#     sh tools/probe/oracles/no-hook-warning.sh <log>...
#
# This oracle reads the session's transcript and not its tree, so a row 11
# probe declares two oracles: one over the tree for "done", and this one.
# Each <log> is a file of the harness's own session log, or a directory that
# holds such files, `*.jsonl`, which is searched. That log, and not the
# stream the recorder reads, holds the hook attachments, as
# `tools/probe/recordings.py` states. Passing the log path of a session to
# this script is a change to the recorder, which #1875 makes.
#
# A hook warning is a record of `"type": "attachment"` whose attachment is
# one of these:
#
# - `hook_additional_context`, context a hook added;
# - `hook_non_blocking_error` or `hook_blocking_error`, a hook that failed or
#   refused;
# - `hook_success` with text in its `content` or its `stderr`, which is how
#   the harness records what a hook such as `intent.sh` printed.
#
# A `hook_success` with no text is a hook that ran and said nothing, and it
# is not a warning. It passes when the logs hold at least one record and no
# hook warning. A log that holds no record fails, because a missing log must
# not read as a silent session.

set -u
. "${0%/*}/lib.sh"
[ $# -ge 1 ] || hw_oracle_usage "no-hook-warning.sh <log>..."
command -v python3 > /dev/null 2>&1 || hw_oracle_usage "no-hook-warning.sh needs python3 to read the log"
for log in "$@"; do
    [ -e "$log" ] || hw_oracle_fail "no log at $log"
done
hw_oracle_tmp
python3 -I - "$@" > "$hw_oracle_scratch/count" 2> "$hw_oracle_scratch/count.err" << 'PY'
import json
import os
import sys

WARNING = ("hook_additional_context", "hook_non_blocking_error", "hook_blocking_error")


def files(paths):
    for path in paths:
        if os.path.isdir(path):
            for top, _, names in sorted(os.walk(path)):
                for name in sorted(names):
                    if name.endswith(".jsonl"):
                        yield os.path.join(top, name)
        else:
            yield path


def text(value):
    return isinstance(value, str) and value.strip() != ""


records = 0
warnings = []
for path in files(sys.argv[1:]):
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            line = line.strip()
            if not line:
                continue
            try:
                record = json.loads(line)
            except ValueError:
                continue
            if not isinstance(record, dict):
                continue
            records += 1
            if record.get("type") != "attachment":
                continue
            att = record.get("attachment") or {}
            kind = att.get("type")
            name = att.get("hookName") or att.get("hookEvent") or "-"
            if kind in WARNING or (kind == "hook_success" and (text(att.get("content")) or text(att.get("stderr")))):
                warnings.append("%s from %s" % (kind, name))
print(records, len(warnings), warnings[0] if warnings else "-")
PY
read -r records count first < "$hw_oracle_scratch/count" || hw_oracle_fail "the log could not be read: $(sed -n '$p' "$hw_oracle_scratch/count.err")"
[ "$records" -gt 0 ] || hw_oracle_fail "the log holds no record"
[ "$count" -eq 0 ] || hw_oracle_fail "$count hook warning(s) in $records record(s), the first $first"
hw_oracle_pass "no hook warning in $records record(s)"
