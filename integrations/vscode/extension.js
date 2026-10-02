// SPDX-License-Identifier: Apache-2.0
//
// Glue between VS Code and `client.js`. Every decision about what to ask the
// server and what counts as an answer is in `client.js`, where `node --test`
// holds it. This file only shows what the client returns: the pointers, the
// line `client.withheldNote` gives when the route budget held pointers back, and
// the engine's silence when a route the user asked for found nothing. It shows
// nothing when the client returns none of these.
//
// It also hands the same answers to Copilot Chat (#1583), in three ways: an MCP
// server definition that runs `relay.js`, two language model tools, and the
// `@headwater` chat participant. What each of them says is decided in
// `client.js`. This file only places it.

'use strict';

const fs = require('node:fs');
const path = require('node:path');
const vscode = require('vscode');
const client = require('./client.js');

function options(folder) {
  const bin = vscode.workspace.getConfiguration('headwater').get('path') || 'headwater';
  return { root: folder.uri.fsPath, bin };
}

// The workspace folder and the path inside it, or null for a document that is
// not a file under an open folder.
function locate(document) {
  if (!document || document.uri.scheme !== 'file') return null;
  const folder = vscode.workspace.getWorkspaceFolder(document.uri);
  if (!folder) return null;
  const relative = path.relative(folder.uri.fsPath, document.uri.fsPath).split(path.sep).join('/');
  if (relative === '' || relative.startsWith('..')) return null;
  return { folder, relative };
}

// The workspace folders that hold a `.headwater/` directory, in order.
function headwaterFolders() {
  return (vscode.workspace.workspaceFolders || []).filter((folder) =>
    fs.existsSync(path.join(folder.uri.fsPath, '.headwater')),
  );
}

// A file named by a tool's input, relative to the first Headwater folder or
// absolute, as the folder and the path inside it, or null.
function locateInput(input) {
  const folders = headwaterFolders();
  if (folders.length === 0 || typeof input !== 'string' || input === '') return null;
  const uri = path.isAbsolute(input) ? vscode.Uri.file(input) : vscode.Uri.joinPath(folders[0].uri, input);
  return locate({ uri });
}

function describe(pointer) {
  return pointer.name ? `${pointer.path} (${pointer.name})` : pointer.path;
}

function activate(context) {
  const status = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
  status.command = 'headwater.governing';
  context.subscriptions.push(status);
  let shown = [];
  let shownFolder = null;
  let generation = 0;

  async function pick(pointers, folder, placeHolder) {
    const items = pointers.map((p) => ({
      label: p.name || p.path,
      description: p.path,
      detail: [p.summary, p.asserted ? `asserted: ${p.asserted}` : null].filter(Boolean).join(' '),
      pointer: p,
    }));
    const chosen = await vscode.window.showQuickPick(items, { placeHolder, matchOnDescription: true });
    if (chosen) {
      const uri = vscode.Uri.joinPath(folder.uri, chosen.pointer.path);
      await vscode.window.showTextDocument(uri);
    }
  }

  // Read time: the documents that govern the file in the active editor.
  async function refresh(editor) {
    const mine = ++generation;
    const place = locate(editor && editor.document);
    if (!place) {
      status.hide();
      return;
    }
    const pointers = await client.governing(place.relative, options(place.folder));
    if (mine !== generation) return;
    shown = pointers;
    shownFolder = place.folder;
    if (pointers.length === 0) {
      status.hide();
      return;
    }
    status.text = `$(book) ${pointers.length === 1 ? describe(pointers[0]) : `${pointers.length} documents govern this file`}`;
    status.tooltip = pointers.map((p) => `${describe(p)}${p.summary ? ` — ${p.summary}` : ''}`).join('\n');
    status.show();
  }

  context.subscriptions.push(
    // Intent time: route a task the user types.
    vscode.commands.registerCommand('headwater.route', async () => {
      const folder = vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders[0];
      if (!folder) return;
      const task = await vscode.window.showInputBox({ prompt: 'What are you about to do?' });
      if (!task) return;
      const answered = await client.route(task, options(folder));
      const note = client.withheldNote(answered);
      if (answered.pointers.length === 0) {
        const said = [answered.silence, note].filter(Boolean).join('; ');
        if (said) vscode.window.showInformationMessage(`Headwater: no document shown; ${said}`);
        return;
      }
      await pick(answered.pointers, folder, `Documents that govern this task${note ? ` (${note})` : ''}`);
    }),
    vscode.commands.registerCommand('headwater.governing', async () => {
      if (shown.length === 0 || !shownFolder) return;
      await pick(shown, shownFolder, 'Documents that govern this file');
    }),
    vscode.window.onDidChangeActiveTextEditor(refresh),
    // Write time: after the save, never before it, so the save waits on nothing.
    vscode.workspace.onDidSaveTextDocument(async (document) => {
      const place = locate(document);
      if (!place) return;
      const pointers = await client.governing(place.relative, options(place.folder));
      if (pointers.length === 0) return;
      vscode.window.showInformationMessage(
        `Headwater: ${place.relative} is governed by ${pointers.map(describe).join(', ')}.`,
      );
    }),
  );

  refresh(vscode.window.activeTextEditor);
  registerCopilot(context);
}

// The Copilot Chat surfaces: an MCP server for each Headwater folder, two
// language model tools and the `@headwater` participant.
function registerCopilot(context) {
  // The provider registers nothing when it finds no engine, and says so only
  // here: View > Output > Headwater. A fail-open extension shows no pop-up,
  // and without this line a person cannot tell why no server is listed.
  const log = vscode.window.createOutputChannel('Headwater', { log: true });
  const changed = new vscode.EventEmitter();
  context.subscriptions.push(
    log,
    changed,
    vscode.lm.registerMcpServerDefinitionProvider('headwater', {
      onDidChangeMcpServerDefinitions: changed.event,
      provideMcpServerDefinitions() {
        const configured = vscode.workspace.getConfiguration('headwater').get('path') || 'headwater';
        const bin = client.resolveBin(configured);
        const folders = headwaterFolders();
        if (!bin) {
          log.warn(`MCP: no server registered, because \`${configured}\` names no binary on PATH or on disk. Set headwater.path to the engine.`);
          return [];
        }
        if (folders.length === 0) {
          log.info('MCP: no server registered, because no workspace folder holds `.headwater/`.');
          return [];
        }
        log.info(`MCP: registering ${folders.map((f) => f.uri.fsPath).join(', ')} with engine ${bin}.`);
        const relay = path.join(context.extensionPath, 'relay.js');
        return folders.map((folder) => {
          const server = new vscode.McpStdioServerDefinition(
            `Headwater (${folder.name})`,
            process.execPath,
            [relay, '--bin', bin, '--root', folder.uri.fsPath],
            { ELECTRON_RUN_AS_NODE: '1' },
            context.extension.packageJSON.version,
          );
          server.cwd = folder.uri;
          return server;
        });
      },
    }),
    vscode.workspace.onDidChangeWorkspaceFolders(() => changed.fire()),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration('headwater.path')) changed.fire();
    }),
    vscode.lm.registerTool('headwater_route', {
      prepareInvocation: () => ({ invocationMessage: 'Asking Headwater which documents govern the task' }),
      async invoke(call) {
        const folder = headwaterFolders()[0];
        const task = String((call.input && call.input.task) || '');
        const answered = folder ? await client.route(task, options(folder)) : { pointers: [], withheld: 0 };
        return textResult(client.toolText(answered, 'this task'));
      },
    }),
    vscode.lm.registerTool('headwater_governing', {
      prepareInvocation: (call) => ({
        invocationMessage: `Asking Headwater which documents govern ${(call.input && call.input.path) || 'the file'}`,
      }),
      async invoke(call) {
        const place = locateInput(call.input && call.input.path);
        if (!place) return textResult('That path is not a file in a Headwater workspace folder.');
        const pointers = await client.governing(place.relative, options(place.folder));
        return textResult(client.toolText(pointers, place.relative));
      },
    }),
    vscode.chat.createChatParticipant('headwater.chat', async (request, _chat, stream) => {
      const folder = headwaterFolders()[0];
      if (!folder) {
        stream.markdown('This workspace has no folder with a `.headwater/` directory, so Headwater has no corpus to read.');
        return;
      }
      if (request.command === 'file') {
        const place = locate({ uri: referencedUri(request) || activeUri() });
        if (!place) {
          stream.markdown('Open a file in this workspace, or attach one, and ask again.');
          return;
        }
        const pointers = await client.governing(place.relative, options(place.folder));
        render(stream, place.folder, client.present(pointers, `\`${place.relative}\``));
        return;
      }
      const answered = await client.route(request.prompt, options(folder));
      render(stream, folder, client.present(answered, 'this task'));
    }),
  );
}

function textResult(text) {
  return new vscode.LanguageModelToolResult([new vscode.LanguageModelTextPart(text)]);
}

function activeUri() {
  const editor = vscode.window.activeTextEditor;
  return editor ? editor.document.uri : undefined;
}

// The first file a chat message references with `#file` or an attachment.
function referencedUri(request) {
  for (const reference of request.references || []) {
    const value = reference.value;
    if (value instanceof vscode.Uri) return value;
    if (value instanceof vscode.Location) return value.uri;
  }
  return undefined;
}

// `client.present` as chat output: a link to each document, then its summary
// as plain text, so no character of a summary is read as Markdown.
function render(stream, folder, presented) {
  stream.markdown(new vscode.MarkdownString().appendText(presented.lead));
  for (const entry of presented.entries) {
    stream.markdown('\n\n- ');
    stream.anchor(vscode.Uri.joinPath(folder.uri, entry.path), entry.label);
    if (entry.detail) stream.markdown(new vscode.MarkdownString().appendText(` — ${entry.detail}`));
  }
  if (presented.tail) {
    stream.markdown('\n\n');
    stream.markdown(new vscode.MarkdownString().appendText(presented.tail));
  }
}

function deactivate() {}

module.exports = { activate, deactivate };
