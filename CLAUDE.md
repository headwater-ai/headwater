# headwater authoring conventions

## No arbitrary line breaks in Markdown

Do not hard-wrap Markdown source. Write each paragraph, list item, and blockquote paragraph as one logical line; editors and renderers reflow as appropriate. Never insert a line break for line-length reasons, and remove such breaks when you edit a file that has them.

Line breaks are structural only: blank lines between blocks, one line per list item or table row, fenced/indented code kept verbatim. A deliberate hard break inside a paragraph (rare) uses a trailing backslash, not two spaces.

## American spelling

Use American English spelling everywhere: organization, artifact, behavior, customize, judgment, license, catalog. This is the ruling that ASD-STE100 rule 1.14 requires (decided 2026-08-10); do not reintroduce British forms. Documents under `docs/reviews/` are point-in-time records and stay as written.

## Avoid stock AI phrasing

Do not use the stock metaphors and intensifiers that AI-assisted prose overuses: "load-bearing", "first-class (citizen)", "battle-tested", "north star", "delve", "seamless", "holistic", "deep dive", "leverage" as a verb, "robust" as filler. Name the concrete thing the metaphor points at instead: who depends on it, what breaks without it, what it enforces.

Exception: documents under `docs/reviews/` are point-in-time review records and stay as written.

## How these rules are enforced on `docs/spec/`

Spec prose is held to the ASD-STE100 **house** profile, and the check is mechanical, not a matter of remembering. Two layers, because they catch different things.

**In a Claude session.** `.claude/settings.json` registers `.claude/hooks/ste-gate.py` on three events. A `PreToolUse` hook refuses `Edit` and `Write` on anything under `docs/spec/` until that session has invoked the `ste-editor` skill; the refusal says so and the session retries after loading the skill. A `PostToolUse` hook records that the skill loaded, and a second one runs the linter on each spec file the moment it is written, so violations come back while the text is still in hand. The hook fails open: if it breaks, edits proceed and the commit hook still catches the result. A spec written through Bash bypasses the gate entirely, which is the reason the second layer exists.

**At commit.** `.githooks/pre-commit` runs `tools/ste-lint.py --staged` over the staged content. Git does not install repository hooks by itself, so each clone needs this once:

    git config core.hooksPath .githooks

**The linter.** `tools/ste-lint.py` is step 4 of the `ste-editor` skill turned into a script, plus the two rules above that are exact string matches. It blocks on hard wraps, semicolons in prose, contractions, British spelling, stock AI phrasing, and sentences past 25 words. It warns, without blocking, on passive voice, "-ing" verb forms, auxiliary constructions, over-long paragraphs, and long headings: those detectors are regular expressions and they misfire often enough that blocking on them would only teach everyone to pass `--no-verify`. Both hooks pass `--errors-only`, so a commit reports warnings as a count and stays readable; run the linter yourself to read them. Run `tools/ste-lint.py` for the whole corpus, `tools/ste-lint.py <file>` for one, and `python3 tools/test_ste_lint.py` after any change to the linter itself.

Two escape hatches, and they mean different things. A trailing comment marks one deliberate exception, with the reason in the source where a reader will find it:

    …the term the cited authors use. <!-- ste-lint: allow stock-phrase # quoting Star and Griesemer -->

`.ste-lint-baseline.json` grandfathers what was already there when the check went in. Each entry is a hash of the file, the rule, and the offending text, so editing that text invalidates its entry: the sentence you touch is the sentence you fix. Regenerate with `tools/ste-lint.py --update-baseline`, and read the diff before you commit it. `tools/ste-lint.py --no-baseline` shows everything the baseline is currently hiding.

Scope is the two constants at the top of `tools/ste-lint.py`. `docs/reviews/` is excluded there for the same reason the rules above exempt it.
