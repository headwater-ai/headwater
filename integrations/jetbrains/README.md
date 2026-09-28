# Headwater for JetBrains IDEs

This plugin shows a person who edits a Headwater repository in IntelliJ IDEA, PyCharm, GoLand, RustRover or another JetBrains IDE the documents that govern their work. It does this at three moments:

- **Intent.** The action *Headwater: What Governs This Task?*, in the Tools menu and in Find Action, asks what you are about to do and lists the documents that govern it. Each row shows a path, a name and a one-line summary, and never the content of the document. When the engine's budget held more documents back, the foot of the list says how many, for example "210 more withheld by the budget". Choose a row to open the document.
- **Read.** The *Headwater* tool window lists the documents that govern the file in the selected editor. It is empty when no document governs the file. Double-click a row, or press Enter on it, to open the document.
- **Write.** When you save a file that a document governs, a balloon names the governing documents. The listener only queues the query and returns, so the save never waits for it, and the balloon comes after the save.

When nothing governs the task or the file, the plugin shows nothing. It does not guess.

## How it gets its answers

The plugin is a client of `headwater mcp`, the corpus MCP server that the engine ships. For each question it starts `headwater mcp --root <project directory>`, sends one request and stops the process. It does not look for a server that is already running, because the server reads standard input and walks the corpus once when it starts. A server kept alive across your edits would answer from an old walk. Every query runs on a pooled thread, never on the thread that draws the IDE.

It calls two tools of the query class, `route` and `governing_docs_for_path`, and no other tool. It never passes `--write`, so the server it starts registers no tool that writes. It reads each answer from the `structuredContent` member, which [the `headwater mcp` interface](../../docs/interfaces/headwater-mcp.md) states as the machine contract of both tools. It takes the path, the kind, the name and the summary of each element of `pointers`, and shows the warrant sentence when the element is `unwarranted`. For `route` it also reads `withheld`, the number of documents the budget held back, so that it reports them rather than drop them in silence. It never reads the text block. A path can hold ` (` and a summary can hold any word, so a pointer read from the text can be wrong. An answer with no `structuredContent` comes from an engine older than this contract, and the plugin shows nothing for it.

`src/main/java/ai/headwater/jetbrains/Client.java` makes every one of these decisions and imports nothing from the IntelliJ Platform. The other classes only show what it returns.

## When it does nothing

The plugin degrades to nothing and says nothing in each of these cases:

- no `headwater` binary is on the path of the IDE process, or at the path the `HEADWATER_BIN` environment variable names
- the project directory has no `.headwater/` directory, or the project has no directory on the local disk
- the file is outside the project directory
- the server exits with an error, answers with an error, answers without `structuredContent`, answers JSON the plugin cannot read, writes more than one megabyte, or takes more than five seconds

This is the same fails-open posture as every hook in this repository. A missing engine never blocks your IDE.

An IDE started from a desktop launcher often does not inherit the path of your shell. If the tool window stays empty for a file you know is governed, start the IDE from a shell where `headwater` is on the path, or set `HEADWATER_BIN` to the binary in the environment the IDE starts in.

## What it does not show

The write-time hook of the three agent harnesses also prints the paths in the governed scope that nothing governs, and the suspect edges of a document. `headwater mcp` does not serve those two accounts, so the save balloon names the governing documents only.

## Install

Build or install the engine so that `headwater` is on your path, or set `HEADWATER_BIN`. Then build the plugin in this folder with Gradle 9.1 or later on a JDK 21 or later:

    gradle buildPlugin

This writes `build/distributions/headwater-jetbrains-0.1.0.zip`. In the IDE, open *Settings*, then *Plugins*, then the gear menu, then *Install Plugin from Disk*, and choose that file. The plugin declares IntelliJ Platform build 243 (2024.3) as its oldest. No Gradle wrapper jar is committed, because that is a binary in the tree, so Gradle comes from your own install.

The plugin has no dependency beyond the IntelliJ Platform. It is licensed under Apache-2.0, and `LICENSE` in this folder is a copy of the repository's license.

What was run and what was not, on 2026-09-28: `gradle buildPlugin` with Gradle 9.8.0 on OpenJDK 25 compiled every class against IntelliJ IDEA Community 2024.3.6 and wrote the plugin archive. `.github/workflows/integrations-jetbrains.yml` runs the same build on every change to this folder. Nobody has yet installed that archive into an IDE, or run the IDE with the plugin loaded, because no JetBrains IDE was available. So the glue classes have compiled against the platform and have not run inside it.

## Tests

    HEADWATER_BIN=engine/target/dev-release/headwater java integrations/jetbrains/test/RunTests.java

Run it from the repository root with a JDK 22 or later. It needs no Gradle, no JUnit and no network: `RunTests.java` compiles `Client.java` and the suite with the compiler module the JDK carries, and runs the suite. `Client.java` makes every decision, and the suite holds it. One case runs against the real engine that `HEADWATER_BIN` names. It skips on a workstation when the variable is unset, and it fails in CI when the variable is unset. The other cases replay sessions that the real server recorded, under `test/fixtures/`, through `test/FakeServer.java`. The glue classes have no tests, because the IntelliJ Platform does not load under a bare JDK. They only show what `Client.java` returns, and `gradle buildPlugin` compiles them.
