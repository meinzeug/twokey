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

## Linux Notes

TwoKey is designed around Linux conventions:

- Config: `~/.config/twokey-ai/`
- Data: `~/.local/share/twokey-ai/`
- Cache: `~/.cache/twokey-ai/`
- Logs/state: `~/.local/state/twokey-ai/`

X11 and Wayland will be handled separately in later phases. Phase 1 only detects and displays the current session type.
