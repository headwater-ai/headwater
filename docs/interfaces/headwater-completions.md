---
id: HW-IFACE-headwater-completions
status: draft
status_since: 2026-08-25
summary: "How to write one of four shell completion scripts, and which caller errors prevent any script output."
last_verified: 2026-08-25
title: "headwater completions"
relations:
  governs:
    - engine/crates/cli/src/lib.rs
    - engine/crates/cli/src/main.rs
---

# headwater completions

## Synopsis

    headwater completions <shell>

The command writes one completion script to standard output for `bash`, `zsh`, `fish` or `powershell`. It writes no file itself.

## Description

`headwater completions <shell>` generates a script from the same command tree that parses the binary and that `headwater help` prints. The script includes every verb, grouped second word and option in that tree.

Redirect standard output to the file and load it with the shell's normal command. For example, `headwater completions bash > f && . f` loads the Bash script in the current shell.

The command does not load the corpus or the taxonomy lock. It answers from the binary, so it works from a directory that is not a repository.

## Preconditions

The selected shell must be available to the caller when the caller loads the script. The binary itself does not start that shell.

## Options

| Option | What it does |
|---|---|
| `<shell>` | Select `bash`, `zsh`, `fish` or `powershell`. |
| `--root <path>` | Accept the repository path for the global parser. Completion generation does not read it. |
| `--wide` | Refused. Completion scripts use a fixed width and do not render help. |
| `--no-color` | Disable color output. Completion scripts contain no color output. |

The global `--help` and `--version` flags are answered before this command runs. The global `--root` and `--no-color` flags are accepted and have no effect. `--wide` is refused because completion scripts use a fixed width. An option that belongs to another verb is refused.

## Exit status

**0** when the selected shell is one of the four supported shells and the script is generated. The script is on standard output, and standard error is empty.

**1** when the shell is omitted, the shell name is not supported, or the command line has an invalid option or extra word. The refusal names the supported shells and writes no script.

## Environment

No environment variable reaches the command. The script is generated from the command tree and a fixed help width.

## Files

None. The command reads no repository file and writes no file. The caller chooses the output path by redirecting standard output.

## See also

[`headwater help`](headwater-help.md) prints the same command tree for a person.

[`headwater --help`](../../engine/crates/cli/src/lib.rs) is the root command grammar from which the script is generated.

[`headwater check`](headwater-check.md) documents a command whose options and status do read a corpus.
