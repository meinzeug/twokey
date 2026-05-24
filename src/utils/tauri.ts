import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type DesktopCapabilities = {
  sessionType: string;
  hotkeysSupported: boolean;
  audioSupported: boolean;
  automationBackend: string;
  warning?: string | null;
};

export type HotkeyEvent = {
  kind: string;
  status: "ready" | "listening" | "transcribing" | "thinking" | "writing" | "error";
  message: string;
  audioPath?: string | null;
  transcript?: string | null;
  provider?: string | null;
};

export async function getDesktopSessionType(): Promise<string> {
  if (!isTauriRuntime()) {
    return "browser-preview";
  }

  return invoke<string>("get_desktop_session_type");
}

export async function getDesktopCapabilities(): Promise<DesktopCapabilities> {
  if (!isTauriRuntime()) {
    return {
      sessionType: "browser-preview",
      hotkeysSupported: false,
      audioSupported: false,
      automationBackend: "browser",
      warning: "Browser-Vorschau ohne native Hotkeys und Audioaufnahme.",
    };
  }

  return invoke<DesktopCapabilities>("get_desktop_capabilities");
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

export async function askOllama(prompt: string): Promise<string> {
  if (!isTauriRuntime()) {
    return "Browser-Vorschau: Ollama ist nur in der nativen Tauri-App verfuegbar.";
  }

  return invoke<string>("ask_ollama", { prompt });
}

export async function insertText(text: string): Promise<void> {
  if (!isTauriRuntime()) {
    throw new Error("Browser-Vorschau kann keinen Text in Desktop-Apps einfuegen.");
  }

  await invoke("insert_text", { text });
}

export async function listenForHotkeyEvents(callback: (event: HotkeyEvent) => void): Promise<UnlistenFn | undefined> {
  if (!isTauriRuntime()) {
    return undefined;
  }

  return listen<HotkeyEvent>("twokey://hotkey-event", (event) => callback(event.payload));
}

function isTauriRuntime() {
  return "__TAURI_INTERNALS__" in window;
}
