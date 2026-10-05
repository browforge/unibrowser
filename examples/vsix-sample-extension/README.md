# Sample VSIX extension (Unibrowser spike)

Minimal extension used by the Node extension-host spike and `unibrowser-extension-host` tests.

## Command

- **VSIX sample: Say hello** (`unibrowser.helloSample.sayHello`) — shows an information toast via the `vscode.window.showInformationMessage` shim.

## Pack and install

```bash
./pack-vsix.sh
```

In Unibrowser: Mod portal → install the generated `.vsix`, enable **Node extension host** under Settings → Advanced, then run the command from the palette (Extensions category).
