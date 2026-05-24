# Security

## Core Rules

- Never execute destructive actions without explicit user intent.
- Make external provider usage transparent.
- Keep secrets out of source code and config JSON.

## Data Protection

- API keys are stored in the OS keyring.
- Runtime history is stored locally in SQLite.
- Settings are stored in user config directory.

## Desktop Automation Safety

- Desktop insertion/replacement is mode-bound and user-triggered via hotkey.
- Escape can cancel active recording.
- Wayland restrictions are reported explicitly.

## Current Trade-offs

- Edit mode currently applies replacement directly after model output to match low-friction workflow.
- For sensitive environments, a preview-confirm toggle should be added as follow-up hardening.

## External Calls

Potential outbound data paths:

- OpenAI-compatible STT/chat
- OpenRouter chat
- release update check (GitHub API)

Users must configure provider keys explicitly; no implicit cloud provider activation.
