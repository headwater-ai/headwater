/* darkmode.js — this file SHADOWS the stock MkDocs theme's file of the same
 * path. A file under `custom_dir` wins over the theme's copy, and this is the
 * only file in `mkdocs-overrides/` that replaces one rather than adding one.
 *
 * WHY IT REPLACES THE THEME'S COPY
 *
 *   `mkdocs.yml` sets `highlightjs: false`, which is HW-OBL-0160's own remedy,
 *   and `color_mode: auto`, so the visual register's `prefers-color-scheme`
 *   block reaches a reader. The theme's `base.html` emits the `#hljs-light`
 *   and `#hljs-dark` link elements only under `highlightjs`, and its
 *   `js/darkmode.js` does `hljs_light.disabled = true` on line 7 with no null
 *   guard. So the two settings together throw on every page that loads it.
 *
 *   Measured, at the commit before this file existed:
 *
 *       python3 tools/check-site-console.py   ->  exit 1
 *       325 pages with a finding, out of 333 served pages
 *
 *   `mkdocs build --strict`, `tools/assemble-site.sh --check` and
 *   `tools/check-site-fragments.py` each exit 0 over that same tree.
 *   `site/_headers` already records the trap in prose for the hand-built half.
 *
 * WHAT IT DOES, AND WHAT IT DELIBERATELY DROPS
 *
 *   It sets `data-bs-theme` on the root element from the system preference and
 *   keeps it in step when that preference changes. That is the whole of what
 *   the stock file still had to do here.
 *
 *   Dropped: the two `hljs` stylesheet swaps, which have no elements to act on.
 *   Dropped: `updateModeToggle` and the click handlers, which drive the
 *   Bootstrap navbar dropdown that `user_color_mode_toggle: false` does not
 *   emit and that this theme has no navbar for. Dropped with them: the
 *   `mkdocs-colormode` value in `localStorage`, which nothing on this site can
 *   set once the toggle is gone. A reader who has one from an older visit is
 *   served the system preference instead, and no page reads a stale choice.
 *
 *   CSS already answers the same question on its own: `css/site-tokens.css`
 *   carries a `@media (prefers-color-scheme: dark)` block, so the register
 *   flips with no script at all. This file exists for Bootstrap's own
 *   components, which read the `data-bs-theme` attribute and not the media
 *   query.
 *
 * `tools/site-console-fixtures.sh` holds it: one case puts the theme's
 * unguarded line back over the real served bytes and asserts the run fails.
 */
(function () {
    var mql = window.matchMedia('(prefers-color-scheme: dark)');

    function apply(dark) {
        document.documentElement.setAttribute('data-bs-theme', dark ? 'dark' : 'light');
    }

    apply(mql.matches);

    // `addEventListener` on a MediaQueryList, with the deprecated
    // `addListener` as the fallback, because a browser that has only the old
    // one must not throw here — an exception in this file is the whole defect
    // this file exists to remove.
    if (typeof mql.addEventListener === 'function') {
        mql.addEventListener('change', function (event) { apply(event.matches); });
    } else if (typeof mql.addListener === 'function') {
        mql.addListener(function (event) { apply(event.matches); });
    }
})();
