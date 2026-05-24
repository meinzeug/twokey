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
| Read selected text automation | yes | partial | Wayland path supports `wtype` + `wl-paste` when available |
| Replace selected text automation | yes | partial | Restricted by compositor/security model |
| Direct dictation insertion | yes | partial | Wayland insertion via `wtype` or `ydotool` |
| Tray + settings window | yes | yes | Independent from compositor input policy |
| Local Whisper STT | yes | yes | Requires runtime dependencies; installed in user space |

## Runtime Integrations Implemented

- Compositor-aware detection in runtime capabilities for:
	- GNOME (Mutter)
	- KDE Plasma (KWin)
	- Sway/wlroots
	- Hyprland
- Backend naming and warning messages now include compositor context.
- Safe fallback remains explicit when compositor security blocks global hold-hotkeys.

## Reproducible Runtime Tests

- CI matrix executes reproducible runtime smoke tests for both `x11` and `wayland` sessions.
- Local reproduction:

```bash
TWOKEY_E2E_SESSION=x11 cargo test --manifest-path src-tauri/Cargo.toml runtime_e2e -- --nocapture
TWOKEY_E2E_SESSION=wayland cargo test --manifest-path src-tauri/Cargo.toml runtime_e2e -- --nocapture
```

## Compositor-Specific Follow-up

Current repo state includes compositor-aware integration and safe automation paths, but deeper compositor-native hotkey integration still needs dedicated work per compositor:

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
