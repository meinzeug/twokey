# TwoKey Linux AI Assistant

TwoKey is a Linux-first desktop AI assistant with a small floating pill overlay.
The core idea matches the video workflow:

- hold a global hotkey, speak, release
- audio is recorded and transcribed
- transcript is sent to the selected AI provider
- result is used directly in your current desktop workflow
- optional TTS reads the answer out loud

## What Works Now

- Global hold hotkey on X11 for voice capture.
- Double-tap hotkey to cycle modes.
- Single-tap hotkey to open file-context picker.
- Voice-triggered toolchains for multi-step desktop actions.
- Four modes:
  - Conversation
  - Edit Text
  - Dictation
  - Feedback
- Conversation mode:
  - transcript -> provider answer
  - optional TTS playback
- Edit mode:
  - read selected text
  - apply spoken transform via AI
  - replace selected text directly
- Dictation mode:
  - insert transcript at cursor
- Feedback mode:
  - local feedback persistence in history DB
- File context:
  - TXT/MD/JSON/CSV/PDF content context
  - image context routing to vision-capable providers
- Provider routing:
  - local Ollama
  - OpenAI-compatible
  - OpenRouter-compatible
  - secure API key storage via local keyring
- Local audit/history persistence in SQLite.
- Tray icon and settings window.
- GitHub release check from settings.
- In-app AppImage update download and launch.

## Current Video Parity

Implemented from video behavior:

- Minimal floating pill UI.
- Hold-to-talk workflow.
- Mode switch via double tap.
- Text workflow without browser/chat-tab context switch.
- File attach + ask flow.
- Hybrid local/online providers.
- Optional TTS answer playback.

Still not fully equivalent to the video vision:

- Toolchains are implemented, but no visual workflow builder exists yet.
- Wayland still has compositor-specific limits for global hold hotkeys and full automation.
- Update install is available for AppImage, but no signed rollback-capable updater pipeline yet.

## Install

```bash
npm install twokey
```

Run:

```bash
twokey
```

Default behavior:

- starts native desktop app in background
- if no native binary is installed, tries to download latest AppImage release

Useful options:

```bash
twokey --help
twokey --cli
twokey --once "Erklaere X11 vs Wayland kurz"
twokey --desktop
```

## Development

Install dependencies:

```bash
npm install
```

Run desktop dev stack:

```bash
npm run tauri:dev
```

Important:

- `target/debug/twokey-ai` alone in dev mode will fail if Vite is not running.
- For development, use `npm run tauri:dev` so frontend + Tauri run together.

Build:

```bash
npm run build
cd src-tauri && cargo check
```

## Hotkey Behavior

Default hotkey can be changed in settings. Examples:

- `Ctrl+Space`
- `Ctrl+Super`
- mouse-button combinations except left/right mouse button in recorder UI

Runtime semantics on X11:

- hold hotkey: start recording
- release hotkey: stop recording -> transcribe -> mode action
- short single tap: open file picker
- short double tap: switch mode
- `Escape` while recording: cancel

Wayland fallback:

- if global hotkeys are blocked, manual start/stop recording is available from the menu
- this keeps the voice pipeline usable on restricted desktops

## STT Providers

Configured in settings (`sttProvider`):

- `mock`
- `local-whisper`
- `external-command`
- `openai-compatible`

`external-command` needs `TWOKEY_STT_COMMAND` with `{audio}` placeholder.

Example:

```bash
TWOKEY_STT_COMMAND='whisper-cli -f {audio} -l de -nt' npm run tauri:dev
```

## Linux Requirements

- Node.js 20+
- npm
- Rust + cargo
- Linux desktop dependencies for Tauri

Ubuntu/Debian example:

```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

Optional runtime tools:

- X11 automation: `xdotool`, `xclip` or `xsel`
- PDF extraction: `poppler-utils` (`pdftotext`)
- TTS backends: `spd-say` or `espeak-ng`/`espeak`

## Data Paths

- Config: `~/.config/twokey-ai/`
- Data: `~/.local/share/twokey-ai/`
- Cache: `~/.cache/twokey-ai/`
- History DB: `~/.local/share/twokey-ai/history.db`
- Toolchains: `~/.config/twokey-ai/toolchains.json`

## Repo

https://github.com/meinzeug/twokey

## License

MIT
