---
name: site-preview
description: Put a pull request's build of https://headwater.tools/ at a private preview URL so a person can read it before the change merges, and tell them when it shows the latest push. Use when a change touches `site/`, the README, a document the site renders, or anything the owner asks to see before merging, and when asked for a preview, a preview link or a preview branch. It labels the pull request, waits for the build, and reports the URL with the commit it shows. It never deploys the live site.
---

# Site preview

A pull request from a branch of this repository gets its own copy of the site at `https://pr-<N>-headwater.j-w-baxter.workers.dev`, where `<N>` is the pull request number. The live site keeps serving its last deploy from `main`. [DEVELOPING.md](../../../DEVELOPING.md) states the mechanism for a person; this file is the procedure for an agent.

## Start a preview

Add the `site-preview` label. Use the REST call, because `gh pr edit --add-label` has failed in this repository on a Projects (classic) error:

    gh api -X POST repos/headwater-ai/headwater/issues/<N>/labels -f 'labels[]=site-preview'

`.github/workflows/preview-site.yml` then runs the `Preview the site` job. It builds the engine and the site the way the production deploy does, and `tools/site/preview-site.sh pr-<N>` uploads the result as a preview version that takes no traffic. The job took 7 minutes 18 seconds on its first run, on pull request #2031.

The job runs only when all three hold:

- the pull request's head branch is in this repository, so a fork gets no preview because its run has no secrets;
- the `site-preview` label is on the pull request;
- on a `labeled` event, the label just added is `site-preview`, so an unrelated label does not rebuild.

It reads `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` from the repository secrets.

## Know which commit the preview shows

The job keeps one comment on the pull request, marked `<!-- headwater:site-preview -->`, and edits it on every build. Its second line is the URL, and its fourth line says `Built from <8-character SHA>`. Each later push to the branch rebuilds the preview under the same URL while the label stays on.

Report a preview only after the comment names the commit you pushed. Before that, the URL serves the previous build, and a reviewer reads text you have already replaced. Read the comment with:

    gh api repos/headwater-ai/headwater/issues/<N>/comments --jq '.[]|select(.body|startswith("<!-- headwater:site-preview -->"))|.body'

To wait for a push, poll that comment every 30 seconds in a background command until it contains the first eight characters of your commit, and stop after 40 minutes. A `Preview the site` check that fails in `gh pr checks <N>` means no comment will come.

## Give the reviewer the link

Send the URL, the pull request, the commit the preview shows, and the pages that changed, named by path on the site (for example `/proof/`). The figures on the hand-built pages are filled at build time ([HW-DR-0097](../../../docs/decisions/0097-a-figure-on-a-hand-built-page-is-measured-when-the-site-is-published-and-the-committed-page-carries-none.md)), so the preview shows them as a deploy of that commit would.

When the owner has asked to see a change before it merges, a preview does not license the merge. Hold the pull request until the owner answers.

## Stop a preview

- Remove the `site-preview` label to stop rebuilding on each push. The URL keeps serving the last build.
- When the pull request closes, merged or not, `.github/workflows/unpreview-site.yml` uploads a version under the same alias that answers 410 Gone, because Cloudflare deletes no preview alias, and it edits the comment to say so.
- For a pull request that closed before that workflow existed, run `Remove the site preview` from the Actions tab with the pull request number, or `gh workflow run unpreview-site.yml -f number=<N>`.

The preview never runs `wrangler deploy`, and `tools/repo/release-guide-fixtures.sh` holds that.
