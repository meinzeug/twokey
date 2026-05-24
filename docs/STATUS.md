# Status

## Current Phase

Phase 5: Diktiermodus.

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
- Recording stop now triggers a transcription pipeline.
- Mock STT provider implemented for deterministic development.
- External real STT command adapter implemented through `TWOKEY_STT_COMMAND`.
- Overlay displays the latest transcript and provider.
- X11 synthetic test verified `recording-started`, `recording-stopped`, and `transcript-ready` events.
- Ollama installed as an enabled systemd service on `127.0.0.1:11434`.
- Default local model `qwen2.5:3b` pulled successfully.
- Ollama API smoke test returned a German response.
- Conversation mode routes transcripts to Ollama and displays the answer.
- Dictation mode inserts transcripts at the active cursor on X11.
- Clipboard content is preserved and restored when `xclip` or `xsel` can read it.
- Initial docs created:
  - `docs/FOUNDATIONS.md`
  - `docs/ARCHITECTURE.md`
  - `docs/STATUS.md`
- README now contains setup and start instructions.

## Open Tasks

- Add a helper daemon boundary for desktop automation.
- Add automated/manual test matrix for X11 and Wayland.
- Add SQLite settings and history storage.
- Add OpenAI/OpenRouter provider abstractions.
- Add file context handling.

## Known Bugs / Limitations

- No real AI, text selection, text injection, tray icon, or persistence yet.
- Wayland hotkeys are detected as unsupported; no portal-based fallback exists yet.
- The overlay position is fixed in configuration and not yet user-persisted.
- The settings page is a placeholder and does not save values.
- `git push origin main` is blocked in this shell because HTTPS Git credentials are not configured.
- No bundled local Whisper model or Whisper binary is installed by the app yet.
- `TWOKEY_STT_COMMAND` is intentionally explicit and must be configured by the developer/user for real STT.
- Ollama is CPU-only on this machine; first response after model load may take around 20 seconds.
- X11 dictation requires `xdotool` and either `xclip` or `xsel`.
- Wayland text insertion is still unsupported and reported explicitly.
- This developer machine had root-owned `~/.local/share`; ownership was corrected so Tauri can create its XDG data directory.

## Next Step

Phase 6: Text markieren, lesen, ersetzen.

Planned Phase 6 work:

- Read currently selected text under X11.
- Preview AI-edited replacement.
- Replace selected text after confirmation.
- Preserve clipboard contents throughout.
