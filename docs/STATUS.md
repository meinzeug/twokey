# Status

## Current Phase

Phase 2: Global Hotkey + Audioaufnahme.

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
- Native Tauri build verified after installing Linux prerequisites and Rust.
- AppImage and `.deb` bundle generation verified.
- X11 `Ctrl+Space` hold detection implemented through `XQueryKeymap`.
- Double-tap mode cycling implemented for short `Ctrl+Space` taps.
- Escape cancellation implemented for active recordings.
- CLI-backed audio recording implemented with `pw-record`, `parec`, or `arecord`.
- Wayland limitation is detected and surfaced instead of failing silently.
- X11 synthetic hotkey test with `xdotool` created a WAV recording in `~/.cache/twokey-ai/recordings/`.
- Initial docs created:
  - `docs/FOUNDATIONS.md`
  - `docs/ARCHITECTURE.md`
  - `docs/STATUS.md`
- README now contains setup and start instructions.

## Open Tasks

- Add a helper daemon boundary for desktop automation.
- Add automated/manual test matrix for X11 and Wayland.
- Add SQLite settings and history storage.
- Add provider abstractions.
- Add file context handling.

## Known Bugs / Limitations

- No real AI, text selection, text injection, tray icon, or persistence yet.
- Wayland hotkeys are detected as unsupported; no portal-based fallback exists yet.
- The overlay position is fixed in configuration and not yet user-persisted.
- The settings page is a placeholder and does not save values.
- `git push origin main` is blocked in this shell because HTTPS Git credentials are not configured.
- STT is not implemented yet, so saved recordings are not transcribed.
- This developer machine had root-owned `~/.local/share`; ownership was corrected so Tauri can create its XDG data directory.

## Next Step

Phase 3: STT Mock + echtes STT.

Planned Phase 3 work:

- Add a mock transcriber for deterministic development.
- Add transcript display in the overlay.
- Wire the recording path from Phase 2 into the transcription pipeline.
- Plan local Whisper and OpenAI-compatible STT provider interfaces.
