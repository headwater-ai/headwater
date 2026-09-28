// SPDX-License-Identifier: Apache-2.0
//
// What holds `Client.java`, the one class of the plugin that decides anything.
// The glue classes import the IntelliJ Platform, which this suite does not
// load, so they are kept thin enough to carry no decision.
//
// The first case runs against the real engine: `HEADWATER_BIN` names it. With
// the variable unset it skips with a printed reason on a workstation, and it
// fails in CI, where a skip would hide the one case that reads a real corpus.
//
// Every other case drives `FakeServer.java`, which replays a session that the
// real server recorded under `test/fixtures/` and writes down what it was
// asked. Run the suite from the repository root with
// `java integrations/jetbrains/test/RunTests.java`, which compiles this file.

import ai.headwater.jetbrains.Client;
import ai.headwater.jetbrains.Client.Answer;
import ai.headwater.jetbrains.Client.Options;
import ai.headwater.jetbrains.Client.Pointer;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Duration;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;

public class ClientTest {
    static final Path TEST = Path.of(System.getProperty("hw.test"));
    static final Path REPO = TEST.getParent().getParent().getParent();
    static final Path FIXTURES = TEST.resolve("fixtures");
    static final String JAVA = ProcessHandle.current().info().command().orElse("java");
    static final String UNWARRANTED = "nobody accepted this document";

    interface Case {
        void run() throws Exception;
    }

    static final class Skip extends RuntimeException {
        private static final long serialVersionUID = 1L;

        Skip(String reason) {
            super(reason);
        }
    }

    record Named(String name, Case body) {}

    static final List<Named> CASES = new ArrayList<>();

    static void test(String name, Case body) {
        CASES.add(new Named(name, body));
    }

    public static void main(String[] args) {
        register();
        int failed = 0;
        int skipped = 0;
        for (Named c : CASES) {
            if (args.length > 0 && !c.name().contains(args[0])) continue;
            try {
                c.body().run();
                System.out.println("ok      " + c.name());
            } catch (Skip s) {
                skipped++;
                System.out.println("skip    " + c.name() + " -- " + s.getMessage());
            } catch (Throwable t) {
                failed++;
                System.out.println("not ok  " + c.name());
                System.out.println("        " + t);
            }
        }
        System.out.println(CASES.size() + " cases, " + failed + " failed, " + skipped + " skipped");
        System.exit(failed == 0 ? 0 : 1);
    }

    // ---- assertions

    static void check(boolean condition, String message) {
        if (!condition) throw new AssertionError(message);
    }

    static void equal(Object actual, Object expected) {
        if (!Objects.equals(actual, expected)) {
            throw new AssertionError("expected " + expected + "\n        but got  " + actual);
        }
    }

    static void throwsBeforeSpawn(Runnable body, String contains) {
        try {
            body.run();
        } catch (IllegalArgumentException e) {
            check(e.getMessage().contains(contains), "the refusal says " + e.getMessage());
            return;
        }
        throw new AssertionError("no refusal");
    }

    // ---- the fake server

    record Fake(Options options, Path record) {
        List<Map<?, ?>> asked() throws IOException {
            List<Map<?, ?>> out = new ArrayList<>();
            if (!Files.exists(record)) return out;
            for (String line : Files.readAllLines(record, StandardCharsets.UTF_8)) {
                if (!line.isBlank()) out.add((Map<?, ?>) Client.Json.parse(line));
            }
            return out;
        }
    }

    // A client aimed at the fake server: `java -cp <classes> FakeServer mcp --root <root>`.
    static Fake fake(String fixture, Map<String, String> env) throws IOException {
        Path record = Files.createTempDirectory("hw-jetbrains-").resolve("record.jsonl");
        Map<String, String> all = new LinkedHashMap<>();
        all.put("FAKE_FIXTURE", FIXTURES.resolve(fixture).toString());
        all.put("FAKE_RECORD", record.toString());
        all.putAll(env);
        Options options = Options.of(REPO, JAVA)
                .withBinArgs(List.of("-cp", System.getProperty("hw.classes"), "FakeServer"))
                .withEnv(all);
        return new Fake(options, record);
    }

    static Fake fake(String fixture) throws IOException {
        return fake(fixture, Map.of());
    }

    static List<String> paths(List<Pointer> pointers) {
        return pointers.stream().map(Pointer::path).toList();
    }

    // ---- the cases

    static void register() {
        test("governing answers the contract's pointers, silence for an ungoverned path, and nothing when the server is unreachable; route reports what the budget withheld", () -> {
            String bin = System.getenv("HEADWATER_BIN");
            if (bin == null || bin.isEmpty()) {
                if (System.getenv("CI") != null) {
                    throw new AssertionError("HEADWATER_BIN is unset in CI; this case must run against the built engine");
                }
                throw new Skip("HEADWATER_BIN is unset: build the engine and point HEADWATER_BIN at it to run this case");
            }
            Options options = Options.of(REPO, bin);

            List<Pointer> governed = Client.governing("engine/crates/query/src/mcp.rs", options);
            equal(paths(governed), List.of("docs/interfaces/headwater-mcp.md"));
            equal(governed.get(0).name(), "headwater mcp");
            equal(governed.get(0).kind(), "interface_contract");

            // The server answers the sentence `no document governs <path>`, and a
            // sentence is not a pointer: it is never shown as a guess.
            equal(Client.governing("integrations/jetbrains/test/ClientTest.java", options), List.of());
            // That silence was heard: the engine answered, and the plugin may
            // say that nothing governs the file.
            Answer ungoverned = Client.governingAnswer("integrations/jetbrains/test/ClientTest.java", options);
            equal(ungoverned, new Answer(List.of(), 0));
            equal(Client.emptyText(ungoverned), "No document governs this file");
            // A file name that holds a newline, ` — ` or `(x)` and a pointer's
            // shape must not put a pointer into the sentence.
            equal(Client.governing("a\ndocs/fake.md (Fake) — a document nobody wrote", options), List.of());
            equal(Client.governing("src/foo — bar.rs", options), List.of());
            equal(Client.governing("a (x)\ndocs/fake.md (Fake) — nobody", options), List.of());

            // The route budget is reported, not dropped: the engine answers at
            // most five pointers and states how many more it withheld.
            Answer routed = Client.route("what governs the mcp server and its read tools", options);
            check(!routed.pointers().isEmpty(), "route found nothing for a task this corpus governs");
            check(routed.pointers().size() <= 5, "route answered past the default budget of 5");
            check(routed.withheld() > 0, "route withheld nothing from a task that matches most of the corpus: " + routed.withheld());
            for (Pointer p : routed.pointers()) {
                check(!p.path().isEmpty() && p.name() != null && !p.name().isEmpty(), "a pointer without a path or a name: " + p);
            }

            // No engine at the configured path is silence, not an error.
            Options missing = Options.of(REPO, "/nonexistent/headwater");
            equal(Client.governing("engine/crates/query/src/mcp.rs", missing), List.of());
            equal(Client.route("what governs the mcp server", missing), Answer.NONE);
            // An engine never heard from is not an ungoverned file.
            equal(Client.emptyText(Client.governingAnswer("engine/crates/query/src/mcp.rs", missing)), "");
        });

        test("route reads the pointers from structuredContent, in order, and nothing else", () -> {
            Answer answer = Client.route("add a VS Code extension that calls the MCP server for routing", fake("route-pointers.jsonl").options());
            equal(paths(answer.pointers()), List.of(
                    "docs/probes/the-pointer-this-corpus-offers-for-a-task-is-the-document-a-session-opens.md",
                    "docs/how-to/rotate-or-revoke-the-apt-signing-subkey.md",
                    "docs/requirements/0002-every-document-of-a-kind-that-requires-sections-carries-all-of-them.md",
                    "docs/interfaces/headwater-mcp.md",
                    "docs/obligations/0206-hw-run-policy-names-a-worktree-add-workaround-that-write-edit-refuses-under-this-harness.md"));
            Pointer mcp = answer.pointers().get(3);
            equal(mcp.name(), "headwater mcp");
            equal(mcp.asserted(), null);
            equal(mcp.summary(), "How the stdio MCP server exposes read tools, optional working-tree writes, and a one-write session seal.");
        });

        test("route reports the count the budget withheld, and governing withholds nothing", () -> {
            Answer withheld = Client.route("add a JetBrains plugin that calls the MCP server for routing", fake("route-withheld.jsonl").options());
            equal(withheld.pointers().size(), 5);
            equal(withheld.withheld(), 210);
            equal(Client.route("x", fake("route-pointers.jsonl").options()).withheld(), 196);
            equal(Client.route("x", fake("route-paren-path.jsonl").options()).withheld(), 0);
            equal(Client.ask("governing_docs_for_path", Map.of("path", "src/my module.rs"), fake("governing-spaced-path.jsonl").options()).withheld(), 0);
            // What the plugin prints under the list: a count above zero, and nothing at zero.
            equal(Client.withheldNote(withheld), "210 more withheld by the budget");
            equal(Client.withheldNote(new Answer(withheld.pointers(), 1)), "1 more withheld by the budget");
            equal(Client.withheldNote(new Answer(withheld.pointers(), 0)), null);
        });

        test("a pointer is named by its path, and by its name when it has one", () -> {
            equal(Client.label(new Pointer("docs/a (b).md", "decision", null, "A (b)", null, null, null)), "docs/a (b).md (A (b))");
            equal(Client.label(new Pointer("docs/a.md", null, null, null, null, "s", null)), "docs/a.md");
        });

        test("a path that holds \" (\", a name with spaces and a summary past 80 columns read back exactly", () -> {
            equal(Client.route("why is quarantine throttling one quota rule", fake("route-paren-path.jsonl").options()),
                    new Answer(List.of(new Pointer(
                            "docs/decisions/a (draft) note.md", "decision", "DR-0099", "Quota notes at the edge", "rationale",
                            "why the quota governs each retry of a tenant, and what quarantine throttling does at the edge",
                            UNWARRANTED)), 0));
        });

        test("an answer with pointers in its text and no structuredContent is no pointers", () -> {
            equal(Client.route("add a VS Code extension that calls the MCP server for routing", fake("route-text-only.jsonl").options()), Answer.NONE);
        });

        test("an [asserted: ...] pointer keeps its warrant", () -> {
            Pointer first = Client.route("anything", fake("route-pointers.jsonl").options()).pointers().get(0);
            equal(first.asserted(), UNWARRANTED);
            check(!first.summary().contains("[asserted"), "the warrant leaked into the summary");
        });

        test("the no-purpose-matched answer is no pointers", () -> {
            equal(Client.route("zzqx", fake("route-no-purpose.jsonl").options()).pointers(), List.of());
        });

        test("isError: true is no pointers", () -> {
            equal(Client.governing("a/b.rs", fake("is-error.jsonl").options()), List.of());
        });

        test("isError: true is no pointers even when the answer carries structuredContent pointers", () -> {
            // `route-paren-path.jsonl` with `isError` set, so the flag alone decides.
            equal(Client.route("why is quarantine throttling one quota rule", fake("is-error-with-pointers.jsonl").options()), Answer.NONE);
        });

        test("output past one megabyte is no pointers, and output below it is read", () -> {
            equal(Client.route("x", fake("route-paren-path.jsonl", Map.of("FAKE_FLOOD", "2000000")).options()), Answer.NONE);
            equal(Client.route("x", fake("route-paren-path.jsonl", Map.of("FAKE_FLOOD", "500000")).options()).pointers().size(), 1);
        });

        test("an empty answer is heard and says nothing governs; every failure is unheard and says nothing at all", () -> {
            for (String fixture : List.of("governing-newline-path.jsonl", "governing-dash-path.jsonl", "governing-paren-path.jsonl")) {
                Answer heard = Client.governingAnswer("a/b.rs", fake(fixture).options());
                equal(heard, new Answer(List.of(), 0, true));
                equal(Client.emptyText(heard), "No document governs this file");
            }
            equal(Client.route("zzqx", fake("route-no-purpose.jsonl").options()).heard(), true);
            Options missing = fake("route-pointers.jsonl").options().withRoot(Files.createTempDirectory("hw-jetbrains-empty-"));
            List<Answer> failures = List.of(
                    Client.governingAnswer("a/b.rs", fake("is-error.jsonl").options()),
                    Client.governingAnswer("a/b.rs", fake("garbage.jsonl").options()),
                    Client.governingAnswer("a/b.rs", fake("rpc-error.jsonl").options()),
                    Client.governingAnswer("a/b.rs", fake("route-text-only.jsonl").options()),
                    Client.governingAnswer("a/b.rs", fake("route-pointers.jsonl", Map.of("FAKE_EXIT", "3")).options()),
                    Client.governingAnswer("a/b.rs", fake("silent").options().withTimeout(Duration.ofMillis(300))),
                    Client.governingAnswer("a/b.rs", missing),
                    Client.governingAnswer("a/b.rs", Options.of(REPO, "/nonexistent/headwater")));
            for (Answer failure : failures) {
                equal(failure, Answer.NONE);
                equal(Client.emptyText(failure), "");
            }
            // A list with pointers needs no empty text.
            equal(Client.emptyText(Client.governingAnswer("src/my module.rs", fake("governing-spaced-path.jsonl").options())), "");
            try {
                new Answer(List.of(), 3, false);
                throw new AssertionError("an unheard answer carried a withheld count");
            } catch (IllegalArgumentException expected) {
                // Refused, as it should be.
            }
        });

        test("a server that floods standard error still answers", () -> {
            // 1 MiB, far past a pipe buffer: a client that pipes standard error
            // and never reads it leaves the server blocked until the timeout.
            equal(Client.route("x", fake("route-paren-path.jsonl", Map.of("FAKE_STDERR", "1048576")).options()).pointers().size(), 1);
        });

        test("garbage on stdout is no pointers", () -> {
            equal(Client.governing("a/b.rs", fake("garbage.jsonl").options()), List.of());
        });

        test("a JSON-RPC error answer is no pointers", () -> {
            equal(Client.route("x", fake("rpc-error.jsonl").options()), Answer.NONE);
        });

        test("a server that exits non-zero is no pointers, even after an answer", () -> {
            equal(Client.route("x", fake("route-pointers.jsonl", Map.of("FAKE_EXIT", "3")).options()), Answer.NONE);
        });

        // The fake server outlives the timeout by ten seconds, so a client whose
        // timeout does not fire fails here rather than passing late.
        test("a server that never answers is no pointers within the timeout", () -> {
            Options options = fake("silent").options().withTimeout(Duration.ofMillis(300));
            long started = System.nanoTime();
            equal(Client.governing("a/b.rs", options), List.of());
            long waited = (System.nanoTime() - started) / 1_000_000;
            check(waited < 3000, "the client waited " + waited + " ms, past its timeout");
        });

        test("a project with no .headwater/ starts nothing and answers nothing", () -> {
            Path empty = Files.createTempDirectory("hw-jetbrains-empty-");
            Fake f = fake("route-pointers.jsonl");
            equal(Client.route("x", f.options().withRoot(empty)), Answer.NONE);
            equal(f.asked(), List.of());
        });

        test("the client speaks only the query class: initialize, initialized, and an allowlisted tool, never --write", () -> {
            Fake f = fake("route-pointers.jsonl");
            Client.route("what governs this", f.options());
            Client.governing("engine/crates/query/src/mcp.rs", f.options());
            List<Map<?, ?>> records = f.asked();
            List<Object> argvs = records.stream().filter(r -> r.containsKey("argv")).<Object>map(r -> r.get("argv")).toList();
            equal(argvs.size(), 2);
            for (Object argv : argvs) {
                equal(argv, List.of("mcp", "--root", REPO.toString()));
                check(!((List<?>) argv).contains("--write"), "a session was started with --write");
            }
            List<Map<?, ?>> messages = records.stream().filter(r -> r.containsKey("message")).<Map<?, ?>>map(r -> (Map<?, ?>) r.get("message")).toList();
            equal(messages.stream().map(m -> m.get("method")).toList(), List.of(
                    "initialize", "notifications/initialized", "tools/call",
                    "initialize", "notifications/initialized", "tools/call"));
            equal(messages.stream().filter(m -> "tools/call".equals(m.get("method"))).map(m -> m.get("params")).toList(), List.of(
                    Map.of("name", "route", "arguments", Map.of("task", "what governs this")),
                    Map.of("name", "governing_docs_for_path", "arguments", Map.of("path", "engine/crates/query/src/mcp.rs"))));
            equal(Client.TOOLS, List.of("route", "governing_docs_for_path"));
            try {
                Client.TOOLS.add("fix");
                throw new AssertionError("the allowlist can be extended at run time");
            } catch (UnsupportedOperationException expected) {
                // The allowlist is frozen.
            }
        });

        test("asking for any tool outside the allowlist throws before anything starts", () -> {
            Fake f = fake("route-pointers.jsonl");
            for (String tool : List.of("new", "fix", "check", "explain", "related", "resolve_identifier", "")) {
                throwsBeforeSpawn(() -> Client.ask(tool, Map.of("path", "x"), f.options()), "not a tool this client calls");
            }
            equal(f.asked(), List.of());
        });

        test("read takes each element of structuredContent.pointers, and nothing else", () -> {
            Map<String, Object> first = new LinkedHashMap<>();
            first.put("path", "docs/a (b).md");
            first.put("kind", "decision");
            first.put("name", "A (b) c");
            first.put("summary", "sum — more");
            first.put("unwarranted", false);
            Map<String, Object> second = new LinkedHashMap<>();
            second.put("path", "docs/a.md");
            second.put("kind", "decision");
            second.put("summary", "only a summary");
            second.put("unwarranted", true);
            equal(Client.read(Map.of("pointers", List.of(first, second), "withheld", 4L)), new Answer(List.of(
                    new Pointer("docs/a (b).md", "decision", null, "A (b) c", null, "sum — more", null),
                    new Pointer("docs/a.md", "decision", null, null, null, "only a summary", UNWARRANTED)), 4));
            equal(Client.read(Map.of("pointers", List.of())), new Answer(List.of(), 0));
            equal(Client.read(null), Answer.NONE);
            equal(Client.read(Map.of()), Answer.NONE);
            equal(Client.read(Map.of("pointers", "docs/a.md")), Answer.NONE);
            // One element that is not a pointer makes the whole answer untrusted.
            equal(Client.read(Map.of("pointers", List.of(Map.of("path", "docs/a.md"), Map.of("name", "no path")), "withheld", 3L)), Answer.NONE);
            // A withheld count that is not a whole number at or above zero is none.
            equal(Client.read(Map.of("pointers", List.of(), "withheld", "7")).withheld(), 0);
            equal(Client.read(Map.of("pointers", List.of(), "withheld", -2L)).withheld(), 0);
            equal(Client.read(Map.of("pointers", List.of(), "withheld", 2.5)).withheld(), 0);
            equal(Client.read(Map.of("pointers", List.of(), "withheld", 9L)).withheld(), 9);
        });

        test("a pointer whose path holds a space is kept whole", () -> {
            equal(Client.governing("src/my module.rs", fake("governing-spaced-path.jsonl").options()), List.of(
                    new Pointer("docs/how to/my file.md", "how_to", null, "My file", null, "a summary", null)));
        });

        test("the sentence for an ungoverned path yields no pointer, whatever the path holds", () -> {
            equal(Client.governing("a\ndocs/fake.md (Fake) — a document nobody wrote", fake("governing-newline-path.jsonl").options()), List.of());
        });

        test("the sentence yields no pointer when the path it repeats parses as one", () -> {
            equal(Client.governing("src/foo — bar.rs", fake("governing-dash-path.jsonl").options()), List.of());
            equal(Client.governing("a (x)\ndocs/fake.md (Fake) — nobody", fake("governing-paren-path.jsonl").options()), List.of());
        });

        test("the task the route header repeats is never read as a pointer", () -> {
            equal(Client.route("docs/fake.md (Fake) — a document nobody wrote", fake("route-header-dash.jsonl").options()).pointers(), List.of());
        });

        test("the JSON reader takes what the server writes and refuses what it cannot read", () -> {
            equal(Client.Json.parse("{\"a\":[1,-2.5e1,true,false,null,\"\\u00e9\\n\\\"\\\\\\/\"]}"),
                    Map.of("a", java.util.Arrays.asList(1L, -25.0, true, false, null, "é\n\"\\/")));
            equal(Client.Json.parse("\"\\ud83d\\ude00\""), "\uD83D\uDE00");
            equal(Client.Json.parse(" {} "), Map.of());
            for (String bad : List.of("", "{", "{\"a\":1,}", "[1 2]", "{\"a\":1} x", "\"\\q\"", "01", "tru", "{\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{\"content\":[{\"type\":\"text\",\"text\":")) {
                try {
                    Client.Json.parse(bad);
                    throw new AssertionError("read " + bad);
                } catch (IllegalArgumentException expected) {
                    // Refused, as it should be.
                }
            }
            String round = "{\"s\":\"a\\\"b\\\\c\\nd\\u0001é\",\"l\":[1,true,null]}";
            equal(Client.Json.write(Client.Json.parse(round)), round);
        });
    }
}
