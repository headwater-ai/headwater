// A HEADING ANCHOR THAT STARTS WITH A DIGIT, AND THE ONE CALL THAT CANNOT READ IT
//
// This corpus writes headings like `## 9. The decision register`, and the
// slugifier `mkdocs.yml` names derives `9--the-decision-register` from one.
// That identifier is correct, it is the one GitHub derives, and #431 and #544
// are the rulings that fixed it: 335 citation links resolve against it and the
// engine's own `fragment.rs` derives the same string. So the slug does not
// move here.
//
// A CSS identifier may not start with a digit. `#9--the-decision-register` is
// therefore a valid URL fragment and an invalid CSS selector, and the theme's
// own script hands the one to the other. Bootstrap's ScrollSpy reads each
// sidebar link's `hash` and passes it to `Element.prototype.querySelector`,
// which throws `SyntaxError: '#9--the-decision-register' is not a valid
// selector`.
//
// The throw lands at the top level of the theme's `js/base.js`, thirteen lines
// before that file defines `keyCodes`. Evaluation stops there, so `keyCodes`
// stays `undefined`, and the `DOMContentLoaded` listener the same file
// registered earlier then throws a second time, `TypeError: Cannot read
// properties of undefined (reading '191')`. Both messages on the 13 affected
// pages come from this one call. Nothing else on those pages is broken.
//
// WHAT THIS DOES, AND WHAT IT DELIBERATELY DOES NOT DO
//
// It wraps `Element.prototype.querySelector` so that a call which *already
// throws* `SyntaxError`, and whose selector is a single `#name` with no
// combinator, comma, bracket or quote in it, is retried once with `CSS.escape`
// applied to the name. `CSS.escape('9--the-decision-register')` yields
// `\39 --the-decision-register`, which selects the same element a browser
// scrolls to when a reader clicks the link.
//
// A call that does not throw never reaches the retry, so no selector that
// works today resolves differently, and no call gains a second pass over the
// document. A call that throws for any other reason — a bad `:not(`, a stray
// bracket, an empty string — is rethrown with its own error object, unchanged.
//
// It patches one method on one interface. Not `querySelectorAll`, not
// `Document.prototype`, not `matches` or `closest`: the failing call is
// `Element.prototype.querySelector` invoked on `document.body`, measured, and
// a patch is worth exactly the calls it has to cover. Adding an interface here
// needs a page that fails without it.
//
// The cost this accepts, stated plainly: it is a global change to a DOM
// prototype, so every script on the page inherits the wrapper, including the
// theme's search and any script a future `extra_javascript` adds. The wrapper
// is transparent for every call that succeeds, which is every call this
// repository makes today, and the alternative — vendoring the theme's 287-line
// `js/base.js` into this repository, where it would drift from the pinned
// MkDocs release in silence — buys a smaller blast radius with a larger one.
//
// `tools/check-site-console.py` is what holds this: it loads every served page
// in a real browser and fails on any console message, and
// `tools/site-console-fixtures.sh` provokes that failure on purpose.

(function () {
    'use strict';

    var proto = window.Element && window.Element.prototype;
    if (!proto || typeof proto.querySelector !== 'function') {
        return;
    }
    if (!window.CSS || typeof window.CSS.escape !== 'function') {
        return;
    }

    // A single id selector and nothing else. Anything carrying a combinator, a
    // comma, a bracket, a parenthesis, a quote or a backslash is somebody
    // else's selector and somebody else's error.
    var LONE_ID = /^#[^\s,>+~()[\]'"\\]+$/;

    var original = proto.querySelector;

    function retryWith(selectors, error) {
        // `error.name`, not `error instanceof SyntaxError`. An invalid selector
        // throws a `DOMException` whose `name` is `'SyntaxError'`, and a
        // `DOMException` is not an instance of `SyntaxError`. Written the other
        // way first, and measured: the wrapper installed, the retry never ran,
        // and both console messages appeared unchanged.
        if (!error || error.name !== 'SyntaxError') {
            return null;
        }
        if (typeof selectors !== 'string' || !LONE_ID.test(selectors)) {
            return null;
        }
        return '#' + window.CSS.escape(selectors.slice(1));
    }

    proto.querySelector = function (selectors) {
        try {
            return original.apply(this, arguments);
        } catch (error) {
            var retry = retryWith(selectors, error);
            if (retry === null) {
                throw error;
            }
            return original.call(this, retry);
        }
    };
}());
