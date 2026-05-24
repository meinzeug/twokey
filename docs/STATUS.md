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
- TTS backend support:
  - spd-say
  - espeak-ng / espeak
- Tray icon/menu support.
- GitHub release update check in settings.
- AppImage update download-and-start from settings.
- Wayland fallback via manual recording controls in the overlay menu.

## Remaining Gaps

- Wayland still has hard global hotkey/automation limitations.
- Auto-update is AppImage-first and not yet a signed rollback-capable installer pipeline.
- Vision answer quality depends on chosen provider/model capability.

## Validation Snapshot

- Frontend build: passing (`npm run build`).
- Backend check: passing (`cargo check`).

## Next Priorities

1. Improve runtime diagnostics for hotkey/STT failures directly in UI.
2. Add explicit mode-level safety toggles (auto-replace vs preview).
3. Add provider capability guardrails for vision/TTS/STT routing.
4. Add test matrix and reproducible integration tests for X11 and Wayland.
