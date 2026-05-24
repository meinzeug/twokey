import { invoke } from "@tauri-apps/api/core";

export async function getDesktopSessionType(): Promise<string> {
  if (!isTauriRuntime()) {
    return "browser-preview";
  }

  return invoke<string>("get_desktop_session_type");
}

export async function openSettingsWindow(): Promise<void> {
  if (!isTauriRuntime()) {
    const url = new URL(window.location.href);
    url.searchParams.set("view", "settings");
    window.open(url.toString(), "twokey-settings", "width=920,height=680");
    return;
  }

  await invoke("open_settings_window");
}

function isTauriRuntime() {
  return "__TAURI_INTERNALS__" in window;
}
