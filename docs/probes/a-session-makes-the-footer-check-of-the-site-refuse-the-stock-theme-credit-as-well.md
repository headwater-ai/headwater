---
id: HW-PROBE-a-session-makes-the-footer-check-of-the-site-refuse-the-stock-theme-credit-as-well
status: current
status_since: 2026-10-30
summary: "A session changes a script that no document governs, so that the footer check also refuses the text `Material for MkDocs`, and a held check that runs the script decides done."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: patched
oracle: "command:r11-site-footer"
title: "A session makes the footer check of the site refuse the stock theme credit as well"
action: ungoverned_control
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-DR-0112
---

# A session makes the footer check of the site refuse the stock theme credit as well

## Task

### Phrasing 1

`tools/site/check-site-footer.sh` refuses a served page that carries the stock MkDocs footer, `Documentation built with MkDocs`. Make it also refuse a page that carries the theme credit text `Material for MkDocs`. A page with either text fails the check, and a clean site still passes.

### Phrasing 2

The footer check of the site, `tools/site/check-site-footer.sh`, catches the plain MkDocs footer and not the Material theme's credit line. Extend it so that any served HTML page holding `Material for MkDocs` also makes it exit 1. Keep the check of the MkDocs footer and of the page count as they are.

## Target

`tools/site/check-site-footer.sh`, a script that no document governs.

## Key

`patched` over `command:r11-site-footer`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/held-check.sh` with a held file under `tools/probe/oracle-args/` that names a held check and the target. The check builds three scratch sites of three pages each, in four runs. It passes when the clean site exits 0, the site with the MkDocs footer exits 1, the site with the theme credit inside a link exits 1, and a minimum of four pages over the clean site exits 2.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r11-site-footer.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change adds the theme credit as a second string to the one search. The wrong change replaces the MkDocs footer with the theme credit, so the site with the MkDocs footer passes, and it fails. A script written beside the target fails, because the check runs the target. A change to the header alone fails. The grading tree as it stands fails, because the site with the theme credit exits 0.

**Nothing governs the target.** `headwater route "tools/site/check-site-footer.sh"` printed `tools/site/check-site-footer.sh is in the governed scope, and nothing governs it` and no `governs` line on 2026-10-30. The same held for the route of phrasing 1. The target is not engine code, and it is not on `base_invalidators`.

**The control has a second condition.** HW-DR-0112 says that row 11 also wants a count of zero hook warnings. `tools/probe/oracles/no-hook-warning.sh` reads that from the harness log, and #1893 wires it to each session. This probe declares the tree oracle only.

**No moments.** The grid gives row 11 no hook moment, so the probe declares no `moments`. HW-OBL-0123 records that a facet cannot say that it applies to one action only.
