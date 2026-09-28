// SPDX-License-Identifier: Apache-2.0
//
// A stand-in for `headwater mcp` that replays a recorded session. `RunTests`
// compiles it, and the suite starts it as `java -cp <classes> FakeServer mcp
// --root <root>`.
//
//   FAKE_FIXTURE  a file under test/fixtures/ with one line per response, as the
//                 real server wrote them. A request with an `id` is answered with
//                 the recorded line of the same `id`. A line that is not JSON is
//                 written out on the first request, which is how the garbage
//                 case reaches the client. A path that names no file is a server
//                 that never answers and exits on its own after ten seconds.
//   FAKE_RECORD   a file this server appends to, one JSON object per line: first
//                 `{"argv": [...]}`, then `{"message": ...}` for each line read.
//   FAKE_EXIT     the status to exit with once standard input closes (default 0).

import ai.headwater.jetbrains.Client;
import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.PrintStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public class FakeServer {
    public static void main(String[] args) throws IOException, InterruptedException {
        String fixture = System.getenv("FAKE_FIXTURE");
        String record = System.getenv("FAKE_RECORD");
        String exit = System.getenv("FAKE_EXIT");
        int status = exit == null || exit.isEmpty() ? 0 : Integer.parseInt(exit);

        note(record, "{\"argv\":" + Client.Json.write(List.of(args)) + "}");

        boolean silent = fixture == null || !Files.isRegularFile(Path.of(fixture));
        Map<Object, String> byId = new HashMap<>();
        List<String> garbage = new ArrayList<>();
        if (!silent) {
            for (String line : Files.readAllLines(Path.of(fixture), StandardCharsets.UTF_8)) {
                if (line.isBlank()) continue;
                try {
                    if (Client.Json.parse(line) instanceof Map<?, ?> m && m.containsKey("id")) byId.put(m.get("id"), line);
                } catch (IllegalArgumentException e) {
                    garbage.add(line);
                }
            }
        }

        PrintStream out = new PrintStream(System.out, true, StandardCharsets.UTF_8);
        BufferedReader in = new BufferedReader(new InputStreamReader(System.in, StandardCharsets.UTF_8));
        boolean answered = false;
        for (String line = in.readLine(); line != null; line = in.readLine()) {
            if (line.isBlank()) continue;
            Object message;
            try {
                message = Client.Json.parse(line);
            } catch (IllegalArgumentException e) {
                note(record, "{\"unparsed\":" + Client.Json.write(line) + "}");
                continue;
            }
            note(record, "{\"message\":" + line + "}");
            if (silent || !(message instanceof Map<?, ?> m) || !m.containsKey("id")) continue;
            if (!answered) {
                for (String g : garbage) out.println(g);
                answered = true;
            }
            String reply = byId.get(m.get("id"));
            if (reply != null) out.println(reply);
        }
        if (silent) {
            // Outlive any client timeout, and still end on its own, so a client
            // whose timeout is broken fails its case rather than hanging the suite.
            Thread.sleep(10_000);
            System.exit(0);
        }
        System.exit(status);
    }

    private static void note(String record, String line) throws IOException {
        if (record == null) return;
        Files.writeString(Path.of(record), line + "\n", StandardCharsets.UTF_8,
                StandardOpenOption.CREATE, StandardOpenOption.APPEND);
    }
}
