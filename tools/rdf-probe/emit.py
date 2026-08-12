#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Emit the Headwater taxonomy as OWL + SKOS, and this corpus as instance triples.

Evidence for docs/evaluations/owl-skos-worked-example.md. Not an emitter that
ships: emitter 4 of the Q13 staging order waits on a named external consumer.
This exists to find out what an RDF projection cannot carry, by building one and
looking at what falls out.

Reads the base package out of the fenced YAML in the first-run walkthrough (the
only committed copy) plus the design-spec bundle, resolves the overlay, and
writes four files into an output directory:

    taxonomy.ttl   the TBox as OWL + SKOS
    corpus.ttl     the ABox, derived from placement and the link graph
    shapes.ttl     SHACL shapes for the same constraints, for the comparison
    loss.json      the declared loss set and the projection census

Then it runs the checks the evaluation reports: syntax, OWL-RL entailment,
SHACL validation, and a round trip.

Usage:  emit.py [--repo PATH] [--out PATH]
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

import yaml
from rdflib import BNode, Graph, Literal, Namespace, URIRef
from rdflib.compare import isomorphic
from rdflib.namespace import DCTERMS, OWL, RDF, RDFS, SKOS, XSD

HW = Namespace("https://w3id.org/headwater/taxonomy/")
HWC = Namespace("https://w3id.org/headwater/corpus/")
SH = Namespace("http://www.w3.org/ns/shacl#")
PROV = Namespace("http://www.w3.org/ns/prov#")

# Where the base package YAML lives. It has no machine-readable home yet, so the
# fenced block in the walkthrough is the source of truth. See the evaluation.
BASE_DOC = "docs/evaluations/default-taxonomy-first-run.md"
BUNDLE = "docs/taxonomies/design-spec/bundle.yml"

XSD_FOR = {"date": XSD.date, "string": XSD.string, "integer": XSD.integer}


class Loss:
    """The declared loss set, accumulated as the emitter refuses to carry things."""

    def __init__(self) -> None:
        self.entries: list[dict] = []

    def add(self, layer: str, construct: str, reason: str, count: int = 1) -> None:
        for e in self.entries:
            if e["layer"] == layer and e["construct"] == construct:
                e["count"] += count
                return
        self.entries.append(
            {"layer": layer, "construct": construct, "reason": reason, "count": count}
        )

    def total(self) -> int:
        return sum(e["count"] for e in self.entries)


def read_base(repo: Path) -> dict:
    """Pull the base package out of the fenced YAML block in the walkthrough."""
    text = (repo / BASE_DOC).read_text(encoding="utf-8")
    m = re.search(r"```yaml\n(.*?)\n```", text, re.S)
    if not m:
        sys.exit(f"no fenced YAML block in {BASE_DOC}")
    return yaml.safe_load(m.group(1))


def apply_overlay(base: dict, bundle: dict) -> dict:
    """Apply the bundle's add operations at their dotted addresses."""
    resolved = yaml.safe_load(yaml.safe_dump(base))  # deep copy through YAML
    for address, value in (bundle.get("add") or {}).items():
        parts = address.split(".")
        node = resolved
        for part in parts[:-1]:
            node = node.setdefault(part, {})
        if parts[-1] in node:
            sys.exit(f"bundle is not add-only: {address} already exists in the base")
        node[parts[-1]] = value
    return resolved


def resolve_vocab_ref(value, resolved: dict):
    """Follow a $vocabularies.<name> reference to its value list."""
    if isinstance(value, str) and value.startswith("$vocabularies."):
        return resolved["vocabularies"][value.split(".", 1)[1]]
    return value


# --------------------------------------------------------------------- TBox


def emit_tbox(resolved: dict, loss: Loss) -> Graph:
    g = Graph()
    for prefix, ns in [
        ("hw", HW), ("owl", OWL), ("rdfs", RDFS), ("skos", SKOS),
        ("xsd", XSD), ("prov", PROV), ("dcterms", DCTERMS),
    ]:
        g.bind(prefix, ns)

    onto = URIRef("https://w3id.org/headwater/taxonomy")
    g.add((onto, RDF.type, OWL.Ontology))
    g.add((onto, DCTERMS.title, Literal("Headwater design-spec taxonomy", lang="en")))

    _emit_vocabularies(g, resolved, loss)
    _emit_purposes(g, resolved)
    _emit_facets(g, resolved, loss)
    _emit_kinds(g, resolved, loss)
    _emit_relations(g, resolved, loss)
    _emit_anchors(g, resolved, loss)
    _note_unrepresentable(resolved, loss)
    return g


def _emit_vocabularies(g: Graph, resolved: dict, loss: Loss) -> None:
    """A controlled vocabulary is a SKOS concept scheme. The role on each value is not."""
    for name, values in (resolved.get("vocabularies") or {}).items():
        scheme = HW[f"scheme/{name}"]
        g.add((scheme, RDF.type, SKOS.ConceptScheme))
        g.add((scheme, SKOS.prefLabel, Literal(name.replace("_", " "), lang="en")))
        for entry in values:
            value = entry["value"] if isinstance(entry, dict) else entry
            concept = HW[f"{name}/{value}"]
            g.add((concept, RDF.type, SKOS.Concept))
            g.add((concept, SKOS.inScheme, scheme))
            g.add((concept, SKOS.prefLabel, Literal(value, lang="en")))
            g.add((scheme, SKOS.hasTopConcept, concept))
            if isinstance(entry, dict) and "role" in entry:
                # SKOS has no slot for the lifecycle role of a value. Carried as a
                # non-standard annotation, which no stock SKOS consumer reads.
                g.add((concept, HW.lifecycleRole, Literal(entry["role"])))
                loss.add(
                    "vocabulary",
                    "lifecycle_state role",
                    "SKOS has no property for the role a value plays in a state "
                    "machine; carried as a Headwater annotation a stock consumer ignores",
                )


def _emit_purposes(g: Graph, resolved: dict) -> None:
    """A purpose is a reader intent. It classifies a kind, so it needs punning."""
    for name, body in (resolved.get("purposes") or {}).items():
        purpose = HW[f"purpose/{name}"]
        g.add((purpose, RDF.type, SKOS.Concept))
        g.add((purpose, SKOS.inScheme, HW["scheme/purpose"]))
        g.add((purpose, SKOS.prefLabel, Literal(name, lang="en")))
        g.add((purpose, SKOS.definition, Literal(body["intent"], lang="en")))
        for answer in body.get("answers", []):
            g.add((purpose, SKOS.scopeNote, Literal(answer, lang="en")))
    g.add((HW["scheme/purpose"], RDF.type, SKOS.ConceptScheme))


def _emit_facets(g: Graph, resolved: dict, loss: Loss) -> None:
    for name, body in (resolved.get("facets") or {}).items():
        prop = HW[name]
        values = resolve_vocab_ref(body.get("values"), resolved)
        if values:
            g.add((prop, RDF.type, OWL.ObjectProperty))
            g.add((prop, RDFS.range, SKOS.Concept))
        else:
            g.add((prop, RDF.type, OWL.DatatypeProperty))
            g.add((prop, RDFS.range, XSD_FOR.get(body.get("type", "string"), XSD.string)))
        g.add((prop, RDFS.label, Literal(name, lang="en")))
        g.add((prop, RDFS.domain, HW.GovernedDocument))

        if body.get("role"):
            # The facet role is what the immutable core is stated in terms of.
            # OWL has no vocabulary for the role a property plays in a schema.
            g.add((prop, HW.facetRole, Literal(body["role"])))
            loss.add(
                "facet",
                "facet role",
                "the core requires facet roles (state, freshness, scent); OWL has "
                "no construct for the role a property plays, so this is an annotation",
            )
        if body.get("stale_after_days"):
            g.add((prop, HW.staleAfterDays, Literal(body["stale_after_days"])))
            loss.add(
                "facet",
                "stale_after_days",
                "freshness is a computation over the current date, not a statement "
                "about the graph; no OWL construct evaluates it",
            )
        if body.get("volatility"):
            loss.add("facet", "volatility", "authoring guidance, not a graph statement")
        if body.get("guidance"):
            # Per-value guidance prose survives as a scope note only if the value
            # space is a vocabulary. doc_type declares values inline, so mint them.
            for value, text in body["guidance"].items():
                concept = HW[f"{name}/{value}"]
                g.add((concept, SKOS.scopeNote, Literal(text, lang="en")))
        if values:
            scheme = HW[f"scheme/{name}"]
            g.add((scheme, RDF.type, SKOS.ConceptScheme))
            for entry in values:
                value = entry["value"] if isinstance(entry, dict) else entry
                concept = HW[f"{name}/{value}"]
                g.add((concept, RDF.type, SKOS.Concept))
                g.add((concept, SKOS.inScheme, scheme))
                g.add((concept, SKOS.prefLabel, Literal(value, lang="en")))


def _emit_kinds(g: Graph, resolved: dict, loss: Loss) -> None:
    for name, body in (resolved.get("kinds") or {}).items():
        cls = HW[_camel(name)]
        g.add((cls, RDF.type, OWL.Class))
        g.add((cls, RDFS.label, Literal(name, lang="en")))
        if body.get("is_a"):
            g.add((cls, RDFS.subClassOf, HW[_camel(body["is_a"])]))

        if body.get("abstract"):
            # OWL cannot forbid direct instantiation of a class. The nearest
            # honest move is an annotation; a reasoner draws nothing from it.
            g.add((cls, HW.abstract, Literal(True)))
            loss.add(
                "kind",
                "abstract kind",
                "OWL has no construct that forbids direct instantiation, so an "
                "instance typed as the abstract kind stays consistent",
            )
        if body.get("purpose"):
            # A purpose classifies the kind itself. In OWL 2 DL a class may only
            # be the subject of an annotation, so nothing is entailed from this.
            g.add((cls, HW.purpose, HW[f"purpose/{body['purpose']}"]))
            loss.add(
                "kind",
                "purpose",
                "purpose classifies a class rather than an instance; OWL 2 DL "
                "reaches it only by punning, and no inference flows through it",
            )
        if body.get("voice"):
            g.add((cls, HW.voice, Literal(body["voice"])))
            loss.add(
                "kind",
                "voice regime",
                "the voice regime forbids categories of prose in the document "
                "body, which is not in the graph at all",
            )
        if body.get("lifecycle"):
            g.add((cls, HW.lifecycle, Literal(body["lifecycle"])))
        if body.get("sections"):
            loss.add(
                "kind",
                "required sections",
                "section structure is a property of the Markdown body; the graph "
                "holds no headings",
            )
        if body.get("identifier"):
            loss.add(
                "kind",
                "identifier scheme",
                "an identifier scheme is a pattern plus an allocation policy; "
                "OWL states neither",
            )

        facets = body.get("facets") or {}
        for facet in facets.get("require", []):
            # An OWL cardinality restriction is an entailment, not a constraint.
            # Under the open-world assumption a missing value is inferred to
            # exist rather than reported. Emitted anyway, because it is what a
            # consumer expects to see, and recorded as the loss that it is.
            restriction = BNode()
            g.add((restriction, RDF.type, OWL.Restriction))
            g.add((restriction, OWL.onProperty, HW[facet]))
            g.add((restriction, OWL.minCardinality, Literal(1)))
            g.add((cls, RDFS.subClassOf, restriction))
            loss.add(
                "kind",
                "required facet",
                "owl:minCardinality entails existence under the open-world "
                "assumption; it never reports a missing value",
            )
        for facet in facets.get("forbid", []):
            restriction = BNode()
            g.add((restriction, RDF.type, OWL.Restriction))
            g.add((restriction, OWL.onProperty, HW[facet]))
            g.add((restriction, OWL.maxCardinality, Literal(0)))
            g.add((cls, RDFS.subClassOf, restriction))

        for _ in (body.get("relations") or {}).get("expect", []):
            loss.add(
                "kind",
                "participation expectation",
                "a participation expectation is a deadline measured from a state "
                "transition; OWL has no time and no severity",
            )


def _emit_relations(g: Graph, resolved: dict, loss: Loss) -> None:
    prov_alignment = {
        "supersedes": PROV.wasRevisionOf,
        "derives_from": PROV.wasDerivedFrom,
    }
    for name, body in (resolved.get("relations") or {}).items():
        prop = HW[name]
        g.add((prop, RDF.type, OWL.ObjectProperty))
        g.add((prop, RDFS.label, Literal(name, lang="en")))
        _domain_or_range(g, prop, RDFS.domain, body.get("from", []))
        _domain_or_range(g, prop, RDFS.range, body.get("to", []))

        if name in prov_alignment:
            g.add((prop, RDFS.subPropertyOf, prov_alignment[name]))
        if body.get("inverse"):
            g.add((prop, OWL.inverseOf, HW[body["inverse"]]))
            g.add((HW[body["inverse"]], RDF.type, OWL.ObjectProperty))
        if body.get("reciprocal") == "symmetric":
            g.add((prop, RDF.type, OWL.SymmetricProperty))
        if body.get("reciprocal") == "required":
            # owl:inverseOf makes the other half *inferred*, which is the
            # opposite of what a reciprocity check does: report the missing half.
            loss.add(
                "relation",
                "required reciprocity",
                "owl:inverseOf infers the missing half rather than reporting it, "
                "so a one-sided edge becomes consistent instead of a finding",
            )
        if body.get("family"):
            g.add((prop, HW.family, Literal(body["family"])))
            loss.add(
                "relation",
                "family defaults",
                "a family supplies nuclearity and acyclicity defaults that OWL "
                "property characteristics do not cover",
            )
        if body.get("nuclearity"):
            g.add((prop, HW.nuclearity, Literal(body["nuclearity"])))
            loss.add(
                "relation",
                "nuclearity",
                "nuclearity drives satellite inheritance, context pruning and "
                "orphan severity; no OWL construct expresses which end stands alone",
            )
        if body.get("created_by"):
            g.add((prop, HW.createdBy, Literal(body["created_by"])))
            loss.add(
                "relation",
                "created_by",
                "created_by is an intent about maintenance that taxonomy audit "
                "measures; it is not a statement about any instance",
            )
        if body.get("cardinality"):
            loss.add(
                "relation",
                "cardinality",
                "declared per relation type rather than per class, which is not "
                "where an OWL cardinality restriction attaches",
            )
        if body.get("on_target"):
            loss.add(
                "relation",
                "on_target state effect",
                "an edge that sets the target's state is an action taken when the "
                "edge is written; OWL states what holds, never what to do",
            )
        if body.get("invalid_when"):
            loss.add(
                "relation",
                "invalid_when",
                "a conditional invalidity over both endpoints' current state; "
                "expressible in SHACL-SPARQL, not in OWL",
            )
        if body.get("attributes"):
            loss.add(
                "relation",
                "instance attributes",
                "an edge with attributes is not a triple; it reifies, and every "
                "consumer that reads plain triples loses the attribute",
            )


def _domain_or_range(g: Graph, prop: URIRef, predicate: URIRef, kinds: list) -> None:
    if not kinds:
        return
    if len(kinds) == 1:
        g.add((prop, predicate, HW[_camel(kinds[0])]))
        return
    union = BNode()
    members = BNode()
    g.add((union, RDF.type, OWL.Class))
    g.add((union, OWL.unionOf, members))
    node = members
    for i, kind in enumerate(kinds):
        g.add((node, RDF.first, HW[_camel(kind)]))
        if i == len(kinds) - 1:
            g.add((node, RDF.rest, RDF.nil))
        else:
            nxt = BNode()
            g.add((node, RDF.rest, nxt))
            node = nxt
    g.add((prop, predicate, union))


def _emit_anchors(g: Graph, resolved: dict, loss: Loss) -> None:
    for name, body in (resolved.get("anchors") or {}).items():
        cls = HW[_camel(name)]
        g.add((cls, RDF.type, OWL.Class))
        g.add((cls, RDFS.label, Literal(name, lang="en")))
        g.add((cls, HW.resolver, Literal(body["resolver"])))
        loss.add(
            "anchor",
            "resolver identity",
            "an anchor's identity comes from a resolver that normalizes strings "
            "against repository content; a URI in an export cannot re-run it",
        )


def _note_unrepresentable(resolved: dict, loss: Loss) -> None:
    """Whole declaration classes with no OWL or SKOS counterpart at all."""
    lifecycle = (resolved.get("regimes") or {}).get("lifecycle") or {}
    if lifecycle:
        loss.add(
            "regime",
            "lifecycle transitions",
            "a legal-transition table is a state machine; OWL cannot say that "
            "draft may become current but superseded may not",
            len(lifecycle),
        )
    voice = (resolved.get("regimes") or {}).get("voice") or {}
    if voice:
        loss.add("regime", "voice regime", "forbids prose categories in the body", len(voice))
    language = (resolved.get("regimes") or {}).get("language") or {}
    if language:
        loss.add(
            "regime",
            "language regime",
            "a controlled-language profile over the body text",
            len(language),
        )
    shelves = resolved.get("shelves") or {}
    if shelves:
        loss.add(
            "shelf",
            "shelf placement",
            "placement is primary for kind resolution, and a path glob with a "
            "layout pattern is not an ontological statement",
            len(shelves),
        )
    schemes = resolved.get("identifier_schemes") or {}
    if schemes:
        loss.add(
            "identifier",
            "allocation policy",
            "reconcile-first allocation is a procedure the engine runs",
            len(schemes),
        )
    projections = resolved.get("projections") or []
    if projections:
        loss.add(
            "projection",
            "projection declaration",
            "a projection generates a document and is held to regeneration; "
            "SHACL-AF infers triples, and OWL infers nothing that is a file",
            len(projections),
        )
    core = (resolved.get("core") or {}).get("requires") or []
    if core:
        loss.add(
            "core",
            "immutable core",
            "the core is a statement about what a taxonomy must declare, which "
            "is an operation on the schema rather than a statement in it",
            len(core),
        )


def _camel(name: str) -> str:
    return "".join(part.capitalize() for part in name.split("_"))


# --------------------------------------------------------------------- ABox


def emit_abox(repo: Path, resolved: dict, loss: Loss) -> tuple[Graph, dict]:
    """Type the corpus from placement, per the spec's placement-is-primary rule."""
    g = Graph()
    for prefix, ns in [("hw", HW), ("hwc", HWC), ("rdfs", RDFS), ("skos", SKOS), ("xsd", XSD)]:
        g.bind(prefix, ns)

    census = {"documents": 0, "typed": 0, "untyped": [], "edges": 0, "facets_present": 0,
              "facets_required": 0}
    docs: dict[Path, str | None] = {}

    for shelf_name, shelf in (resolved.get("shelves") or {}).items():
        base_dir = repo / shelf["path"].replace("/**", "")
        if not base_dir.is_dir():
            continue
        for path in sorted(base_dir.glob("*.md")):
            rel = path.relative_to(repo)
            census["documents"] += 1
            if shelf.get("homogeneous"):
                kind = shelf["kind"]
            else:
                # A heterogeneous shelf resolves its kind through the
                # discriminator facet, which lives in front matter this corpus
                # does not have. The kind is therefore unresolved.
                kind = None
                census["untyped"].append(str(rel))
            docs[path] = kind
            subject = HWC[str(rel)]
            if kind:
                census["typed"] += 1
                g.add((subject, RDF.type, HW[_camel(kind)]))
            g.add((subject, HW.shelf, Literal(shelf_name)))
            g.add((subject, RDFS.label, Literal(_title(path), lang="en")))
            seq = re.match(r"^(\d+)-", path.name)
            if seq and "sequence" in str(shelf.get("layout", "")):
                g.add((subject, HW.sequence, Literal(int(seq.group(1)))))

    required = _required_facets(resolved)
    for path, kind in docs.items():
        if not kind:
            continue
        census["facets_required"] += len(required.get(kind, set()))

    census["edges"] = _emit_links(g, repo, docs)
    loss.add(
        "instance",
        "finding anchor",
        "a finding anchors to a line in a file; RDF has no byte offset, so no "
        "check result can be carried back through this export",
    )
    loss.add(
        "instance",
        "warrant",
        "every document carries exactly one warrant, and this corpus records "
        "none in machine-readable form, so the export cannot state it",
    )
    return g, census


def _required_facets(resolved: dict) -> dict[str, set]:
    out: dict[str, set] = {}
    kinds = resolved.get("kinds") or {}
    for name, body in kinds.items():
        req: set = set()
        cursor = body
        seen = set()
        while cursor is not None:
            req |= set(((cursor.get("facets") or {}).get("require")) or [])
            parent = cursor.get("is_a")
            if not parent or parent in seen:
                break
            seen.add(parent)
            cursor = kinds.get(parent)
        out[name] = req
    return out


def _emit_links(g: Graph, repo: Path, docs: dict) -> int:
    """Every Markdown link between two corpus documents becomes an edge."""
    count = 0
    seen: set[tuple[str, str]] = set()
    for path in docs:
        text = path.read_text(encoding="utf-8")
        for target in re.findall(r"\]\(([^)#]+\.md)(?:#[^)]*)?\)", text):
            resolved_target = (path.parent / target).resolve()
            if resolved_target not in docs:
                continue
            src = str(path.relative_to(repo))
            dst = str(resolved_target.relative_to(repo))
            if (src, dst) in seen:
                # Duplicate edges collapse to one, which is what the spec says
                # and also all a triple store can do. The finding is lost.
                continue
            seen.add((src, dst))
            g.add((HWC[src], HW["cites_evidence"], HWC[dst]))
            count += 1
    return count


def _title(path: Path) -> str:
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith("# "):
            return line[2:].strip()
    return path.stem


# ------------------------------------------------------------------- SHACL


def emit_shapes(resolved: dict) -> Graph:
    """The same required-facet constraints, in the language that reports them."""
    g = Graph()
    g.bind("sh", SH)
    g.bind("hw", HW)
    required = _required_facets(resolved)
    for kind, facets in required.items():
        body = (resolved.get("kinds") or {}).get(kind) or {}
        if body.get("abstract") or not facets:
            continue
        shape = HW[f"shape/{_camel(kind)}"]
        g.add((shape, RDF.type, SH.NodeShape))
        g.add((shape, SH.targetClass, HW[_camel(kind)]))
        for facet in sorted(facets):
            prop = BNode()
            g.add((shape, SH.property, prop))
            g.add((prop, SH.path, HW[facet]))
            g.add((prop, SH.minCount, Literal(1)))
            g.add((prop, SH.severity, SH.Violation))
            g.add((prop, SH.message, Literal(f"{kind} requires {facet}")))
    return g


# ------------------------------------------------------------------- checks


def run_checks(tbox: Graph, abox: Graph, shapes: Graph) -> dict:
    import owlrl
    from pyshacl import validate

    report: dict = {}

    combined = Graph()
    for triple in tbox:
        combined.add(triple)
    for triple in abox:
        combined.add(triple)
    report["tbox_triples"] = len(tbox)
    report["abox_triples"] = len(abox)

    before = len(combined)
    owlrl.DeductiveClosure(owlrl.OWLRL_Semantics).expand(combined)
    report["owl_entailed_triples"] = len(combined) - before
    report["owl_inconsistent"] = bool(
        list(combined.triples((None, RDF.type, OWL.Nothing)))
    )

    conforms, _, text = validate(
        abox, shacl_graph=shapes, ont_graph=tbox, inference="rdfs", advanced=True
    )
    report["shacl_conforms"] = conforms
    report["shacl_violations"] = text.count("Constraint Violation")

    turtle = tbox.serialize(format="turtle")
    reparsed = Graph().parse(data=turtle, format="turtle")
    report["roundtrip_isomorphic"] = isomorphic(tbox, reparsed)
    report["roundtrip_triples"] = len(reparsed)
    return report


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", default=".", type=Path)
    parser.add_argument("--out", default="out", type=Path)
    args = parser.parse_args()

    repo = args.repo.resolve()
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)

    loss = Loss()
    base = read_base(repo)
    bundle = yaml.safe_load((repo / BUNDLE).read_text(encoding="utf-8"))
    resolved = apply_overlay(base, bundle)

    tbox = emit_tbox(resolved, loss)
    abox, census = emit_abox(repo, resolved, loss)
    shapes = emit_shapes(resolved)

    (out / "taxonomy.ttl").write_text(tbox.serialize(format="turtle"), encoding="utf-8")
    (out / "corpus.ttl").write_text(abox.serialize(format="turtle"), encoding="utf-8")
    (out / "shapes.ttl").write_text(shapes.serialize(format="turtle"), encoding="utf-8")

    report = run_checks(tbox, abox, shapes)
    payload = {"census": census, "checks": report,
               "loss_set": sorted(loss.entries, key=lambda e: (e["layer"], e["construct"])),
               "loss_total": loss.total()}
    (out / "loss.json").write_text(json.dumps(payload, indent=2), encoding="utf-8")

    print(f"TBox      {report['tbox_triples']} triples -> {out / 'taxonomy.ttl'}")
    print(f"ABox      {report['abox_triples']} triples -> {out / 'corpus.ttl'}")
    print(f"census    {census['documents']} documents, {census['typed']} typed, "
          f"{len(census['untyped'])} unresolved, {census['edges']} edges")
    print(f"OWL       {report['owl_entailed_triples']} entailed, "
          f"inconsistent={report['owl_inconsistent']}")
    print(f"SHACL     conforms={report['shacl_conforms']}, "
          f"violations={report['shacl_violations']}")
    print(f"roundtrip isomorphic={report['roundtrip_isomorphic']}")
    print(f"loss set  {len(loss.entries)} declared constructs, "
          f"{loss.total()} declarations dropped")
    return 0


if __name__ == "__main__":
    sys.exit(main())
