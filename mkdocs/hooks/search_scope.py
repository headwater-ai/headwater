"""Keep the campaign's recorded sessions out of the site's search index.

WHY A HOOK AND NOT A PLUGIN OPTION

The base `mkdocs` theme builds `search/search_index.json` with MkDocs's own
search plugin, and that plugin has no option that leaves a page out. The page
is still served and still linked from the nav; only its text is withheld from
the index a reader's browser downloads.

WHY THESE TWO SHELVES

`docs/probe-runs/` and `docs/probe-results/` hold one transcript and one result
per probe line of the paid layer campaign (#1659). They are tables of recorded
sessions, so a reader searches them by identifier and never by prose. Landing
them (#1697, `42851aca`) took `search_index.json` to 43.9 MiB, and Cloudflare
Workers refuses an asset above 25 MiB, so the deploy stopped at `wrangler` and
the site has not updated since.

WHAT HOLDS IT

`tools/site/check-site-asset-size.sh` fails a build whose served directory holds
any file above the limit, and `tools/site/site-asset-size-fixtures.sh` holds that
check. The hook is the remedy and the check is the gate: a third shelf that
grows past the limit fails a pull request, and this list is where its remedy goes.

NO NEW DEPENDENCY

`hooks:` is a core MkDocs option since 1.4 and this repository pins 1.6.1.
"""

import json
from pathlib import Path

# Page locations under `site_dir`, as the search plugin writes them: relative,
# with `use_directory_urls` giving `probe-runs/<stem>/` and a README its shelf root.
WITHHELD = ("probe-runs/", "probe-results/")


def on_post_build(config, **kwargs):
    index = Path(config["site_dir"]) / "search" / "search_index.json"
    if not index.is_file():
        return
    data = json.loads(index.read_text(encoding="utf-8"))
    kept = [d for d in data["docs"] if not d["location"].startswith(WITHHELD)]
    dropped = len(data["docs"]) - len(kept)
    data["docs"] = kept
    index.write_text(json.dumps(data, separators=(",", ":")), encoding="utf-8")
    print(
        "search_scope: withheld %d of %d index entries under %s"
        % (dropped, dropped + len(kept), ", ".join(WITHHELD))
    )
