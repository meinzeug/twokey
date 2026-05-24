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

export type AppSettings = {
  autostart: boolean;
  overlayPosition: string;
  overlaySize: string;
  theme: string;
  accentColor: string;
  mainHotkey: string;
  doubleTapMs: number;
  escapeCancel: boolean;
  sttProvider: string;
  defaultLanguage: string;
  punctuationCleanup: boolean;
  ttsEnabled: boolean;
  ttsVoice: string;
  ttsSpeed: number;
  ollamaModel: string;
  preferLocal: boolean;
  saveHistory: boolean;
  logApiRequests: boolean;
  updateChannel: string;
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

export async function readSelectedText(): Promise<string> {
  if (!isTauriRuntime()) {
    throw new Error("Browser-Vorschau kann keine Desktop-Textauswahl lesen.");
  }

  return invoke<string>("read_selected_text");
}

export async function replaceSelectedText(text: string): Promise<void> {
  if (!isTauriRuntime()) {
    throw new Error("Browser-Vorschau kann keine Desktop-Textauswahl ersetzen.");
  }

  await invoke("replace_selected_text", { text });
}

export async function getSettings(): Promise<AppSettings> {
  if (!isTauriRuntime()) {
    return {
      autostart: false,
      overlayPosition: "top-left",
      overlaySize: "compact",
      theme: "dark",
      accentColor: "#75e0c3",
      mainHotkey: "Ctrl+Space",
      doubleTapMs: 420,
      escapeCancel: true,
      sttProvider: "mock",
      defaultLanguage: "de",
      punctuationCleanup: false,
      ttsEnabled: false,
      ttsVoice: "piper-default",
      ttsSpeed: 1,
      ollamaModel: "qwen2.5:3b",
      preferLocal: true,
      saveHistory: true,
      logApiRequests: false,
      updateChannel: "stable",
    };
  }

  return invoke<AppSettings>("get_settings");
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  if (!isTauriRuntime()) {
    return;
  }

  await invoke("save_settings", { settings });
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
