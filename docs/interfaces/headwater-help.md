---
id: HW-IFACE-headwater-help
status: current
status_since: 2026-09-06
summary: "How to print the command tree or one verb's help, and the caller errors that return status 1."
last_verified: 2026-08-30
title: "headwater help"
relations:
  governs:
    - engine/crates/cli/src/lib.rs
    - engine/crates/cli/src/main.rs
    - engine/crates/cli/src/paint.rs
  traces_to:
    - HW-DR-0045
---

# headwater help

## Synopsis

    headwater help [<verb> [<word>]]

The command prints the root help screen when it has no operand. It prints the help for one verb, and for its second word where that word exists.

## Description

`headwater help` reads the command tree that the parser uses. The root screen lists every verb and its summary. A selected node prints its synopsis, description, options and subcommands.

The root screen also prints a masthead above `Usage:`: the binary name and version, a rule, then the existing tagline. A verb's own help page prints none of it. [HW-DR-0045](../decisions/0045-coloring-the-cli-and-where-the-banner-goes.md) rules that the root screen is the only place it appears.

The command does not load the corpus or the taxonomy lock. It answers from the command tree, so it also works from a directory that is not a repository.

`headwater help <verb>` and `<verb> --help` print the same bytes. The `-h` form also prints the same bytes for a verb that accepts the short flag.

## Preconditions

None. The command tree is available in the binary.

## Options

| Option | What it does |
|---|---|
| `<verb>` | Select the top-level verb to describe. Omit it to print the root screen. |
| `<word>` | Select the second word of a grouped verb, such as `taxonomy diff`. |
| `--root <path>` | Accept the repository path for the global parser. Help does not read it. |
| `--wide` | Use the wider help layout. It changes the rendered help width. |
| `--no-color` | Force plain text on both streams: bold and dim weight plus glyphs, no escape sequence. The default already senses whether each stream is a terminal, and renders color only there. |
| `--no-banner` | Suppress the masthead. It is accepted here and does nothing beyond the root screen, since only the root screen prints one. |

The global `--help` and `--version` flags are answered before this command runs. The global `--root`, `--wide`, `--no-color` and `--no-banner` flags are accepted when the parser reaches this command. An option that belongs to another verb is refused.

## Exit status

**0** when the selected command node exists and its help is printed. This includes the root screen.

**1** when a requested verb or second word does not exist, or when the command line has an invalid option or extra word. The refusal names the word and points to `headwater --help`. It renders in the error role when standard error is a terminal, and plainly otherwise, the rule every command follows.

Help output goes to standard output. A successful run writes nothing to standard error. A refusal writes its message to standard error and writes no help screen.

## Environment

`NO_COLOR`, set to any value, has the same effect as `--no-color`. `HEADWATER_NO_BANNER`, set to any value, has the same effect as `--no-banner`. `COLUMNS` affects the global width helper only when a caller uses the width option on a command that reads it. This command uses the parser's command tree and does not read the corpus.

## Files

None. The command reads no repository file and writes no file.

## See also

[`headwater --help`](../../engine/crates/cli/src/lib.rs) is the root command grammar.

[`headwater completions`](headwater-completions.md) writes shell completion scripts from the same command tree.

[`headwater check`](headwater-check.md) documents a command whose help output describes a corpus-reading verb.
