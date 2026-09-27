"""Stub: reads the fields where the issue text said they were. Replaced by the build."""

NONE_STATED = "none stated"


class ExportRefused(Exception):
    pass


class Model:
    def __init__(self, documents, anchors):
        self.documents = documents
        self.anchors = anchors


def load(export, corpus_identity):
    documents = [
        {"id": d["id"], "last_verified": d.get("last_verified"), "warrant": d.get("warrant", "asserted")}
        for d in export["graph"]["documents"]
        if d.get("last_verified")
    ]
    return Model(documents, [])


def staleness_view(model):
    return sorted(model.documents, key=lambda row: row["last_verified"])


def warrant_view(model):
    groups = {}
    for row in model.documents:
        groups.setdefault(row["warrant"], []).append(row)
    return list(groups.items())


def coverage_view(model, tree=None):
    raise NotImplementedError


def render(model):
    return ""
