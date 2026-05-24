# Status

## Current Phase

Phase 12: Stabilisierung.

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
- Text edit mode reads selected text under X11.
- Text edit mode asks Ollama for a replacement using the spoken instruction.
- Text replacement requires explicit confirmation in the overlay.
- Settings are persisted under `~/.config/twokey-ai/settings.json`.
- Settings UI has real controls for general, hotkey, STT, Ollama, privacy, and update-channel values.
- Ollama chat is routed through a provider abstraction.
- Provider metadata is exposed to the settings UI.
- OpenAI-compatible and OpenRouter-compatible providers are represented as disabled placeholders.
- File context can be added from the overlay menu.
- TXT/MD/JSON/CSV files are read directly.
- PDF text extraction uses `pdftotext`.
- Image files are registered as vision-ready metadata.
- Extracted file text is cached under `~/.cache/twokey-ai/file-contexts/`.
- AppImage and `.deb` bundle generation remains verified.
- Autostart setting writes/removes `~/.config/autostart/twokey-ai.desktop`.
- Settings can check GitHub Releases for a newer version.
- Update checks do not auto-download or auto-install.
- Final `npm run tauri:build` succeeded and produced AppImage plus `.deb`.
- `npm audit --omit=dev` reports 0 vulnerabilities.
- Ollama service is active with `qwen2.5:3b` installed.
- Repository is pushed to `origin/main`.
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
- Some settings are persisted before all runtime systems consume them.
- Online provider API-key storage and execution are not implemented yet.
- Office files are not supported yet.
- Image files are not analyzed until a vision provider is active.
- Tray icon is still not implemented.
- Release signing and rollback strategy are not defined, so automatic updates are intentionally not implemented.
- TTS is not implemented yet.
- Tray menu is not implemented yet.
- Online provider execution is represented but disabled until secure API-key storage is added.
- This developer machine had root-owned `~/.local/share`; ownership was corrected so Tauri can create its XDG data directory.

## Next Step

Post-Phase Hardening.

Recommended next work:

- Add tray icon/menu.
- Add SQLite history/audit tables.
- Add real local Whisper integration.
- Add Piper TTS.
- Add secure API-key storage for online providers.
- Add Wayland-specific UX paths and portal investigation.
