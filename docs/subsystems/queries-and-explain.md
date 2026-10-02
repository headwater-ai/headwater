---
id: HW-SPEC-queries-and-explain
status: current
status_since: 2026-10-01
summary: "How two crates turn the census and graph of a run into route pointers, explanations and MCP answers, and pin an offline embedding model."
last_verified: 2026-10-01
title: "Queries and explain"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: measure+draft+revise
  evidence_basis: evidenced
relations:
  governs:
    - engine/crates/query/src/**
    - engine/crates/embed/src/**
  traces_to:
    - HW-SPEC-engine-architecture
    - HW-SPEC-ai-integration
    - HW-SPEC-check-layer
    - HW-SPEC-taxonomy-model
    - HW-IFACE-headwater-route
    - HW-IFACE-headwater-explain
    - HW-IFACE-headwater-show
    - HW-IFACE-headwater-mcp
    - HW-IFACE-headwater-neighbors
    - HW-IFACE-headwater-query
    - HW-DR-0064
    - HW-DR-0070
    - HW-DR-0071
    - HW-DR-0074
---

# Queries and explain

## Scope

This spec describes the inside of the queries and explain stages of [spec 6](../spec/06-engine-architecture.md#pipeline). Two crates under `engine/crates/` build these stages: `query` and `embed`.

The stages have two inputs. The first is the census that [Parse and census](parse-and-census.md) takes. The second is the graph that [Graph build](graph-build.md) builds from that census. The stages give three outputs: pointers, explanations, and the answers of the MCP server. They add no phase to a run. Each answer is a projection of the census and the graph that the run already holds.

The `embed` crate is the offline embedding path of [HW-DR-0064](../decisions/0064-q64-whether-intent-time-routing-gains-an-offline-embedding-path-in-shadow-mode.md). It runs in shadow mode, so no route reads a vector and no agent reads one.

The bodies of the `route`, `neighbors`, `explain` and `show` verbs are in `engine/crates/cli/src/main.rs`. They belong to the Command surface row of spec 6. This spec describes the library that those verbs call.

Other documents state what the stages do, and this spec does not repeat them:

- [Spec 6](../spec/06-engine-architecture.md#library) states the library promise, and its [MCP server](../spec/06-engine-architecture.md#mcp-server) section states the promise of the server.
- [Spec 5](../spec/05-ai-integration.md#intent-time-routing) states how routing behaves, and its [agent surfaces](../spec/05-ai-integration.md#agent-surfaces) section lists the tools of each class.
- [Spec 2](../spec/02-taxonomy-model.md#kind-resolution) states what `explain` prints.
- The contracts of [`headwater route`](../interfaces/headwater-route.md), [`headwater explain`](../interfaces/headwater-explain.md), [`headwater show`](../interfaces/headwater-show.md), [`headwater mcp`](../interfaces/headwater-mcp.md), [`headwater neighbors`](../interfaces/headwater-neighbors.md) and [`headwater query`](../interfaces/headwater-query.md) state the behavior of each verb.

The public Rust API of each crate is not in this spec ([HW-DR-0098](../decisions/0098-an-engine-subsystem-is-described-by-a-technical-design-spec-on-a-shelf-of-its-own-and-its-behavior-stays-where-it-is-already-written.md)). The doc comments of the crates state it.

## Design

### A read of what the run built, and never of content

`Surface` borrows the census, the graph, the resolved taxonomy and the relation declarations. It does not own a copy of them. A surface that copied them could answer about a corpus that no run evaluated.

Spec 5 names the tools of the query class: `route`, `governing_docs_for_path`, `resolve_identifier`, `related`, `explain`, `check` and `kinds`. The first five are reads in this crate. `check` is the runner of the check layer, and this crate calls the runner and does not implement it again. `kinds` reads the resolved taxonomy and not the graph. The `kinds` module holds its one renderer, and `headwater taxonomy kinds` calls the same renderer. So the tool and the verb give the same bytes for one lock.

Spec 6 also lists `headwater query <expression>`, and no document states what an expression is. Thus the crate has no `query` function. [HW-OBL-0029](../obligations/0029-what-headwater-query-takes.md) carries that gap, and the [`headwater query`](../interfaces/headwater-query.md) contract states the refusal.

A read gives a `Pointer` and never a body. A pointer carries a path, an identifier, a kind, a name, a purpose, a summary and a warrant flag. The name comes from the facet in the `name` role, and the summary comes from the facet in the `scent` role. The crate reads a facet role and never a facet name, because `title` can mean something different in another taxonomy. A traversal gives a `Neighbour`: the far end of one edge and the cue at that end. Where the far end is an anchor, it also gives the reach of the anchor.

### Determinism

The crate holds three rules so that a read is byte-identical across runs. Every score is an integer. Every order is total, and the last key is the path. No read uses a clock, an environment variable or a network. [Spec 12](../spec/12-check-layer.md#determinism-concretely) states the bar that these rules satisfy.

### Route

`route.rs` turns a task into a ranked set of pointers in the three steps that spec 5 states. First it matches the declared purpose of each kind against the task. Then it ranks the documents under each matched purpose by the terms of the task. Then reading precedence breaks ties.

A term scores only where it separates. `weights` gives each term the number of documents that it rules out. A term that every document carries weighs nothing, so the engine needs no word list for one language. A corpus that declares one purpose has no alternative to separate, so there every term counts.

The matched purposes take turns at the budget ([HW-DR-0070](../decisions/0070-the-matched-purposes-take-turns-at-a-route-budget-and-each-pointer-states-what-reached-it.md)). `in_turns` and `taken_in_turn` give each purpose its best remaining candidate in turn, and the kinds of one purpose take turns the same way. Before this rule, on this repository, the purpose with the highest score took every slot, and the answering specification part was past rank 100.

The gate is a term that reaches the document as well as its purpose. A task that matched a purpose and no document gives `Silence::NoDocumentReached`. `Silence` keeps four reasons apart, because each one is a different fact about the corpus. `Silence::token` is the stable name that a JSON reader compares, and `Silence::name` is the sentence for a person.

A route reads four surfaces of each document: the summary, the facet values, the path, and the anchors that the edges of the document reach. It never opens the body, and it offers a document and never a heading in it ([HW-DR-0071](../decisions/0071-a-route-offers-a-document-and-never-a-heading-inside-it.md)). `Surfaces` keeps the four apart, because a term in the summary weighs more than the same term in the path.

A task can name a path. `readings` reads a word in the forms that a person writes. An example is a path at the end of a sentence, or a path with a line number after it. A document that governs a named path is carried with `Evidence::Named`, outside the budget, because an anchor named it and no ranking chose it. The [route contract](../interfaces/headwater-route.md) states that the budget never cuts such a pointer. A ranked pointer carries `Evidence::Ranked` with the terms that reached it and its rank, and never a score or a confidence. `Route::withheld` counts the ranked pointers that the budget removed. `Ungoverned` names a path of the task that the governed scope admits and that no edge governs.

The crate reads no file. The verb gives `route_in` a function that answers `Entry::File`, `Entry::Directory` or `Entry::Absent` for a path, and the MCP read tools answer `Absent` for each path. [HW-OBL-0061](../obligations/0061-an-anchor-resolver-normalizes-and-nothing-states-how.md) is open because no document states how the anchor resolver normalizes a path. [HW-OBL-0006](../obligations/0006-route-latency-at-a-harvesting-tier-is-unmeasured.md) is open because nobody has measured the latency of a route at a harvesting tier.

### Explain

`explain.rs` builds an `Explanation`. Its members hold the list that spec 2 gives for `headwater explain`: the derivation, the purpose, the required facets and sections, and the permitted relations. They also hold the summary, the warrant and the edges that the document already has. `Permitted` is one relation that the kind may declare, with the kinds that it may reach.

The derivation is the one that the census recorded, and `explain` does not resolve the kind again. A second resolution could give a different answer from the one that the run used. An untyped document also has an explanation, in the same shape, with no kind and so no requirements. A reader needs that case explained more than any other.

The [`headwater show`](../interfaces/headwater-show.md) verb finds a document through the same resolver as `explain`, so the two verbs accept the same spellings of a path.

### JSON

`json.rs` writes a `Route` and an `Explanation` as JSON documents. The MCP `route` tool sends the route document as its structured content. Three rules hold for each document that the module writes:

- `VERSION` is the version of the shape of the document and not the version of the engine. A reader outside this repository has no clone of the engine, so the bytes must say what they are.
- An absent value is an absent member, because the JSON writer has no `null`. A member that is ambiguous when absent is written on each run, and `pointers` is one of them.
- Each function destructures its source completely. A field added to a source type and not to the document does not compile.

`explain_bounded` cuts the `paths` list of each reach to a bound and never cuts a count. The edit hook passes that bound. [Spec 6](../spec/06-engine-architecture.md#performance-targets) states the performance target that the bound serves.

### MCP

`mcp.rs` is the server of [spec 6](../spec/06-engine-architecture.md#mcp-server). Its tools call the reads beside them and render with the functions of the CLI. Thus a client and a terminal read the same text.

`QUERY_CLASS` holds the six read tools, and `WRITE_CLASS` holds `new` and `fix`. `registered` decides which tools exist. A server with no write switch registers the six reads only, and no table holds a landed write. [Spec 5](../spec/05-ai-integration.md#what-the-server-may-do-and-the-axis-that-decides-it) states why the registration carries the property and the `readOnlyHint` annotation does not. [HW-OBL-0004](../obligations/0004-working-tree-write-tools-have-no-measured-effect.md) records that nobody has measured the effect of the write tools.

The server walks the corpus once, before it accepts a message, and each tool answers from that walk. `Server` does not change after it starts. `Session` holds the one bit that changes: whether a call moved a byte of the checkout. After such a call, `respond` refuses each later `tools/call` by name with the reason. A `fix` that wrote no patch, or a `new` that the scaffolder refused, is not a write, and the session stays open.

`check` gets its three inputs and chooses none of them. The clock is the one that the server read when it started. The run uses no cache, because a cache write is a write. The corpus is the walk that the server already holds. The format is a required argument with no default, so the answer is byte-identical to `headwater check --format <name>`.

`Writing` holds the two write verbs as functions that the caller supplies, so a tool call runs the same verb as the terminal. `serve` reads one JSON object per line and writes one per line, which is the MCP stdio framing. `respond` takes the text of one request and gives the text of one response, so a test needs no process.

### The embedding path

`embed` holds the four rulings of [HW-DR-0064](../decisions/0064-q64-whether-intent-time-routing-gains-an-offline-embedding-path-in-shadow-mode.md), and each ruling has a type. Three constants name where its files are. `PIN` is the committed pin. `MODELS` is the ignored directory for the model files, and `CACHE` is the ignored directory for the vectors.

`Pin` reads the committed pin, which names each model file by URL and digest. `Pin::digest` covers each pinned file, so a second model or a second vocabulary gives a second digest. No crate here opens a socket, and the caller fetches the files.

`Model::load` reads each file through `verified`, which refuses a file whose digest is not the pinned one. A model file that fails its digest stops the run, as the [`headwater neighbors`](../interfaces/headwater-neighbors.md) contract states. Inference is pure Rust, through `tract-onnx`. `similarity` is the dot product of two unit vectors, which is their cosine.

`Cache` keeps the vectors for one model digest, keyed on the digest of each text. A vector is a fact about this machine and not about the tree, because quantized arithmetic differs between instruction sets. Thus no emitter writes a vector. A missing or malformed cache file is an empty cache and never an error, because each entry can be computed again.

`wordpiece.rs` is the BERT uncased WordPiece tokenizer that the pinned model uses. It is in this crate because the default build of the `tokenizers` crate links a C regular-expression library. It lowercases the text and removes accents before it splits the text, as the configuration of the model states.

## Invariants

A change to these crates must keep each of these. A test holds each one that names a test.

- **The fixture tree gives the recorded reads** (`the_fixture_tree_answers_the_recorded_reads` in `engine/crates/query/tests/reads.rs`), **and two reads of one corpus are byte-identical** (`two_reads_of_one_corpus_are_byte_identical`).
- **A route ranks no more pointers than its budget and never cuts an anchor** (`a_route_ranks_no_more_than_its_budget_and_never_cuts_an_anchor`, `a_route_carries_every_governing_document_whatever_the_budget`).
- **A pointer carries the declared summary and no body** (`a_pointer_carries_the_declared_summary_and_no_body`), **and states an `asserted` warrant** (`a_pointer_to_an_asserted_document_states_the_warrant`).
- **A route says how many ranked pointers the budget withheld** (`a_route_says_how_many_ranked_pointers_the_budget_withheld`).
- **The anchor resolver and the path tools agree on each spelling that both accept** (`the_anchor_resolver_and_the_path_tools_agree_on_every_spelling_both_accept`).
- **A session runs to the recorded transcript** (`the_session_runs_to_the_recorded_transcript` in `engine/crates/query/tests/mcp.rs`). The other cases of that file hold the registration, the tool arguments, the check tool and the path tools.
- **No tool outside the query class is registered without the switch** (`no_tool_outside_the_query_class_is_registered_without_the_switch`), **and the switch registers no landed write** (`the_switch_registers_two_tools_and_no_landed_write`).
- **A call that moved a byte ends the session** (`a_call_that_moved_a_byte_ends_the_session`), **and a session writes nothing to the corpus that it reads** (`a_session_writes_nothing_to_the_corpus_it_reads`).
- **The check tool gives the bytes that the CLI gives** (`the_check_tool_answers_the_bytes_the_cli_answers`).
- **A file that is not the pinned bytes is refused before anything loads** (`a_file_that_is_not_the_pinned_bytes_is_refused_before_anything_loads` in `engine/crates/embed/src/lib.rs`).
- **The model digest moves with a file digest and not with a URL** (`the_model_digest_moves_with_a_file_digest_and_not_with_a_url`). The other unit tests of that file hold the pin, the stamp and the cache.
- **The tokenizer splits a word into the longest pieces that the vocabulary holds** (`a_word_splits_into_the_longest_pieces_the_vocabulary_holds` in `engine/crates/embed/src/wordpiece.rs`). The other unit tests of that file hold the accents, the unknown token and the limit.
