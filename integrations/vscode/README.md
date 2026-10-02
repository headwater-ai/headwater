# Headwater for VS Code

This extension shows a person who edits a Headwater repository in plain VS Code the documents that govern their work. It does this at three moments:

- **Intent.** The command *Headwater: What governs this task?* asks what you are about to do and lists the documents that govern it. Each entry shows a path, a name and a one-line summary, and never the content of the document. Select an entry to open the document.
- **Read.** When you open a file that a document governs, the status bar names that document. Click it to list every document that governs the file.
- **Write.** When you save a file that a document governs, a notification names the governing documents. The notification comes after the save, so the save never waits for it.

When nothing governs a file, the extension shows nothing. It does not guess. When nothing governs a task you asked about, a message gives the engine's own reason, such as `no declared purpose answers this task`, so you can tell an answer of nothing from no answer. When the route held pointers back, the message also says how many.

## For Copilot Chat

The extension also gives the same answers to GitHub Copilot Chat, in three ways ([#1583](https://github.com/headwater-ai/headwater/issues/1583)):

- **An MCP server.** For each workspace folder that holds `.headwater/`, the extension registers a server named *Headwater (folder)* with VS Code. Copilot agent mode can then call the six read tools of `headwater mcp`: `route`, `explain`, `related`, `resolve_identifier`, `governing_docs_for_path` and `check`. Nobody has to write a `.vscode/mcp.json`.
- **Two language model tools.** Type `#headwaterRoute` or `#headwaterGoverning` in a chat message to attach the documents that govern a task or a file. Agent mode can also call both tools on its own.
- **The `@headwater` participant.** Type `@headwater` and a task in the chat panel to list the documents that govern the task. Type `@headwater /file` to list the documents that govern the file in the active editor, or a file that you attach to the message. Each document is a link.

The two tools and the participant use the same client as the three moments above. The MCP server is `relay.js`, which VS Code runs with its own Node. VS Code keeps a registered server alive for the whole chat session, but `headwater mcp` walks the corpus once when it starts. So the relay stays alive in its place and starts one `headwater mcp` for each request, as [HW-DR-0102](../../docs/decisions/0102-an-editor-integration-starts-headwater-mcp-for-each-query-and-does-not-embed-the-library.md) rules for every query of an editor integration. An answer never comes from a walk older than the request, even after the agent edits the tree. The relay never passes `--write`.

When a route names no document, the two tools and the participant quote the engine's own reason, as the route command does. Any other empty answer says "Headwater named no document for this. Either nothing governs it, or the engine did not answer", because the client reads a failed session as an empty one. A model must not read that as "nothing governs this file".

## How it gets its answers

The extension is a client of `headwater mcp`, the corpus MCP server that the engine ships. For each question it starts `headwater mcp --root <workspace folder>`, sends one request and stops the process. It does not look for a server that is already running, because the server reads standard input and walks the corpus once when it starts. A server kept alive across your edits would answer from an old walk. One session takes about 0.16 seconds on this repository.

It calls two tools of the query class, `route` and `governing_docs_for_path`, and no other tool. It never passes `--write`, so the server it starts registers no tool that writes. It reads each answer from the `structuredContent` member, which [the `headwater mcp` interface](../../docs/interfaces/headwater-mcp.md) states as the machine contract of both tools. It takes the path, the name and the summary of each element of `pointers`, and shows the warrant sentence when the element is `unwarranted`. From a `route` answer it also reads `withheld`, the number of ranked pointers that the budget held back. When that number is more than zero, the pick list shows a line such as `196 more withheld by the budget`, so you can tell three answers from three of fifteen. When `pointers` is empty, it reads `silence.says`, the sentence the engine gives for a route it answered with nothing. It never reads the text block. A path can hold ` (` and a summary can hold any word, so a pointer read from the text can be wrong. An answer with no `structuredContent` comes from an engine older than this contract, and the extension shows nothing for it.

## When it does nothing

The extension degrades to nothing and says nothing in each of these cases:

- no `headwater` binary is on the path, or at the path the `headwater.path` setting names
- the workspace folder has no `.headwater/` directory
- the server exits with an error, answers with an error, answers without `structuredContent`, answers JSON the extension cannot read, or takes more than five seconds

This is the same fails-open posture as every hook in this repository. A missing engine never blocks your editor.

For Copilot Chat the posture is the same, with one change for the reader that is a model. When the extension finds no engine, it registers no MCP server. When a relay session fails, the relay answers with a JSON-RPC error and not with an empty list. It stops a session after 60 seconds, because the `check` tool reads the whole corpus.

## What it does not show

The write-time hook of the three agent harnesses also prints the paths in the governed scope that nothing governs, and the suspect edges of a document. `headwater mcp` does not serve those two accounts, so the save notification names the governing documents only.

## Install

The extension needs VS Code 1.101 or later. That is the first release in which the MCP server provider, the language model tools and the chat participant API are all stable. The Copilot surfaces also need GitHub Copilot Chat.

Build or install the engine so that `headwater` is on your path, or set `headwater.path` to the binary. Then choose one of these:

- Open this folder in VS Code and press F5. `.vscode/launch.json` starts a second VS Code window with the extension loaded and the repository root open, because the extension activates only in a folder that holds `.headwater/`. It puts `engine/target/dev-release` and `engine/target/release` first on the path of that window, so a built engine needs no `headwater.path` setting. The path uses a colon separator, so on Windows set `headwater.path` instead.
- Package it with `npx @vscode/vsce package` in this folder, and install the result with `code --install-extension headwater-vscode-0.2.0.vsix`.

The extension has no npm dependencies and no build step. It is licensed under Apache-2.0, and `LICENSE` in this folder is a copy of the repository's license. `.vscodeignore` keeps `test/` out of the package.

What was run and what was not, on 2026-10-02: `npx @vscode/vsce package` in this folder exited 0 with no warning and packaged six files (`LICENSE.txt`, `client.js`, `extension.js`, `package.json`, `readme.md`, `relay.js`). `relay.js` ran by hand against the `dev-release` engine over this repository. It forwarded `initialize`, listed the six read tools and answered a `governing_docs_for_path` call. Nobody has yet installed the package with `code --install-extension`, or loaded the extension with F5, because no VS Code was available. So `extension.js` has not run inside VS Code, and no Copilot session has called the relay, the tools or the participant.

## Tests

    HEADWATER_BIN=engine/target/dev-release/headwater node --test integrations/vscode/test/client.test.js integrations/vscode/test/relay.test.js

Run it from the repository root. `client.js` makes every decision, and the suite holds it, including the text of the two tools and the lines of the participant. One case runs against the real engine that `HEADWATER_BIN` names. It skips on a workstation when the variable is unset, and it fails in CI when the variable is unset. The other cases replay sessions that the real server recorded, under `test/fixtures/`. The relay suite starts `relay.js` over the same recorded sessions and counts the servers that each request started. `extension.js` has no tests, because the `vscode` module does not load under `node --test`. It only shows what `client.js` returns.
