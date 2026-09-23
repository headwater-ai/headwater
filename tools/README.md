# tools

Scripts this repository runs on itself. Every file here is local to this repository, which is population 4 of [HW-DR-0077](../docs/decisions/0077-the-consumer-surface-is-what-an-adopter-receives-runs-and-must-have-installed-and-it-is-a-closed-and-declared-list.md), and no page for an adopter may tell a reader to run one. `surface.local_path.instructed` holds that, over the pages that `surface.adopter_documents` in `.headwater/overlay.yml` lists. Two scripts here are still on an adopter's path today, and each one leaves: `headwater-bootstrap.sh` when `headwater taxonomy vendor` takes a location (#959), and `cite/check-citations.py` when it moves out of this directory as a companion. Nothing here is a governed document: the corpus root is `docs` and no shelf claims this directory.

One directory per subject, because the thing a script acts on is what a reader is looking for. A fixture suite lives beside the thing it holds rather than with the other fixture suites, which is the convention `.githooks/fixtures.sh` and `.claude/skills/fixtures.sh` already follow.

| directory | what its scripts act on |
|---|---|
| `site/` | the website: the two halves, the assembly that composes them, the checkers CI runs over the served pages, and the refreshers that write a measured figure into a hand-built page |
| `engine/` | the Rust workspace under `engine/`: the declarations its build makes about itself, the terminal-sensing promise of the command line, the container recipe an outsider runs, and the local build speedup |
| `repo/` | this repository's own artifacts: the root `README.md`, `DEVELOPING.md`, the library index, the `diataxis-site` entry, the identifier claim store under `.headwater/ids/`, a fetched-added-built worktree, and the worktrees and branches a finished change leaves behind |
| `run/` | a build-order run: its run directory, the census of what a parent's turns were spent on, the dispatch of a run to another harness, a bounded wait that re-issues itself under the cache lifetime, and this repository's issues and project board over `gh` |
| `probe/` | the recorder that turns a harness session log into probe events, and the blessed input it is held against |
| `cite/` | a citation comment in a code tree: the checker that holds one against the corpus that licensed it, and the fixtures planted at governed and ungoverned paths that hold the checker |
| `taxonomy/` | the fixture corpora under `docs/taxonomies/`: the recipe each README prints and the figures it states about the run |

Three scripts stand at the top of this directory rather than in one of those, and all three stay there. They are entry points a person types by name, where every script below is one that a subject owns. `hw-cargo-fixtures.sh` stands beside the one it holds, the same rule that puts a fixture suite beside its subject everywhere else.

`headwater-bootstrap.sh` is the one script written for somebody outside this repository, and `site/tutorial/index.html` publishes a `curl` of its raw URL on `main`. Moving it breaks an install command that people have already copied.

`hw-cargo` wraps every cargo command of a session on a host that runs several at once. [DEVELOPING.md](../DEVELOPING.md) and the `hw-run-policy` skill both spell the path out in a command line a reader retypes, so a longer one costs something on every use and buys nothing.

`cap-run` bounds a command's output to a head and a tail on request, the whole of it kept on disk, for a session that expects a command to print a lot. Its own header states why this is a plain script rather than a `PreToolUse` hook, which is what bounded every command's output unconditionally until a worktree-isolated session started refusing the shape it generated.
