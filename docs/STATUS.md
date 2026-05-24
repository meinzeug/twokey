# Status

## Current Focus

Align runtime behavior with the video UX:

- hold hotkey -> record -> transcribe -> AI action
- minimal context-switch workflow
- direct text operations in current app
- optional spoken AI response

## Implemented

- Floating overlay pill with mode menu and settings window.
- X11 global hotkey pipeline:
  - hold to record
  - release to process
  - single tap for file picker event
  - double tap for mode cycle
  - escape cancel
- Modes:
  - Conversation
  - Edit Text
  - Dictation
  - Feedback
- Conversation mode:
  - transcript to selected provider
  - optional toolchain execution for multi-step actions
  - optional TTS answer playback
- Edit mode:
  - read selected text
  - AI rewrite
  - direct replacement
- Dictation mode insertion at current cursor.
- Feedback mode local persistence in SQLite history.
- File context load:
  - text/markdown/json/csv
  - PDF extraction
  - image context support
- Provider routing:
  - Ollama local
  - OpenAI-compatible
  - OpenRouter-compatible
  - secure API keys via keyring
- STT provider wiring from settings:
  - mock
  - local-whisper
  - external-command
  - openai-compatible
- Local runtime dependency bootstrap:
  - managed `ffmpeg` in `~/.local/share/twokey/bin/`
  - managed Whisper venv in `~/.local/share/twokey/whisper-venv/`
- Local Whisper settings:
  - model selection
  - beam size tuning
- Save/install UX in settings shows explicit operation status (including install phase).
- Edit mode safety toggle available (`direkt ersetzen` vs preview/confirm).
- Provider guardrail for image-context + non-vision provider mismatch.
- Local Whisper runtime diagnostics endpoint wired into settings.
- TTS backend support:
  - spd-say
  - espeak-ng / espeak
- Tray icon/menu support.
- GitHub release update check in settings.
- AppImage update download-and-start from settings.
- Release pipeline publishes checksum + keyless signatures for AppImage assets.
- CI workflow validates TS and Rust compile checks on each push/PR.
- Wayland fallback via manual recording controls in the overlay menu.
- Wayland compatibility matrix tracking document added.
- Wayland compositor-aware runtime detection for GNOME/KDE/Sway/Hyprland.
- Wayland automation integration paths:
  - insertion via `wtype`/`ydotool`
  - selected-text read via `wtype` + `wl-paste` when available
- Runtime smoke E2E tests for X11/Wayland added and wired in CI matrix.
- Updater supports channel-aware release selection and checksum verification.
- Updater restores previous local AppImage if updated binary start fails.
- Updater supports staged rollout cohorts via release marker (`twokey-rollout:<percent>`).

## Remaining Gaps

- Wayland still has hard limits for truly global hold-hotkeys depending on compositor security policy.
- Auto-update remains AppImage-first.
- Vision answer quality depends on chosen provider/model capability.

## Hard Gaps Requiring Larger Milestones

1. Remote rollout management (dashboard/control plane) beyond release-note marker control.
2. Deeper compositor-native Wayland integrations beyond current safe integration paths.
3. Full end-to-end runtime integration tests on real compositor sessions (beyond smoke-level checks).
4. Visual toolchain/workflow builder.

## Validation Snapshot

- Frontend build: passing (`npm run build`).
- Backend check: passing (`cargo check`).

## Next Priorities

1. Improve runtime diagnostics for hotkey/STT failures directly in UI.
2. Expand provider capability guardrails for full feature matrix (vision/TTS/STT).
3. Expand runtime tests from smoke checks to real interaction E2E scenarios.
