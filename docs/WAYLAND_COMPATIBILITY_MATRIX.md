# Wayland Compatibility Matrix

This document tracks current behavior per desktop session and compositor profile.

## Legend

- `yes`: supported and validated
- `partial`: works with fallback/limitations
- `no`: currently unavailable
- `todo`: planned but not implemented

## Feature Matrix

| Feature | X11 | Wayland (generic) | Notes |
|---|---|---|---|
| Global hold hotkey | yes | partial | Wayland depends on compositor policies; fallback is manual start/stop in overlay |
| Double-tap mode cycle | yes | partial | Works where global hotkey dispatch is available |
| Single-tap file picker trigger | yes | partial | Same constraints as global hotkeys |
| Record/transcribe pipeline | yes | yes | Manual trigger available even with restricted global hotkeys |
| Read selected text automation | yes | partial | Restricted by compositor/security model |
| Replace selected text automation | yes | partial | Restricted by compositor/security model |
| Direct dictation insertion | yes | partial | Backend-dependent on synthetic input restrictions |
| Tray + settings window | yes | yes | Independent from compositor input policy |
| Local Whisper STT | yes | yes | Requires runtime dependencies; installed in user space |

## Compositor-Specific Follow-up

Current repo state does not yet include compositor-specific integration layers. Track and validate for:

1. GNOME (Mutter)
2. KDE Plasma (KWin)
3. Sway/wlroots
4. Hyprland

For each compositor, add:

- tested distro and version
- working feature set
- fallback behavior
- known blockers
- workaround commands or settings
