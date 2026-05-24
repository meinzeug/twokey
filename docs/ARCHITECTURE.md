# Architecture

## Phase 1 Choice

The app starts as a Tauri 2 desktop application with a React frontend and Rust native layer.

This matches the Linux desktop requirements better than a browser-only app because Tauri can own native windows, tray integration, packaging, and later IPC to a helper daemon.

## Components

```text
React UI
  Overlay pill
  Mode menu
  Settings window

Tauri Rust shell
  Window lifecycle
  Desktop session detection
  Future bridge to helper daemon

Future helper daemon
  Global hotkeys
  Audio capture
  Clipboard and text automation
  X11/Wayland capability checks
```

## Why a Helper Daemon Is Planned

Global hotkeys, text selection, clipboard preservation, and text injection are desktop-automation concerns. Tauri should not become a large automation backend.

The planned split is:

1. Tauri GUI for overlay, settings, tray, status, and packaging.
2. Linux helper daemon/service for hotkeys, audio, clipboard, and automation.
3. IPC over a Unix socket by default, with localhost only if needed.

## Linux Desktop Strategy

X11 and Wayland will be treated as separate capability profiles.

- X11: start with clipboard-based selected-text read/replace and keyboard simulation fallbacks.
- Wayland: detect restrictions, prefer portals where available, and show clear explanations when an action is unavailable.

Phase 1 only detects `XDG_SESSION_TYPE` and displays it in the overlay menu.

## Data Locations

- Config: `~/.config/twokey-ai/`
- Data: `~/.local/share/twokey-ai/`
- Cache: `~/.cache/twokey-ai/`
- Logs/state: `~/.local/state/twokey-ai/`

## Provider Interface

Future providers should implement:

- `chat(messages, options)`
- `transcribe(audioFile, options)`
- `speak(text, options)`
- `vision(imageFile, prompt, options)` when supported

## Current Phase 1 Scope

Implemented:

- Tauri app skeleton
- Overlay window
- Settings window placeholder
- Dummy mode switching
- Desktop session detection command

Not implemented:

- Global hotkeys
- Audio capture
- STT/TTS
- Ollama/OpenAI/OpenRouter providers
- SQLite storage
- Clipboard and text replacement
- File context extraction
