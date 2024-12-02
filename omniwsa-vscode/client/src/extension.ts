import * as path from "path";
import { ExtensionContext } from "vscode";

import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  const serverOptions: ServerOptions = {
    run: {
      command: context.asAbsolutePath(path.join("..", "target", "release", "omniwsa-ls")),
      transport: TransportKind.stdio
    },
    debug: {
      command: context.asAbsolutePath(path.join("..", "target", "debug", "omniwsa-ls")),
      transport: TransportKind.stdio
    },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "whitespace-assembly" }],
  };

  client = new LanguageClient(
    "omniwsa",
    "Whitespace assembly",
    serverOptions,
    clientOptions,
  );
  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
