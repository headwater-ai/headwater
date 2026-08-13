# rdf-probe

A throwaway emitter, kept because an evaluation cites its numbers.

[The OWL and SKOS worked example](../../docs/evaluations/owl-skos-worked-example.md) asks what the taxonomy loses when it is emitted as an ontology and the corpus as instance triples. `emit.py` is what produced that document's figures. It is not emitter 4 of the [Q13](../../docs/spec/09-decisions.md#q13--linkml-and-shacl-as-substrate) staging order, and it is not a draft of one — that emitter ships when a named external consumer asks, and this exists to say what it will owe when it does.

It reads the base package out of the fenced YAML in [the first-run walkthrough](../../docs/evaluations/default-taxonomy-first-run.md#the-base-derived-rather-than-chosen), because that is the only committed copy, applies the [design-spec bundle](../../docs/taxonomies/design-spec/bundle.yml) as add-only operations, and writes four files.

| File | What it holds |
|---|---|
| `taxonomy.ttl` | the TBox as OWL and SKOS |
| `corpus.ttl` | the ABox, typed by placement, with the Markdown link graph as edges |
| `shapes.ttl` | the same required-facet constraints in SHACL, for the comparison |
| `loss.json` | the projection census and the declared loss set |

## Running it

    python3 -m venv .venv
    .venv/bin/pip install rdflib pyshacl owlrl pyyaml
    .venv/bin/python tools/rdf-probe/emit.py --repo . --out out

Four third-party dependencies is why this is not in the commit hook and not in `tools/` proper. The commit hook runs the engine, which is one built binary and no interpreter at all. This probe should not grow until an emitter has a consumer to be written for.

## What it does not do

No SPARQL, no OKF, no LinkML. It emits what a knowledge-organization consumer would read and reports what fell out. The three checks it runs — OWL-RL closure, SHACL validation, and a serialize-parse round trip — are there to make the losses observable rather than asserted.
