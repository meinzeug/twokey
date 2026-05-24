# Linux Desktop Limitations

## X11

X11 allows broad desktop automation. This makes global hotkeys, selected text access, clipboard workflows, and keyboard simulation practical, but also increases security responsibility.

Phase 2 and Phase 6 should implement X11 paths first and audit clipboard restoration carefully.

## Wayland

Wayland intentionally restricts global input capture and synthetic input. This protects users, but it means some workflows cannot be implemented generically across all compositors.

TwoKey must detect Wayland and explain unavailable capabilities instead of failing silently.

Possible Wayland routes:

- Desktop portals where available.
- Compositor-specific protocols where acceptable.
- `ydotool` with clear installation and permission guidance.
- Clipboard tools such as `wl-copy`/`wl-paste` only where they actually solve the workflow.

## Current Phase

Phase 1 only displays the session type from `XDG_SESSION_TYPE`.
