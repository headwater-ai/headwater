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

## How these rules are enforced on `docs/spec/`

Every rule above is a check that `headwater check` runs, and the taxonomy of this repository is where each one is declared. Nothing enforces them by memory, and no second copy of any rule lives in a script.

**What declares them.** `.headwater/overlay.yml` binds `regimes.language.ste_house` to the three kinds that carry this repository's own prose: `design_spec`, `decision_register` and `obligation_register`. The regime names the controlled language and the profile, fixes the source form as `one_line_per_block`, and lists the stock phrasing under `retired_terms` with the reason each term is retired. The base package declares the voice regime that forbids future intent, change narration and phased rollout. A document under `docs/reviews/` answers to none of the language rules, which is the exemption the sections above state.

**What runs them.** Six rules read prose. `language.controlled.not_met` holds a sentence to 25 words and refuses a contraction, a British spelling and a semicolon in running prose. `language.source_form.not_met` reports a block written over more than one line. `language.retired_term.used` reports a term the regime retired. `voice.forbidden_construction` reports the three voice categories. `section.required.missing` and `link.fragment.unresolved` read the body for other reasons.

**What blocks.** `.githooks/pre-commit` runs `headwater check --strict`, which fails on an error and reports everything else. A rule is an error when its remediation is mechanical and total, which is the [fixability](docs/spec/12-check-layer.md#fixability) bar: a contraction, a British spelling, a hard-wrapped block, and a retired term that names its replacement. A rule is advisory when the remediation is a rewrite: a sentence past the word limit, a semicolon, a stock metaphor, and every voice category. Git does not install repository hooks by itself, so each clone needs this once:

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

**The escape hatches, and what each one means.** A directive on the offending block marks a deliberate exception, with the reason in the source where a reader will find it:

    …the term the cited authors use. <!-- headwater allow=language.retired_term.used scope=block until=2027-12-31 reason=false_positive note=quoting Star and Griesemer -->

`reason` is `false_positive` where the rule is wrong and `accepted_deviation` where the rule is right and the text stands anyway. `scope` is `file` or `block`. `until` is required, so no exception is permanent, and the report counts an expired directive and names it.

The second hatch is for debt rather than for an exception. The `adoption` block of `.headwater/taxonomy.lock` holds `(document, rule)` pairs under a task with an owner and an expiry. A pending finding is reported with its task beside it and does not fail a strict run. `headwater infer --owner <name> --write` writes one, and `headwater check` reports how many pairs remain on every run. This repository declares one task, which carries the last entry of the linter baseline that the engine still reports.

**Writing spec prose.** Invoke the `ste-editor` skill before you rewrite a document under `docs/spec/`, and run `headwater check` on the result. The skill carries the rules that no check reads: consistent terms, multi-word nouns kept to three words, and the judgment calls that a lexical rule cannot make.

**The seven skills, and when each one loads.** Nothing makes a skill load. A harness reads a description and a model picks, which [spec 5](docs/spec/05-ai-integration.md#how-a-skill-reaches-an-agent-and-what-nothing-does) states as a measurement rather than a property. This paragraph is the one mechanism stronger than a description, and it costs context on every session.

| skill | invoke it before |
|---|---|
| `headwater-orient` | you search `docs/` or open a specification part to find out what this repository already says |
| `headwater-authoring` | you add or revise any document under `docs/`, and whenever `headwater new` refuses |
| `headwater-taxonomy` | you touch `packages/`, `docs/taxonomies/` or `.headwater/overlay.yml` |
| `ste-editor` | you rewrite prose under `docs/spec/` |
| `headwater-engine` | the first cargo or CLI command of a session, and whenever a build fails on the toolchain |
| `repo-cleanup` | you retire a worktree, a local branch or a remote branch that a merged change left behind |
| `headwater-sweep` | you are asked to read a slice of the corpus for what no check can see |

The first two are ordered: orientation finds the document, and authoring writes one. A session that opens a specification part to discover it was the wrong part has paid for a read it did not need, and `headwater explain` answers from the graph the engine already built.

The last one is the only mechanism here that no engine performs. `headwater sweep plan` writes the briefing, you read the documents, and `headwater sweep report` says what the engine could confirm about what you wrote back. Nothing gates on it: the verb exits 0 whatever it finds, no gate and no CI job runs it, and no crate of this engine opens a socket.

`.claude/agents/headwater-maintainer.md` is the maintainer subagent: hand it a change and it reports what the change touched, what is now stale, what the corpus is owed, and what it could not decide. It proposes and never accepts.

`sh .claude/skills/fixtures.sh` holds every claim these files make about the engine. It is a blocking CI step, half of its cases are derived from the skill files rather than listed, and it writes only into a scratch copy of the corpus.
