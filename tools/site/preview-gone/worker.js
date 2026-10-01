// SPDX-License-Identifier: Apache-2.0
//
// What a closed pull request's preview URL serves once
// `tools/site/unpreview-site.sh` has run: 410 Gone on every path, a pointer
// to the live site, and a header that keeps the URL out of every index.
// PR_NUMBER is the `--var` that script passes.
export default {
  fetch(request, env) {
    const pr = env.PR_NUMBER ? ` #${env.PR_NUMBER}` : "";
    return new Response(
      `This preview was removed when pull request${pr} closed.\nThe live site is https://headwater.tools/\n`,
      {
        status: 410,
        headers: {
          "content-type": "text/plain; charset=utf-8",
          "cache-control": "no-store",
          "x-robots-tag": "noindex",
        },
      },
    );
  },
};
