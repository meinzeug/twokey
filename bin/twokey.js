#!/usr/bin/env node

import { spawn } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import readline from "node:readline";

const VERSION = process.env.npm_package_version || "1.0.5";
const DEFAULT_MODEL = process.env.TWOKEY_OLLAMA_MODEL || "qwen2.5:3b";
const DEFAULT_OLLAMA_URL = process.env.TWOKEY_OLLAMA_URL || "http://127.0.0.1:11434";
const LATEST_RELEASE_API = "https://api.github.com/repos/meinzeug/twokey/releases/latest";
const APPIMAGE_DIR = path.join(os.homedir(), ".local", "share", "twokey", "bin");
const APPIMAGE_PATH = path.join(APPIMAGE_DIR, "twokey-ai.AppImage");
const APPIMAGE_META_PATH = path.join(APPIMAGE_DIR, "twokey-ai.meta.json");

const args = process.argv.slice(2);
const QUIET = args.includes("--quiet");

if (args.includes("--help") || args.includes("-h")) {
  printHelp();
  process.exit(0);
}

if (args.includes("--version") || args.includes("-v")) {
  console.log(VERSION);
  process.exit(0);
}

if (args.includes("--desktop")) {
  launchDesktopApp().then(async (startedCommand) => {
    if (startedCommand) {
      if (args.includes("--enable-autostart")) {
        try {
          await ensureUserService(startedCommand);
        } catch (error) {
          if (!QUIET) {
            const message = error instanceof Error ? error.message : String(error);
            console.warn(`Autostart setup skipped: ${message}`);
          }
        }
      }
      if (!QUIET) {
        console.log("TwoKey desktop app started in background.");
      }
      process.exit(0);
    }
    if (!QUIET) {
      console.error("No native desktop binary found in PATH.");
      console.error("Install the .deb/.AppImage release and ensure 'twokey-ai' is available in PATH.");
    }
    process.exit(1);
  });
}

const onceIndex = args.findIndex((value) => value === "--once");
if (onceIndex >= 0) {
  const prompt = args.slice(onceIndex + 1).join(" ").trim();
  if (!prompt) {
    console.error("Missing prompt after --once");
    process.exit(1);
  }

  runSinglePrompt(prompt).catch((error) => {
    console.error(error.message || String(error));
    process.exit(1);
  });
} else if (args.includes("--cli")) {
  startRepl().catch((error) => {
    console.error(error.message || String(error));
    process.exit(1);
  });
} else {
  launchDesktopApp().then(async (startedCommand) => {
    if (startedCommand) {
      if (args.includes("--enable-autostart")) {
        try {
          await ensureUserService(startedCommand);
        } catch (error) {
          if (!QUIET) {
            const message = error instanceof Error ? error.message : String(error);
            console.warn(`Autostart setup skipped: ${message}`);
          }
        }
      }
      if (!QUIET) {
        console.log("TwoKey desktop app started in background.");
      }
      process.exit(0);
    }

    if (!QUIET) {
      console.error("Could not start desktop app.");
      console.error("Tried system binaries and auto-download from GitHub Releases.");
      console.error("Use 'twokey --cli' to run terminal mode.");
    }
    process.exit(1);
  });
}

async function runSinglePrompt(prompt) {
  const answer = await askOllama(prompt);
  console.log(answer);
}

async function startRepl() {
  console.log("TwoKey CLI");
  console.log("Type your prompt. Commands: /help, /exit");
  console.log(`Ollama endpoint: ${DEFAULT_OLLAMA_URL}`);
  console.log(`Model: ${DEFAULT_MODEL}`);

  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
    prompt: "twokey> ",
  });

  rl.prompt();

  rl.on("line", async (line) => {
    const input = line.trim();
    if (!input) {
      rl.prompt();
      return;
    }

    if (input === "/exit" || input === "/quit") {
      rl.close();
      return;
    }

    if (input === "/help") {
      printHelp();
      rl.prompt();
      return;
    }

    try {
      const answer = await askOllama(input);
      console.log(`\n${answer}\n`);
    } catch (error) {
      console.error(error.message || String(error));
    }

    rl.prompt();
  });

  rl.on("close", () => {
    console.log("bye");
    process.exit(0);
  });
}

async function askOllama(prompt) {
  const response = await fetch(`${DEFAULT_OLLAMA_URL.replace(/\/$/, "")}/api/chat`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      model: DEFAULT_MODEL,
      stream: false,
      messages: [
        {
          role: "system",
          content: "You are TwoKey, a concise Linux assistant.",
        },
        {
          role: "user",
          content: prompt,
        },
      ],
      options: {
        temperature: 0.3,
        num_predict: 384,
      },
    }),
  });

  if (!response.ok) {
    throw new Error(`Ollama request failed with HTTP ${response.status}`);
  }

  const payload = await response.json();
  const content = payload?.message?.content?.trim();
  if (!content) {
    throw new Error("Ollama returned an empty response");
  }

  return content;
}

async function launchDesktopApp() {
  let appImageReady = false;
  try {
    appImageReady = await ensureLocalAppImage();
  } catch {
    appImageReady = false;
  }

  const candidates = [];
  if (process.env.TWOKEY_DESKTOP_CMD) {
    candidates.push(process.env.TWOKEY_DESKTOP_CMD);
  }
  if (appImageReady) {
    candidates.push(APPIMAGE_PATH);
  }
  candidates.push("twokey-ai", "twokey-desktop");

  for (const command of candidates) {
    const started = await spawnDetached(command);
    if (started) {
      return command;
    }
  }

  return null;
}

async function ensureLocalAppImage() {
  await fs.promises.mkdir(APPIMAGE_DIR, { recursive: true });

  const latestAsset = await resolveLatestAppImageAsset();
  if (!latestAsset) {
    return hasExecutable(APPIMAGE_PATH);
  }

  if (await isCurrentAppImage(latestAsset)) {
    return true;
  }

  const response = await fetch(latestAsset.url, {
    headers: {
      "User-Agent": "twokey-cli",
      Accept: "application/octet-stream",
    },
  });

  if (!response.ok) {
    return false;
  }

  const data = Buffer.from(await response.arrayBuffer());
  await fs.promises.writeFile(APPIMAGE_PATH, data, { mode: 0o755 });

  await fs.promises.chmod(APPIMAGE_PATH, 0o755);
  const meta = {
    releaseTag: latestAsset.releaseTag,
    assetName: latestAsset.name,
    assetId: latestAsset.id,
    assetSize: latestAsset.size,
    assetUpdatedAt: latestAsset.updatedAt,
    downloadedAt: new Date().toISOString(),
  };
  await fs.promises.writeFile(APPIMAGE_META_PATH, `${JSON.stringify(meta, null, 2)}\n`, "utf8");
  return true;
}

async function resolveLatestAppImageAsset() {
  const response = await fetch(LATEST_RELEASE_API, {
    headers: {
      "User-Agent": "twokey-cli",
      Accept: "application/vnd.github+json",
    },
  });

  if (!response.ok) {
    return null;
  }

  const payload = await response.json();
  const assets = Array.isArray(payload.assets) ? payload.assets : [];
  const appImage = assets.find(
    (asset) => typeof asset?.name === "string" && asset.name.endsWith(".AppImage") && asset.name.includes("amd64"),
  ) || assets.find((asset) => typeof asset?.name === "string" && asset.name.endsWith(".AppImage"));

  if (!appImage?.browser_download_url) {
    return null;
  }

  return {
    releaseTag: typeof payload?.tag_name === "string" ? payload.tag_name : "unknown",
    id: Number.isFinite(appImage.id) ? appImage.id : 0,
    name: appImage.name,
    size: Number.isFinite(appImage.size) ? appImage.size : 0,
    updatedAt: typeof appImage.updated_at === "string" ? appImage.updated_at : "",
    url: appImage.browser_download_url,
  };
}

async function isCurrentAppImage(latestAsset) {
  if (!(await hasExecutable(APPIMAGE_PATH))) {
    return false;
  }

  const [meta, stats] = await Promise.all([readAppImageMeta(), fs.promises.stat(APPIMAGE_PATH)]);
  if (!meta) {
    return false;
  }

  return (
    meta.releaseTag === latestAsset.releaseTag
    && meta.assetId === latestAsset.id
    && meta.assetUpdatedAt === latestAsset.updatedAt
    && Number(meta.assetSize) === Number(stats.size)
  );
}

async function readAppImageMeta() {
  try {
    const content = await fs.promises.readFile(APPIMAGE_META_PATH, "utf8");
    return JSON.parse(content);
  } catch {
    return null;
  }
}

async function hasExecutable(filePath) {
  try {
    await fs.promises.access(filePath, fs.constants.X_OK);
    return true;
  } catch {
    return false;
  }
}

function spawnDetached(command) {
  return new Promise((resolve) => {
    const child = spawn(command, [], {
      detached: true,
      stdio: "ignore",
      shell: false,
    });

    child.once("spawn", () => {
      child.unref();
      resolve(true);
    });

    child.once("error", () => {
      resolve(false);
    });
  });
}

function printHelp() {
  console.log("twokey <command/options>");
  console.log("");
  console.log("Options:");
  console.log("  --help, -h       Show help");
  console.log("  --version, -v    Show version");
  console.log("  --cli            Start interactive terminal mode");
  console.log("  --once <prompt>  Send one prompt to Ollama and print response");
  console.log("  --desktop        Start native desktop app in background");
  console.log("");
  console.log("Without options, twokey starts the native desktop app in background.");
  console.log("If no desktop binary is installed, twokey tries to download an AppImage from latest GitHub release.");
}

async function ensureUserService(command) {
  const configHome = process.env.XDG_CONFIG_HOME || path.join(os.homedir(), ".config");
  const systemdDir = path.join(configHome, "systemd", "user");
  const servicePath = path.join(systemdDir, "twokey.service");
  await fs.promises.mkdir(systemdDir, { recursive: true });

  const content = [
    "[Unit]",
    "Description=TwoKey Desktop Assistant",
    "After=graphical-session.target",
    "",
    "[Service]",
    `ExecStart=/bin/sh -lc ${shellEscape(command)}`,
    "Restart=on-failure",
    "RestartSec=3",
    "",
    "[Install]",
    "WantedBy=default.target",
    "",
  ].join("\n");

  await fs.promises.writeFile(servicePath, content, "utf8");

  await runSystemctlUser(["daemon-reload"]);
  await runSystemctlUser(["enable", "--now", "twokey.service"]);

  if (!QUIET) {
    console.log("TwoKey systemd user service enabled: twokey.service");
  }
}

async function runSystemctlUser(argsList) {
  return new Promise((resolve, reject) => {
    const child = spawn("systemctl", ["--user", ...argsList], {
      stdio: QUIET ? "ignore" : "pipe",
      shell: false,
    });

    let stderr = "";
    if (child.stderr) {
      child.stderr.on("data", (chunk) => {
        stderr += String(chunk);
      });
    }

    child.on("error", (error) => reject(error));
    child.on("close", (code) => {
      if (code === 0) {
        resolve();
        return;
      }
      reject(new Error(stderr.trim() || `systemctl --user failed with code ${code}`));
    });
  });
}

function shellEscape(value) {
  return `'${String(value).replace(/'/g, `'\\''`)}'`;
}
