#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# Stub: prints the four headings and nothing else. The decisive fixture must
# fail against this.
import sys

out = sys.argv[2]
with open(out, "w", encoding="utf-8") as f:
    f.write("## Touched\n\n## Stale\n\n## Owed\n\n## Unmeasured\n")
