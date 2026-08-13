# Contributing to Headwater

Headwater is in its design phase. The specification under [`docs/spec/`](docs/spec/) is the current product, and [`docs/spec/09-decisions.md`](docs/spec/09-decisions.md) is the register of what the design has settled and why. [`docs/spec/13-open-obligations.md`](docs/spec/13-open-obligations.md) carries what it still owes.

## The terms

Code is licensed under the Apache License, Version 2.0. See [`LICENSE`](LICENSE).

Prose under `docs/` is licensed under Creative Commons Attribution 4.0 International. See [`docs/LICENSE`](docs/LICENSE).

## Sign your work: the Developer Certificate of Origin

Headwater takes contributions under the [Developer Certificate of Origin](https://developercertificate.org/), version 1.1. You certify the DCO by adding a `Signed-off-by` line to each commit message:

    Signed-off-by: Your Name <your.email@example.com>

`git commit -s` writes that line for you. The name must be a real name, and the address must be one that reaches you.

**There is no contributor license agreement, and that is deliberate.** A contributor agreement assigns rights or grants a license broad enough to let the project change its terms later without asking the people who wrote the code. That power is the only thing such an agreement adds over the DCO for a project under a permissive license. Headwater does not want it. Every contributor keeps the copyright in what they wrote, and the terms above bind the project the same way they bind everyone else. Changing them would require asking.

## The license identifier in source files

Every source file carries its identifier on the first line that permits a comment:

    // SPDX-License-Identifier: Apache-2.0

Use the comment syntax of the language. A file that a tool generates carries the same line, emitted by the generator.

The identifier is the whole header. Do not add a copyright line naming yourself or anyone else to individual files. Authorship lives in the commit history, where it stays accurate, and `NOTICE` carries the project-level statement.

## Security

Do not report a vulnerability through an issue or a pull request. [`SECURITY.md`](SECURITY.md) states the process.

## Authoring conventions

[`CLAUDE.md`](CLAUDE.md) records the conventions that specification prose follows, and how they are enforced. Two of them catch every newcomer once. Markdown source is not hard-wrapped, so a paragraph is one long line. Spelling is American.

Enable the repository hooks before your first commit:

    git config core.hooksPath .githooks

If you work through Claude Code, `.claude/settings.json` registers three more hooks that load on their own. They route your task before you open a file, refuse a raw write of a new document, name the documents that govern code you just edited, and run the commit gate at the end of a turn. [`CLAUDE.md`](CLAUDE.md) says what each one does and how to turn them off. None of them blocks a commit, and none is a substitute for the two lines above.
