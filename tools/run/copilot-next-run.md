# Running the Headwater build order on the Copilot CLI

A real binding of `.claude/commands/next-run.md` to GitHub Copilot, not a workaround. Unlike Codex, Copilot already reads `AGENTS.md`, `.claude/skills/*/SKILL.md` and `.claude/agents/*.md` directly from the checkout, its IDE reads `.claude/settings.json`, and `.github/hooks/hooks.json` already binds C3 through C6 the same way `.claude/settings.json` does. `tools/run/copilot-dispatch.sh` is the one new piece: a driver that runs each stage as its own `copilot -p` session and lets the veto resume it by id.

Everything below marked **verified** was run against the installed `copilot-cli 1.0.83` during this design session, live, on a scratch clone with the real engine. Everything marked **documented** is a claim of the vendor that nobody re-ran here. Everything marked **open** is a real capability this design does not yet use.

## What the investigation found

**`--agent <name>` resolves a persona from `.claude/agents/<name>.md` directly, with no `.github/agents/*.agent.md` mirror required** (**verified**: deleting `.github/agents/hw-queue.agent.md` from a scratch clone and dispatching `--agent hw-queue` again produced an identical result, including a warning about the persona's own `model: opus` front matter). Spec 16's table names only `.github/agents/*.agent.md` for Copilot's C9; this repository's own `.github/agents/*.agent.md` files, however many of the eight personas they cover, are a discoverability convenience for a picker, not a dispatch requirement. This design reads `.claude/agents/*.md` straight, the same single copy of each persona that Claude Code and the two mirrored files both point at.

**A session id is ours to assign, and a resumed session survives across separate process invocations** (**verified**: `--session-id <uuid>` on one `copilot -p` call, then `--resume=<uuid>` on a wholly separate later invocation, correctly recalled a planted word). This is the direct equivalent of Codex's `codex exec resume <thread-id>`, except Copilot does not make the caller parse an id out of a JSON stream first — the id is chosen before the first call, not read back after it.

**Identity does not survive a bare `--resume`, and the fix is to pass `--agent` again** (**verified**: a resume without `--agent` answered "I'm speaking as GitHub Copilot CLI, not a specialized sub-agent" while half-recalling the prior persona's constraint from the transcript — a genuinely confused answer, not a clean fallback. Passing `--agent <name>` again on the resume call restored both identity and context together, confirmed by repeating the same round trip with `--agent` on both calls). Every dispatch and every veto in this design carries `--agent`, because of this.

**`-p` takes its prompt as a command-line argument, not stdin** (**verified**: `-p -` was tried first, on the model Codex's `-` convention that reads a piped file; the model received the two-character string `-` as its whole prompt). The driver passes the composed prompt as `-p "$(cat ...)"` instead, which is one reason the composed prompt here is much shorter than Codex's: Copilot needs no translation of skills, standing context or the persona body, so there is far less to inline before hitting a practical argument-length concern.

**The live hook suite this repository already ships had two bugs hiding a working install** (**verified, and fixed in `.claude/hooks/fixtures-live.sh`**): its 30-second reachability probe and 100-second per-case timeout both undercount how slow `copilot -p` actually is — a trivial "say pong" took 44 seconds wall clock on a fully authenticated, working install — so the suite read a live install as "not authenticated, or unreachable" and silently skipped every Copilot case. Raised to 90s and 180s, all three live cases pass: a raw `docs/` write is refused, a clean turn ends without a block, and a real check finding blocks the turn until fixed.

## What this design does not use yet

**Copilot's native `task` tool.** A running Copilot agent can dispatch a named persona as an in-process sub-task and get its report back synchronously, or in the background with a completion notification and a `write_agent` follow-up to an idle agent — a closer, more capable equivalent to Claude Code's own `Agent`/`SendMessage` tools than anything external. **Verified working** for a synchronous dispatch. **Open**: the native tool takes a persona's `model:` front matter literally and fails hard when the model id is not one Copilot recognizes (`model: opus` errored outright, where the top-level `--agent` CLI flag instead warns and substitutes `claude-sonnet-5`), so every one of the five build-order stage personas needs a model override on the call, or a stripped `model:` field, before this path works with them unmodified. It is also untested for cross-process durability: the whole run currently depends on `tools/run/run-dir.sh`'s external ledger surviving a killed or restarted parent, and `task`/`read_agent`/`write_agent` are described by the CLI itself as scoped to the current session's lifetime. A single long-running interactive `copilot` parent using `task` natively, with real width from its background mode, is a stronger design than the external driver below — it is just not the one that has been proven end to end here.

## The shape (what is proven)

One `copilot -p` process per dispatch, driven by `tools/run/copilot-dispatch.sh`. The parent runs the same procedure `.claude/commands/next-run.md` already states.

    parent (a copilot session, or a person, following .github/prompts/next-run.prompt.md)
      └── sh tools/run/copilot-dispatch.sh dispatch <stage> <run> <tag> <dispatch-file>
            └── copilot -C <root> --agent <persona> --session-id <uuid> --allow-all-tools --silent -p "<composed prompt>"

`<stage>` is one of `product-owner`, `maintainer`, `queue`, `adjudicate`, `build`, `verify`, `integrate`; the driver maps each to its persona file under `.claude/agents/`. `<tag>` names the dispatch inside the run, as `issue-<N>-<stage>` or `queue`. `tools/run/run-dir.sh` is reused unmodified, the same ledger Claude Code and the Codex spike both use.

### The composed prompt

Far shorter than Codex's, because there is far less to translate. Three gaps, not a whole mechanism vocabulary: `EnterWorktree` does not exist, so a worktree is `git worktree add` by hand, exactly as the personas already document as their fallback. The scratch directory is stated once, in the harness's own terms, mapping `$CLAUDE_JOB_DIR/tmp/issue-<N>/` onto the run's own scratch path. And a second opinion from another model is named as a real, available capability (the native `task` tool) rather than an absent one, though this driver does not exercise it — a stage that wants one can call it directly, and reports whether it did.

### The veto

    sh tools/run/copilot-dispatch.sh veto <run> <tag> <finding-file> [stage]

Resumes the exact session that built the branch, with `--agent` and `--resume` both set, so it keeps its settled design and its identity together, confirmed by the round trip above. When it returns, dispatch `verify` again with a new tag.

## What you gain over the Codex spike

No persona translation, because Copilot reads the one canonical file. No thread-id parsing, because the id is chosen up front. Repository-shippable commands (`.github/prompts/*.prompt.md`) and, partially, agents (`.github/agents/*.agent.md`) where Codex's are personal-only. A graceful model fallback at the CLI dispatch layer, where Codex has none recorded. And a real path to width later, through the native `task` tool's background mode, that Codex does not offer at all.

## What you still lose, stated plainly

**Width, for now.** This driver blocks the parent's turn per dispatch, same limitation as the Codex spike, for the same reason: proving the external, resumable, ledger-backed shape first. The native `task` tool's background mode is the answer, once the model-override gap above is closed.

**A verified in-process second opinion.** The synchronous `task` dispatch works; a background one with a `write_agent` follow-up, which is what a genuine parallel second opinion needs, has not been exercised here.

## Start it today

1. `copilot login`, if not already authenticated, and confirm the repository is a trusted folder — `~/.copilot/config.json`'s `trustedFolders` should list this checkout's path. Nothing else needs bootstrapping: no personal prompt file to copy, no config to hand-edit, because `.github/prompts/*.prompt.md` and `.github/hooks/hooks.json` already ship in the repository.
2. `chmod +x tools/run/copilot-dispatch.sh` if the checkout does not preserve the executable bit.
3. `sh tools/run/run-dir.sh start` for a run directory, then dispatch the first stage:

        sh tools/run/copilot-dispatch.sh dispatch product-owner "$run" po-1 <dispatch-file>

4. Read `.github/prompts/next-run.prompt.md` for the loop itself; it now names this driver rather than telling the parent to run every iteration inline.

**Prove the first real iteration before trusting a run further than this session did.** Every case above was a synthetic plumbing check — a persona confirming which file it read, a planted word recalled across a resume — not a real `hw-build` constructing an issue. Run one real `adjudicate` → `build` → `verify` cycle by hand and read whether the report at the end is one a person would accept, the same caution the Codex design carries for its own first run.
