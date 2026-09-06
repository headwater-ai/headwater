---
id: HW-DR-0055
status: current
status_since: 2026-09-06
summary: "A harness payload is read by a verb of this engine rather than by an interpreter a session does not otherwise require. The verb answers nothing about a corpus, which is what puts it outside the term that forbids a hook verb."
last_verified: 2026-09-06
title: "A hook reads a wire format through the engine and not through an interpreter"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  governs:
    - .claude/hooks/lib.sh
    - .claude/hooks/intent.sh
    - .claude/hooks/review.sh
---

# A hook reads a wire format through the engine and not through an interpreter

## Context

A harness hands a hook one JSON object on standard input. Each of the three positions reads a member of it. The intent position reads the prompt, the write position reads the moment and the path, and the review position reads the re-entry guard. `.claude/hooks/lib.sh` read all of them by a call to `python3`, and one more call in `.claude/hooks/intent.sh` read the answer of `headwater route --json`.

[HW-OBL-0146](../obligations/0146-the-stop-hook-reads-its-re-entry-guard-with-an-interpreter-it-does-not-require-so-a-machine-with-no-python3-re-blocks-the-same-turn.md) priced that dependency. On a machine with no `python3` the review position cannot read `stop_hook_active`. The read resolves to an empty string and the guard is gone. The position then runs the commit gate on the second stop as well as on the first. A failing tree therefore stops the turn, and stops the re-entered turn, with no exit from the loop. Every other position degrades into silence, which the [hook contract](../spec/05-ai-integration.md#the-hook-contract-and-what-a-hook-cannot-bind) makes an outcome.

The engine is the one program a session running these hooks already has. It parses JSON today: `engine/crates/yaml/src/json.rs` states that the reader is the YAML loader, because JSON is a subset of the YAML 1.2 core schema. The MCP server reads every request through that loader, over standard input. The matching writer in the same file is what one line of the interpreter reimplemented.

Three answers were open, and the hook contract constrains all three. Its third term forbids a `headwater hook <moment>` verb, and the reason it gives is drift. Two entry points to one answer are two answers as soon as one drifts.

## Decision

`headwater json` reads the JSON object on standard input. `field` prints one member, addressed by a path of keys. `count` prints how many elements the array or the object at a path holds. `quote` writes standard input back as one JSON string literal. The three hooks call these and call no interpreter.

**The verb is outside the third term rather than inside an exception to it.** It answers nothing about a corpus, so no second answer exists for it to drift from. It names no moment, so no position of a harness is spelled anywhere in the engine. A harness field name stays in the hook, which is the file that knows which harness it serves.

**A tool such as `yq` or `jq` is refused for the reason the interpreter is refused.** Either one is a dependency beyond the engine, and neither is installed as widely as the interpreter that this ruling removes. A parser written in the shell is refused as well. It is a second implementation of a format the engine already reads, and it is the weaker of the two.

**An input form on `route` and on `explain` is refused because of what it moves.** Those verbs would then carry the field names of three harnesses, which is a fact about a client rather than about a corpus. The review position reads a payload and calls no verb of the engine, so no input form reaches it at all.

**The verb prints its member and nothing else, and it exits non-zero where the read reaches no scalar.** Six states share that one answer. They are a document that does not parse, and a step of the path that is not an object. The other four are an absent key, an array, an object, and a null. A caller that told them apart would act on the shape of a message it did not write.

**Segregation is the group and not the name.** The verb prints under a heading of its own, which is the mechanism the first screen already carries. A hidden verb is refused for three reasons. `engine/crates/cli/tests/verbs.rs` holds the parsed command tree against `headwater_verbs::VERBS` in both directions. `headwater generate` writes the verb index out of the same table. A name a reader cannot discover is a name nobody types.

## Consequences

The three hooks require a built engine at every position. Two of them required one already. The review position requires one for the guard, and it exits 0 where it has none. A machine with no engine therefore ends the turn rather than stopping it twice. The commit gate holds that tree, and the gate prints its own line about a missing engine.

**The intent position spawns one process more than it did, and it costs less than it did.** Spec 5 states that a hook is invisible or it is bypassed. The count is therefore the wrong measure, and the clock is the right one. Ten reads of one member cost 0.077 seconds through this verb and 0.698 seconds through the interpreter, measured on an idle machine on 2026-09-06. That is 8 milliseconds against 70 for one read. The position now makes three calls where it made two, and the three are faster than the two by a factor of six.

`headwater json` is the first verb of this binary that reads no corpus. It takes `--root` because every verb does, and it reads nothing under it. A later utility joins the group rather than becoming a second bucket.

The interpreter stays where a session never meets it. `.claude/tutorial/fixtures.sh` and the two oracles under `engine/crates/adapter/tests/` and `engine/crates/generate/tests/` run in CI. Each oracle is written in another language on purpose, so that a defect in an emitter cannot hide behind the matching defect in a checker. The site tools under `tools/` are a build path and not a session path, and this ruling reaches neither.

`.claude/hooks/fixtures.sh` drives every position under a `PATH` with no interpreter on it. The pair that HW-OBL-0146 asks for is there. A second stop under that `PATH` exits 0, and a first stop over a tree the gate refuses exits 2.
