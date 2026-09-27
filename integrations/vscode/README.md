# Headwater for VS Code

This extension shows a person who edits a Headwater repository in plain VS Code the documents that govern their work. It does this at three moments:

- **Intent.** The command *Headwater: What governs this task?* asks what you are about to do and lists the documents that govern it. Each entry shows a path, a name and a one-line summary, and never the content of the document. Select an entry to open the document.
- **Read.** When you open a file that a document governs, the status bar names that document. Click it to list every document that governs the file.
- **Write.** When you save a file that a document governs, a notification names the governing documents. The notification comes after the save, so the save never waits for it.

When nothing governs the task or the file, the extension shows nothing. It does not guess.

## How it gets its answers

The extension is a client of `headwater mcp`, the corpus MCP server that the engine ships. For each question it starts `headwater mcp --root <workspace folder>`, sends one request and stops the process. It does not look for a server that is already running, because the server reads standard input and walks the corpus once when it starts. A server kept alive across your edits would answer from an old walk. One session takes about 0.16 seconds on this repository.

It calls two tools of the query class, `route` and `governing_docs_for_path`, and no other tool. It never passes `--write`, so the server it starts registers no tool that writes. It reads each answer by the pointer shape that [the `headwater mcp` interface](../../docs/interfaces/headwater-mcp.md) states as the wire format a caller may rely on: `path (name) — summary [asserted: …]`. A line of any other shape is not a pointer, and the extension does not show it. A line with neither a name nor a summary is not a pointer either. The server repeats your own words in two places, and the extension never reads a pointer from them: the first line of a `route` answer repeats the task, and the answer for an ungoverned path repeats the path. So the extension recognizes the answer `no document governs <path>` by that prefix and shows nothing for it, whatever the path holds. Any other `governing_docs_for_path` answer counts only when every line of it is a pointer.

## When it does nothing

The extension degrades to nothing and says nothing in each of these cases:

- no `headwater` binary is on the path, or at the path the `headwater.path` setting names
- the workspace folder has no `.headwater/` directory
- the server exits with an error, answers with an error, answers text the extension cannot read, or takes more than five seconds

This is the same fails-open posture as every hook in this repository. A missing engine never blocks your editor.

## What it does not show

The write-time hook of the three agent harnesses also prints the paths in the governed scope that nothing governs, and the suspect edges of a document. `headwater mcp` does not serve those two accounts, so the save notification names the governing documents only.

## Install

Build or install the engine so that `headwater` is on your path, or set `headwater.path` to the binary. Then choose one of these:

- Open this folder in VS Code and press F5. This starts a second VS Code window with the extension loaded.
- Package it with `npx @vscode/vsce package` in this folder, and install the result with `code --install-extension headwater-vscode-0.1.0.vsix`.

The extension has no npm dependencies and no build step. It is licensed under Apache-2.0, and `LICENSE` in this folder is a copy of the repository's license. `.vscodeignore` keeps `test/` out of the package.

What was run and what was not, on 2026-09-27: `npx @vscode/vsce package` in this folder exited 0 with no warning and packaged five files (`LICENSE.txt`, `client.js`, `extension.js`, `package.json`, `readme.md`). Nobody has yet installed that package with `code --install-extension`, or loaded the extension with F5, because no VS Code was available. So `extension.js` has not run inside VS Code.

## Tests

    HEADWATER_BIN=engine/target/dev-release/headwater node --test integrations/vscode/test/client.test.js

Run it from the repository root. `client.js` makes every decision, and the suite holds it. One case runs against the real engine that `HEADWATER_BIN` names. It skips on a workstation when the variable is unset, and it fails in CI when the variable is unset. The other cases replay sessions that the real server recorded, under `test/fixtures/`. `extension.js` has no tests, because the `vscode` module does not load under `node --test`. It only shows what `client.js` returns.
