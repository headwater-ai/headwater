# Draft: a pull-request body for n8n-io/n8n

**Nobody has sent this.** It is a draft for a person to rewrite in their own words and to open under their own name, after the issue in [`issue.md`](issue.md) is filed and linked. n8n's `CONTRIBUTING.md` section 3 closes a pull request with no real description, section 4 forbids pasted model output and invites a disclosure of AI assistance, and section 5 rejects a typo-only change, which is why all three fixes go in one pull request rather than three.

The commits are on the branch [`fix-agents-doc-defects`](https://github.com/headwater-ai/n8n/tree/fix-agents-doc-defects) of the fork `headwater-ai/n8n`, cut from `b0550cb3cb4d1752546a69056c55eccfb9111a12`. Rebase onto current `master` before opening. All three files carried the same bytes at that commit as on `master` on 2026-09-08.

Suggested title, following `.github/pull_request_title_conventions.md`: `docs: Fix three factual defects in .agents/ and the expression-runtime doc`

---

Fixes the three items in #<issue number>.

**`packages/@n8n/expression-runtime/ARCHITECTURE.md`.** The reference link to the workflow package pointed one directory too high, at `packages/@n8n/workflow/`, which does not exist. It now points at `packages/workflow/`.

**`.agents/skills/spec-driven-development/SKILL.md`.** The skill opened by stating that specs live in `.agents/specs/` and are the source of truth, and the directory is not in the tree. Rather than guess whether the directory is planned, the claim is now conditional and the skill says what to do on a checkout that has no specs at all: read an empty or failing listing as "no spec exists" for every feature and go to step 3, which is the step that already handles that case. The `ls` in step 1 no longer prints an error on a checkout with no such directory. Tell me if you would rather commit the directory and keep the claim flat, and I will send that instead.

**`.agents/review-rules/README.md`.** The sentence about which agents deliberately do not link `testing/` named two, and `cubic.yaml` has three: Security, DB migrations and QA & DX. The Layout table above it is already correct. The stated reason now names a migration alongside a credential fix and a Dockerfile.

No tests. Three documentation lines, no behavior, and `pnpm check:cubic-config` reads paths and character counts rather than prose, so it does not cover any of this.

**Disclosure.** I used an AI assistant to locate these three defects while reading this repository's `.agents/` tree. I checked each one by hand against `master`, I wrote the fixes, and I wrote this description.
