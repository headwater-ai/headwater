---
id: HW-OBL-0164
status: current
status_since: 2026-09-06
summary: "`generate --check` reads byte equality before it reads the marker, so a generated file the census dropped from the generated set still passes it"
last_verified: 2026-09-06
title: "The regeneration check tests byte equality before the marker, so a file the census stopped counting as generated still passes"
waiting_on: adopter
---

# The regeneration check tests byte equality before the marker, so a file the census stopped counting as generated still passes

## Context

`headwater generate --check` decides one verdict for each output it planned. The arms sit in `engine/crates/generate/src/lib.rs:940` and the engine tests them in the order they are written. The first arm is byte equality. The second arm is the generated-file marker.

```rust
(Some(text), _) if text == &output.bytes => Verdict::Unchanged,
// The marker decides before the difference does. A file that is
// there and unmarked is authored, and reporting it as drift would
// tell a reader to run the verb that destroys it.
(Some(text), _) if !headwater_mark::carries_marker(&output.path, text) => {
    Verdict::Occupied
}
```

The comment states that the marker decides before the difference does. The code tests the difference first. The comment is the intent and the order is the behavior, and the two disagree.

`headwater_mark::carries_marker` reads line 1 of a Markdown file that carries no front matter, and it reads nowhere else. `engine/crates/census/src/census.rs:342` reads the same function to classify a file as generated. So one position of one comment line decides the census classification and the regeneration verdict, and it decides them through two different readers.

## Obligation

**An emitter that wrote its marker anywhere but line 1 would leave both verbs green over a corpus that had lost its generated set.** The census would move the files to `untyped`. `generate --check` would still see matching bytes, so the first arm would fire, and no gate of this repository would report the move.

Measured on `0efe2ba`, by moving the marker below the heading in `docs/probes/README.md` alone and restoring it:

```
before:  11 generated, 2 untyped, 7 findings, check --strict exit 0, generate --check exit 0
after:   10 generated, 3 untyped, 7 findings, check --strict exit 0, generate --check exit 1
```

**The check layer reports none of that, and it cannot.** No `Outcome::Generated` appears anywhere in `engine/crates/check/src`, and `check/src/suppression.rs:230` reads `Outcome::Typed` alone. The run statistic held at 395 seen, 283 classified, 283 checked and 5194 check instances across both arms, and the finding count held at 7.

`generate --check` exited 1 in that arm for a reason the silent state does not carry. The committed bytes stopped matching what the emitter writes, so the arms were never the question. An emitter that also wrote the marker in the new position would produce matching bytes. The first arm would fire and the exit would be 0.

**The state after that is worse than the silence.** The first corpus edit that changes an index moves the output bytes, the second arm fires, and `Verdict::Occupied` refuses to rewrite the engine's own output. The doc comment on `Verdict::is_error` says that it fails a write and a check alike. So `headwater generate` freezes on a file it wrote, and it tells the reader that the file is authored.

**Nothing here is reached today.** Every emitter of this engine writes the marker on line 1. [#567](https://github.com/headwater-ai/headwater/issues/567) named a move of the marker as one of three candidate fixes for the shelf index titles. This record is the price of that candidate, measured rather than estimated. The change that closed #567 took a different route and left every marker where it was.

## Discharge

**This record discharges when a case pins the arm order.** The case plans one output, commits bytes that match the plan and carry no marker on line 1, and asserts that `generate --check` refuses. It fails on the order as written and passes on the reversed order. `engine/crates/generate/tests/fixtures.rs` is where it goes, beside the cases that already read a verdict.

**A second discharge makes the census classification visible to a gate.** A rule that reads the generated set turns a silent move into a red run. So does a recorded fixture that pins the `generated` and `untyped` counts. `engine/crates/census/fixtures/corpus.census` already records both numbers. The smaller version of this is a test that reads them rather than a new rule.

**What does not discharge this.** A comment that restates the intended order, which the second arm already carries and which is the half that went stale. A green `generate --check` over a corpus whose markers all sit on line 1, which is every run this repository has made.
