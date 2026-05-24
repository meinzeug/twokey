# Architecture

## Stack

- UI: React + TypeScript
- Desktop shell: Tauri 2 + Rust
- Providers: local + online routing layer
- Persistence: JSON settings + SQLite history + keyring secrets

## Runtime Data Flow

1. Global hotkey event starts/stops recording.
2. Recorded audio is sent to configured STT provider.
3. Transcript event is emitted to UI.
4. UI dispatches mode-specific action:
   - conversation -> provider chat -> optional TTS
   - edit -> selected-text read + rewrite + replace
   - dictation -> direct insertion
   - feedback -> local feedback persistence
5. Events and outcomes are written to history DB.

## Main Modules

- hotkeys: global hotkey polling + event lifecycle
- audio: recording lifecycle
- stt: provider-specific transcription
- provider: chat routing and vision-capable request paths
- tts: local speech output backends
- desktop: read/replace/insert automation
- file_context: file picker + extraction
- history: SQLite audit and history APIs
- secrets: API key storage via keyring
- tray: tray menu and quick actions
- updater: release check API

## Context Routing

- Text context is merged into prompt body.
- Image context routes through vision-capable provider APIs.
- Provider selection considers user preference and local-first policy.

## Platform Profile

- X11: primary hotkey and desktop-automation path.
- Wayland: explicit limitations and fallback messaging.

## Storage

- Config: ~/.config/twokey-ai/settings.json
- History DB: ~/.local/share/twokey-ai/history.db
- Cache: ~/.cache/twokey-ai/
- Secrets: OS keyring
