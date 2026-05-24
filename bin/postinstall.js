#!/usr/bin/env node

import { spawn, spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

if (process.env.CI === "true" || process.env.TWOKEY_SKIP_POSTINSTALL === "1") {
  process.exit(0);
}

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const cliPath = path.join(__dirname, "twokey.js");

const isRoot = typeof process.getuid === "function" && process.getuid() === 0;
const sudoUser = process.env.SUDO_USER;

if (isRoot && sudoUser && sudoUser !== "root") {
  const uid = resolveUid(sudoUser);
  const homeDir = resolveHomeDir(sudoUser);
  const env = { ...process.env };

  if (homeDir) {
    env.HOME = homeDir;
  }
  if (uid) {
    env.XDG_RUNTIME_DIR = `/run/user/${uid}`;
    env.DBUS_SESSION_BUS_ADDRESS = `unix:path=${env.XDG_RUNTIME_DIR}/bus`;
  }

  const runtimePrep = spawn(
    "sudo",
    ["-u", sudoUser, "-H", process.execPath, cliPath, "--prepare-runtime-only", "--quiet"],
    {
      stdio: "ignore",
      shell: false,
      env,
    },
  );
  runtimePrep.on("error", () => undefined);

  if (isDesktopRunningForUser(sudoUser)) {
    process.exit(0);
  }

  const delegated = spawn(
    "sudo",
    ["-u", sudoUser, "-H", process.execPath, cliPath, "--prepare-runtime", "--desktop", "--enable-autostart"],
    {
      stdio: "ignore",
      shell: false,
      env,
    },
  );

  delegated.on("error", () => process.exit(0));
  delegated.on("close", () => process.exit(0));
} else {
  if (isRoot) {
    ensureSystemTtsBackend();
  }

  const runtimePrep = spawn(process.execPath, [cliPath, "--prepare-runtime-only", "--quiet"], {
    stdio: "ignore",
    shell: false,
  });

  runtimePrep.on("error", () => undefined);

  if (isDesktopRunningForUser(process.env.USER || "")) {
    process.exit(0);
  }

  const child = spawn(process.execPath, [cliPath, "--prepare-runtime", "--desktop", "--enable-autostart"], {
    stdio: "ignore",
    shell: false,
  });

  child.on("error", () => process.exit(0));
  child.on("close", () => process.exit(0));
}

function resolveUid(username) {
  const out = spawnSync("id", ["-u", username], { encoding: "utf8" });
  if (out.status !== 0) {
    return "";
  }
  return String(out.stdout || "").trim();
}

function resolveHomeDir(username) {
  const out = spawnSync("getent", ["passwd", username], { encoding: "utf8" });
  if (out.status !== 0) {
    return "";
  }

  const entry = String(out.stdout || "").trim();
  const parts = entry.split(":");
  return parts[5] || "";
}

function isDesktopRunningForUser(username) {
  if (!username) {
    return false;
  }

  const out = spawnSync("pgrep", ["-u", username, "-f", "twokey-ai(\\.AppImage)?"], { encoding: "utf8" });
  return out.status === 0;
}

function ensureSystemTtsBackend() {
  if (hasAnyTtsBackend()) {
    return;
  }

  if (!commandExists("apt-get")) {
    return;
  }

  try {
    const result = spawnSync("apt-get", ["install", "-y", "espeak-ng"], {
      stdio: "ignore",
      shell: false,
    });
    if (result.status !== 0) {
      return;
    }
  } catch {
    // best effort
  }
}

function hasAnyTtsBackend() {
  return commandExists("spd-say") || commandExists("espeak-ng") || commandExists("espeak");
}

function commandExists(name) {
  const out = spawnSync("sh", ["-c", `command -v ${name} >/dev/null 2>&1`], { encoding: "utf8" });
  return out.status === 0;
}
