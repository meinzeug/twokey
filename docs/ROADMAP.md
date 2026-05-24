# Roadmap

## Done

### Core Interaction

- Floating pill overlay.
- Hold hotkey voice pipeline.
- Double-tap mode cycle.
- Single-tap file-context trigger.

### AI Workflows

- Conversation mode with provider routing.
- Toolchain trigger execution from spoken transcript.
- Direct edit/replace flow for selected text.
- Dictation insertion flow.
- Feedback capture and persistence.

### Context + Providers

- File context load (text/pdf/image metadata).
- Context-aware provider chat routing.
- Vision request path for image context.
- Hybrid local/online provider strategy with local preference option.

### Runtime Services

- STT providers wired from settings.
- Local Whisper runtime bootstrap in user space (managed ffmpeg + managed whisper venv).
- Local Whisper quality/speed tuning controls (model + beam size).
- Optional TTS answer playback.
- Tray menu and settings.
- SQLite history/audit.
- Secure API key storage.

## In Progress

- Better guardrails and failure UX for provider capability mismatches.
- Better hotkey diagnostics and live status observability.
- Signed updater pipeline and rollback-ready install flow design.

## Planned

### Desktop Automation

- Visual editor for reusable toolchains/macros.
- Richer action catalog (window placement, sequence timing, app state checks).

### Platform Hardening

- Wayland-specific UX paths and clearer fallbacks.
- Expanded Linux compositor compatibility matrix (tracking doc added).

### Update/Release Hardening

- Signed release artifacts.
- Safer update install flow with rollback.
- Optional background update download.

### Quality

- Integration tests for end-to-end voice pipeline.
- Snapshot/regression tests for settings and overlay interactions.
- CI validation workflow for TypeScript + Rust checks.
