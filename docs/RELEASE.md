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
- AppImage release pipeline publishes signed artifacts (`.sig`/`.pem`) and checksum files.

## Updates

Current in-app behavior:

- check latest GitHub release
- notify user if newer version exists
- download latest AppImage and launch it from settings
- local runtime dependencies can be prepared explicitly via CLI (`--prepare-runtime`, `--prepare-runtime-only`)
- update checks respect configured channel (`stable`, `beta`, `dev`)
- update install verifies checksum when `.sha256` release asset exists
- if starting the updated AppImage fails, previous local AppImage is restored
- staged rollout gating is supported per release body marker:
	- `twokey-rollout:<0-100>` (example: `twokey-rollout:25`)
	- for `beta`, default rollout is 30% if marker is missing
	- for `stable`/`dev`, default rollout is 100%

Not yet implemented:

- remote cohort management/dashboard beyond release-note marker control
