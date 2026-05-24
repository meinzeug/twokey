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
- Visual toolchain editor in settings with persistent create/edit/delete flow.
- Toolchain dry-run and manual execution controls with safety guardrails.
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

- Expand runtime tests from interaction E2E to compositor-native real-session E2E.
- Signed updater pipeline and rollback-ready install flow design.

## Planned

### Desktop Automation

- Richer action catalog (window placement, sequence timing, app state checks).
- Workflow safety controls (dry-run/validation/permission prompts).

Delivered baseline:
- `wait_ms` and `check_command` steps.
- dangerous shell pattern blocking.
- Dry-run preview + manual execution confirmation from settings.

### Platform Hardening

- Wayland-specific UX paths and clearer fallbacks.
- Expanded Linux compositor compatibility matrix (tracking doc added).
- Compositor-aware runtime detection and backend messaging for GNOME/KDE/Sway/Hyprland.
- Runtime diagnostics panel in settings (session/backend/STT/TTS/recent failures).

### Update/Release Hardening

- Signed release artifacts.
- Safer update install flow with rollback.
- Optional background update download.
- Staged rollout by channel/cohort marker in release notes.

Delivered baseline:
- Background update download to staged AppImage path with checksum verification.

### Quality

- Integration tests for end-to-end voice pipeline.
- Snapshot/regression tests for settings and overlay interactions.
- CI validation workflow for TypeScript + Rust checks.
- Reproducible runtime smoke tests for X11 and Wayland in CI session matrix.
- Interaction-level toolchain E2E tests in CI.
