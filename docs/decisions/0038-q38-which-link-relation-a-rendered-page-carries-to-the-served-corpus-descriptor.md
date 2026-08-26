---
id: HW-DR-0038
status: draft
status_since: 2026-08-26
summary: "A rendered page points at the served descriptor with `rel=\"describedby\"`, because the IANA registry carries that token and the Headwater extension address answers 404."
last_verified: 2026-08-26
title: "Q38 — Which link relation a rendered page carries to the served corpus descriptor"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  governs:
    - mkdocs-overrides/main.html
  traces_to:
    - HW-SPEC-distribution-and-federation
    - HW-DR-0014
---

# Q38 — Which link relation a rendered page carries to the served corpus descriptor

## Context

**[Spec 7](../spec/07-distribution-and-federation.md#arriving-at-a-corpus-cold) requires the pointer and names no token for it.** It states the requirement once: "A rendered page therefore carries a link relation to the served copy". The same sentence adds that the copy may sit anywhere that the site can put it. No part of the specification names a relation. The string `describedby` appears nowhere under `docs/`.

**No rendered page carried such a link, and the descriptor sits outside the rendered tree.** [Q14](0014-discovery-surface.md) fixed the descriptor at `.headwater/corpus.json`, and `mkdocs.yml` sets `docs_dir: docs`. A build therefore copies no descriptor into the site and emits no pointer to one. A search of a built site for `describedby` found no page.

**Two candidate tokens stood in front of this question.** The first is `describedby`, which the IANA link relations registry carries under a W3C Recommendation. The second is `https://w3id.org/headwater/rel/corpus`, an extension relation under the namespace that [the w3id registration](../w3id/README.md) records.

**Three adjudications answered the question, and they did not agree.** The first recommended the extension relation. The second answered `describedby`. The third derived `describedby` again from RFC 8288 and from both registries. That pass also ran the W3C Nu validator and traced the namespace live. It replaced one of the three reasons the second gave, and it restated the other two. The reasons below are therefore the measured ones and not the inherited ones.

## Decision

**A rendered page carries one element, and the relation on it is `describedby`.** `mkdocs-overrides/main.html` emits it once, inside the `extrahead` block that the theme's `base.html` declares:

```html
<link rel="describedby" type="application/json" href="{{ 'corpus.json'|url }}">
```

Every page that MkDocs renders through `main.html` carries it. The `url` filter resolves the address against the page, so a page two directories down carries `href="../../corpus.json"`. That filter is the one `base.html` already uses for each of its own assets.

**Reach decides this question, and conformance decides nothing in it.** Section 3.3 of RFC 8288 requires the URI form of an extension relation type inside a `Link` header field. Section 2.1.2 permits another serialization to express the same relation type in another form. The RFC therefore puts no rule of its own on a `<link>` element. No conformance checker in reach separates the two forms either. The W3C Nu validator accepted both candidates, and it accepted an invented token as well. What divides them is what a stranger already knows. `describedby` sits in the IANA registry with a W3C Recommendation behind it. `https://w3id.org/headwater/rel/corpus` sits in no registry that a reader of HTML consults, so its meaning stays private to this corpus until something publishes it. A pointer whose whole job is to be followed by a stranger is the wrong place for a private token.

**The extension address answers 404, and every page would advertise it.** A request to `https://w3id.org/headwater/rel/corpus` answers 302 and then 404, measured on 2026-08-26. [The w3id registration](../w3id/README.md) already accepts a 404 under the same namespace for a Turtle request. The two are not the same 404. Nothing points at the Turtle address, so no reader meets it. The relation address would sit in the head of every rendered page, which is an address this corpus publishes and does not answer. To answer it costs a new live page under `site/`, which [Q37](0037-q37-which-parts-of-the-site-are-hand-built-and-which-are-a-projection-of-this-corpus.md) governs and which Cloudflare deploys on merge.

**The work that a Headwater token would do at the link is already done at the document.** Spec 7 requires a reader to test the shape of what it fetched before it believes the descriptor. A response of the wrong shape reads as an absent descriptor and not as a malformed one. The same section names the member that answers the rule, which is the top-level generated-file marker. `.headwater/corpus.json` carries it as `headwater:generated`, beside `descriptor_version` and `corpora`. A private relation token is a second and weaker copy of a test that the specification already requires.

**The `type` attribute stays on the element.** The descriptor states no media type of its own, so `type="application/json"` is the only place a pointer to it states one.

**Three conditions reopen this question.**

- Headwater registers a relation type, in the IANA registry or on the microformats `existing-rel-values` page that WHATWG HTML points a checker at.
- Headwater publishes a document at the extension address and serves the pair in an HTTP `Link` header field. Section 3.3 of RFC 8288 makes the URI form the required one there.
- A named consumer reports a collision, which is a second `describedby` on the same page that the consumer cannot separate from this one.

**Four alternatives are refused, and each one is refused for a stated reason.**

- **Both tokens in one `rel` attribute.** HTML takes `rel` as a set of tokens and RFC 8288 allows several relation types, so the form is legal. It is refused because it advertises the 404 on every page, and it buys a discrimination that the document already carries.
- **A `title` attribute on the element**, as free discrimination that mints no address. It is refused because `type` already narrows the target and the generated-file marker decides it. The third reopening condition covers a collision, and the attribute puts an unchecked human-readable string on every page.
- **A symlink or a committed copy at `docs/corpus.json`**, so that MkDocs copies it as a static asset. It is refused because it puts a file that is not Markdown into the corpus root. [Q29](0029-q29-whether-a-corpus-root-may-contain-code-and-what-an-interface-contract-may-reach.md) fixed that root at `docs`, and such a file then needs an exclusion and a reason for it.
- **A MkDocs plugin that copies the file.** It is refused because the theme is the built-in one and it needs no dependency. The CI step pins MkDocs so that the defaults of that theme are the policy. A plugin dependency undoes both.

## Consequences

**The 404 page carries no pointer, and the cause is structural.** The theme declares `404.html` as a static template, and that template extends `base.html` without a pass through `main.html`. An override in `main.html` therefore cannot reach it. A reader who arrives at a 404 reached no corpus to be pointed at.

**A build on a contributor machine leaves the pointer dangling.** The copy of the descriptor into the built site runs in the CI job, because `mkdocs build` cleans the output directory before it writes anything. So a contributor who runs `mkdocs build` by hand gets a link to a file that the build did not put there. One command after the build repairs it: `cp .headwater/corpus.json .headwater/site-build/corpus.json`.

**A strict build cannot hold this ruling, so a separate CI step holds it.** With the `custom_dir` line deleted from `mkdocs.yml`, `mkdocs build --strict` exits 0 with no warning while every page loses the element. MkDocs has no opinion about a `<link>` that a theme does not emit. The step that holds Q38 reads the built tree and names every page with no pointer in it.

**Nothing about the built site reaches a reader outside this repository today.** [Q31](0031-q31-whether-this-repository-becomes-public-and-when.md) rules that this repository becomes public and leaves the date to the owner, and [HW-OBL-0130](../obligations/0130-the-publication-date-of-this-repository-is-unset-and-the-owner-alone-sets-it.md) refuses a date that an agent proposes. The element is therefore verifiable in the CI job and in no public place.

**This record narrows no other decision, and it declares no `constrains` edge.** It answers a requirement that spec 7 already states, where [Q37](0037-q37-which-parts-of-the-site-are-hand-built-and-which-are-a-projection-of-this-corpus.md) narrowed the population of an earlier ruling.

**Two things sit outside this record.** The visual design of the rendered half is one. The hand-built pages under `site/` are the other, and Q37 governs those. No page under `site/` carries this pointer.

**This record governs one file.** `mkdocs-overrides/main.html` is the whole of the mechanism, and it is the only reason that the theme has a custom directory at all. A `governs` edge reaches the path it names and no path under it, which [HW-OBL-0104](../obligations/0104-a-governs-edge-reaches-the-path-it-names-and-nothing.md) prices.
