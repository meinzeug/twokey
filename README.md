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

No STT, TTS, text injection, or AI provider calls are implemented yet.

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
