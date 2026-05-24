# Linux Desktop Limitations

## X11

Supported and used as primary path:

- global hold hotkeys
- selected text read/replace automation
- clipboard-based insertion flows

Dependencies may be needed:

- xdotool
- xclip or xsel

## Wayland

Known constraints:

- generic global hold hotkeys are often blocked
- synthetic input and global capture are compositor-restricted

TwoKey behavior on Wayland:

- reports capability limits explicitly
- allows manual start/stop recording from overlay menu as fallback
- avoids silent failure or fake success

## Practical Guidance

For full current feature set, use X11 session.
Wayland support requires compositor-specific or portal-based expansion and remains partial.
