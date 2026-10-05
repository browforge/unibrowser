"use strict";

/** Synthetic Open VSX-style formatter extension (Apache-2.0, in-repo only). */
function activate(vscode) {
  vscode.languages.registerDocumentFormattingEditProvider(
    { language: "javascript" },
    {
      provideDocumentFormattingEdits(document) {
        const text = document.getText();
        if (!text.endsWith("\n")) {
          const lines = text.split("\n");
          const last = lines.length - 1;
          const col = lines[last].length;
          return [
            vscode.TextEdit.insert(new vscode.Position(last, col), "\n"),
          ];
        }
        return [];
      },
    },
  );

  vscode.commands.registerCommand("fixture.format", () => {
    vscode.commands.executeCommand("unibrowser.editor.formatDocument");
  });
}

module.exports = { activate };
