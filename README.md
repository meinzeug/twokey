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
- Wayland compositor detection (GNOME/KDE/Sway/Hyprland) with compositor-specific runtime capability messaging.
- Wayland automation integration paths:
  - text insertion via `wtype` or `ydotool`
  - selected-text read via `wtype` + `wl-paste` when available
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
- Local Whisper auto-setup on first use:
  - managed `ffmpeg` in `~/.local/share/twokey/bin/`
  - managed Whisper venv in `~/.local/share/twokey/whisper-venv/`
- Local Whisper tuning in settings:
  - model selection (`tiny` .. `large-v3`)
  - beam size tuning (speed vs quality)
- Visible install/save status in settings (including install spinner).

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

- Toolchains have a visual settings editor; richer action templates and safety controls are still pending.
- Wayland still has compositor-specific limits for global hold hotkeys and full automation.
- Update install is AppImage-first with channel-aware checks (`stable`/`beta`/`dev`), checksum validation and local rollback fallback.

## Install

```bash
npm install -g twokey
```

Run:

```bash
twokey
```

Default behavior:

- starts native desktop app in background
- if no native binary is installed, tries to download latest AppImage release
- prepares runtime dependencies in user space (best effort, no system package mutation)

Useful options:

```bash
twokey --help
twokey --cli
twokey --once "Erklaere X11 vs Wayland kurz"
twokey --desktop
twokey --prepare-runtime
twokey --prepare-runtime-only
```

Updater behavior:

- Uses update channel from settings (`stable`, `beta`, `dev`).
- Verifies AppImage checksum when a `.sha256` asset is available.
- Restores previous local AppImage if updated binary cannot be started.

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

Runtime smoke tests:

```bash
cargo test --manifest-path src-tauri/Cargo.toml runtime_e2e -- --nocapture
TWOKEY_E2E_SESSION=x11 cargo test --manifest-path src-tauri/Cargo.toml runtime_e2e -- --nocapture
TWOKEY_E2E_SESSION=wayland cargo test --manifest-path src-tauri/Cargo.toml runtime_e2e -- --nocapture
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

For `local-whisper`, TwoKey attempts runtime setup in user space and checks required binaries before transcription.

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

Notes:

- `ffmpeg` is auto-provisioned to `~/.local/share/twokey/bin/ffmpeg` if missing.
- Local Whisper CLI is auto-provisioned to `~/.local/share/twokey/whisper-venv/bin/whisper` if missing.

## Data Paths

- Config: `~/.config/twokey-ai/`
- Data: `~/.local/share/twokey-ai/`
- Cache: `~/.cache/twokey-ai/`
- History DB: `~/.local/share/twokey-ai/history.db`
- Toolchains: `~/.config/twokey-ai/toolchains.json`
- Managed runtime bin: `~/.local/share/twokey/bin/`
- Managed Whisper venv: `~/.local/share/twokey/whisper-venv/`

## Repo

https://github.com/meinzeug/twokey

## Additional Docs

- Wayland matrix: `docs/WAYLAND_COMPATIBILITY_MATRIX.md`

## License

MIT
