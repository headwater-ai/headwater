---
id: HW-DR-0077
status: current
status_since: 2026-09-20
summary: "An adopter needs the binary, a data-only package, Git, `sh`, `curl`, `tar` and eight POSIX utilities, and nothing else. Every other program reaches Headwater through a verb that the binary configures on request, and no adopter-facing page instructs this repository's own tooling."
last_verified: 2026-09-24
title: "The consumer surface is what an adopter receives, runs and must have installed, and it is a closed and declared list"
relations:
  constrains:
    - HW-DR-0072
    - HW-DR-0049
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-fable-5-1
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
---

# The consumer surface is what an adopter receives, runs and must have installed, and it is a closed and declared list

## Context

[HW-DR-0072](0072-the-binary-is-the-only-interface-an-adopter-must-run-and-every-integration-point-outside-it-is-declared.md) rules on what an adopter runs for the governed loop. The loop is what reads the corpus, checks it, writes it and resolves its taxonomy. That record is silent on everything else an adopter meets. It does not cover installation, CI, the agent harness or site publication. It does not say what an adopter receives or what an adopter must have installed. Its last paragraph keeps `tools/` and `.githooks/` exactly as they are, so it draws no line in the tree.

An audit of the tree on 2026-09-20 measured what that silence costs. [#892](https://github.com/headwater-ai/headwater/issues/892) holds the full inventory, and five of its findings carry this ruling.

**The first prerequisite of the tutorial is a Rust toolchain.** [The tutorial](../tutorials/your-first-governed-corpus.md) lists Rust 1.91 and `curl` before step 1. The README and the hand-built site lead with `cargo install`. The one archive with no toolchain runs on x86_64 Linux with a recent glibc and on nothing else.

**The one check of the boundary allows the toolchain.** `.claude/tutorial/adopter_interface.py` reads the fenced command blocks of `README.md` and the tutorial. Its allow list passes `cargo install` and `cargo build`. It reads no verb contract, no how-to guide, no site page and no text of the shipped package.

**Documents for an adopter name scripts that only this repository has.** Six verb contracts under [the command surface](../interfaces/README.md) name one, and `.githooks/change-manifest` and `tools/site/refresh-figures.sh` are two of them. The shipped `conformance.yml` instructs `git config core.hooksPath .githooks`, and nothing gives an adopter that directory. `tools/README.md` says that nothing in `tools/` ships to an adopter, and two scripts beside it address one.

**The merge hazard ships and the protection does not.** `headwater generate` writes folds over the whole corpus into every adopter's tree. [HW-DR-0049](0049-a-corpus-wide-fold-is-derived-and-never-stored.md) names the hazard and says that an adopter inherits the rule rather than the artifacts. The hooks that protect this repository resolve an engine under `engine/target`, which no adopter has. Each `python3` call in them guards a check of this repository's own site and fails open. The general half of the merged-state check is already two verbs, `headwater generate --check` and `headwater taxonomy resolve --check`.

**Two specification parts promise what no release carries.** [Spec 0](../spec/00-vision-and-scope.md) says that the skills and the hooks ship with the first release. [Spec 5](../spec/05-ai-integration.md) calls that machinery the adoption model. The release archive holds a binary and a license, and the package holds no harness file.

The owner's statement for release 0.2 is that an adopter needs nothing but the `headwater` binary. No Rust toolchain, no Python and no script of this repository is part of it. The owner ruled on ten questions from the audit on 2026-09-20, and the comment of that date on #892 records each answer. This record is where those rulings live.

## Decision

**The consumer surface is everything that an adopter receives, is told to run, or must have installed.** It is a closed list and a declaration holds it. A thing that is not on the list is not the adopter's, and no page for an adopter instructs it.

**Four populations make the whole tree, and every file belongs to exactly one.**

1. **Mandatory.** The `headwater` binary and a taxonomy package that holds data and no executable file. The platform prerequisites are Git, `sh`, and `curl` and `tar` as the download and unpack tools of the operating system. Eight POSIX utilities and `sh` builtins are also platform prerequisites, and a page for an adopter may assume each one. They are `cd`, `echo`, `grep`, `mkdir`, `printf`, `rm`, `wc` and `xargs`. These prerequisites are not this project's, and the surface declares each one by name. The surface also declares `headwater` as a command, and it is not a prerequisite, because it is the mandatory binary of this population. No Rust toolchain, no Python and no script of this repository is mandatory.
2. **A declared integration point.** Another program calls Headwater here. The five points are git, a CI forge, an agent harness, a shell and a site generator. At each point the executable that the other program calls is a `headwater` verb. The binary writes or prints the configuration, and the explicit command of the adopter is the consent. Each point declares its dependencies beyond the binary.
3. **A companion.** A tool that an adopter may run and that no conformance level requires. The citation checker, the probe recorder and the model fetch are companions. `headwater probe`, `headwater neighbors` and `headwater sweep` sit outside the bar of 0.2 with their helper scripts. A companion states its dependencies and lives outside `tools/`.
4. **Local to this repository.** `tools/`, `.githooks/`, `.claude/`, `.github/`, `site/` and `mkdocs/`. No page for an adopter instructs a file of this population. Such a page may cite one as the way this repository does a thing, in a passage that says so.

**The test for a new case is the question of who holds the file.** A file that an adopter must hold belongs to population 1 or 2. It is the binary, data, or a line that the binary wrote. A file that only this repository holds belongs to population 4. The necessity test of HW-DR-0072 still decides whether an integration point is legal at all.

**Merge safety ships as verbs and as configuration that `headwater init` writes.** A verb of the binary is the merge driver. It keeps the current side of a derived fold, exits non-zero and names the producer command. `headwater generate --check` and `headwater taxonomy resolve --check` hold the merged state. On request, `headwater init` writes the `.gitattributes` lines and the shims that git needs. A shim is one line of `sh` that calls a verb, because git already requires `sh`. No script body ships. The half of `.githooks/` that protects the hand-built pages of this repository never ships.

**The binary prints the `git config` lines by default and runs them only under an explicit flag.** Git takes no executable from a repository without the consent of the clone, and a printed line keeps that consent with the adopter.

**A merge-driver verb does not break the rule of spec 5 that no hook introduces a verb.** That rule forbids a second entry point to `route` or to `check`. The driver answers a question that no verb answers, which is what a merge does with a derived fold.

**Skills and harness hooks are not part of the surface of 0.2.** The vehicle is ruled here and the release is not. The binary emits them, and a hook configuration calls verbs only, as [HW-DR-0055](0055-a-hook-reads-a-wire-format-through-the-engine-and-not-through-an-interpreter.md) already requires. 0.3 is the earliest release that carries them.

**The supported platforms of 0.2 are a static Linux x86_64 build and a macOS arm64 build at the least.** `cargo install` is a labeled alternative and never the lead route.

**Site publication through a static-site generator is a declared integration point and is outside the mandatory surface.** MkDocs needs Python, and that dependency belongs to the generator. Headwater builds no renderer, as spec 0 states.

**`tools/headwater-bootstrap.sh` leaves the surface when `headwater taxonomy vendor` takes a location.** [HW-DR-0075](0075-the-vendor-verb-may-take-a-location-and-the-fetch-lives-only-in-a-crate-the-checking-loop-never-links.md) rules that verb. A stub stays at the published URL for one release and names the verb to run.

## Consequences

**HW-DR-0072 stands, and this record narrows its last paragraph.** It remains the rule for the governed loop and for the necessity test. `tools/` and `.githooks/` stay as producers that no adopter sees, and population 4 is the name for them. What changes is that a page for an adopter may not instruct them. The sentence of its Consequences that calls the bootstrap fetch the one declared exception is stale against its own Decision, and [#959](https://github.com/headwater-ai/headwater/issues/959) corrects it.

**HW-DR-0072 is accepted and rests on HW-DR-0075, which is a draft.** #959 does not build until HW-DR-0075 is accepted.

**The sentence of HW-DR-0049 about an adopter did not describe what ships.** An adopter inherits the rule and also the verbs that hold it. [#574](https://github.com/headwater-ai/headwater/issues/574) accepted HW-DR-0049 on 2026-09-23 and rewrote that sentence to say so.

**Spec 0 and spec 5 state a delivery that no release makes.** Spec 0 says that release 0.1 carries the skills and the hooks, and no release carries them. An obligation record holds that gap, and the correction of the sentence is part of the work under #892.

**Nothing checks this record.** The population of a file is stated here and in no declaration. [#976](https://github.com/headwater-ai/headwater/issues/976) declares the surface as a manifest, generates a page under `docs/interfaces/` from it, and reads every page for an adopter against it. [#933](https://github.com/headwater-ai/headwater/issues/933) rejected a wide rule because a list of one entry reports zero by construction. That reason does not hold here, because the audit counted at least 12 live findings under `docs/interfaces/` alone.

**Four issues carry the build.**

- [#974](https://github.com/headwater-ai/headwater/issues/974) makes the merge driver a verb, and `headwater init` writes the git configuration on request.
- [#975](https://github.com/headwater-ai/headwater/issues/975) attaches the two static archives to every release and leads every page with the download.
- #976 declares the surface and holds the pages for an adopter against it.
- #959 removes the bootstrap script from the tutorial, the README, the site and the release notes.

[#833](https://github.com/headwater-ai/headwater/issues/833) packages the CI integration point, and its action downloads a release archive and never runs `cargo`. [#977](https://github.com/headwater-ai/headwater/issues/977) and [#978](https://github.com/headwater-ai/headwater/issues/978) carry the site generator point in release 0.6.

**One directory holds what the binary cannot carry.** A CI action and a generator configuration are files that an adopter copies, and the binary cannot be either. They live together in one directory outside `tools/`. The location of a file then answers which population it belongs to. #833 names that directory when it lands the first file.

**Amended 2026-09-24: population 1 names the POSIX utilities that the pages for an adopter use.** The first text named Git, `sh` and the download and unpack tools. The tutorial and the relocation how-to also tell an adopter to run eight POSIX utilities. The owner ruled at the fifth merge of run 20260924-0411 to widen the clause by name and not to rewrite the pages. [#1051](https://github.com/headwater-ai/headwater/issues/1051) raised the ruling, and [#1070](https://github.com/headwater-ai/headwater/issues/1070) carries the amendment. The record is not superseded, and the other clauses do not change.

**Whether `headwater neighbors` ships at all is a separate decision.** Its only caller is `.claude/hooks/intent.sh`, which is local to this repository. This record places the verb among the companions and rules nothing more about it.
