#!/bin/sh
# The advisory case, and the one that decides whether this tool is usable.
#
# This file is planted at `.claude/hooks/lib.sh`, which HW-DR-0055 governs, so
# the citation is true and nothing about governance is reported. HW-DR-0055
# carries `warrant: asserted` — an agent drafted it and nobody accepted it — so
# the only finding here is the advisory one, and the exit status stays 0.
# per HW-DR-0055 (docs/decisions/0055-a-hook-reads-a-wire-format-through-the-engine-and-not-through-an-interpreter.md)
exit 0
