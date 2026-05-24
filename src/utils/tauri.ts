import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

export type DesktopCapabilities = {
  sessionType: string;
  compositor?: string;
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
  whisperModel: string;
  whisperBeamSize: number;
  editAutoApply: boolean;
  punctuationCleanup: boolean;
  ttsEnabled: boolean;
  ttsBackend: string;
  ttsVoice: string;
  ttsSpeed: number;
  ollamaModel: string;
  preferredChatProvider: string;
  openaiBaseUrl: string;
  openaiModel: string;
  openrouterBaseUrl: string;
  openrouterModel: string;
  openrouterModelTier: string;
  preferLocal: boolean;
  saveHistory: boolean;
  logApiRequests: boolean;
  trayEnabled: boolean;
  updateChannel: string;
};

export type ProviderInfo = {
  id: string;
  label: string;
  kind: string;
  enabled: boolean;
  supportsChat: boolean;
  supportsStt: boolean;
  supportsTts: boolean;
  supportsVision: boolean;
  note: string;
};

export type OpenRouterModelInfo = {
  id: string;
  name: string;
  description: string;
  isFree: boolean;
  contextLength?: number | null;
  pricingPrompt?: string | null;
  pricingCompletion?: string | null;
};

export type FileContext = {
  path: string;
  name: string;
  kind: string;
  summary: string;
  extractedText?: string | null;
  cachePath?: string | null;
};

export type UpdateStatus = {
  currentVersion: string;
  latestVersion?: string | null;
  updateAvailable: boolean;
  releaseUrl?: string | null;
  message: string;
};

export type SecretStatus = {
  providerId: string;
  configured: boolean;
};

export type HistoryEntry = {
  id: number;
  tsUnixMs: number;
  kind: string;
  mode?: string | null;
  provider?: string | null;
  inputText?: string | null;
  outputText?: string | null;
  metadataJson?: string | null;
  success: boolean;
};

export type LocalWhisperDiagnostics = {
  whisperAvailable: boolean;
  ffmpegAvailable: boolean;
  managedWhisperPath: string;
  managedFfmpegPath: string;
  message: string;
};

export type ToolchainStep = {
  kind: string;
  value: string;
};

export type Toolchain = {
  id: string;
  name: string;
  trigger: string;
  steps: ToolchainStep[];
};

export type ToolchainDryRun = {
  toolchainId: string;
  toolchainName: string;
  executable: boolean;
  steps: string[];
  warnings: string[];
};

export type RuntimeDiagnostics = {
  desktop: DesktopCapabilities;
  whisper: LocalWhisperDiagnostics;
  tts: {
    backend: string;
    available: boolean;
    message: string;
  };
  providers: ProviderInfo[];
  sttProvider: string;
  chatProvider: string;
  recentFailures: HistoryEntry[];
};

export type TtsBackendStatus = {
  backend: string;
  available: boolean;
  message: string;
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

export async function askAssistant(prompt: string, fileContext?: FileContext | null): Promise<string> {
  if (!isTauriRuntime()) {
    return "Browser-Vorschau: Assistent ist nur in der nativen Tauri-App verfuegbar.";
  }

  return invoke<string>("ask_assistant", {
    prompt,
    file_context: fileContext
      ? {
          path: fileContext.path,
          name: fileContext.name,
          kind: fileContext.kind,
          summary: fileContext.summary,
          extractedText: fileContext.extractedText ?? null,
        }
      : null,
  });
}

export async function submitFeedback(text: string): Promise<string> {
  if (!isTauriRuntime()) {
    return "Feedback wurde in der Browser-Vorschau nicht gespeichert.";
  }

  return invoke<string>("submit_feedback", { text });
}

export async function speakText(text: string): Promise<string> {
  if (!isTauriRuntime()) {
    throw new Error("Browser-Vorschau kann kein TTS starten.");
  }

  return invoke<string>("speak_text", { text });
}

export async function installTtsBackend(backend: string, sudoPassword?: string | null): Promise<string> {
  if (!isTauriRuntime()) {
    throw new Error("Browser-Vorschau kann kein TTS-Backend installieren.");
  }

  return invoke<string>("install_tts_backend", { backend, sudoPassword: sudoPassword ?? null });
}

export async function getTtsBackendStatus(backend: string): Promise<TtsBackendStatus> {
  if (!isTauriRuntime()) {
    return {
      backend,
      available: false,
      message: "Browser-Vorschau kann die TTS-Backend-Verfuegbarkeit nicht pruefen.",
    };
  }

  return invoke<TtsBackendStatus>("get_tts_backend_status", { backend });
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
      whisperModel: "base",
      whisperBeamSize: 5,
      editAutoApply: true,
      punctuationCleanup: false,
      ttsEnabled: false,
      ttsBackend: "auto",
      ttsVoice: "piper-default",
      ttsSpeed: 1,
      ollamaModel: "qwen2.5:3b",
      preferredChatProvider: "ollama",
      openaiBaseUrl: "https://api.openai.com/v1",
      openaiModel: "gpt-4o-mini",
      openrouterBaseUrl: "https://openrouter.ai/api/v1",
      openrouterModel: "openai/gpt-4o-mini",
      openrouterModelTier: "free",
      preferLocal: true,
      saveHistory: true,
      logApiRequests: false,
      trayEnabled: true,
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

export async function setAutostart(enabled: boolean): Promise<void> {
  if (!isTauriRuntime()) {
    return;
  }

  await invoke("set_autostart", { enabled });
}

export async function listProviders(): Promise<ProviderInfo[]> {
  if (!isTauriRuntime()) {
    return [];
  }

  return invoke<ProviderInfo[]>("list_providers");
}

export async function listOpenRouterModels(): Promise<OpenRouterModelInfo[]> {
  if (!isTauriRuntime()) {
    return [];
  }

  return invoke<OpenRouterModelInfo[]>("list_openrouter_models");
}

export async function addFileContext(): Promise<FileContext> {
  if (!isTauriRuntime()) {
    throw new Error("Browser-Vorschau kann keinen nativen Dateidialog oeffnen.");
  }

  return invoke<FileContext>("add_file_context");
}

export async function checkForUpdates(): Promise<UpdateStatus> {
  if (!isTauriRuntime()) {
    return {
      currentVersion: "0.1.0",
      latestVersion: null,
      updateAvailable: false,
      releaseUrl: null,
      message: "Browser-Vorschau kann GitHub Releases nicht nativ pruefen.",
    };
  }

  return invoke<UpdateStatus>("check_for_updates");
}

export async function installLatestUpdate(): Promise<string> {
  if (!isTauriRuntime()) {
    throw new Error("Browser-Vorschau kann kein Desktop-Update installieren.");
  }

  return invoke<string>("install_latest_update");
}

export async function downloadLatestUpdateBackground(): Promise<string> {
  if (!isTauriRuntime()) {
    throw new Error("Browser-Vorschau kann kein Hintergrund-Update laden.");
  }

  return invoke<string>("download_latest_update_background");
}

export async function runToolchainFromText(text: string): Promise<string | null> {
  if (!isTauriRuntime()) {
    return null;
  }

  return invoke<string | null>("run_toolchain_from_text", { text });
}

export async function runToolchainById(id: string): Promise<string> {
  if (!isTauriRuntime()) {
    return "Browser-Vorschau: Toolchain-Ausfuehrung nicht verfuegbar.";
  }

  return invoke<string>("run_toolchain_by_id", { id });
}

export async function dryRunToolchainById(id: string): Promise<ToolchainDryRun> {
  if (!isTauriRuntime()) {
    return {
      toolchainId: id,
      toolchainName: "Browser-Vorschau",
      executable: false,
      steps: [],
      warnings: ["Browser-Vorschau: Dry-Run nicht verfuegbar."],
    };
  }

  return invoke<ToolchainDryRun>("dry_run_toolchain_by_id", { id });
}

export async function listToolchains(): Promise<Toolchain[]> {
  if (!isTauriRuntime()) {
    return [];
  }

  return invoke<Toolchain[]>("list_toolchains");
}

export async function saveToolchains(toolchains: Toolchain[]): Promise<void> {
  if (!isTauriRuntime()) {
    return;
  }

  await invoke("save_toolchains", { toolchains });
}

export async function startManualCapture(): Promise<void> {
  if (!isTauriRuntime()) {
    return;
  }

  await invoke("start_manual_capture");
}

export async function stopManualCapture(): Promise<void> {
  if (!isTauriRuntime()) {
    return;
  }

  await invoke("stop_manual_capture");
}

export async function setProviderApiKey(providerId: string, apiKey: string): Promise<void> {
  if (!isTauriRuntime()) {
    return;
  }

  await invoke("set_provider_api_key", { providerId, apiKey });
}

export async function clearProviderApiKey(providerId: string): Promise<void> {
  if (!isTauriRuntime()) {
    return;
  }

  await invoke("clear_provider_api_key", { providerId });
}

export async function providerApiKeyStatus(providerId: string): Promise<SecretStatus> {
  if (!isTauriRuntime()) {
    return { providerId, configured: false };
  }

  return invoke<SecretStatus>("provider_api_key_status", { providerId });
}

export async function getHistoryRecent(limit = 50): Promise<HistoryEntry[]> {
  if (!isTauriRuntime()) {
    return [];
  }

  return invoke<HistoryEntry[]>("history_recent", { limit });
}

export async function getLocalWhisperDiagnostics(): Promise<LocalWhisperDiagnostics> {
  if (!isTauriRuntime()) {
    return {
      whisperAvailable: false,
      ffmpegAvailable: false,
      managedWhisperPath: "",
      managedFfmpegPath: "",
      message: "Browser-Vorschau verfuegbar, keine Runtime-Diagnose.",
    };
  }

  return invoke<LocalWhisperDiagnostics>("get_local_whisper_diagnostics");
}

export async function ensureLocalWhisperRuntime(sudoPassword?: string | null): Promise<string> {
  if (!isTauriRuntime()) {
    throw new Error("Browser-Vorschau kann keine lokale Whisper-Runtime installieren.");
  }

  return invoke<string>("ensure_local_whisper_runtime", { sudoPassword: sudoPassword ?? null });
}

export async function getRuntimeDiagnostics(): Promise<RuntimeDiagnostics> {
  if (!isTauriRuntime()) {
    return {
      desktop: {
        sessionType: "browser-preview",
        hotkeysSupported: false,
        audioSupported: false,
        automationBackend: "browser",
        warning: "Browser-Vorschau ohne native Runtime-Diagnose.",
      },
      whisper: {
        whisperAvailable: false,
        ffmpegAvailable: false,
        managedWhisperPath: "",
        managedFfmpegPath: "",
        message: "Browser-Vorschau verfuegbar, keine Runtime-Diagnose.",
      },
      tts: {
        backend: "none",
        available: false,
        message: "Browser-Vorschau verfuegbar, keine Runtime-Diagnose.",
      },
      providers: [],
      sttProvider: "mock",
      chatProvider: "ollama",
      recentFailures: [],
    };
  }

  return invoke<RuntimeDiagnostics>("get_runtime_diagnostics");
}

export async function listenForHotkeyEvents(callback: (event: HotkeyEvent) => void): Promise<UnlistenFn | undefined> {
  if (!isTauriRuntime()) {
    return undefined;
  }

  return listen<HotkeyEvent>("twokey://hotkey-event", (event) => callback(event.payload));
}

export async function setOverlayWindowExpanded(expanded: boolean): Promise<void> {
  if (!isTauriRuntime()) {
    return;
  }
  await invoke("set_overlay_window_expanded", { expanded });
}

export async function exportDebugReport(userNotes: string): Promise<string> {
  if (!isTauriRuntime()) {
    throw new Error("Browser-Vorschau kann keinen Debug-Bericht exportieren.");
  }

  return invoke<string>("export_debug_report", { userNotes });
}

export async function startOverlayDrag(): Promise<void> {
  if (!isTauriRuntime()) {
    return;
  }

  await getCurrentWindow().startDragging();
}

function isTauriRuntime() {
  return "__TAURI_INTERNALS__" in window;
}
