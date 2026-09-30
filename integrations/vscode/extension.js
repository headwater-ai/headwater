// SPDX-License-Identifier: Apache-2.0
//
// Glue between VS Code and `client.js`. Every decision about what to ask the
// server and what counts as an answer is in `client.js`, where `node --test`
// holds it. This file only shows what the client returns: the pointers, and the
// line `client.withheldNote` gives when the route budget held pointers back. It
// shows nothing when the client returns neither.

'use strict';

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
        if (note) vscode.window.showInformationMessage(`Headwater: no document shown; ${note}`);
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
}

function deactivate() {}

module.exports = { activate, deactivate };
