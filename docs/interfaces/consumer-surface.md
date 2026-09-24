<!-- headwater:generated consumer_surface. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file. -->

# The consumer surface

This page lists what an adopter receives, what an adopter runs, and what an adopter must have installed. `headwater generate` writes it from the `surface` block of the taxonomy, and `headwater generate --check` reports it stale when the block changes.

## What the archive holds

The files of a release archive, as paths inside it.

- `headwater`
- `LICENSE`

## Integration points

The places where an adopter connects the binary, and what each one needs beyond the binary.

| Integration point | What it needs beyond the binary |
|---|---|
| `agent_harness` | nothing |
| `ci_forge` | nothing |
| `git` | `git`, `sh` |
| `shell` | `sh` |
| `site_generator` | `mkdocs`, `python3` |

## Prerequisites

Programs that an adopter installs and that this project does not supply.

- `git`
- `sh`
- `curl`
- `tar`

## Companions

Tools that an adopter may run and that no conformance level requires.

- `headwater probe`
- `headwater neighbors`
- `headwater sweep`

## Commands

The first word of every command that a page for an adopter may tell a reader to run.

- `headwater`
- `git`
- `sh`
- `curl`
- `tar`
- `cd`
- `echo`
- `grep`
- `mkdir`
- `printf`
- `rm`
- `wc`
- `xargs`

The block also names the pages for an adopter and the roots that only this repository holds. They configure the check that holds each such page, and they are not something an adopter receives or runs, so this page does not list them.
