# Release

## Packaging

Current build targets:

- AppImage
- deb

Build command:

```bash
npm run tauri:build
```

## Distribution

- GitHub Releases used for release metadata and downloadable assets.
- npm package includes CLI entrypoint for desktop launch and fallback AppImage download.
- npm global install bootstraps known runtime dependencies in user space (managed `ffmpeg` + managed Whisper CLI venv).

## Updates

Current in-app behavior:

- check latest GitHub release
- notify user if newer version exists
- download latest AppImage and launch it from settings
- local runtime dependencies can be prepared explicitly via CLI (`--prepare-runtime`, `--prepare-runtime-only`)

Not yet implemented:

- signed auto-install with rollback
- staged channels with automatic safe rollout
