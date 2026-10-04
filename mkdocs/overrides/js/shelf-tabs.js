// Turns each group tab of the shelf switcher into a toggle for its panel
// (#1681). The template writes each tab as a link to its group's first shelf,
// and writes every panel, with the panel of the group that holds the page open.
// So with no script every shelf is still two clicks away, and this file only
// saves the page load. A tab opens its panel and closes the others, and a
// second click on an open tab closes it. The underline of the current group
// is the `cur` class the template wrote, and this file never moves it.
(function () {
  var tabs = document.querySelectorAll('.hw-tab[data-panel]');
  if (!tabs.length) { return; }
  function panelOf(tab) { return document.getElementById(tab.getAttribute('data-panel')); }
  tabs.forEach(function (tab) {
    var panel = panelOf(tab);
    if (!panel) { return; }
    tab.setAttribute('role', 'button');
    tab.setAttribute('aria-controls', panel.id);
    tab.setAttribute('aria-expanded', panel.hidden ? 'false' : 'true');
    tab.addEventListener('click', function (event) {
      event.preventDefault();
      var open = tab.getAttribute('aria-expanded') === 'true';
      tabs.forEach(function (other) {
        var p = panelOf(other);
        if (p) { p.hidden = true; }
        other.setAttribute('aria-expanded', 'false');
      });
      if (!open) {
        panel.hidden = false;
        tab.setAttribute('aria-expanded', 'true');
      }
    });
  });
})();
