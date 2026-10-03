// Turns each fenced mermaid block into a diagram. The page emits
// `<pre><code class="language-mermaid">`, and mermaid reads a node's text, so
// each `<pre>` is replaced by a `<div class="mermaid">` carrying that text.
// Loaded only on a page that has such a block, after `js/mermaid.min.js`.
(function () {
  var blocks = document.querySelectorAll('pre > code.language-mermaid');
  if (!blocks.length || typeof mermaid === 'undefined') { return; }
  blocks.forEach(function (code) {
    var div = document.createElement('div');
    div.className = 'mermaid';
    div.textContent = code.textContent;
    code.parentNode.replaceWith(div);
  });
  var dark = document.documentElement.getAttribute('data-bs-theme') === 'dark';
  mermaid.initialize({ startOnLoad: false, securityLevel: 'strict', theme: dark ? 'dark' : 'default' });
  mermaid.run({ querySelector: 'div.mermaid' });
})();
