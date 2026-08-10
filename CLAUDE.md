# Headwater authoring conventions

## The name is Headwater

**Headwater** is the name of the system, not a working name (decided 2026-08-10, [Q10](docs/spec/09-open-questions.md#q10--naming)). Capitalize it in prose, including compounds: Headwater's, Headwater-specific, Headwater-native.

Write it as `headwater` in lower case only when it is an identifier: the CLI verb (`headwater check`), package names (`acme/headwater-taxonomy`), the `.headwater/` directory, the `headwater:` annotation prefix, and the `https://headwater.dev/` namespace. Do not capitalize inside code spans or fenced code. Documents under `docs/reviews/` are point-in-time records and stay as written.

## No arbitrary line breaks in Markdown

Do not hard-wrap Markdown source. Write each paragraph, list item, and blockquote paragraph as one logical line; editors and renderers reflow as appropriate. Never insert a line break for line-length reasons, and remove such breaks when you edit a file that has them.

Line breaks are structural only: blank lines between blocks, one line per list item or table row, fenced/indented code kept verbatim. A deliberate hard break inside a paragraph (rare) uses a trailing backslash, not two spaces.

## American spelling

Use American English spelling everywhere: organization, artifact, behavior, customize, judgment, license, catalog. This is the ruling that ASD-STE100 rule 1.14 requires (decided 2026-08-10); do not reintroduce British forms. Documents under `docs/reviews/` are point-in-time records and stay as written.

## Avoid stock AI phrasing

Do not use the stock metaphors and intensifiers that AI-assisted prose overuses: "load-bearing", "first-class (citizen)", "battle-tested", "north star", "delve", "seamless", "holistic", "deep dive", "leverage" as a verb, "robust" as filler. Name the concrete thing the metaphor points at instead: who depends on it, what breaks without it, what it enforces.

Exception: documents under `docs/reviews/` are point-in-time review records and stay as written.
