"use strict";

/** @param {*} vscode minimal shim from the Node host */
function activate(vscode) {
  vscode.commands.registerCommand("unibrowser.helloSample.sayHello", () => {
    vscode.window.showInformationMessage(
      "Hello from the Unibrowser sample VSIX extension (Node host spike).",
    );
  });
}

module.exports = { activate };
