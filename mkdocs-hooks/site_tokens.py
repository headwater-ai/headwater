"""Serve `tools/site-tokens.css` to the generated half, as the same file.

WHY A HOOK AND NOT A COPY

HW-DR-0050 rules that the visual register of this site is seven color tokens,
three type stacks and two classes that name the stacks, and that
`tools/site-tokens.css` is the one copy a person edits. It scopes the mechanism
to the hand-built half: `tools/refresh-site-tokens.sh` interpolates that block
into every `index.html` under `site/`, because `site/_headers` gives those paths
`style-src 'unsafe-inline'` with no `'self'`, so a hand-built page can reach
nothing at read time and an inlined block is the only form available.

The generated half is served from `/*`, whose policy does carry `style-src
'self'`. So it can link the register instead of restating it, and it must,
because a second copy of `#fbfaf8` and `#1d5c54` in a theme stylesheet is
exactly what HW-DR-0050 exists to prevent, and nothing in this repository would
catch it drifting: `tools/refresh-site-tokens.sh --check` walks `site/` alone.
HW-DR-0050's fourth clause is the ruling this file implements.

WHAT IT DOES

One `File`, generated at build time, whose bytes are read from
`tools/site-tokens.css` — not copied into it, and not written to disk anywhere
in the checkout. `mkdocs build` places it at `css/site-tokens.css` in the
output, `mkdocs-overrides/main.html` links it ahead of the theme's own
stylesheet, and `mkdocs serve` rebuilds it when the source file changes because
the file is registered with its real source path.

`tools/site-tokens.css` sits outside `docs_dir`, so no build step would
otherwise reach it. That is the same reason `.headwater/corpus.json` needs
`tools/assemble-site.sh` to place it.

WHAT HOLDS IT

`tools/site-fragments-fixtures.sh` and the CI step that runs
`tools/check-site-fragments.py` walk the served bytes. The narrower assertion
this file needs is that the served copy is byte-identical to the source, and
`tools/site-tokens-fixtures.sh` is where that lives.

NO NEW DEPENDENCY

`hooks:` is a core MkDocs option since 1.4 and this repository pins 1.6.1. It
adds nothing to the two places that must stay identical, `.github/workflows/ci.yml`
and `tools/cloudflare-build.sh`.
"""

from pathlib import Path

from mkdocs.structure.files import File

# The one copy, relative to the repository root. `mkdocs.yml` sits at that root,
# so `config.config_file_path` is what makes this path independent of the
# working directory a build was started from.
SOURCE = Path("tools") / "site-tokens.css"

# Where the generated half serves it. `mkdocs-overrides/main.html` links this
# exact path and `css/` is where the theme keeps its own stylesheets, so the
# register arrives beside them rather than at a path of its own invention.
TARGET = "css/site-tokens.css"


def on_files(files, config):
    root = Path(config.config_file_path).parent
    source = root / SOURCE
    if not source.is_file():
        raise FileNotFoundError(
            f"{SOURCE} is the one copy of the visual register that HW-DR-0050 "
            f"rules, and it is not at {source}. The generated half links it "
            f"rather than restating it, so a build without it would serve "
            f"every page unstyled and exit 0."
        )
    files.append(File.generated(config, TARGET, abs_src_path=str(source)))
    return files
