![Headwater. Documentation you can validate.](assets/headwater-social-preview.png)

**Documentation rots because nothing holds it accountable.** Headwater types your corpus, checks it as a graph, and accounts for every file it saw. Nothing else can answer *"is this corpus still true?"*

[![CI](https://github.com/headwater-ai/headwater/actions/workflows/ci.yml/badge.svg)](https://github.com/headwater-ai/headwater/actions/workflows/ci.yml) [![Release](https://img.shields.io/github/v/release/headwater-ai/headwater)](https://github.com/headwater-ai/headwater/releases/latest) [![License](https://img.shields.io/github/license/headwater-ai/headwater)](LICENSE)

**Start here: [Your first governed corpus](docs/tutorials/your-first-governed-corpus.md).** Sixteen steps from an empty directory to a passing check, and it teaches the model rather than the commands. Every step states what you should now see, and `sh .claude/tutorial/fixtures.sh` runs the page against the engine in CI so that no output block on it can go stale quietly. That page, and the rest of the `docs/` tree, is rendered at <https://headwater.tools/>. The specification itself is indexed at [`docs/spec/README.md`](docs/spec/README.md), which `headwater generate` writes from the documents themselves in the reading order this corpus derives from its own relations.

## Obtaining a named version

**The simplest route now, and it works.** All 23 workspace crates are on crates.io as of `v0.1.2`, published in dependency order by `.github/workflows/publish-crates.yml`.

```
cargo install headwater-cli
```

The command needs nothing this repository ships: no clone, no toolchain floor beyond what `cargo` itself resolves from the crate's declared `rust-version`. It gets you the `headwater` binary alone, at whatever the newest published version is; it does not get you `taxonomy-source`, so a reader who also wants the base taxonomy package still needs one of the two routes below.

`v0.1.2` is the name of the newest tagged release, and building it from source is the version to build if you want a tree that does not move under you and you also want the taxonomy package beside it. The block below is a source build, and it needs a Rust toolchain at **1.90 or later**, a floor `engine/README.md` explains and `engine/Cargo.toml` declares.

```
git clone https://github.com/headwater-ai/headwater.git
cd headwater
git checkout v0.1.2
cargo build --release -p headwater-cli --manifest-path engine/Cargo.toml --locked
```

The binary lands at `engine/target/release/headwater`, and `headwater --version` prints the number the tag names. [The GitHub release for `v0.1.0`](https://github.com/headwater-ai/headwater/releases/tag/v0.1.0) states one digest: the value of the `release.digest` field inside `packages/headwater-standard/release.yml`. It is not what `sha256sum` prints for that file, because the field covers the files the record lists and cannot cover the record itself. Pass the stated value to `headwater taxonomy vendor packages/headwater-standard --expect <digest>`, from the directory the block above leaves you in. The directory has to be named, because this engine opens no socket and checks a package somebody already fetched: `packages/headwater-standard` is the one the clone gave you, and a package vendors over itself. A matching digest exits 0 and reports the version and the file count; a digest that does not match exits 1, names both values and writes nothing. The check then rests on a number published outside the artifact rather than on one read out of it. `tools/headwater-bootstrap.sh` runs this same fetch-and-vendor step without a clone at all; the tutorial's step 3 shows it.

**An engine tag pins whatever `headwater/standard` version happened to ship with it, which is not always the newest one.** An engine tag is cut when the engine changes, not when the taxonomy does, so the version a `v<n>` tag carries can lag behind the version `taxonomy-source/headwater-standard/package.yml` states on the default branch for as long as nobody cuts the next engine release — [#757](https://github.com/headwater-ai/headwater/issues/757) is that gap, and it is a gap in the route rather than in the package. `taxonomy/headwater-standard/v<version>` is a second tag namespace, distinct from `v<version>` so the two never collide, cut independently by whoever maintains `headwater/standard` whenever they choose to publish a fixed version of it alone: no engine version bump and no engine tag required. `.github/workflows/release-taxonomy.yml` ([#760](https://github.com/headwater-ai/headwater/issues/760)) builds the engine at that tag, runs `headwater taxonomy publish` against `taxonomy-source/headwater-standard`, and attaches `headwater-standard-<version>.zip` to the tag's GitHub release, with the `release.digest` value `publish` printed stated in the release notes the same way the engine route states it on the release page. `tools/headwater-bootstrap.sh --tag taxonomy/headwater-standard/v<version> --expect <digest>` fetches it exactly the way it fetches an engine tag, because it reads whatever source tree a tag names and does not care which workflow cut it. [`taxonomy/headwater-standard/v4.2.0`](https://github.com/headwater-ai/headwater/releases/tag/taxonomy/headwater-standard/v4.2.0) is the first release this route ever cut, and the tutorial's step 3 fetches it.

**`v0.1.2` carries a binary and a checksum on its GitHub release.** `.github/workflows/release.yml` attaches `headwater-<tag>-x86_64-unknown-linux-gnu.tar.gz`, which holds the `headwater` binary and the license, beside `headwater-<tag>-x86_64-unknown-linux-gnu.tar.gz.sha256` for `sha256sum -c`, to every tag whose name starts with `v`. `v0.1.1` remains without a GitHub binary after its release collided with an immutable release object. That build is `x86_64` Linux on `ubuntu-24.04`, so it needs glibc 2.39 or later; every other platform still takes the source build above. [The releases page](https://github.com/headwater-ai/headwater/releases) is where you see which tags carry a binary.

The tutorial above builds whatever tree you cloned, which is the default branch and moves. That is deliberate, because the tutorial is a claim about the default branch and CI holds it there. This paragraph is where the fixed version is.

## Status

> **The engine runs, and two of its layers are unfinished.** This repository types its own corpus, checks it on every commit, and scaffolds, explains, routes, generates and exports it. Distribution and the measurement layer are the unfinished layers, and the canonical taxonomy library, ecosystem tooling, taxonomy expressiveness and this first release are the rest of what is still open. Every efficacy claim in this repository is still marked unmeasured.

## What problem this solves

Specs drift from code, rationale evaporates, standards multiply and contradict each other, and the AI assistants now reading that documentation as context inherit every one of those faults — silently, and at scale.

The usual answers are a style guide (unenforced), a wiki (unstructured), or a static-site generator (renders whatever you feed it). None of them can answer *"is this corpus still true?"*, because none of them know what kind of document anything is.

Headwater's premise: **make the structure of the corpus a machine-readable contract**, then derive everything else from it — validation, navigation, templates, AI instruction context, publishing, and the evidence that the whole thing is working.

## The three commitments

1. **Taxonomy is data.** What shelves exist, what document kinds live on them, what metadata they carry, what voice and lifecycle they obey, and how they may reference each other — all declared in one versioned schema. Two organizations with different documentation cultures run the same engine over different taxonomies. Customizing the taxonomy never means forking the tooling.

2. **The corpus is a graph.** Documents are typed nodes; front-matter references are typed edges. Every validation rule is a constraint on that graph, every query is a traversal of it, and every derived artifact (indexes, site navigation, AI rules, agent context) is a projection of it.

3. **AI assistants are readers in their own right, and measured ones.** The corpus feeds agents at intent time, read time, write time, and review time — and the system probes whether that context actually changes agent behavior, rather than assuming it does.

## License

Code is licensed under the Apache License, Version 2.0. See [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE).

The prose under `docs/` is licensed under Creative Commons Attribution 4.0 International. See [`docs/LICENSE`](docs/LICENSE). Code samples inside that prose stay under Apache-2.0, so copying an example into your own project carries no attribution obligation.

Contributions arrive under the [Developer Certificate of Origin](https://developercertificate.org/), with a `Signed-off-by` line and no contributor license agreement. [`CONTRIBUTING.md`](CONTRIBUTING.md) states why. [`SECURITY.md`](SECURITY.md) carries the disclosure process, and [`TRADEMARKS.md`](TRADEMARKS.md) states the trademark position, which is that there is no registered mark.

[Q11](docs/spec/09-decisions.md#q11--license-and-distribution-posture) records the reasoning, and the [first-contact evaluation](docs/evaluations/first-contact.md) carries the full argument. The design narrowed the field and did not choose within it: internal-only and source-available terms are refused by rulings the specification already made, and network copyleft is refused for the library that spec 6 requires. The choice among the terms that remained belongs to the owner, who made it on 2026-08-11.
