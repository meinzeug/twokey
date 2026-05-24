#!/usr/bin/env node

import { spawn } from "node:child_process";
import readline from "node:readline";

const VERSION = "1.0.1";
const DEFAULT_MODEL = process.env.TWOKEY_OLLAMA_MODEL || "qwen2.5:3b";
const DEFAULT_OLLAMA_URL = process.env.TWOKEY_OLLAMA_URL || "http://127.0.0.1:11434";

const args = process.argv.slice(2);

if (args.includes("--help") || args.includes("-h")) {
  printHelp();
  process.exit(0);
}

if (args.includes("--version") || args.includes("-v")) {
  console.log(VERSION);
  process.exit(0);
}

if (args.includes("--desktop")) {
  launchDesktopApp();
  process.exit(0);
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
} else {
  startRepl().catch((error) => {
    console.error(error.message || String(error));
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

function launchDesktopApp() {
  const candidates = ["twokey-ai", "twokey"];
  for (const command of candidates) {
    const child = spawn(command, { stdio: "inherit" });
    child.on("error", () => undefined);
    child.on("spawn", () => {
      process.exit(0);
    });
  }

  console.error("No native desktop binary found in PATH.");
  console.error("Install the .deb/.AppImage release or run plain 'twokey' for CLI mode.");
}

function printHelp() {
  console.log("twokey <command/options>");
  console.log("");
  console.log("Options:");
  console.log("  --help, -h       Show help");
  console.log("  --version, -v    Show version");
  console.log("  --once <prompt>  Send one prompt to Ollama and print response");
  console.log("  --desktop        Start native desktop app if installed in PATH");
  console.log("");
  console.log("Without options, twokey starts interactive CLI mode.");
}
