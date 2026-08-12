#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Check this repository's own docs/ tree against the taxonomy that types it.

The throwaway validator that issue #4 asks for. There is no engine, so this
script stands in for one over exactly the rules that the typed corpus can be
wrong against today. It is not a design and nothing should grow it. When
`headwater check` exists, every rule below belongs to the check layer and this
file goes away.

It resolves the base package (from the fenced YAML in the first-run walkthrough,
which is still its only committed copy), applies the design-spec bundle, applies
this repository's overlay, and then reads every document on a declared shelf.

    usage:  tools/abox-check.py [--repo PATH] [-v]

Exit status is 1 when an error-severity rule fails. Warnings and advisories are
reported and never change the status, which is the posture that spec 4 requires
of a detective check.
"""

from __future__ import annotations

import argparse
import posixpath
import re
import sys
from collections import Counter, defaultdict
from datetime import date, timedelta
from pathlib import Path

import yaml

# --- resolution --------------------------------------------------------------

BASE_SOURCE = "docs/evaluations/default-taxonomy-first-run.md"
BUNDLE = "docs/taxonomies/design-spec/bundle.yml"
OVERLAY = ".headwater/overlay.yml"

# The four provenance rules of spec 3, as (requires, forbids).
WARRANT_RULES = {
    "accepted": (["accepted_by"], []),
    "regenerated": ([], ["accepted_by"]),
    "transcribed": ([], ["accepted_by"]),
    "asserted": (["drafted_by", "activity"], ["accepted_by"]),
}


def load_base(repo: Path) -> dict:
    text = (repo / BASE_SOURCE).read_text()
    blocks = re.findall(r"```yaml\n(.*?)```", text, re.S)
    for b in blocks:
        doc = yaml.safe_load(b)
        if isinstance(doc, dict) and doc.get("package") == "headwater/standard":
            return doc
    raise SystemExit(f"no base package found in {BASE_SOURCE}")


def apply_adds(tax: dict, adds: dict) -> None:
    """Apply `add` operations at dotted addresses, the way the resolver will."""
    for address, value in adds.items():
        parts = address.split(".")
        node = tax
        for p in parts[:-1]:
            node = node.setdefault(p, {})
        leaf = parts[-1]
        if leaf in node and isinstance(node[leaf], dict) and isinstance(value, dict):
            node[leaf].update(value)
        else:
            node[leaf] = value


def resolve(repo: Path) -> dict:
    tax = load_base(repo)
    for path in (BUNDLE, OVERLAY):
        overlay = yaml.safe_load((repo / path).read_text())
        apply_adds(tax, overlay.get("add", {}))
    return tax


# --- the corpus --------------------------------------------------------------

FM = re.compile(r"\A---\n(.*?)\n---\n", re.S)
LINK = re.compile(r"\[([^\]]*)\]\(([^)]+)\)")


class Doc:
    def __init__(self, path: str, fm: dict, body: str):
        self.path = path
        self.fm = fm
        self.body = body
        self.kind = None
        self.id = fm.get("id")


def shelf_of(tax: dict, path: str):
    for name, shelf in tax["shelves"].items():
        pattern = shelf["path"].replace("**", "")
        if path.startswith(pattern):
            return name, shelf
    return None, None


def read_corpus(repo: Path, tax: dict):
    docs, unclassified = {}, []
    for path in sorted(p.relative_to(repo).as_posix() for p in (repo / "docs").rglob("*")):
        if not path.endswith(".md") or (repo / path).is_dir():
            continue
        if "/taxonomies/" in path or "/fixtures/" in path:
            continue  # package content, not corpus content
        name, shelf = shelf_of(tax, path)
        if shelf is None:
            unclassified.append(path)
            continue
        text = (repo / path).read_text()
        m = FM.match(text)
        if not m:
            docs[path] = Doc(path, {}, text)
            continue
        docs[path] = Doc(path, yaml.safe_load(m.group(1)) or {}, text[m.end():])
    return docs, unclassified


# --- checks ------------------------------------------------------------------

class Report:
    def __init__(self):
        self.rows = []

    def add(self, severity, rule, where, message):
        self.rows.append((severity, rule, where, message))

    def count(self, severity):
        return sum(1 for r in self.rows if r[0] == severity)


def check(repo: Path, verbose: bool) -> Report:
    tax = resolve(repo)
    docs, unclassified = read_corpus(repo, tax)
    rep = Report()

    kinds = tax["kinds"]
    relations = tax["relations"]
    facets = tax["facets"]

    # inverse name -> (relation name, declared inverse)
    inverse_of = {}
    for rname, rel in relations.items():
        if "inverse" in rel:
            inverse_of[rel["inverse"]] = rname

    # 1. kind resolution, from placement and the discriminator
    for path, doc in docs.items():
        _, shelf = shelf_of(tax, path)
        if not doc.fm:
            rep.add("error", "front-matter", path, "no front matter")
            continue
        dt = doc.fm.get("doc_type")
        if shelf.get("homogeneous"):
            doc.kind = shelf["kind"]
            if dt is not None and "doc_type" in kinds[doc.kind].get("facets", {}).get("forbid", []):
                rep.add("error", "facet-forbidden", path,
                        f"doc_type on a homogeneous shelf, which {doc.kind} forbids")
        else:
            if dt is None:
                rep.add("error", "discriminator", path,
                        f"no {shelf['discriminator']} on a heterogeneous shelf")
                continue
            if dt not in shelf["kinds"]:
                rep.add("error", "discriminator", path, f"doc_type {dt} is not on this shelf")
                continue
            doc.kind = dt

    # 2. required facets, per kind, including the four the base requires
    for path, doc in docs.items():
        if not doc.kind:
            continue
        required = set(kinds["governed_document"]["facets"]["require"])
        required |= set(kinds[doc.kind].get("facets", {}).get("require", []))
        for f in sorted(required):
            if doc.fm.get(f) in (None, ""):
                rep.add("error", "required-facet", path, f"missing {f}")
        for f in doc.fm:
            if f in ("provenance", "relations", "id"):
                continue
            if f not in facets and f not in ("doc_type", "sequence"):
                rep.add("error", "unknown-facet", path, f"unknown facet {f}")

    # 3. dates: format, no future stamps, freshness after entry
    today = date.today()
    for path, doc in docs.items():
        for f in ("status_since", "last_verified"):
            v = doc.fm.get(f)
            if v is None:
                continue
            if not isinstance(v, date):
                rep.add("error", "date-format", path, f"{f} is not a date")
                continue
            if v > today:
                rep.add("error", "date-future", path, f"{f} is in the future")
        s, l = doc.fm.get("status_since"), doc.fm.get("last_verified")
        if isinstance(s, date) and isinstance(l, date) and l < s:
            rep.add("error", "date-order", path, "last_verified is before status_since")

    # 4. identifiers: present, unique, and shaped by the scheme of the kind
    seen = {}
    for path, doc in docs.items():
        if not doc.kind:
            continue
        scheme_name = kinds[doc.kind].get("identifier", {}).get("scheme")
        if not scheme_name:
            continue
        if not doc.id:
            rep.add("error", "identifier", path, f"{doc.kind} declares a scheme and the document has no id")
            continue
        scheme = tax["identifier_schemes"][scheme_name]
        prefix = scheme["pattern"].split("{")[0] + str(scheme["namespace"]) + "-"
        if not doc.id.startswith(prefix):
            rep.add("error", "identifier", path, f"{doc.id} does not match scheme {scheme_name} ({prefix}…)")
        if doc.id in seen:
            rep.add("error", "identifier-unique", path, f"{doc.id} is also on {seen[doc.id]}")
        seen[doc.id] = path
    by_id = {d.id: d for d in docs.values() if d.id}

    # 5. the warrant, and what each value requires and forbids
    for path, doc in docs.items():
        prov = doc.fm.get("provenance")
        if not isinstance(prov, dict):
            if doc.kind:
                rep.add("error", "provenance", path, "no provenance block")
            continue
        w = prov.get("warrant")
        if w not in WARRANT_RULES:
            rep.add("error", "warrant", path, f"warrant {w!r} is not in the closed set")
            continue
        req, forb = WARRANT_RULES[w]
        for f in req:
            if not prov.get(f):
                rep.add("error", "warrant", path, f"warrant {w} requires {f}")
        for f in forb:
            if prov.get(f):
                rep.add("error", "warrant", path, f"warrant {w} forbids {f}")
        if prov.get("evidence_basis") not in ("evidenced", "reconstructed", "unevidenced"):
            rep.add("error", "evidence-basis", path, "evidence_basis is missing or outside the set")
        if prov.get("evidence_basis") == "reconstructed" and not prov.get("reconstructed_from"):
            rep.add("error", "evidence-basis", path, "reconstructed needs reconstructed_from")

    # 6. relations: names, endpoints, targets, duplicates
    triples = Counter()
    declared = defaultdict(set)          # (doc, relation) -> targets
    for path, doc in docs.items():
        rels = doc.fm.get("relations") or {}
        if not doc.kind:
            continue
        for rname, entries in rels.items():
            forward = rname if rname in relations else inverse_of.get(rname)
            if forward is None:
                rep.add("error", "unknown-relation", path, f"{rname} is not a declared relation")
                continue
            targets = entries if isinstance(entries, list) else [entries]
            for t in targets:
                target = t["to"] if isinstance(t, dict) else t
                declared[(doc.id, rname)].add(target)
                key = (doc.id, rname, target)
                triples[key] += 1
                if triples[key] > 1:
                    rep.add("error", "repeated-triple", path, f"{rname} -> {target} declared twice")
                if target.endswith("/") or "/" in target:
                    continue      # an anchor target, resolved against the source tree
                if target not in by_id:
                    rep.add("error", "dangling-target", path, f"{rname} -> {target} resolves to nothing")
                    continue
                rel = relations[forward]
                src_kind, dst_kind = doc.kind, by_id[target].kind
                if rname == forward:
                    frm, to = src_kind, dst_kind
                else:
                    frm, to = dst_kind, src_kind
                def permitted(kind, ends):
                    if kind in ends:
                        return True
                    parent = kinds.get(kind, {}).get("is_a")
                    return parent in ends
                if not permitted(frm, rel["from"]) or not permitted(to, rel["to"]):
                    rep.add("error", "endpoint", path,
                            f"{forward} does not permit {frm} -> {to}")

    # 7. reciprocity, both halves present
    for (src_id, rname), targets in list(declared.items()):
        forward = rname if rname in relations else inverse_of.get(rname)
        if forward is None:
            continue
        rel = relations[forward]
        if rel.get("reciprocal") != "required":
            continue
        other = rel["inverse"] if rname == forward else forward
        for t in targets:
            if "/" in t:
                continue
            if src_id not in declared.get((t, other), set()):
                rep.add("error", "reciprocity", by_id[src_id].path,
                        f"{rname} -> {t} has no {other} on the other end")

    # 8. lifecycle: a live document may not depend on a terminal one
    terminal = {v["value"] for v in tax["vocabularies"]["lifecycle_state"]
                if str(v["role"]).startswith("terminal")}
    for (src_id, rname), targets in declared.items():
        forward = rname if rname in relations else inverse_of.get(rname)
        if forward is None or relations[forward].get("family") != "succession":
            continue
        src = by_id[src_id]
        if src.fm.get("status") in terminal:
            continue
        for t in targets:
            if t in by_id and by_id[t].fm.get("status") in terminal and rname == forward:
                continue  # a successor names its predecessor, which is the point
            if t in by_id and by_id[t].fm.get("status") in terminal:
                rep.add("error", "lifecycle-sensitive", src.path,
                        f"live document depends on terminal {t} through {rname}")

    # 9. participation expectations, windowed from status_entered
    for kind_name, kind in kinds.items():
        for exp in kind.get("relations", {}).get("expect", []):
            window = timedelta(days=int(str(exp["within"]).rstrip("d")))
            for path, doc in docs.items():
                if doc.kind != kind_name:
                    continue
                if doc.fm.get("status") != exp.get("when", {}).get("status", "current"):
                    continue
                held = declared.get((doc.id, exp["relation"]), set())
                if held:
                    continue
                since = doc.fm.get("status_since")
                if isinstance(since, date) and today - since > window:
                    rep.add(exp.get("severity", "warn"), exp["id"], path, exp["rationale"])

    # 10. a prose link with no declared relation is an advisory finding
    path_by_id = {d.id: d.path for d in docs.values() if d.id}
    id_by_path = {d.path: d.id for d in docs.values() if d.id}
    advisory = 0
    for path, doc in docs.items():
        body = re.sub(r"```.*?```", "", doc.body, flags=re.S)
        related = set()
        for (sid, _), targets in declared.items():
            if sid == doc.id:
                related |= {path_by_id.get(t, t) for t in targets}
        for _, target in LINK.findall(body):
            if target.startswith(("http", "#")):
                continue
            p = target.split("#")[0]
            if not p:
                continue
            dest = posixpath.normpath(posixpath.join(posixpath.dirname(path), p))
            if dest in docs and dest not in related:
                advisory += 1
                if verbose:
                    rep.add("advisory", "undeclared-link", path, f"prose link to {dest}")

    # --- output ---
    print(f"corpus: {len(docs)} documents on {len(tax['shelves'])} shelves, "
          f"{sum(triples.values())} declared edge halves")
    kind_counts = Counter(d.kind for d in docs.values())
    for k, n in sorted(kind_counts.items(), key=lambda x: (x[0] or "")):
        print(f"    {k or '(unresolved)':22s} {n}")

    if unclassified:
        print(f"\ncensus: {len(unclassified)} files under docs/ that no shelf claims")
        for p in unclassified:
            print(f"    {p}")

    print(f"\nadvisory: {advisory} prose links to a corpus document with no declared relation")

    for severity in ("error", "warn", "advisory"):
        rows = [r for r in rep.rows if r[0] == severity]
        if not rows:
            continue
        print(f"\n{severity} ({len(rows)}):")
        for _, rule, where, message in rows:
            print(f"    [{rule}] {where}: {message}")
    if rep.count("error") == 0:
        print("\nno errors")
    return rep


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", default=Path(__file__).resolve().parent.parent)
    ap.add_argument("-v", "--verbose", action="store_true",
                    help="list every advisory finding rather than the count")
    args = ap.parse_args()
    rep = check(Path(args.repo), args.verbose)
    return 1 if rep.count("error") else 0


if __name__ == "__main__":
    sys.exit(main())
