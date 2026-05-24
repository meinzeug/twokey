#!/usr/bin/env node

import { spawn } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

if (process.env.CI === "true" || process.env.TWOKEY_SKIP_POSTINSTALL === "1") {
  process.exit(0);
}

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const cliPath = path.join(__dirname, "twokey.js");

const child = spawn(process.execPath, [cliPath, "--desktop", "--enable-autostart", "--quiet"], {
  stdio: "ignore",
  shell: false,
});

child.on("error", () => process.exit(0));
child.on("close", () => process.exit(0));
