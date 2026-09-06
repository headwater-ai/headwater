---
id: HW-OBL-0165
status: current
status_since: 2026-09-06
summary: "The CI step that holds every rendered page to a corpus pointer exits 0 over a served directory with no page in it"
last_verified: 2026-09-06
title: "A rendered-page gate greps for a string it requires and states no denominator, so an empty build passes it"
waiting_on: adopter
---

# A rendered-page gate greps for a string it requires and states no denominator, so an empty build passes it

## Context

`.github/workflows/ci.yml:360` carries the step named "Every rendered page points at the corpus descriptor". It is a blocking step and it is the only reader of that property.

```sh
cp .headwater/corpus.json .headwater/site-build/corpus.json
missing=$(grep -rL 'rel="describedby"' .headwater/site-build --include='*.html' | grep -v '/404\.html$' || true)
if [ -n "$missing" ]; then
```

`grep -rL` names the files that lack the string. Over a directory with no file in it, `grep -rL` names nothing and exits 1, `|| true` absorbs the status, `missing` is empty, and the step passes. The step never states how many pages it read.

`tools/check-site-fragments.py` states the opposite posture about the same served directory. Its own header says that a suite that cannot tell "nothing is wrong" from "nothing ran" is not a gate. That file exits 2 over an empty root rather than reporting a clean run.

## Obligation

**A build that wrote no page passes this step, and the passing run looks the same as a correct one.** Provoked in the shell GitHub Actions runs a `run:` block in, over three arms of one scratch directory:

```
an empty served directory      exit 0
one page that carries it       exit 0
one page that does not         exit 1, and it names the page
```

The first two arms are indistinguishable from the outside. Only the third arm proves that the step reads anything.

**A missing directory is caught, and by the wrong line.** The `cp` above the grep fails when `.headwater/site-build` is absent, and `bash -e` ends the step there. So the step refuses the absent case as a side effect of copying a file into it. It refuses nothing about the empty case, which is the case a broken build actually produces.

**The population this step protects is 303 served pages.** A silent pass over zero of them would carry through a release. No later step reads the same property, and the deploy reads the directory rather than the report.

## Discharge

**This record discharges when the step states a non-zero denominator and fails without one.** One line does it. Count the pages the walk reached with `grep -rl`, and refuse a count of zero, in the shape `tools/check-site-fragments.py` already uses over the same tree.

**A stronger discharge moves the property into that file.** It already walks every served page, parses each one, refuses an empty or a missing root, and prints its own denominator. [#567](https://github.com/headwater-ai/headwater/issues/567) added a second property to it for those four reasons. A third property there costs no CI step and inherits the refusal.

**What does not discharge this.** A green CI run, which is what the step has reported on every build so far. A comment beside the step that states the assumption, because the run would still pass without it.
