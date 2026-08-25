# Headwater authoring conventions

## The name is Headwater

**Headwater** is the name of the system, not a working name (decided 2026-08-10, [Q10](docs/spec/09-decisions.md#q10--naming)). Capitalize it in prose, including compounds: Headwater's, Headwater-specific, Headwater-native.

Write it as `headwater` in lower case only when it is an identifier: the CLI verb (`headwater check`), package names (`acme/headwater-taxonomy`), the `.headwater/` directory, the `headwater:` annotation prefix, and the `https://w3id.org/headwater/` namespace. Do not capitalize inside code spans or fenced code. Documents under `docs/reviews/` are point-in-time records and stay as written.

## No arbitrary line breaks in Markdown

Do not hard-wrap Markdown source. Write each paragraph, list item, and blockquote paragraph as one logical line; editors and renderers reflow as appropriate. Never insert a line break for line-length reasons, and remove such breaks when you edit a file that has them.

Line breaks are structural only: blank lines between blocks, one line per list item or table row, fenced/indented code kept verbatim. A deliberate hard break inside a paragraph (rare) uses a trailing backslash, not two spaces.

## American spelling

Use American English spelling everywhere: organization, artifact, behavior, customize, judgment, license, catalog. This is the ruling that ASD-STE100 rule 1.14 requires (decided 2026-08-10); do not reintroduce British forms. Documents under `docs/reviews/` are point-in-time records and stay as written.

## Avoid stock AI phrasing

Do not use the stock metaphors and intensifiers that AI-assisted prose overuses: "load-bearing", "first-class (citizen)", "battle-tested", "north star", "delve", "seamless", "holistic", "deep dive", "leverage" as a verb, "robust" as filler. Name the concrete thing the metaphor points at instead: who depends on it, what breaks without it, what it enforces. The list a check reads is `retired_terms` in `.headwater/overlay.yml`, and this paragraph is the statement of why. Add a term in both places, with the reason in the overlay.

Exception: documents under `docs/reviews/` are point-in-time review records and stay as written.

## How these rules are enforced on the shelves that carry this repository's prose

Every rule above is a check that `headwater check` runs, and the taxonomy of this repository is where each one is declared. Nothing enforces them by memory, and no second copy of any rule lives in a script.

**What declares them.** `.headwater/overlay.yml` binds `regimes.language.ste_house` to the kinds that carry this repository's own prose: `design_spec`, `decision`, `decision_register`, `evaluation`, `obligation_register`, `obligation_record`, `interface_contract`, `tutorial`, `requirement` and `acceptance_criterion`. The regime names the controlled language and the profile, fixes the source form as `one_line_per_block`, and lists the stock phrasing under `retired_terms` with the reason each term is retired. The base package declares the voice regime that forbids future intent, change narration and phased rollout. A document under `docs/reviews/` answers to none of the language rules, which is the exemption the sections above state.

**What runs them.** Six rules read prose. `language.controlled.not_met` holds a sentence to 25 words and refuses a contraction, a British spelling and a semicolon in running prose. `language.source_form.not_met` reports a block written over more than one line. `language.retired_term.used` reports a term the regime retired. `voice.forbidden_construction` reports the three voice categories. `section.required.missing` and `link.fragment.unresolved` read the body for other reasons.

**What blocks.** `.githooks/pre-commit` runs `headwater check --strict`, which fails on an error and reports everything else. It first runs `.githooks/change-manifest HEAD`, which turns the committed `HEAD` and your working tree into the manifest that `--change` reads, so a state movement the declared lifecycle refuses is refused here too. That producer runs every `git` command on the path from a commit to a verdict, because the engine runs none. It fails open, so no git and no `mktemp` costs you one rule and never the commit. `sh .githooks/fixtures.sh` drives the gate and the producer over a scratch copy of this corpus, including every refusal, and CI runs it. A rule is an error when its remediation is mechanical and total, which is the [fixability](docs/spec/12-check-layer.md#fixability) bar: a contraction, a British spelling, a hard-wrapped block, and a retired term that names its replacement. A rule is advisory when the remediation is a rewrite: a sentence past the word limit, a semicolon, a stock metaphor, and every voice category. Git does not install repository hooks by itself, so each clone needs this once:

    git config core.hooksPath .githooks

`headwater check --fix` writes the corrections the engine can derive without judgment, in your working tree, and prints what it wrote on standard error. Today that is a British spelling, a contraction whose expansion is one word, a retired term that names a replacement, and a missing reciprocal link. It leaves every finding whose remedy is a rewrite, and it refuses a file rather than half-writing one. Read the diff before you commit it.

The hook needs a built engine and fails open with one printed line when there is none. Build it once with `cargo build --release -p headwater-cli --manifest-path engine/Cargo.toml`. The CI job builds the engine and runs the same check on every pull request, so an unbuilt clone delays a finding rather than losing it. Run `headwater check` yourself to read the advisory findings, which the hook does not print.

**What runs before commit time, in this harness.** `.claude/settings.json` registers three hooks in `.claude/hooks/`, one for each moment [spec 5](docs/spec/05-ai-integration.md#the-hook-contract-and-what-a-hook-cannot-bind) names. Git does not install them and Claude Code loads them when the repository opens, which is the opposite of the line above.

| Position | Script | What it does |
|---|---|---|
| `UserPromptSubmit` | `intent.sh` | `headwater route` on your prompt, and nothing at all when the route is silent |
| `PreToolUse` on `Write`/`Edit` | `write.sh` | refuses a raw write of a document that does not exist yet, and names `headwater new`. An edit to an existing document passes |
| `PostToolUse` on `Write`/`Edit` | `write.sh` | names the documents that declare `governs` over the path you just edited. Advisory, and it blocks nothing |
| `Stop` | `review.sh` | runs `.githooks/pre-commit` and stops the turn on what would stop the commit |

Each one calls a verb that already ships, and none carries a rule of its own. The review hook invokes the commit hook rather than repeating it, so this repository still runs exactly one thing at commit time. Every one of them fails open: no built engine, no `python3`, or an input it cannot read, and the action proceeds.

None of them binds. A `Bash` call that writes a file matches no matcher, `disableAllHooks` turns all of them off with no record anywhere, and `git commit --no-verify` skips the gate below them. What holds a change is the commit gate and the CI job. `sh .claude/hooks/fixtures.sh` runs all four positions against recorded input, including every refusal.

The same three scripts are registered a second and a third time, in `.codex/hooks.json` and `.github/hooks/*.json`, for the two harnesses [spec 16](docs/spec/16-harness-support.md) records. `sh .claude/hooks/fixtures-live.sh` is what holds that binding rather than the shape it assumes: a real `codex exec` and a real `copilot -p`, over a scratch clone, spending real AI credits against a real login. It skips a harness that is not installed or not authenticated rather than failing on it, nothing gates on it, and no CI job runs it, the same posture as `headwater probe`.

**The escape hatches, and what each one means.** A directive on the offending block marks a deliberate exception, with the reason in the source where a reader will find it:

    …the term the cited authors use. <!-- headwater allow=language.retired_term.used scope=block until=2027-12-31 reason=false_positive note=quoting Star and Griesemer -->

`reason` is `false_positive` where the rule is wrong and `accepted_deviation` where the rule is right and the text stands anyway. `scope` is `file` or `block`. `until` is required, so no exception is permanent, and the report counts an expired directive and names it.

The second hatch is for debt rather than for an exception. The `adoption` block of `.headwater/taxonomy.lock` holds `(document, rule)` pairs under a task with an owner and an expiry. A pending finding is reported with its task beside it and does not fail a strict run. `headwater infer --owner <name> --write` writes one, and `headwater check` reports how many pairs remain on every run. That write adds to the block rather than producing it: it proposes only the findings no open task holds, it mints an identifier past every identifier the block declares, and it prints what it carried through. A second run of it declares nothing and writes nothing. This repository declares one task, `AD-1`, and its one pair is closed. The pair carried the last entry of the retired linter's baseline, and that sentence was rewritten rather than waited out, so `headwater check` now reports the task as holding 0 findings.

**The on-ramp, and the suite that holds it.** `docs/tutorials/your-first-governed-corpus.md` takes a reader from an empty directory to a passing strict run, and it states after every step what the reader should now see. Each of those is a claim about this engine copied into prose, so `sh .claude/tutorial/fixtures.sh` is the verb that produces them: it reads the commands out of the document, runs them against a scratch repository under the temporary directory, and diffs each result against the block the document prints. It blocks in CI, and it writes nothing inside this checkout. Edit an output block only by running the command and taking what it printed.

**Writing governed prose.** Invoke the `ste-editor` skill before you rewrite a document under `docs/spec/`, `docs/decisions/`, `docs/evaluations/`, `docs/obligations/`, `docs/interfaces/`, `docs/requirements/`, `docs/acceptance-criteria/` or `docs/tutorials/`, and run `headwater check` on the result. The skill carries the rules that no check reads: consistent terms, multi-word nouns kept to three words, and the judgment calls that a lexical rule cannot make.

**The seven skills, and when each one loads.** Nothing makes a skill load. A harness reads a description and a model picks, which [spec 5](docs/spec/05-ai-integration.md#how-a-skill-reaches-an-agent-and-what-nothing-does) states as a measurement rather than a property. This paragraph is the one mechanism stronger than a description, and it costs context on every session.

| skill | invoke it before |
|---|---|
| `headwater-orient` | you search `docs/` or open a specification part to find out what this repository already says |
| `headwater-authoring` | you add or revise any document under `docs/`, and whenever `headwater new` refuses |
| `headwater-taxonomy` | you touch `packages/`, `docs/taxonomies/` or `.headwater/overlay.yml` |
| `ste-editor` | you rewrite prose under `docs/spec/`, `docs/decisions/`, `docs/evaluations/`, `docs/obligations/`, `docs/interfaces/`, `docs/requirements/`, `docs/acceptance-criteria/` or `docs/tutorials/` |
| `headwater-engine` | the first cargo or CLI command of a session, and whenever a build fails on the toolchain |
| `repo-cleanup` | you retire a worktree, a local branch or a remote branch that a merged change left behind |
| `headwater-sweep` | you are asked to read a slice of the corpus for what no check can see |

The first two are ordered: orientation finds the document, and authoring writes one. A session that opens a specification part to discover it was the wrong part has paid for a read it did not need, and `headwater explain` answers from the graph the engine already built.

The last one is the only mechanism here that no engine performs. `headwater sweep plan` writes the briefing, you read the documents, and `headwater sweep report` says what the engine could confirm about what you wrote back. Nothing gates on it: the verb exits 0 whatever it finds, no gate and no CI job runs it, and no crate of this engine opens a socket.

`.claude/agents/headwater-maintainer.md` is the maintainer subagent: hand it a change and it reports what the change touched, what is now stale, what the corpus is owed, and what it could not decide. It proposes and never accepts.

`.claude/agents/headwater-product-owner.md` is the product-owner subagent, and it reads the whole board rather than a change: whether the milestones are being completed in order, which of them is finished and left open, which issues are filed against the wrong one, what an adopter still cannot do, and what needs a ruling. `/product-owner` runs it standalone at any time, and the build order also runs it at the top of a run and every fifth iteration inside one, never in the context of the agent doing the building.

It owns the structure of the board and never its scope. It closes a finished milestone, creates one, moves an issue between milestones, writes the `adopter-blocking` and `self-audit` labels, and files an issue only where it can quote the bar of an existing milestone that names work no open issue carries. It closes no issue and edits no Done-when, because what work is and when it is done is the owner's. That boundary is the reason it may hold a pen at all: closing the current milestones cleanly needs one, and deciding what the product should do next does not.

**The value rule that both of them serve.** Work must name a reader who is not this repository. `.claude/commands/next-run.md` states it once, under *The value rule*, and every other file cites it rather than restating it. Two labels carry it on the issue tracker: `self-audit` for work found by running the engine over this corpus with no reader outside it, and `adopter-blocking` for work an outside adopter cannot proceed without. `self-audit` means the work waits, not that it is wrong.

## Filing an issue

`.github/ISSUE_TEMPLATE/issue.md` is the shape, and it is the only copy of it. GitHub offers it in the web form and nowhere else, so anything filing through `gh` reads that file first and follows it by hand.

**An issue opens with an `## ELI5` section.** Two to five sentences of plain language, for a reader who has never opened this repository: what is wrong, what a person would notice because of it, and what the fix looks like from outside. No spec numbers, no rule identifiers, no verb or crate name that the same sentence does not define. The rest of the body then addresses a reader who does know the repository, which is what the bodies here have always done. The ELI5 is the part that was missing, and it is written last and read first.

Nothing checks this. No hook, no rule and no CI job reads an issue body, so the template and this paragraph are the whole mechanism.

`sh .claude/skills/fixtures.sh` holds every claim these files make about the engine. It is a blocking CI step, half of its cases are derived from the skill files rather than listed, and it writes only into a scratch copy of the corpus.
