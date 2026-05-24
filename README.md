# TwoKey Linux AI Assistant

TwoKey is a Linux-first desktop assistant prototype. The goal is a small always-available overlay that will later let users hold two keys, speak, and have the result processed by local or online AI providers without switching to a browser or chat window.

Phase 1 implements the foundation only:

- Tauri + React project structure
- Minimal dark overlay pill
- Dummy modes: Conversation, Edit Text, Dictation, Feedback
- Clickable mode menu
- Placeholder settings window
- Initial architecture and product documentation
- X11 Phase 2 prototype: hold `Ctrl+Space` to record audio, double tap `Ctrl+Space` to cycle modes
- Phase 3 prototype: mock STT and external STT command adapter
- Phase 4 prototype: local Ollama conversation mode with `qwen2.5:3b`

No TTS or text injection is implemented yet.

## Installation

```bash
npm install twokey
```

Start the tool directly:

```bash
twokey
```

Default behavior: start the native desktop app in background.
If no desktop binary is installed yet, `twokey` attempts to download an AppImage from the latest GitHub release into `~/.local/share/twokey/bin/` and starts it.

Useful CLI options:

```bash
twokey --help
twokey --cli
twokey --once "Erklaere kurz den Unterschied zwischen X11 und Wayland"
twokey --desktop
```

## Minimal Usage

```ts
import { getPackageInfo } from "twokey";

const info = getPackageInfo();
console.log(info.name);
console.log(info.runtimeStatus.waylandGlobalHotkeys);
```

The package exports runtime status metadata. Current status includes planned/limited areas such as Wayland global hotkeys, TTS, tray menu, SQLite history/audit, and online provider execution until secure API-key storage is implemented.

## Requirements

- Linux desktop
- Node.js 20+
- npm
- Rust toolchain with `cargo` for running the Tauri app
- Tauri Linux system dependencies, for example on Ubuntu/Debian:

```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

This workspace currently has Node/npm available. `cargo` is required before the native app can be started.

## Start Development

Install JavaScript dependencies:

```bash
npm install
```

Run only the web UI:

```bash
npm run dev
```

Run the Linux desktop app:

```bash
npm run tauri:dev
```

Phase 2 hotkey behavior on X11:

- Hold `Ctrl+Space`: start recording after a short hold delay.
- Release `Ctrl+Space`: stop recording and save a WAV file under `~/.cache/twokey-ai/recordings/`.
- Double tap `Ctrl+Space`: cycle to the next mode.
- Press `Escape`: cancel an active recording.

On Wayland, generic global hold-hotkeys are reported as unavailable instead of failing silently.

Phase 3 STT behavior:

- Default STT provider is a deterministic mock transcriber.
- To test a real local or custom STT command, set `TWOKEY_STT_COMMAND`.
- The command must print the transcript to stdout and include `{audio}` as placeholder.

Example:

```bash
TWOKEY_STT_COMMAND='whisper-cli -f {audio} --language de --no-timestamps' npm run tauri:dev
```

Phase 4 Ollama behavior:

- Ollama runs as a systemd service on `127.0.0.1:11434`.
- Default model: `qwen2.5:3b`.
- Conversation mode sends finished transcripts to Ollama and displays the answer in the overlay.
- Override model or endpoint with:

```bash
TWOKEY_OLLAMA_MODEL='llama3.2:3b' TWOKEY_OLLAMA_URL='http://127.0.0.1:11434' npm run tauri:dev
```

Phase 5 dictation behavior:

- In dictation mode, a finished transcript is pasted at the active cursor position.
- X11 uses `xclip` or `xsel` plus `xdotool`.
- Clipboard content is read before insertion and restored afterward when possible.
- Wayland insertion is deliberately reported as unsupported for now.

Phase 6 text editing behavior:

- In edit mode, TwoKey reads the current X11 selection with `Ctrl+C`.
- The spoken instruction and selected text are sent to Ollama.
- The overlay shows a replacement preview.
- The selected text is replaced only after pressing `Ersetzen`.

Phase 7 settings behavior:

- Settings are persisted at `~/.config/twokey-ai/settings.json`.
- The settings window saves general, hotkey, STT, Ollama, privacy, and update-channel values.
- Some settings are stored before they are fully applied at runtime; later phases will wire them into the helper/provider layers.

Phase 8 provider behavior:

- Ollama chat is routed through a provider abstraction.
- Provider metadata is exposed to the settings UI.
- OpenAI-compatible and OpenRouter-compatible providers are visible as planned online providers, but disabled until API-key storage and routing are implemented.

Phase 9 file context behavior:

- File context can be added from the overlay menu.
- `txt`, `md`, `markdown`, `json`, and `csv` files are read directly.
- PDFs are extracted with `pdftotext` from `poppler-utils`.
- Images are registered as context metadata for future vision providers.
- Extracted text is cached under `~/.cache/twokey-ai/file-contexts/`.

Phase 10 packaging behavior:

- `npm run tauri:build` creates AppImage and `.deb` bundles.
- Settings autostart writes `~/.config/autostart/twokey-ai.desktop`.
- Generated bundles live under `src-tauri/target/release/bundle/`.

Phase 11 update behavior:

- Settings can check GitHub Releases for a newer version.
- The app only reports availability; it does not auto-download or auto-install.

Current stabilization status:

- AppImage and `.deb` builds complete successfully.
- Ollama runs locally as a systemd service with `qwen2.5:3b`.
- Production dependency audit reports no vulnerabilities.
- Wayland limitations, TTS, tray menu, SQLite history, and online provider execution remain future hardening work.

Build the frontend:

```bash
npm run build
```

Build the desktop bundle:

```bash
npm run tauri:build
```

## Project Layout

```text
docs/                 Product, architecture, security, and status docs
src/                  React frontend
src-tauri/            Rust/Tauri desktop shell
```

## Repository

GitHub: https://github.com/meinzeug/twokey

## License

MIT. See `LICENSE`.

## Linux Notes

TwoKey is designed around Linux conventions:

- Config: `~/.config/twokey-ai/`
- Data: `~/.local/share/twokey-ai/`
- Cache: `~/.cache/twokey-ai/`
- Logs/state: `~/.local/state/twokey-ai/`

X11 and Wayland will be handled separately in later phases. Phase 1 only detects and displays the current session type.
