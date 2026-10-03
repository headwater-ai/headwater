// Turns each fenced mermaid block into a diagram. The page emits
// `<pre><code class="language-mermaid">`, and mermaid reads a node's text, so
// each `<pre>` is replaced by a `<div class="mermaid">` carrying that text.
// Loaded only on a page that has such a block, after `js/mermaid.min.js`.
//
// The colors come from the register's custom properties (`--bg`, `--fg`,
// `--muted`, `--rule`, `--accent`, `--code-bg`), which `css/site-tokens.css`
// flips under `prefers-color-scheme`. Reading them keeps a diagram in step with
// the page in both modes. The script cannot read `data-bs-theme` for this:
// `js/darkmode.js` sets it after this file runs. A change of the system
// preference redraws each diagram from the text kept in `data-source`.
(function () {
  var blocks = document.querySelectorAll('pre > code.language-mermaid');
  if (!blocks.length || typeof mermaid === 'undefined') { return; }
  blocks.forEach(function (code) {
    var div = document.createElement('div');
    div.className = 'mermaid';
    div.setAttribute('data-source', code.textContent);
    div.textContent = code.textContent;
    code.parentNode.replaceWith(div);
  });

  function token(name) {
    return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  }

  function draw() {
    var bg = token('--bg');
    var fg = token('--fg');
    var muted = token('--muted');
    var rule = token('--rule');
    var accent = token('--accent');
    var panel = token('--code-bg');
    mermaid.initialize({
      startOnLoad: false,
      securityLevel: 'strict',
      theme: 'base',
      themeVariables: {
        background: bg,
        fontFamily: token('--sans'),
        primaryColor: panel,
        primaryTextColor: fg,
        primaryBorderColor: accent,
        secondaryColor: panel,
        secondaryTextColor: fg,
        secondaryBorderColor: rule,
        tertiaryColor: bg,
        tertiaryTextColor: fg,
        tertiaryBorderColor: rule,
        lineColor: muted,
        textColor: fg,
        mainBkg: panel,
        nodeBorder: accent,
        clusterBkg: bg,
        clusterBorder: rule,
        edgeLabelBackground: bg,
        titleColor: fg
      }
    });
    document.querySelectorAll('div.mermaid').forEach(function (div) {
      div.removeAttribute('data-processed');
      div.textContent = div.getAttribute('data-source');
    });
    mermaid.run({ querySelector: 'div.mermaid' });
  }

  draw();
  if (window.matchMedia) {
    var query = window.matchMedia('(prefers-color-scheme: dark)');
    if (query.addEventListener) { query.addEventListener('change', draw); }
  }
})();
