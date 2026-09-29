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
REPO = os.path.join(HERE, "..", "..")


def repo_export():
    """This repository's graph export, which is computed and never committed (#1251).

    `HEADWATER_EXPORT` names a file a caller already computed, which is what
    `tools/repo/integrations-fixtures.sh` passes. Without it the newer of the two
    built engines computes one. With neither there is nothing to read, and the
    case says so rather than reading a stale file.
    """
    named = os.environ.get("HEADWATER_EXPORT")
    if named:
        with open(named, encoding="utf-8") as handle:
            return json.load(handle)
    engines = [
        os.path.join(REPO, "engine", "target", profile, "headwater")
        for profile in ("release", "dev-release")
    ]
    engines = [path for path in engines if os.access(path, os.X_OK)]
    if not engines:
        raise unittest.SkipTest("no built engine to compute the export, and HEADWATER_EXPORT is unset")
    engine = max(engines, key=os.path.getmtime)
    written = subprocess.run(
        [engine, "export", "--format", "json", "--root", REPO],
        check=True,
        capture_output=True,
    )
    return json.loads(written.stdout)


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


def with_list_anchor(export):
    """The fixture export plus one governs edge onto a list anchor of two files.

    The `id` is the length-prefixed identity the engine writes for a list
    (HW-DR-0074), which matches no file of the tree as a path. The members
    are under `patterns`, as an export at 1.2 or later writes them (#1247).
    """
    target = {
        "bound": "anchor",
        "anchor_kind": "code_path",
        "id": "10:README.txt9:docs/a.md",
        "resolver": "source-tree",
        "patterns": ["README.txt", "docs/a.md"],
    }
    export["graph"]["anchors"].append(dict((key, target[key]) for key in ("anchor_kind", "id", "resolver", "patterns")))
    export["graph"]["edges"].append(
        {"source": "docs/decisions/0002-recent.md", "relation": "governs", "written_as": "governs", "target": target}
    )
    return export


def add_list_anchor(export, patterns):
    """One more governs edge onto a list anchor, with the identity the engine would write."""
    members = sorted(patterns)
    target = {
        "bound": "anchor",
        "anchor_kind": "code_path",
        "id": "".join("%d:%s" % (len(member.encode("utf-8")), member) for member in members),
        "resolver": "source-tree",
        "patterns": members,
    }
    export["graph"]["anchors"].append(dict((key, target[key]) for key in ("anchor_kind", "id", "resolver", "patterns")))
    export["graph"]["edges"].append(
        {"source": "docs/obligations/0001-old.md", "relation": "governs", "written_as": "governs", "target": target}
    )
    return export


class AListAnchorIsCountedByItsMembers(unittest.TestCase):
    """#1247: a list anchor covers the union of its members, and never its identity as a path."""

    def test_the_row_matches_every_file_its_members_match(self):
        model = dashboard.load(with_list_anchor(load_fixture()), corpus_identity="fixture")
        coverage = dashboard.coverage_view(model, tree=TREE)
        matched = dict((row["id"], count) for row, count in coverage.rows)
        self.assertEqual(matched["10:README.txt9:docs/a.md"], 2)
        self.assertEqual((coverage.covered, coverage.total), (4, 4))

    def test_the_page_names_the_members_and_counts_them_as_paths(self):
        model = dashboard.load(with_list_anchor(load_fixture()), corpus_identity="fixture")
        self.assertEqual(
            dashboard.governed_paths(model),
            ["README.txt", "docs/a.md", "missing/path.rs", "src/**", "src/main.rs"],
        )
        page = dashboard.render(model)
        self.assertIn("5 governed code paths", page)
        self.assertIn("<code>README.txt</code><br><code>docs/a.md</code>", page)
        self.assertNotIn("10:README.txt", page)

    def test_a_member_two_lists_share_is_one_governed_path(self):
        export = with_list_anchor(load_fixture())
        add_list_anchor(export, ["docs/a.md", "src/lib.rs"])
        model = dashboard.load(export, corpus_identity="fixture")
        # `docs/a.md` is a member of both lists. It is one path, so the header
        # counts six and not seven.
        self.assertEqual(
            dashboard.governed_paths(model),
            ["README.txt", "docs/a.md", "missing/path.rs", "src/**", "src/lib.rs", "src/main.rs"],
        )
        self.assertIn("6 governed code paths", dashboard.render(model))

    def test_a_file_two_members_of_one_list_match_is_one_file_of_the_row(self):
        export = load_fixture()
        add_list_anchor(export, ["src/**", "src/main.rs"])
        model = dashboard.load(export, corpus_identity="fixture")
        coverage = dashboard.coverage_view(model, tree=TREE)
        matched = dict((row["id"], count) for row, count in coverage.rows)
        # `src/main.rs` matches both members. The tree holds two files under
        # `src/`, so the row matches two, and never three.
        self.assertEqual(matched["6:src/**11:src/main.rs"], 2)

    def test_an_anchor_with_no_patterns_is_its_one_id(self):
        model = dashboard.load(load_fixture(), corpus_identity="fixture")
        self.assertEqual(dict((row["id"], row["patterns"]) for row in model.anchors)["src/**"], ["src/**"])

    def test_patterns_that_are_not_a_list_of_strings_are_refused(self):
        export = with_list_anchor(load_fixture())
        export["graph"]["edges"][-1]["target"]["patterns"] = "README.txt, docs/a.md"
        with self.assertRaises(dashboard.ExportRefused) as raised:
            dashboard.load(export, corpus_identity="fixture")
        self.assertIn("target.patterns", str(raised.exception))


DEAD_MEMBER_REASON = "`code_path`: `src/gone.rs` no `src/gone.rs` in the source tree"


def with_dead_member_list(export):
    """The fixture export plus `governs: [[README.txt, src/gone.rs]]` from FX-DR-0002.

    `README.txt` is in the tree and `src/gone.rs` is not. The engine binds
    nothing for such a list (HW-DR-0074), and exports the edge with the shape
    `Target::Unbound` writes: `bound` and `reason`, and no members.
    """
    export["graph"]["edges"].append(
        {
            "source": "docs/decisions/0002-recent.md",
            "relation": "governs",
            "written_as": "governs",
            "target": {"bound": "nothing", "reason": DEAD_MEMBER_REASON},
        }
    )
    return export


class AListWithADeadMemberIsShownRefused(unittest.TestCase):
    """#1307: a governs edge that binds nothing is reported on the page, and covers no file."""

    def test_the_page_names_the_refused_edge_and_its_reason(self):
        model = dashboard.load(with_dead_member_list(load_fixture()), corpus_identity="fixture")
        page = dashboard.render(model, dashboard.coverage_view(model, tree=TREE))
        # Before #1307 the edge left no trace on the page, and this line failed.
        self.assertIn("<tr><td>FX-DR-0002</td><td>%s</td></tr>" % dashboard.html.escape(DEAD_MEMBER_REASON), page)
        self.assertIn("Governs edges that bind nothing", page)
        self.assertIn("including the paths in it that do match", page)
        self.assertEqual(
            [(row["key"], row["governed_by"], row["reason"]) for row in model.refused],
            [(("fixture", "FX-DR-0002"), "FX-DR-0002", DEAD_MEMBER_REASON)],
        )

    def test_the_matching_member_is_not_counted_as_covered(self):
        before = dashboard.coverage_view(dashboard.load(load_fixture(), corpus_identity="fixture"), tree=TREE)
        model = dashboard.load(with_dead_member_list(load_fixture()), corpus_identity="fixture")
        after = dashboard.coverage_view(model, tree=TREE)
        self.assertEqual((after.covered, after.total), (before.covered, before.total))
        self.assertEqual([row["id"] for row, _ in after.rows], [row["id"] for row, _ in before.rows])

    def test_governed_paths_is_unchanged(self):
        before = dashboard.governed_paths(dashboard.load(load_fixture(), corpus_identity="fixture"))
        model = dashboard.load(with_dead_member_list(load_fixture()), corpus_identity="fixture")
        self.assertEqual(dashboard.governed_paths(model), before)
        self.assertNotIn("README.txt", dashboard.governed_paths(model))

    def test_a_page_with_no_refused_edge_says_there_is_none(self):
        model = dashboard.load(load_fixture(), corpus_identity="fixture")
        self.assertEqual(model.refused, [])
        self.assertIn("No governs edge binds nothing", dashboard.render(model))

    def test_a_refused_edge_of_another_relation_is_not_listed(self):
        export = with_dead_member_list(load_fixture())
        export["graph"]["edges"][-1]["relation"] = "cites"
        model = dashboard.load(export, corpus_identity="fixture")
        self.assertEqual(model.refused, [])

    def test_a_refused_edge_with_no_reason_is_refused_as_input(self):
        export = with_dead_member_list(load_fixture())
        del export["graph"]["edges"][-1]["target"]["reason"]
        with self.assertRaises(dashboard.ExportRefused) as raised:
            dashboard.load(export, corpus_identity="fixture")
        self.assertIn("target.reason", str(raised.exception))


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


def run_cli(export, scratch):
    source = os.path.join(scratch, "export.json")
    with open(source, "w", encoding="utf-8") as handle:
        json.dump(export, handle)
    out = os.path.join(scratch, "page.html")
    result = subprocess.run(
        [sys.executable, os.path.join(HERE, "dashboard.py"), "--export", source, "--out", out],
        capture_output=True,
        text=True,
    )
    return result, os.path.exists(out)


class AMalformedExportIsRefused(unittest.TestCase):
    """A malformed input exits 2 with one line naming the document and field, never a traceback."""

    def refused(self, mutate, names):
        export = load_fixture()
        mutate(export)
        with tempfile.TemporaryDirectory() as scratch:
            result, wrote = run_cli(export, scratch)
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertNotIn("Traceback", result.stderr)
        self.assertFalse(wrote, "a refused export must not write a page")
        for name in names:
            self.assertIn(name, result.stderr)

    def test_a_document_with_neither_id_nor_path(self):
        def mutate(export):
            del export["graph"]["documents"][1]["id"]
            del export["graph"]["documents"][1]["path"]
        self.refused(mutate, ["graph.documents[1]", "`id`", "`path`"])

    def test_a_last_verified_that_is_not_a_string(self):
        def mutate(export):
            export["graph"]["documents"][1]["facets"]["last_verified"] = 20260115
        self.refused(mutate, ["FX-OBL-0001", "facets.last_verified"])

    def test_a_list_valued_warrant(self):
        def mutate(export):
            export["graph"]["documents"][0]["warrant"] = ["accepted"]
        self.refused(mutate, ["FX-DR-0002", "`warrant`"])

    def test_a_document_that_is_not_an_object(self):
        def mutate(export):
            export["graph"]["documents"].append("FX-DR-0003")
        self.refused(mutate, ["graph.documents[2]", "not an object"])

    def test_a_governs_edge_whose_target_has_no_id(self):
        def mutate(export):
            del export["graph"]["edges"][0]["target"]["id"]
        self.refused(mutate, ["graph.edges[0]", "target.id"])

    def test_a_graph_with_no_documents_key(self):
        def mutate(export):
            del export["graph"]["documents"]
        self.refused(mutate, ["graph", "`documents`"])

    def test_a_date_that_is_not_on_the_calendar(self):
        def mutate(export):
            export["graph"]["documents"][1]["facets"]["last_verified"] = "2026-02-30"
        self.refused(mutate, ["FX-OBL-0001", "facets.last_verified", "2026-02-30"])

    def test_a_last_verified_that_is_not_a_date_at_all(self):
        def mutate(export):
            export["graph"]["documents"][1]["facets"]["last_verified"] = "last tuesday"
        self.refused(mutate, ["FX-OBL-0001", "facets.last_verified", "last tuesday"])

    def test_the_basic_iso_form_is_refused_on_every_python(self):
        def mutate(export):
            export["graph"]["documents"][1]["facets"]["last_verified"] = "20260115"
        self.refused(mutate, ["FX-OBL-0001", "facets.last_verified", "20260115"])

    def test_an_iso_week_date_is_refused_on_every_python(self):
        def mutate(export):
            export["graph"]["documents"][1]["facets"]["last_verified"] = "2026-W03-4"
        self.refused(mutate, ["FX-OBL-0001", "facets.last_verified", "2026-W03-4"])

    def test_a_date_with_a_trailing_newline(self):
        def mutate(export):
            export["graph"]["documents"][1]["facets"]["last_verified"] = "2026-01-15\n"
        self.refused(mutate, ["FX-OBL-0001", "facets.last_verified"])

    def test_two_documents_with_one_id(self):
        def mutate(export):
            export["graph"]["documents"][1]["id"] = "FX-DR-0002"
        self.refused(mutate, ["FX-DR-0002", "more than one document"])


class ThePageHoldsWhatItPrints(unittest.TestCase):
    def test_a_title_is_escaped_and_never_markup(self):
        export = load_fixture()
        export["graph"]["documents"][0]["facets"]["title"] = "<script>alert(1)</script> & more"
        page = dashboard.render(dashboard.load(export, corpus_identity="fixture"))
        self.assertNotIn("<script>", page)
        self.assertIn("&lt;script&gt;alert(1)&lt;/script&gt; &amp; more", page)

    def test_a_code_path_is_escaped_and_never_markup(self):
        export = load_fixture()
        export["graph"]["edges"][0]["target"]["id"] = "src/<img src=x onerror=alert(1)>.rs"
        page = dashboard.render(dashboard.load(export, corpus_identity="fixture"))
        self.assertNotIn("<img", page)
        self.assertIn("<code>src/&lt;img src=x onerror=alert(1)&gt;.rs</code>", page)

    def test_an_empty_corpus_renders_and_says_it_holds_no_documents(self):
        export = load_fixture()
        export["graph"]["documents"] = []
        export["graph"]["edges"] = []
        with tempfile.TemporaryDirectory() as scratch:
            result, wrote = run_cli(export, scratch)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue(wrote)
            with open(os.path.join(scratch, "page.html"), encoding="utf-8") as handle:
                page = handle.read()
        self.assertIn("This export holds no documents", page)

    def test_a_corpus_with_documents_carries_no_empty_notice(self):
        page = dashboard.render(dashboard.load(load_fixture(), corpus_identity="fixture"))
        self.assertNotIn("holds no documents", page)

    def test_documents_with_no_governed_code_path_carry_no_empty_notice(self):
        export = load_fixture()
        export["graph"]["anchors"] = []
        export["graph"]["edges"] = []
        model = dashboard.load(export, corpus_identity="fixture")
        self.assertEqual((len(model.documents), len(model.anchors)), (2, 0))
        self.assertNotIn("holds no documents", dashboard.render(model))

    def test_staleness_order_and_values_are_date_text(self):
        export = load_fixture()
        export["graph"]["documents"].append(
            {"path": "docs/y.md", "kind": "decision", "id": "FX-DR-0003",
             "facets": {"title": "Mid-year", "last_verified": "2026-06-30"}}
        )
        model = dashboard.load(export, corpus_identity="fixture")
        rows = dashboard.staleness_view(model)
        self.assertEqual(
            [(row["id"], row["last_verified"]) for row in rows],
            [("FX-OBL-0001", "2026-01-15"), ("FX-DR-0003", "2026-06-30"), ("FX-DR-0002", "2026-09-01")],
        )
        for row in rows:
            self.assertIs(type(row["last_verified"]), str)

    def test_an_undated_document_is_listed_first_as_never_verified(self):
        export = load_fixture()
        export["graph"]["documents"].append(
            {"path": "docs/x.md", "kind": "decision", "id": "FX-DR-0009", "facets": {"title": "Never checked"}}
        )
        model = dashboard.load(export, corpus_identity="fixture")
        order = [row["id"] for row in dashboard.staleness_view(model)]
        self.assertEqual(order, ["FX-DR-0009", "FX-OBL-0001", "FX-DR-0002"])
        page = dashboard.render(model)
        self.assertIn("<td>%s</td><td>FX-DR-0009</td>" % dashboard.NEVER_VERIFIED, page)


class TheWorkedExample(unittest.TestCase):
    """This repository's graph export is the input the issue names."""

    def test_every_document_of_the_repository_export_is_on_the_page(self):
        export = repo_export()
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
