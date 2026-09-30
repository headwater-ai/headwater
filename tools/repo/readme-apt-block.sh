#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# readme-apt-block.sh [README] — print the APT install block of the page, as a
# reader who is already root runs it.
#
# It prints the lines of the first fence after the paragraph that opens
# `**Install it with apt`, with every `sudo ` removed, because the paragraph
# tells a reader in a container to remove it. It is the only copy of those
# commands outside the page: `.github/workflows/readme-apt.yml` runs what this
# script prints, and group 12 of `tools/repo/readme-fixtures.sh` holds that the
# workflow calls this script and carries no copy of its own (#1408).
#
# It exits 1 and prints nothing when the paragraph is missing, when no fence
# follows it, or when the fence is empty. The workflow depends on that refusal:
# `sh -e` of an empty script exits 0, so a paragraph that moved would turn the
# job green while it ran nothing.
#
# The fence test is the one `apt_fence_lines` in `tools/repo/readme-fixtures.sh`
# uses: a line that opens with optional blanks and three backquotes toggles it.

set -u

readme=${1:-$(cd "$(dirname "$0")/../.." && pwd)/README.md}

if [ ! -f "$readme" ]; then
    echo "readme-apt-block: no file at $readme" >&2
    exit 1
fi

block=$(awk '
    !seen && /^\*\*Install it with apt/ { seen = 1; next }
    !seen { next }
    /^[ \t]*```/ { if (fence) exit; fence = 1; next }
    fence { print }
' "$readme" | sed 's/sudo //g')

if [ -z "$(printf '%s' "$block" | tr -d ' \t\n')" ]; then
    echo "readme-apt-block: $readme has no fence with commands after the paragraph that opens \`**Install it with apt\`" >&2
    exit 1
fi

printf '%s\n' "$block"
