---
id: HW-IFACE-headwater-json
status: current
status_since: 2026-09-06
summary: "How one member of the JSON object on standard input reaches a caller, and the one answer that stands for every way a read reaches no scalar."
last_verified: 2026-09-06
title: "headwater json"
relations:
  governs:
    - engine/crates/cli/src/main.rs
    - engine/crates/yaml/src/json.rs
---

# headwater json

## Synopsis

    headwater json field <key>...
    headwater json count [<key>...]
    headwater json quote

The command reads one JSON object on standard input. `field` and `count` write one line to standard output. `quote` writes one JSON string literal with no newline after it.

## Description

`headwater json` is the one verb of this binary that reads no corpus. A harness hands a hook one JSON object on standard input. A hook that read it alone would need an interpreter that nothing else in a session requires. [HW-DR-0055](../decisions/0055-a-hook-reads-a-wire-format-through-the-engine-and-not-through-an-interpreter.md) rules that the engine answers this and that no `headwater hook <moment>` verb exists.

`field` takes a path of keys, outermost first, and prints the member at the end of it. `headwater json field tool_input file_path` prints the `file_path` member of the `tool_input` member. A string arrives with its escapes resolved, a number as it was written, and a boolean as `true` or `false`.

`count` prints how many elements the array or the object at that path holds. It is the read `field` cannot do. An empty array and an absent member both give a caller nothing back through `field`, and they are different facts about a message.

`quote` reads standard input whole and writes it back as one JSON string literal. A caller needs it to put a path or a report inside the object it writes to a harness.

The reader is the YAML loader of this engine, because JSON is a subset of the YAML 1.2 core schema that the loader implements. The writer is the one that every JSON this engine emits is written with.

## Preconditions

Standard input carries one JSON document and is text. The verb reads it to the end, so a caller that holds the stream open holds the verb open.

## Options

| Option | What it does |
|---|---|
| `<key>...` | The path to the member, outermost first. `field` requires at least one. `count` with none counts the document on standard input itself. |
| `--root <path>` | Accepted for the global parser. This verb reads nothing under it. |
| `--no-color` | Force plain text on both streams. The artifact carries no color at any setting, and the account on standard error does. |
| `--no-banner` | Accepted and does nothing, since only the root help screen prints a masthead. |
| `--wide` | Refused. This verb renders no help of its own. |

## Exit status

**0** when the read reaches a scalar for `field`, an array or an object for `count`, or text for `quote`. The artifact is on standard output and standard error is empty.

**1** when the read reaches nothing. One answer stands for six states. They are a document that does not parse, and a step of the path that is not an object. The other four are an absent key, an array, an object, and a null. Standard output is empty and standard error names the path. A caller that told the six apart would act on the shape of a message it did not write.

**1** when standard input is not text, when a second word is absent, and when `field` is given no key. A second word this verb does not carry is refused the same way.

## Environment

No environment variable reaches this verb. `NO_COLOR` and `HEADWATER_NO_BANNER` reach the global parser as they do for every verb.

## Files

None. The verb reads standard input and writes standard output, and it opens no file under the repository root.

## See also

[HW-DR-0055](../decisions/0055-a-hook-reads-a-wire-format-through-the-engine-and-not-through-an-interpreter.md) rules why a verb answers a wire format and an interpreter does not.

[Spec 5](../spec/05-ai-integration.md#the-hook-contract-and-what-a-hook-cannot-bind) carries the hook contract, whose third term this verb answers to.

[`headwater route`](headwater-route.md) writes the document that the intent position reads back through this verb.
