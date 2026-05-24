# Release

## Initial Packaging Targets

- AppImage.
- `.deb`.
- Optional Flatpak later.

## Desktop Integration

Planned integration:

- `.desktop` launcher.
- Tray icon.
- Autostart via `~/.config/autostart/`.
- Logs under `~/.local/state/twokey-ai/` or `~/.cache/twokey-ai/logs/`.

## Updates

Auto-update is planned, not implemented.

Initial strategy:

- Check GitHub Releases.
- Compare local version.
- Show update notice.
- Open download link.

Fully automatic updates should only be added when signing and rollback behavior are clear.
