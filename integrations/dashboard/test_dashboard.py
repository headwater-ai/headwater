"""What holds integrations/dashboard/dashboard.py.

Run it from anywhere, with python3 and nothing else:

    python3 -m unittest discover -s integrations/dashboard -p 'test_*.py'

The first case is the contract HW-DR-0080 states: every row the dashboard
stores or renders is keyed by (corpus_identity, id). The second is the
decisive fixture of #505: an export whose documents carry last_verified only
under `facets`, one of which has no `warrant` key. A view that reads the
wrong path publishes an empty page, and a view that fills in a default labels
an unstated warrant as `asserted`. Both defects fail here.
"""

import json
import os
import subprocess
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import dashboard  # noqa: E402

FIXTURE = os.path.join(HERE, "fixtures", "two-documents.json")
TREE = os.path.join(HERE, "fixtures", "tree")
REPO_EXPORT = os.path.join(HERE, "..", "..", ".headwater", "export.json")


def load_fixture():
    with open(FIXTURE, encoding="utf-8") as handle:
        return json.load(handle)


class EveryRowIsKeyedByCorpusIdentityAndId(unittest.TestCase):
    """HW-DR-0080: the key is the pair, from the first commit."""

    def test_document_and_anchor_rows_carry_the_pair(self):
        model = dashboard.load(load_fixture(), corpus_identity="fixture-corpus")
        rows = list(model.documents) + list(model.anchors)
        self.assertEqual(len(model.documents), 2)
        self.assertEqual(len(model.anchors), 3)
        for row in rows:
            self.assertEqual(row["key"], ("fixture-corpus", row["id"]))
            self.assertEqual(row["corpus_identity"], "fixture-corpus")

    def test_a_single_corpus_page_never_displays_the_identity(self):
        model = dashboard.load(load_fixture(), corpus_identity="fixture-corpus-7f3a")
        page = dashboard.render(model)
        self.assertNotIn("fixture-corpus-7f3a", page)

    def test_two_corpora_never_fold_under_one_key(self):
        one = dashboard.load(load_fixture(), corpus_identity="a")
        two = dashboard.load(load_fixture(), corpus_identity="b")
        keys = [row["key"] for row in one.documents + two.documents]
        self.assertEqual(len(keys), len(set(keys)))


class DecisiveFixture(unittest.TestCase):
    """#505: last_verified lives under facets, and an absent warrant is not asserted."""

    def test_staleness_lists_both_documents_oldest_first(self):
        model = dashboard.load(load_fixture(), corpus_identity="fixture")
        order = [(row["id"], row["last_verified"]) for row in dashboard.staleness_view(model)]
        self.assertEqual(order, [("FX-OBL-0001", "2026-01-15"), ("FX-DR-0002", "2026-09-01")])

    def test_warrant_view_shows_none_stated_in_its_own_group(self):
        model = dashboard.load(load_fixture(), corpus_identity="fixture")
        groups = dict((label, [row["id"] for row in rows]) for label, rows in dashboard.warrant_view(model))
        self.assertEqual(groups.get(dashboard.NONE_STATED), ["FX-OBL-0001"])
        self.assertEqual(groups.get("accepted"), ["FX-DR-0002"])
        self.assertNotIn("asserted", groups)

    def test_the_rendered_page_says_none_stated_and_never_asserted(self):
        model = dashboard.load(load_fixture(), corpus_identity="fixture")
        page = dashboard.render(model)
        self.assertIn("none stated", page)
        self.assertNotIn("asserted", page)
        self.assertLess(page.index("FX-OBL-0001"), page.index("FX-DR-0002"))


class CoverageView(unittest.TestCase):
    def test_only_code_path_anchors_a_governs_edge_reaches(self):
        model = dashboard.load(load_fixture(), corpus_identity="fixture")
        ids = [row["id"] for row in model.anchors]
        self.assertEqual(ids, ["missing/path.rs", "src/**", "src/main.rs"])
        governing = dict((row["id"], row["governed_by"]) for row in model.anchors)
        self.assertEqual(governing["src/**"], ["FX-DR-0002"])
        self.assertEqual(governing["src/main.rs"], ["FX-OBL-0001"])

    def test_share_of_a_tree_counts_each_file_once(self):
        model = dashboard.load(load_fixture(), corpus_identity="fixture")
        coverage = dashboard.coverage_view(model, tree=TREE)
        self.assertEqual((coverage.covered, coverage.total), (2, 4))
        matched = dict((row["id"], count) for row, count in coverage.rows)
        self.assertEqual(matched, {"missing/path.rs": 0, "src/**": 2, "src/main.rs": 1})

    def test_without_a_tree_there_is_no_share(self):
        model = dashboard.load(load_fixture(), corpus_identity="fixture")
        coverage = dashboard.coverage_view(model, tree=None)
        self.assertIsNone(coverage.total)
        page = dashboard.render(model)
        self.assertIn("src/**", page)


class TheCommandLine(unittest.TestCase):
    def test_writes_one_page_and_nothing_else(self):
        with tempfile.TemporaryDirectory() as scratch:
            out = os.path.join(scratch, "page.html")
            result = subprocess.run(
                [sys.executable, os.path.join(HERE, "dashboard.py"), "--export", FIXTURE, "--tree", TREE, "--out", out],
                capture_output=True,
                text=True,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(os.listdir(scratch), ["page.html"])
            with open(out, encoding="utf-8") as handle:
                page = handle.read()
            self.assertIn("2 of 4", page)

    def test_refuses_an_export_below_version_1_1(self):
        export = load_fixture()
        export["export_version"] = "1.0"
        with self.assertRaises(dashboard.ExportRefused):
            dashboard.load(export, corpus_identity="fixture")


class TheWorkedExample(unittest.TestCase):
    """.headwater/export.json in this repository is the input the issue names."""

    def test_every_document_of_the_repository_export_is_on_the_page(self):
        with open(REPO_EXPORT, encoding="utf-8") as handle:
            export = json.load(handle)
        model = dashboard.load(export, corpus_identity="headwater")
        documents = export["graph"]["documents"]
        self.assertEqual(len(dashboard.staleness_view(model)), len(documents))
        self.assertEqual(sum(len(rows) for _, rows in dashboard.warrant_view(model)), len(documents))
        unstated = sum(1 for document in documents if "warrant" not in document)
        groups = dict(dashboard.warrant_view(model))
        self.assertEqual(len(groups.get(dashboard.NONE_STATED, [])), unstated)
        self.assertTrue(all(row["last_verified"] for row in model.documents))


if __name__ == "__main__":
    unittest.main()
