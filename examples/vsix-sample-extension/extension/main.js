"use strict";

/** @param {*} vscode minimal shim from the Node host */
function activate(vscode) {
  vscode.commands.registerCommand("unibrowser.helloSample.sayHello", () => {
    vscode.window.showInformationMessage(
      "Hello from the Unibrowser sample VSIX extension (Node host spike).",
    );
  });

  vscode.commands.registerCommand("unibrowser.helloSample.probeEditor", () => {
    const editor = vscode.window.activeTextEditor;
    const lang = editor ? editor.document.languageId : "none";
    const previewChars =
      vscode.workspace.getConfiguration("helloSample").get("previewChars", 40) ?? 40;
    const snippet = editor
      ? editor.document.getText().slice(0, Number(previewChars))
      : "(no active editor)";
    vscode.commands.executeCommand("unibrowser.font.size.increase");
    vscode.window.showInformationMessage(
      `probe: lang=${lang} snippet=${snippet}`,
    );
  });

  vscode.commands.registerCommand("unibrowser.helloSample.formatViaEdit", () => {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showWarningMessage("No active editor");
      return;
    }
    editor.edit((builder) => {
      builder.insert(new vscode.Position(0, 0), "FORMATTED ");
    });
  });

  vscode.languages.registerDocumentFormattingEditProvider({ language: "markdown" }, {
    provideDocumentFormattingEdits(document) {
      const text = document.getText();
      if (text.endsWith("\n")) {
        return [];
      }
      const lines = text.split("\n");
      const lastLine = lines.length - 1;
      const col = lines[lastLine].length;
      return [
        vscode.TextEdit.insert(new vscode.Position(lastLine, col), "\n"),
      ];
    },
  });
}

module.exports = { activate };
