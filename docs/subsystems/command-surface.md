---
id: HW-SPEC-command-surface
status: current
status_since: 2026-10-01
summary: "A verb list that the help and the verb index read, a parser that dispatches, help folded at a fixed width, and one palette."
last_verified: 2026-10-01
title: "Command surface"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: measure+draft+revise
  evidence_basis: evidenced
relations:
  governs:
    - engine/crates/cli/src/**
    - engine/crates/verbs/src/**
    - engine/crates/paint/src/**
  traces_to:
    - HW-SPEC-engine-architecture
    - HW-IFACE-headwater-help
    - HW-IFACE-headwater-completions
    - HW-IFACE-headwater-json
    - HW-IFACE-headwater-merge-driver
    - HW-DR-0029
    - HW-DR-0033
    - HW-DR-0042
    - HW-DR-0043
    - HW-DR-0045
    - HW-DR-0074
    - HW-DR-0082
    - HW-DR-0098
---

# Command surface

## Scope

This spec describes the inside of the command surface of [spec 6](../spec/06-engine-architecture.md#subsystems). Three crates under `engine/crates/` build it: `cli`, `verbs` and `paint`. The subsystem runs no stage of the pipeline. It reads a command line, selects one verb, calls the crates that do the work, and writes the result on the two standard streams.

The subsystem has three parts:

- `verbs` holds the list of every verb and every second word, with the words that a caller reads about each one.
- `cli` holds the parser, the help, the color and width of the text, the body of each verb, and the Mermaid renderer of `headwater taxonomy graph`.
- `paint` holds the palette and the pure functions that apply it.

Other documents state what a caller gets, and this spec does not repeat them:

- The `### CLI` section of [spec 6](../spec/06-engine-architecture.md#cli) states the grammar of each verb, its exit status and the stream that it writes. Its [verb index](../spec/06-engine-architecture.md#a-verb-index-reads-the-command-surface-of-the-engine) section states the artifact that `headwater generate` writes from the verb list.
- The interface contract of each verb under `docs/interfaces/` states the behavior of that verb. The contracts of [`headwater help`](../interfaces/headwater-help.md), [`headwater completions`](../interfaces/headwater-completions.md), [`headwater json`](../interfaces/headwater-json.md) and [`headwater merge-driver`](../interfaces/headwater-merge-driver.md) describe four verbs whose body is a function in `main.rs`. The `json` body calls `headwater_yaml::json`, and the `merge-driver` body reads `headwater_census::derived::LOCK`.
- [HW-DR-0033](../decisions/0033-q33-whether-the-command-line-is-derived-and-who-a-flag-belongs-to.md) rules that the command line is derived and that a flag belongs to the verb that reads it. [HW-DR-0042](../decisions/0042-q42-what-one-screen-means-for-the-first-help-screen.md) rules on the first help screen. [HW-DR-0043](../decisions/0043-q43-whether-a-refusal-under-json-is-a-json-document.md) rules that a refusal under `--json` is one English sentence. [HW-DR-0045](../decisions/0045-coloring-the-cli-and-where-the-banner-goes.md) rules on the palette and the masthead.

Most verbs have their body in `engine/crates/cli/src/main.rs`, and that body calls a library crate of another subsystem. This spec describes how a command line reaches that body. The other subsystem specs describe the library. The public Rust API of each crate is not in this spec ([HW-DR-0098](../decisions/0098-an-engine-subsystem-is-described-by-a-technical-design-spec-on-a-shelf-of-its-own-and-its-behavior-stays-where-it-is-already-written.md)). The doc comments of the crates state it.

## Design

### One list of verbs, and a parser that dispatches

`headwater_verbs::VERBS` is a constant array of `Verb`. Each `Verb` carries a name, a group, a summary, a description and a slice of `Word`. A `Word` carries the same three texts for a second word. A verb with second words names one command line for each word and no bare one, so `headwater sweep` is refused and `headwater sweep plan` runs. `Verb::forms` gives these command lines.

The help, the refusals and the verb index read the list from this array. `command_in` in `cli/src/lib.rs` puts its summaries and descriptions on the `clap` command tree. `first_screen` prints its groups in the order that `headwater_verbs::groups` reads off the array. The refusals in `main.rs` name the legal words through `headwater_verbs::listed` and `headwater_verbs::words_of`. The verb index of `headwater generate` takes the array as an argument. One edit to the array thus moves the first help screen, the page of one verb and a committed artifact together.

The array does not decide which verb runs. The `clap` derive in `cli/src/lib.rs` declares `Cli`, the `Verb` enum and four enums of second words: `JsonWord`, `SweepWord`, `ProbeWord` and `TaxonomyWord`. `clap` builds the command tree from that derive, and `dispatch` in `main.rs` matches the parsed `Verb` exhaustively. A variant with no arm does not compile. `engine/crates/cli/tests/verbs.rs` walks the tree to its leaves and holds it against the array in both directions. Thus the list and the parser are two declarations, and a test, not a lookup, keeps them equal.

The list is a crate of its own because two crates read it. `headwater-generate` writes the verb index, and `headwater-cli` depends on `headwater-generate`. So the list cannot be in the binary, and a crate below both is the one position that serves both. The `Cargo.toml` of `verbs` declares no dependency. The module comment of `verbs/src/lib.rs` records the four copies of the list that disagreed before this crate existed.

### The parse, and the words a caller reads

A summary or a description of a verb is not written in the derive. `command_in` walks `VERBS` and calls `mut_subcommand` for each name that the tree carries. A name that the tree does not carry is skipped rather than added, and `tests/verbs.rs` reports it. A doc comment on a derived item becomes help text, so the comments on the derive types are `//` comments.

A flag is the opposite case. HW-DR-0033 rules that a flag belongs to the verb that reads it. So the description of a flag is written at its declaration in the derive. The global flags are in `GLOBALS`, a table of `Global` entries. Each entry carries a one-line summary for the first screen and a description that `clap` prints on every verb page. `first_screen` prints the summaries in its own template, because `clap` would print the whole description of each flag.

`parsed` builds the tree through `command` and parses the process arguments with it. `Cli::parse` would build a second tree from the derive alone, with no words on it. One entry point keeps the tree that parses and the tree that prints help the same. `parsed` also refuses `--wide` on a run that lays out no text, because the flag would do nothing there.

Each level of the tree declares an external subcommand. These are `Verb::Other` at the root and an `Other` variant in each enum of second words. An unknown word thus reaches `dispatch`, and its refusal names every word that the binary carries. A refusal from `clap` names one near miss at most. `headwater help` is a variant of `Verb`, and the derive sets `disable_help_subcommand`. The subcommand that `clap` injects would add a copy of the whole tree under `help`, and `tests/verbs.rs` would report each copy.

`chosen` in `main.rs` turns `--json` into the value `json` of `--format`, so no verb body can tell which spelling a caller typed. `typed` carries the spelling past that substitution, for a refusal that names the flag.

### Two streams and one failing status

`main.rs` defines its own `print!`, `println!`, `eprint!` and `eprintln!`. Each of them calls `emit`, which writes and flushes one stream. The standard macros panic when a write fails, and the process then exits 101, which a caller cannot tell from a defect. When a write to standard output fails, `unwritten` writes one sentence to standard error and unwinds with the `Unwritten` payload. When a write to standard error fails, `emit_bytes` unwinds with that payload and writes nothing.

`main` runs `entered` inside `catch_unwind`. An `Unwritten` payload becomes exit status 1. Any other payload is a real panic, and `main` resumes it, so it exits 101. The unwind drops each value between the failed write and `main`, and `std::process::exit` would drop none. A value such as the temporary directory of `headwater_fetch::Fetched` thus removes itself.

`clap` exits 2 on a parse error. `entered` uses `try_get_matches` through `parsed`, so `clap` never exits by itself. An error for which `clap::Error::use_stderr` is true goes to `fail`, which exits 1. The other errors are `--help` and the `clap` form of `--version`, and `clap` prints them to standard output with status 0. `headline` keeps the message of `clap` and drops its usage block, and `fail` adds the prefix and the pointer to the help.

### The width of the help, which no terminal decides

The workspace takes `clap` without its `wrap_help` feature, so `clap` folds no text. That feature measures the terminal, and a width that depends on the terminal makes a piped run and a terminal run write different bytes. Every recorded fixture would then read the terminal of whoever ran it.

`cli/src/paint.rs` folds every string before `clap` sees it. `painted` walks the whole tree, sets `next_line_help` on each command, and folds each string at the width that `width` gives. With `next_line_help`, the help of each argument starts at one indent, `INDENT`, at every node. One folded string is then correct wherever `clap` prints it. `reserved` measures the text that `clap` appends after a help string, such as `[possible values: …]`, so the fold keeps that suffix with the last word.

`width` gives `WIDTH`, which is 80, unless the raw arguments carry `--wide`. Only then does it read `COLUMNS`, and `width_of` holds the value in the band from `WIDTH` to `WIDEST`, which is 120. The scan reads the raw arguments, because the width is needed to build the tree that parses them. `WIDTH`, `WIDEST`, `fold` and `fold_at` are in `headwater_check::fill`, and `cli/src/paint.rs` re-exports them. The text report of `headwater check` is laid out by the same functions, so the report and the help have one layout.

`completions` builds the tree at `WIDTH` and in plain mode, and `flattened` puts each string back on one line. A completion script thus carries no fold and no escape byte, and its bytes do not change with the terminal.

### Color, which reads the terminal on purpose

HW-DR-0045 departs from the rule of the width on purpose. An escape sequence in a pipe or a log file harms the reader, and a fold never does. So `stdout_color` and `stderr_color` each read whether their own stream is a terminal, together with `--no-color` and `NO_COLOR`. They pass these three facts to `color_of`, which is a pure function in `paint`. `--no-color` and `--no-banner` are read from the raw arguments, as `--wide` is, because the tree is built before the parse.

The help gets its mode from `command_at`, which reads standard output. `command_in` takes the mode as a parameter, so a test and `completions` can state it. The masthead is printed by `entered` before the parse, and not in the template. `wants_root_help` scans the raw arguments for `-h` or `--help` with no word that names a verb. `banner` then writes the masthead in the mode of standard output.

`paint` is a leaf crate, and its `Cargo.toml` declares no dependency. `headwater-census`, `headwater-check` and `headwater-resolve` depend on it directly. The `[dependencies]` of `headwater-check` hold `headwater-census` and `headwater-resolve`. Cargo refuses a cycle, so a palette in `headwater-check` was out of reach of a renderer in either crate. The module comment of `paint/src/lib.rs` records that move. `headwater-check` keeps the functions that read its `Severity` type and re-exports the rest as `headwater_check::paint`. `cli/src/paint.rs` re-exports the palette from `headwater_check::paint`, and `headwater-cli` declares no direct dependency on `headwater-paint`.

`paint` and `dim` take a `ColorMode` and return a `String`. `color_of` takes three booleans and returns a `ColorMode`. No function in `paint` opens a stream or reads an environment variable. `ColorMode::Plain` writes the text back with no escape sequence. This purity lets a case table test each renderer with no terminal. It also lets a call site pass `Plain` where it should pass the mode of its stream, and no headless test sees that defect. [HW-OBL-0180](../obligations/0180-a-renderer-s-color-mode-is-wired-at-a-call-site-that-no-type-forbids-from-being-wrong.md) records it. `tools/engine/color-fixtures.sh` attaches a pseudo-terminal and counts escape bytes. The module comment of `paint/src/lib.rs` names it as the one check that sees this defect.

### The drawing of the taxonomy

`cli/src/taxonomy_graph.rs` renders the resolved taxonomy as a Mermaid flowchart for `headwater taxonomy graph`. [HW-DR-0082](../decisions/0082-the-resolved-taxonomy-is-drawn-by-a-verb-that-prints-mermaid-and-writes-no-file.md) rules that a verb prints it and that no projection writes it. `render` takes the `resolved` block of the lock, and the package name and version of the lock for the title. The file names no kind, purpose or relation, so a release that adds a kind moves the drawing with no edit here.

`View` selects one of two drawings. The concrete view answers which kind can relate to which, and it draws no edge with an abstract kind at either end. The abstract view answers what an abstract kind gives to the kinds under it. `split_loops` takes each relation from a kind to itself out of the edges, and the view writes it on the node. Mermaid draws such an edge as a long detour. `PALETTE` holds eight stroke colors. The renderer gives them to the families in sorted order, and a ninth family takes the first color again. The output is sorted and carries no clock and no digest, so two runs over one lock write the same bytes.

### The body of each verb

`dispatch` sends each `Verb` to a function in `main.rs`. That function reads what its verb needs, calls the library crates that do the work, and writes the result through `emit`. [HW-OBL-0138](../obligations/0138-the-rest-of-the-editorial-pass-with-cli-src-main-rs-at-the-head-of-the-distribution.md) records that `main.rs` is the file at the head of the editorial pass that the corpus still owes.

## Invariants

A change to these crates must keep each of these. A test holds each one that names a test.

- **The parser answers to exactly the command lines that the verb list carries** (`the_parser_answers_to_exactly_the_command_lines_the_table_carries` in `engine/crates/cli/tests/verbs.rs`).
- **The verb index names exactly the verbs that the binary dispatches** (`the_verb_index_names_exactly_the_verbs_this_binary_dispatches` in `engine/crates/cli/tests/verbs.rs`).
- **The verbs of one group are contiguous in the list** (`the_verbs_of_one_group_are_contiguous` in `engine/crates/verbs/src/lib.rs`), **and no two verbs carry one name** (`no_two_verbs_carry_one_name`).
- **Every argument that the parser admits says what it is** (`every_argument_the_parser_admits_says_what_it_is` in `engine/crates/cli/tests/help.rs`), **and every global flag is one line on the first screen** (`every_global_flag_is_one_line_on_the_first_screen`).
- **The long description of a verb is the same through `help <verb>`, `--help` and `-h`** (`the_long_description_of_a_verb_is_reachable_three_ways` in `engine/crates/cli/tests/help.rs`).
- **A failed write makes no verb panic** (`a_reader_that_closes_early_does_not_make_a_verb_panic` in `engine/crates/cli/tests/unwritable.rs`).
- **No line of the help is wider than 80 columns** (`no_line_of_the_help_surface_is_wider_than_eighty_columns` in `engine/crates/cli/tests/width.rs`).
- **A run that states `COLUMNS` writes the same bytes as one that does not** (`a_run_that_states_columns_and_one_that_does_not_write_the_same_bytes` in `engine/crates/cli/tests/width.rs`).
- **Stripping the escape bytes from the painted help gives the plain help byte for byte** (`stripping_the_painted_help_gives_the_plain_help_byte_for_byte` in `engine/crates/cli/tests/width.rs`), **and no escape byte reaches a machine format** (`no_escape_byte_reaches_a_machine_format`).
- **`Plain` writes no escape sequence for any role** (`plain_writes_no_escape_sequence_for_any_role` in `engine/crates/paint/src/lib.rs`).
- **A completion script does not move with the terminal of the caller** (`the_script_does_not_move_with_the_terminal_of_whoever_asked_for_it` in `engine/crates/cli/tests/completions.rs`).
- **Two runs of `taxonomy graph` over one lock write the same bytes** (`two_runs_over_one_lock_write_the_same_bytes` in `engine/crates/cli/tests/taxonomy_graph.rs`).
