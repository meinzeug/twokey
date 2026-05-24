# Status

## Current Phase

Phase 1: Projektstruktur und Dummy-Overlay.

## Completed

- Tauri + React + TypeScript project structure created.
- Overlay pill implemented with dark minimal styling.
- Dummy modes implemented:
  - Gespräch
  - Text bearbeiten
  - Diktieren
  - Feedback
- Click on the pill opens the mode menu.
- Double click or menu action opens the settings window.
- Placeholder settings UI created.
- Desktop session type is detected through the Tauri Rust layer.
- Initial docs created:
  - `docs/FOUNDATIONS.md`
  - `docs/ARCHITECTURE.md`
  - `docs/STATUS.md`
- README now contains setup and start instructions.

## Open Tasks

- Install Rust/Cargo on the development machine before running the native Tauri app.
- Add real global hotkey handling.
- Add hold-to-record audio capture.
- Add a helper daemon boundary for desktop automation.
- Add SQLite settings and history storage.
- Add provider abstractions.
- Add file context handling.

## Known Bugs / Limitations

- Phase 1 has no real hotkeys, audio, AI, text selection, text injection, tray icon, or persistence.
- Wayland and X11 behavior is only detected, not acted on.
- The overlay position is fixed in configuration and not yet user-persisted.
- The settings page is a placeholder and does not save values.

## Next Step

Phase 2: globale Hotkeys + Audioaufnahme.

Planned Phase 2 work:

- Define helper daemon responsibilities.
- Implement global hotkey detection for X11 first.
- Add Wayland capability detection and user-facing warnings.
- Add hold-to-record audio plumbing using PipeWire/PulseAudio-compatible APIs or a pragmatic CLI-backed prototype.
- Update docs with tested desktop limitations.
