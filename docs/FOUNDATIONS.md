# Foundations

## Product Vision

TwoKey should let users operate AI during normal desktop work with minimal friction:

- hold hotkey, speak, release
- AI handles the request in the current work context
- no browser tab or dedicated chat app switching required

## Core UX Principles

- The pill is ambient and lightweight.
- Status states are always visible (ready/listening/transcribing/thinking/writing/error).
- Potentially long operations (for example runtime install/setup) are visibly represented in settings status.
- Mode switching is fast and predictable.
- Recovery paths are explicit when OS capabilities are missing.

## Modes

- Conversation: ask and receive AI answers, optional TTS playback.
- Edit Text: transform selected text by voice instruction and apply replacement.
- Dictation: insert spoken text at cursor.
- Feedback: capture product feedback into local history.

## Context Model

- Single-tap hotkey can trigger file-context selection.
- Context can include text files, PDF extraction, or image references.
- Providers receive context-aware prompts.

## Privacy and Routing

- Prefer local model routing where possible.
- Use secure key storage for online APIs.
- Keep local audit trail of STT/chat/TTS/feedback events.
- Prefer user-space runtime setup over system-wide package mutation.

## Constraints

- Linux desktop target first.
- X11 currently offers best hotkey/automation reliability.
- Wayland remains constrained by compositor/security model.

## Non-Goals (Current Scope)

- No full autonomous multi-step OS orchestration yet.
- No silent bypass of platform security restrictions.
- No unsigned automatic background update installer flow.
