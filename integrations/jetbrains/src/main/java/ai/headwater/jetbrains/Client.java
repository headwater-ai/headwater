// SPDX-License-Identifier: Apache-2.0
//
// The one class of the JetBrains plugin that decides anything. It imports no
// `com.intellij` class, so `test/RunTests.java` holds every decision it makes
// with a bare JDK.
//
// It starts `<bin> mcp --root <root>` once per query. `headwater mcp` is stdio
// only, so there is no running server to find. It walks the corpus once when it
// starts, so a server kept alive across edits would answer from a stale walk.
//
// It calls two tools of the query class and no other: `route` and
// `governing_docs_for_path`. It never passes `--write`. `ask` refuses any other
// tool name before it starts anything.
//
// It reads the pointers out of `structuredContent`, which
// `docs/interfaces/headwater-mcp.md` names as the machine contract of both
// tools, and never out of the text block. The text is for a person: a path may
// hold ` (` and a summary any word, so no parse of it is exact. For `route` it
// also reads `withheld`, the count of pointers the budget held back, so that the
// plugin can say that more exist rather than drop them in silence.
//
// It fails open. A start error, a non-zero exit, a timeout, `isError: true`, a
// JSON-RPC error, an answer without `structuredContent` (an engine older than
// #1248) and output that does not parse all give `Answer.NONE`. No call throws,
// except `ask` with a tool outside the allowlist, which is a defect in the
// caller and not a state of the project.

package ai.headwater.jetbrains;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Duration;
import java.util.ArrayList;
import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.TimeUnit;

public final class Client {
    private Client() {}

    /** The only tools this client sends. Nothing else can reach the server. */
    public static final List<String> TOOLS = List.of("route", "governing_docs_for_path");

    static final Duration DEFAULT_TIMEOUT = Duration.ofSeconds(5);
    static final int MAX_OUTPUT = 1 << 20;

    // The sentence `route` and `governing_docs_for_path` write after a pointer
    // to a document nobody accepted, as `engine/crates/query/src/lib.rs` writes
    // it. The structured pointer states the fact as `unwarranted`, and this is
    // the one sentence of the server that this class copies.
    static final String UNWARRANTED = "nobody accepted this document";

    /** One document an answer points at. Any member but `path` may be null. */
    public record Pointer(String path, String kind, String id, String name, String purpose, String summary, String asserted) {}

    /** The pointers of one answer, in order, and how many more the budget withheld. */
    public record Answer(List<Pointer> pointers, int withheld) {
        public static final Answer NONE = new Answer(List.of(), 0);

        public Answer {
            pointers = List.copyOf(pointers);
        }
    }

    /** Where the engine is and how to start it. */
    public record Options(Path root, String bin, List<String> binArgs, Map<String, String> env, Duration timeout) {
        public static Options of(Path root, String bin) {
            return new Options(root, bin, List.of(), Map.of(), DEFAULT_TIMEOUT);
        }

        public Options withRoot(Path value) {
            return new Options(value, bin, binArgs, env, timeout);
        }

        public Options withBinArgs(List<String> value) {
            return new Options(root, bin, List.copyOf(value), env, timeout);
        }

        public Options withEnv(Map<String, String> value) {
            return new Options(root, bin, binArgs, Map.copyOf(value), timeout);
        }

        public Options withTimeout(Duration value) {
            return new Options(root, bin, binArgs, env, value);
        }
    }

    /** How the plugin names one pointer: its path, and its name when it has one. */
    public static String label(Pointer pointer) {
        return pointer.name() == null ? pointer.path() : pointer.path() + " (" + pointer.name() + ")";
    }

    /**
     * The line the plugin shows under a route answer when the budget held
     * pointers back, or null when it held none back and there is nothing to say.
     */
    public static String withheldNote(Answer answer) {
        return answer.withheld() > 0 ? answer.withheld() + " more withheld by the budget" : null;
    }

    /** The documents that govern the task the user typed, and the count withheld. */
    public static Answer route(String task, Options options) {
        return ask("route", Map.of("task", String.valueOf(task)), options);
    }

    /** The documents that govern a project-relative path. */
    public static List<Pointer> governing(String relativePath, Options options) {
        return ask("governing_docs_for_path", Map.of("path", String.valueOf(relativePath)), options).pointers();
    }

    /**
     * One session: start, initialize, one `tools/call`, read the answer, stop.
     * Throws when `tool` is not in `TOOLS`, and gives `Answer.NONE` on every
     * failure.
     */
    public static Answer ask(String tool, Map<String, String> arguments, Options options) {
        if (!TOOLS.contains(tool)) {
            throw new IllegalArgumentException("`" + tool + "` is not a tool this client calls; it calls " + String.join(", ", TOOLS));
        }
        Path root = options.root();
        if (root == null || !Files.isDirectory(root.resolve(".headwater"))) return Answer.NONE;

        List<String> argv = new ArrayList<>();
        argv.add(options.bin() == null || options.bin().isEmpty() ? "headwater" : options.bin());
        argv.addAll(options.binArgs());
        argv.addAll(List.of("mcp", "--root", root.toString()));

        Process process;
        try {
            ProcessBuilder builder = new ProcessBuilder(argv)
                    .directory(root.toFile())
                    .redirectError(ProcessBuilder.Redirect.DISCARD);
            builder.environment().putAll(options.env());
            process = builder.start();
        } catch (IOException | RuntimeException e) {
            return Answer.NONE;
        }
        try {
            return session(process, tool, arguments, options.timeout());
        } finally {
            process.destroyForcibly();
        }
    }

    private static Answer session(Process process, String tool, Map<String, String> arguments, Duration timeout) {
        long deadline = System.nanoTime() + timeout.toNanos();
        ByteArrayOutputStream stdout = new ByteArrayOutputStream();
        boolean[] overflow = {false};
        Thread reader = Thread.ofVirtual().start(() -> {
            byte[] buffer = new byte[8192];
            try (InputStream in = process.getInputStream()) {
                for (int n = in.read(buffer); n >= 0; n = in.read(buffer)) {
                    synchronized (stdout) {
                        if (stdout.size() + n > MAX_OUTPUT) {
                            overflow[0] = true;
                            process.destroyForcibly();
                            return;
                        }
                        stdout.write(buffer, 0, n);
                    }
                }
            } catch (IOException ignored) {
                // The process died or was killed; what was read is what there is.
            }
        });

        Map<String, Object> clientInfo = new LinkedHashMap<>();
        clientInfo.put("name", "headwater-jetbrains");
        clientInfo.put("version", "0.1.0");
        Map<String, Object> init = new LinkedHashMap<>();
        init.put("protocolVersion", "2024-11-05");
        init.put("capabilities", Map.of());
        init.put("clientInfo", clientInfo);
        Map<String, Object> call = new LinkedHashMap<>();
        call.put("name", tool);
        call.put("arguments", new LinkedHashMap<>(arguments));
        String messages = message(1, "initialize", init) + "\n"
                + message(null, "notifications/initialized", null) + "\n"
                + message(2, "tools/call", call) + "\n";
        try (OutputStream in = process.getOutputStream()) {
            in.write(messages.getBytes(StandardCharsets.UTF_8));
        } catch (IOException ignored) {
            // A server that exits before it reads is judged by its exit below.
        }

        try {
            long left = deadline - System.nanoTime();
            if (left <= 0 || !process.waitFor(left, TimeUnit.NANOSECONDS)) return Answer.NONE;
            left = deadline - System.nanoTime();
            reader.join(Duration.ofNanos(Math.max(left, 1)));
            if (reader.isAlive()) return Answer.NONE;
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            return Answer.NONE;
        }
        if (process.exitValue() != 0) return Answer.NONE;
        String text;
        synchronized (stdout) {
            if (overflow[0]) return Answer.NONE;
            text = stdout.toString(StandardCharsets.UTF_8);
        }
        return answer(text);
    }

    private static String message(Integer id, String method, Map<String, Object> params) {
        Map<String, Object> m = new LinkedHashMap<>();
        m.put("jsonrpc", "2.0");
        if (id != null) m.put("id", id);
        m.put("method", method);
        if (params != null) m.put("params", params);
        return Json.write(m);
    }

    // The answer to the `tools/call`, or `Answer.NONE`.
    static Answer answer(String stdout) {
        for (String line : stdout.split("\n")) {
            Object message;
            try {
                message = Json.parse(line);
            } catch (IllegalArgumentException e) {
                continue;
            }
            if (!(message instanceof Map<?, ?> m) || !(m.get("id") instanceof Long id) || id != 2L) continue;
            if (!(m.get("result") instanceof Map<?, ?> result) || Boolean.TRUE.equals(result.get("isError"))) return Answer.NONE;
            return read(result.get("structuredContent"));
        }
        return Answer.NONE;
    }

    /**
     * The pointers of one answer's `structuredContent`, in order, and its
     * `withheld` count. An answer with no pointer list, or with any element
     * that is not a pointer, is `Answer.NONE`.
     */
    public static Answer read(Object structured) {
        if (!(structured instanceof Map<?, ?> s) || !(s.get("pointers") instanceof List<?> elements)) return Answer.NONE;
        List<Pointer> pointers = new ArrayList<>();
        for (Object element : elements) {
            Pointer p = pointerOf(element);
            if (p == null) return Answer.NONE;
            pointers.add(p);
        }
        int withheld = s.get("withheld") instanceof Long n && n >= 0 && n <= Integer.MAX_VALUE ? n.intValue() : 0;
        return new Answer(pointers, withheld);
    }

    private static Pointer pointerOf(Object element) {
        if (!(element instanceof Map<?, ?> e) || !(e.get("path") instanceof String path)) return null;
        return new Pointer(path, text(e.get("kind")), text(e.get("id")), text(e.get("name")), text(e.get("purpose")),
                text(e.get("summary")), Boolean.TRUE.equals(e.get("unwarranted")) ? UNWARRANTED : null);
    }

    private static String text(Object value) {
        return value instanceof String s ? s : null;
    }

    /**
     * The JSON this client reads and writes, and no more. The JDK ships no
     * parser, and the client takes no dependency. An object is a
     * `LinkedHashMap`, an array a `List`, a whole number a `Long`, any other
     * number a `Double`, and `null` is `null`. Input it cannot read throws
     * `IllegalArgumentException`.
     */
    public static final class Json {
        private Json() {}

        private static final int MAX_DEPTH = 64;

        public static Object parse(String text) {
            Json.Reader r = new Json.Reader(text);
            r.space();
            Object value = r.value(0);
            r.space();
            if (r.at < text.length()) throw r.fail("text after the value");
            return value;
        }

        public static String write(Object value) {
            StringBuilder out = new StringBuilder();
            write(value, out);
            return out.toString();
        }

        private static void write(Object value, StringBuilder out) {
            switch (value) {
                case null -> out.append("null");
                case String s -> string(s, out);
                case Boolean b -> out.append(b);
                case Integer n -> out.append(n);
                case Long n -> out.append(n);
                case Double n when Double.isFinite(n) -> out.append(n);
                case Map<?, ?> m -> {
                    out.append('{');
                    boolean first = true;
                    for (Map.Entry<?, ?> entry : m.entrySet()) {
                        if (!first) out.append(',');
                        first = false;
                        string(String.valueOf(entry.getKey()), out);
                        out.append(':');
                        write(entry.getValue(), out);
                    }
                    out.append('}');
                }
                case List<?> l -> {
                    out.append('[');
                    for (int i = 0; i < l.size(); i++) {
                        if (i > 0) out.append(',');
                        write(l.get(i), out);
                    }
                    out.append(']');
                }
                default -> throw new IllegalArgumentException("no JSON form for " + value.getClass().getName());
            }
        }

        private static void string(String s, StringBuilder out) {
            out.append('"');
            for (int i = 0; i < s.length(); i++) {
                char c = s.charAt(i);
                switch (c) {
                    case '"' -> out.append("\\\"");
                    case '\\' -> out.append("\\\\");
                    case '\n' -> out.append("\\n");
                    case '\r' -> out.append("\\r");
                    case '\t' -> out.append("\\t");
                    case '\b' -> out.append("\\b");
                    case '\f' -> out.append("\\f");
                    default -> {
                        if (c < 0x20) out.append(String.format("\\u%04x", (int) c));
                        else out.append(c);
                    }
                }
            }
            out.append('"');
        }

        private static final class Reader {
            final String s;
            int at;

            Reader(String s) {
                this.s = s;
            }

            IllegalArgumentException fail(String what) {
                return new IllegalArgumentException("unreadable JSON at " + at + ": " + what);
            }

            void space() {
                while (at < s.length() && " \t\r\n".indexOf(s.charAt(at)) >= 0) at++;
            }

            char peek() {
                if (at >= s.length()) throw fail("end of input");
                return s.charAt(at);
            }

            void expect(char c) {
                if (peek() != c) throw fail("expected " + c);
                at++;
            }

            Object value(int depth) {
                if (depth > MAX_DEPTH) throw fail("nesting too deep");
                char c = peek();
                return switch (c) {
                    case '{' -> object(depth);
                    case '[' -> array(depth);
                    case '"' -> string();
                    case 't' -> word("true", Boolean.TRUE);
                    case 'f' -> word("false", Boolean.FALSE);
                    case 'n' -> word("null", null);
                    default -> {
                        if (c == '-' || (c >= '0' && c <= '9')) yield number();
                        throw fail("unexpected " + c);
                    }
                };
            }

            Object word(String word, Object value) {
                if (!s.startsWith(word, at)) throw fail("expected " + word);
                at += word.length();
                return value;
            }

            Map<String, Object> object(int depth) {
                expect('{');
                Map<String, Object> out = new LinkedHashMap<>();
                space();
                if (peek() == '}') {
                    at++;
                    return out;
                }
                while (true) {
                    space();
                    String key = string();
                    space();
                    expect(':');
                    space();
                    out.put(key, value(depth + 1));
                    space();
                    if (peek() == ',') {
                        at++;
                        continue;
                    }
                    expect('}');
                    return out;
                }
            }

            List<Object> array(int depth) {
                expect('[');
                List<Object> out = new ArrayList<>();
                space();
                if (peek() == ']') {
                    at++;
                    return Collections.unmodifiableList(out);
                }
                while (true) {
                    space();
                    out.add(value(depth + 1));
                    space();
                    if (peek() == ',') {
                        at++;
                        continue;
                    }
                    expect(']');
                    return Collections.unmodifiableList(out);
                }
            }

            String string() {
                expect('"');
                StringBuilder out = new StringBuilder();
                while (true) {
                    char c = peek();
                    at++;
                    if (c == '"') return out.toString();
                    if (c < 0x20) throw fail("a control character in a string");
                    if (c != '\\') {
                        out.append(c);
                        continue;
                    }
                    char e = peek();
                    at++;
                    switch (e) {
                        case '"', '\\', '/' -> out.append(e);
                        case 'b' -> out.append('\b');
                        case 'f' -> out.append('\f');
                        case 'n' -> out.append('\n');
                        case 'r' -> out.append('\r');
                        case 't' -> out.append('\t');
                        case 'u' -> {
                            if (at + 4 > s.length()) throw fail("a short \\u escape");
                            try {
                                out.append((char) Integer.parseInt(s.substring(at, at + 4), 16));
                            } catch (NumberFormatException x) {
                                throw fail("a bad \\u escape");
                            }
                            at += 4;
                        }
                        default -> throw fail("a bad escape \\" + e);
                    }
                }
            }

            Object number() {
                int start = at;
                if (peek() == '-') at++;
                if (peek() == '0') {
                    at++;
                } else {
                    digits();
                }
                boolean whole = true;
                if (at < s.length() && s.charAt(at) == '.') {
                    whole = false;
                    at++;
                    digits();
                }
                if (at < s.length() && (s.charAt(at) == 'e' || s.charAt(at) == 'E')) {
                    whole = false;
                    at++;
                    if (at < s.length() && (s.charAt(at) == '+' || s.charAt(at) == '-')) at++;
                    digits();
                }
                String n = s.substring(start, at);
                if (whole) {
                    try {
                        return Long.parseLong(n);
                    } catch (NumberFormatException x) {
                        return Double.parseDouble(n);
                    }
                }
                return Double.parseDouble(n);
            }

            void digits() {
                int start = at;
                while (at < s.length() && s.charAt(at) >= '0' && s.charAt(at) <= '9') at++;
                if (at == start) throw fail("expected a digit");
            }
        }
    }
}
