use std::{
    env,
    fs,
    path::PathBuf,
    process::Command,
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("env lock poisoned")
}

fn should_run_session(session: &str) -> bool {
    match env::var("TWOKEY_E2E_SESSION") {
        Ok(selected) => selected == session,
        Err(_) => true,
    }
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_millis();
    let dir = env::temp_dir().join(format!("twokey-{label}-{ts}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[test]
fn runtime_e2e_x11_mock_pipeline() {
    if !should_run_session("x11") {
        return;
    }

    let _guard = lock_env();
    let workspace = env!("CARGO_MANIFEST_DIR");

    let status = Command::new("cargo")
        .current_dir(workspace)
        .args(["check", "--manifest-path", "Cargo.toml"])
        .env("XDG_SESSION_TYPE", "x11")
        .status()
        .expect("cargo check should start");

    assert!(status.success(), "cargo check failed for x11 runtime smoke");
}

#[test]
fn runtime_e2e_wayland_mock_pipeline() {
    if !should_run_session("wayland") {
        return;
    }

    let _guard = lock_env();
    let workspace = env!("CARGO_MANIFEST_DIR");

    let config_home = unique_temp_dir("config-wayland");
    let data_home = unique_temp_dir("data-wayland");
    let cache_home = unique_temp_dir("cache-wayland");

    let status = Command::new("cargo")
        .current_dir(workspace)
        .args(["check", "--manifest-path", "Cargo.toml"])
        .env("XDG_SESSION_TYPE", "wayland")
        .env("XDG_CURRENT_DESKTOP", "Hyprland")
        .env("XDG_CONFIG_HOME", &config_home)
        .env("XDG_DATA_HOME", &data_home)
        .env("XDG_CACHE_HOME", &cache_home)
        .status()
        .expect("cargo check should start");

    assert!(status.success(), "cargo check failed for wayland runtime smoke");
}

#[test]
fn runtime_e2e_cli_runtime_prepare_smoke() {
    if !should_run_session("x11") {
        return;
    }

    let _guard = lock_env();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf();

    let status = Command::new("node")
        .current_dir(&root)
        .args(["./bin/twokey.js", "--prepare-runtime-only", "--quiet"])
        .status()
        .expect("node runtime prepare should start");

    assert!(status.success(), "runtime preparation smoke failed");
}
