use std::{
    env,
    fs,
    path::PathBuf,
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use twokey_ai_lib::toolchains;

static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("env lock poisoned")
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
fn toolchain_e2e_executes_wait_check_and_shell_steps() {
    let _guard = lock_env();
    let config_home = unique_temp_dir("toolchain-config");
    let output_dir = unique_temp_dir("toolchain-output");
    let output_file = output_dir.join("result.txt");
    let toolchain_dir = config_home.join("twokey-ai");
    fs::create_dir_all(&toolchain_dir).expect("toolchain dir");

    let payload = format!(
        r#"[
  {{
    "id": "interaction-e2e",
    "name": "Interaction E2E",
    "trigger": "run interaction",
    "steps": [
      {{ "kind": "wait_ms", "value": "15" }},
      {{ "kind": "check_command", "value": "sh" }},
      {{ "kind": "shell", "value": "echo ok > {}" }}
    ]
  }}
]"#,
        output_file.to_string_lossy()
    );

    fs::write(toolchain_dir.join("toolchains.json"), payload).expect("write toolchains");
    env::set_var("XDG_CONFIG_HOME", &config_home);

    let result = toolchains::run_by_id("interaction-e2e").expect("toolchain run should succeed");
    assert!(result.contains("Interaction E2E"));

    let written = fs::read_to_string(&output_file).expect("output should be written");
    assert_eq!(written.trim(), "ok");
}

#[test]
fn toolchain_e2e_dry_run_blocks_dangerous_shell_patterns() {
    let _guard = lock_env();
    let config_home = unique_temp_dir("toolchain-config-danger");
    let toolchain_dir = config_home.join("twokey-ai");
    fs::create_dir_all(&toolchain_dir).expect("toolchain dir");

    let payload = r#"[
  {
    "id": "danger",
    "name": "Danger",
    "trigger": "do danger",
    "steps": [
      { "kind": "shell", "value": "rm -rf /tmp/example" }
    ]
  }
]"#;

    fs::write(toolchain_dir.join("toolchains.json"), payload).expect("write toolchains");
    env::set_var("XDG_CONFIG_HOME", &config_home);

    let dry = toolchains::dry_run_by_id("danger").expect("dry-run should work");
    assert!(!dry.executable);
    assert!(!dry.warnings.is_empty());
    assert!(dry.warnings[0].contains("gefaehrliches shell-Muster"));

    let run_error = toolchains::run_by_id("danger").expect_err("dangerous toolchain must be blocked");
    assert!(run_error.contains("Sicherheitsgruenden") || run_error.contains("gefaehrliches shell-Muster"));
}
