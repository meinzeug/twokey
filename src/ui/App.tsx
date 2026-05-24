import { useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import {
  Bot,
  Check,
  ChevronDown,
  FilePlus2,
  Keyboard,
  MessageSquareText,
  Mic,
  PencilLine,
  Settings,
  ShieldAlert,
  X,
} from "lucide-react";
import {
  askAssistant,
  addFileContext,
  checkForUpdates,
  clearProviderApiKey,
  getHistoryRecent,
  getDesktopCapabilities,
  getSettings,
  insertText,
  listProviders,
  listenForHotkeyEvents,
  openSettingsWindow,
  providerApiKeyStatus,
  readSelectedText,
  replaceSelectedText,
  saveSettings,
  setProviderApiKey,
  setAutostart,
  speakText,
  type AppSettings,
  type DesktopCapabilities,
  type FileContext,
  type HistoryEntry,
  type ProviderInfo,
  type SecretStatus,
} from "../utils/tauri";

type AssistantMode = "conversation" | "edit" | "dictation" | "feedback";
type AssistantStatus = "ready" | "listening" | "transcribing" | "thinking" | "writing" | "error";

type PendingReplacement = {
  original: string;
  replacement: string;
  instruction: string;
};

const modes: Record<
  AssistantMode,
  {
    label: string;
    shortLabel: string;
    description: string;
    icon: typeof MessageSquareText;
  }
> = {
  conversation: {
    label: "Gespräch",
    shortLabel: "Gespräch",
    description: "Fragen stellen und Antworten im Overlay erhalten.",
    icon: MessageSquareText,
  },
  edit: {
    label: "Text bearbeiten",
    shortLabel: "Text",
    description: "Markierten Text später lesen, verbessern und ersetzen.",
    icon: PencilLine,
  },
  dictation: {
    label: "Diktieren",
    shortLabel: "Diktat",
    description: "Gesprochenen Text später direkt einfügen.",
    icon: Keyboard,
  },
  feedback: {
    label: "Feedback",
    shortLabel: "Feedback",
    description: "Feedback lokal sammeln und für Verbesserungen speichern.",
    icon: Bot,
  },
};

const statusLabels: Record<AssistantStatus, string> = {
  ready: "Bereit",
  listening: "Höre zu",
  transcribing: "Transkribiere",
  thinking: "Denke",
  writing: "Schreibe",
  error: "Fehler",
};

export function App() {
  const view = new URLSearchParams(window.location.search).get("view");

  if (view === "settings") {
    return <SettingsWindow />;
  }

  return <OverlayApp />;
}

function OverlayApp() {
  const [mode, setMode] = useState<AssistantMode>("conversation");
  const [status, setStatus] = useState<AssistantStatus>("ready");
  const [menuOpen, setMenuOpen] = useState(false);
  const [capabilities, setCapabilities] = useState<DesktopCapabilities>({
    sessionType: "unknown",
    hotkeysSupported: false,
    audioSupported: false,
    automationBackend: "unknown",
  });
  const [eventMessage, setEventMessage] = useState("Phase 2 startet Hotkeys und Audioaufnahme.");
  const [lastAudioPath, setLastAudioPath] = useState<string | null>(null);
  const [lastTranscript, setLastTranscript] = useState<string | null>(null);
  const [lastProvider, setLastProvider] = useState<string | null>(null);
  const [assistantAnswer, setAssistantAnswer] = useState<string | null>(null);
  const [pendingReplacement, setPendingReplacement] = useState<PendingReplacement | null>(null);
  const [fileContext, setFileContext] = useState<FileContext | null>(null);
  const modeRef = useRef(mode);
  const activeMode = modes[mode];
  const ActiveIcon = activeMode.icon;

  useEffect(() => {
    modeRef.current = mode;
  }, [mode]);

  useEffect(() => {
    let mounted = true;

    getDesktopCapabilities()
      .then((value) => {
        if (mounted) {
          setCapabilities(value);
          setEventMessage(value.warning ?? "Ctrl+Space halten zum Aufnehmen, doppelt tippen zum Moduswechsel.");
        }
      })
      .catch(() => {
        if (mounted) {
          setEventMessage("Desktop-Fähigkeiten konnten nicht gelesen werden.");
        }
      });

    return () => {
      mounted = false;
    };
  }, []);

  useEffect(() => {
    let unlisten: Awaited<ReturnType<typeof listenForHotkeyEvents>> | undefined;
    let mounted = true;

    listenForHotkeyEvents((event) => {
      if (!mounted) {
        return;
      }

      if (event.kind === "mode-cycle") {
        const modeKeys = Object.keys(modes) as AssistantMode[];
        setMode((currentMode) => {
          const currentIndex = modeKeys.indexOf(currentMode);
          return modeKeys[(currentIndex + 1) % modeKeys.length] ?? "conversation";
        });
      }

      setStatus(event.status);
      setEventMessage(event.message);

      if (event.audioPath) {
        setLastAudioPath(event.audioPath);
      }

      if (event.transcript) {
        setLastTranscript(event.transcript);
      }

      if (event.provider) {
        setLastProvider(event.provider);
      }

      if (event.kind === "transcript-ready" && event.transcript) {
        if (modeRef.current === "conversation") {
          setStatus("thinking");
          setEventMessage("Ollama denkt...");
          setAssistantAnswer(null);

          askAssistant(buildConversationPrompt(event.transcript, fileContext))
            .then((answer) => {
              if (!mounted) {
                return;
              }

              setStatus("ready");
              setEventMessage("Antwort bereit.");
              setAssistantAnswer(answer);

              getSettings()
                .then((appSettings) => {
                  if (appSettings.ttsEnabled) {
                    return speakText(answer).catch(() => undefined);
                  }
                  return undefined;
                })
                .catch(() => undefined);
            })
            .catch((error: unknown) => {
              if (!mounted) {
                return;
              }

              setStatus("error");
              setEventMessage(error instanceof Error ? error.message : String(error));
            });
        } else if (modeRef.current === "edit") {
          setStatus("thinking");
          setEventMessage("Lese Auswahl und bereite Textvorschau vor...");
          setPendingReplacement(null);

          readSelectedText()
            .then((selectedText) =>
              askAssistant(
                [
                  "Bearbeite den folgenden markierten Text gemaess Anweisung.",
                  "Gib ausschliesslich den finalen Ersatztext aus, ohne Erklaerung.",
                  `Anweisung: ${event.transcript}`,
                  "Markierter Text:",
                  selectedText,
                ].join("\n\n"),
              ).then((replacement) => ({ selectedText, replacement })),
            )
            .then(({ selectedText, replacement }) => {
              if (!mounted) {
                return;
              }

              setStatus("ready");
              setEventMessage("Textvorschau bereit. Bitte bestaetigen.");
              setPendingReplacement({
                original: selectedText,
                replacement,
                instruction: event.transcript ?? "",
              });
            })
            .catch((error: unknown) => {
              if (!mounted) {
                return;
              }

              setStatus("error");
              setEventMessage(error instanceof Error ? error.message : String(error));
            });
        } else if (modeRef.current === "dictation") {
          setStatus("writing");
          setEventMessage("Fuege Diktat ein...");

          insertText(event.transcript)
            .then(() => {
              if (!mounted) {
                return;
              }

              setStatus("ready");
              setEventMessage("Diktat eingefuegt.");
            })
            .catch((error: unknown) => {
              if (!mounted) {
                return;
              }

              setStatus("error");
              setEventMessage(error instanceof Error ? error.message : String(error));
            });
        } else if (modeRef.current === "feedback") {
          setEventMessage("Du bist gerade im Feedbackmodus. Wechsle den Modus per Doppeltipp oder ueber das Menue.");
        }
      }
    }).then((cleanup) => {
      unlisten = cleanup;
    });

    return () => {
      mounted = false;
      unlisten?.();
    };
  }, []);

  const statusPreview = useMemo(() => {
    if (mode === "edit") {
      return "Textvorschau folgt in Phase 6";
    }

    if (mode === "dictation") {
      return "Diktat startet ab Phase 5";
    }

    if (mode === "feedback") {
      return "Feedbackspeicher folgt ab Phase 4";
    }

    return "KI-Antworten folgen ab Phase 4";
  }, [mode]);

  const cycleStatus = () => {
    const order: AssistantStatus[] = ["ready", "listening", "transcribing", "thinking", "writing"];
    const currentIndex = order.indexOf(status);
    setStatus(order[(currentIndex + 1) % order.length] ?? "ready");
  };

  return (
    <main className="overlay-shell" onContextMenu={(event) => event.preventDefault()}>
      <button className="pill" type="button" onClick={() => setMenuOpen((open) => !open)} onDoubleClick={openSettingsWindow}>
        <span className="mode-icon" aria-hidden="true">
          <ActiveIcon size={18} strokeWidth={2.2} />
        </span>
        <span className="mode-label">{activeMode.shortLabel}</span>
        <span className="separator" />
        <span className="status-label">{statusLabels[status]}</span>
        <ChevronDown className={menuOpen ? "chevron open" : "chevron"} size={16} aria-hidden="true" />
      </button>

      {menuOpen ? (
        <section className="menu-panel" aria-label="TwoKey menu">
          <div className="menu-header">
            <div>
              <p className="eyebrow">TwoKey</p>
              <h1>Linux AI Assistant</h1>
            </div>
            <button className="icon-button" type="button" onClick={() => setMenuOpen(false)} aria-label="Menü schließen">
              <X size={16} />
            </button>
          </div>

          <div className="mode-list">
            {(Object.keys(modes) as AssistantMode[]).map((modeKey) => {
              const item = modes[modeKey];
              const ItemIcon = item.icon;
              const active = modeKey === mode;

              return (
                <button
                  className={active ? "mode-item active" : "mode-item"}
                  key={modeKey}
                  type="button"
                  onClick={() => {
                    setMode(modeKey);
                    setStatus("ready");
                    setMenuOpen(false);
                  }}
                >
                  <ItemIcon size={18} aria-hidden="true" />
                  <span>
                    <strong>{item.label}</strong>
                    <small>{item.description}</small>
                  </span>
                  {active ? <Check size={16} aria-hidden="true" /> : null}
                </button>
              );
            })}
          </div>

          <div className="menu-actions">
            <button type="button" onClick={openSettingsWindow}>
              <Settings size={16} aria-hidden="true" />
              Einstellungen
            </button>
            <button
              type="button"
              onClick={() => {
                setEventMessage("Oeffne Dateiauswahl...");
                addFileContext()
                  .then((context) => {
                    setFileContext(context);
                    setEventMessage(context.summary);
                  })
                  .catch((error: unknown) => setEventMessage(error instanceof Error ? error.message : String(error)));
              }}
            >
              <FilePlus2 size={16} aria-hidden="true" />
              Datei hinzufügen
            </button>
            <button type="button" onClick={cycleStatus}>
              <Mic size={16} aria-hidden="true" />
              Status testen
            </button>
          </div>

          <div className="system-note">
            <ShieldAlert size={16} aria-hidden="true" />
            <span>
              Session: <strong>{capabilities.sessionType}</strong>. Backend: <strong>{capabilities.automationBackend}</strong>.
              Hotkeys: <strong>{capabilities.hotkeysSupported ? "aktiv" : "nicht verfügbar"}</strong>. Audio:{" "}
              <strong>{capabilities.audioSupported ? "bereit" : "nicht verfügbar"}</strong>.
            </span>
          </div>

          <p className="event-text">{eventMessage}</p>
          {lastTranscript ? (
            <div className="transcript-box">
              <strong>Transkript{lastProvider ? ` (${lastProvider})` : ""}</strong>
              <p>{lastTranscript}</p>
            </div>
          ) : null}
          {assistantAnswer ? (
            <div className="answer-box">
              <strong>Antwort</strong>
              <p>{assistantAnswer}</p>
            </div>
          ) : null}
          {fileContext ? (
            <div className="file-context-box">
              <strong>{fileContext.name}</strong>
              <span>{fileContext.summary}</span>
            </div>
          ) : null}
          {pendingReplacement ? (
            <div className="replacement-box">
              <strong>Text ersetzen?</strong>
              <small>{pendingReplacement.instruction}</small>
              <p>{pendingReplacement.replacement}</p>
              <div className="replacement-actions">
                <button
                  type="button"
                  onClick={() => {
                    setStatus("writing");
                    setEventMessage("Ersetze markierten Text...");
                    replaceSelectedText(pendingReplacement.replacement)
                      .then(() => {
                        setStatus("ready");
                        setEventMessage("Markierter Text ersetzt.");
                        setPendingReplacement(null);
                      })
                      .catch((error: unknown) => {
                        setStatus("error");
                        setEventMessage(error instanceof Error ? error.message : String(error));
                      });
                  }}
                >
                  Ersetzen
                </button>
                <button type="button" onClick={() => setPendingReplacement(null)}>
                  Verwerfen
                </button>
              </div>
            </div>
          ) : null}
          {lastAudioPath ? <p className="path-text">{lastAudioPath}</p> : null}
          <p className="preview-text">{statusPreview}</p>
        </section>
      ) : null}
    </main>
  );
}

function buildConversationPrompt(transcript: string, context: FileContext | null) {
  if (!context) {
    return transcript;
  }

  const contextText = context.extractedText
    ? ["Dateikontext:", context.name, context.extractedText].join("\n\n")
    : ["Dateikontext:", context.name, context.summary, "Diese Datei ist fuer spaetere Vision-Provider vorgemerkt."].join("\n\n");

  return [contextText, "Nutzerfrage:", transcript].join("\n\n");
}

function SettingsWindow() {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [providers, setProviders] = useState<ProviderInfo[]>([]);
  const [secretStatus, setSecretStatus] = useState<Record<string, SecretStatus>>({});
  const [apiKeyInputs, setApiKeyInputs] = useState<Record<string, string>>({
    "openai-compatible": "",
    openrouter: "",
  });
  const [historyEntries, setHistoryEntries] = useState<HistoryEntry[]>([]);
  const [saveState, setSaveState] = useState("Bereit");
  const [updateState, setUpdateState] = useState("Nicht geprueft");
  const settingsSections = [
    "Allgemein",
    "Hotkeys",
    "Sprache",
    "Sprachausgabe",
    "KI-Provider",
    "Datenschutz",
    "Updates",
  ];

  useEffect(() => {
    getSettings()
      .then(setSettings)
      .catch((error: unknown) => setSaveState(error instanceof Error ? error.message : String(error)));
    listProviders().then(setProviders).catch(() => setProviders([]));
    Promise.all([providerApiKeyStatus("openai-compatible"), providerApiKeyStatus("openrouter")])
      .then(([openai, openrouter]) => {
        setSecretStatus({
          [openai.providerId]: openai,
          [openrouter.providerId]: openrouter,
        });
      })
      .catch(() => setSecretStatus({}));
    getHistoryRecent(25).then(setHistoryEntries).catch(() => setHistoryEntries([]));
  }, []);

  const updateSetting = <Key extends keyof AppSettings>(key: Key, value: AppSettings[Key]) => {
    if (!settings) {
      return;
    }

    const nextSettings = { ...settings, [key]: value };
    setSettings(nextSettings);
    setSaveState("Speichere...");
    const sideEffect = key === "autostart" ? setAutostart(Boolean(value)) : Promise.resolve();
    sideEffect
      .then(() => saveSettings(nextSettings))
      .then(() => setSaveState("Gespeichert"))
      .catch((error: unknown) => setSaveState(error instanceof Error ? error.message : String(error)));
  };

  const saveProviderKey = (providerId: "openai-compatible" | "openrouter") => {
    const value = (apiKeyInputs[providerId] ?? "").trim();
    if (!value) {
      setSaveState("API-Key ist leer");
      return;
    }

    setSaveState("Speichere API-Key...");
    setProviderApiKey(providerId, value)
      .then(() => providerApiKeyStatus(providerId))
      .then((status) => {
        setSecretStatus((current) => ({ ...current, [providerId]: status }));
        setApiKeyInputs((current) => ({ ...current, [providerId]: "" }));
        return listProviders();
      })
      .then(setProviders)
      .then(() => setSaveState("API-Key sicher gespeichert"))
      .catch((error: unknown) => setSaveState(error instanceof Error ? error.message : String(error)));
  };

  const removeProviderKey = (providerId: "openai-compatible" | "openrouter") => {
    setSaveState("Entferne API-Key...");
    clearProviderApiKey(providerId)
      .then(() => providerApiKeyStatus(providerId))
      .then((status) => {
        setSecretStatus((current) => ({ ...current, [providerId]: status }));
        return listProviders();
      })
      .then(setProviders)
      .then(() => setSaveState("API-Key entfernt"))
      .catch((error: unknown) => setSaveState(error instanceof Error ? error.message : String(error)));
  };

  if (!settings) {
    return (
      <main className="settings-shell loading">
        <p>Settings werden geladen...</p>
      </main>
    );
  }

  return (
    <main className="settings-shell">
      <aside className="settings-nav">
        <div className="settings-brand">
          <span className="brand-dot" />
          <div>
            <p>TwoKey</p>
            <strong>Einstellungen</strong>
          </div>
        </div>

        {settingsSections.map((section) => (
          <button key={section} type="button" className={section === "Allgemein" ? "active" : ""}>
            {section}
          </button>
        ))}
      </aside>

      <section className="settings-content">
        <div className="settings-title">
          <div>
            <p className="eyebrow">Phase 7</p>
            <h1>Einstellungen</h1>
          </div>
          <span>{saveState}</span>
        </div>

        <div className="settings-form">
          <SettingsGroup title="Allgemein">
            <label>
              <span>Autostart</span>
              <input type="checkbox" checked={settings.autostart} onChange={(event) => updateSetting("autostart", event.target.checked)} />
            </label>
            <label>
              <span>Overlay-Position</span>
              <select value={settings.overlayPosition} onChange={(event) => updateSetting("overlayPosition", event.target.value)}>
                <option value="top-left">Oben links</option>
                <option value="top-right">Oben rechts</option>
                <option value="bottom-left">Unten links</option>
                <option value="bottom-right">Unten rechts</option>
              </select>
            </label>
            <label>
              <span>Theme</span>
              <select value={settings.theme} onChange={(event) => updateSetting("theme", event.target.value)}>
                <option value="dark">Dunkel</option>
                <option value="light">Hell</option>
                <option value="system">System</option>
              </select>
            </label>
          </SettingsGroup>

          <SettingsGroup title="Hotkeys">
            <label>
              <span>Haupt-Hotkey</span>
              <input value={settings.mainHotkey} onChange={(event) => updateSetting("mainHotkey", event.target.value)} />
            </label>
            <label>
              <span>Doppeltipp ms</span>
              <input
                type="number"
                min="200"
                max="1200"
                value={settings.doubleTapMs}
                onChange={(event) => updateSetting("doubleTapMs", Number(event.target.value))}
              />
            </label>
            <label>
              <span>Escape-Abbruch</span>
              <input type="checkbox" checked={settings.escapeCancel} onChange={(event) => updateSetting("escapeCancel", event.target.checked)} />
            </label>
          </SettingsGroup>

          <SettingsGroup title="Sprache und KI">
            <label>
              <span>STT-Anbieter</span>
              <select value={settings.sttProvider} onChange={(event) => updateSetting("sttProvider", event.target.value)}>
                <option value="mock">Mock</option>
                <option value="external-command">Externer Befehl</option>
                <option value="local-whisper">Lokal Whisper</option>
              </select>
            </label>
            <label>
              <span>Chat-Provider</span>
              <select value={settings.preferredChatProvider} onChange={(event) => updateSetting("preferredChatProvider", event.target.value)}>
                <option value="ollama">Ollama lokal</option>
                <option value="openai-compatible">OpenAI-kompatibel</option>
                <option value="openrouter">OpenRouter</option>
              </select>
            </label>
            <label>
              <span>Ollama-Modell</span>
              <input value={settings.ollamaModel} onChange={(event) => updateSetting("ollamaModel", event.target.value)} />
            </label>
            <label>
              <span>OpenAI Base URL</span>
              <input value={settings.openaiBaseUrl} onChange={(event) => updateSetting("openaiBaseUrl", event.target.value)} />
            </label>
            <label>
              <span>OpenAI Modell</span>
              <input value={settings.openaiModel} onChange={(event) => updateSetting("openaiModel", event.target.value)} />
            </label>
            <label>
              <span>OpenRouter Base URL</span>
              <input value={settings.openrouterBaseUrl} onChange={(event) => updateSetting("openrouterBaseUrl", event.target.value)} />
            </label>
            <label>
              <span>OpenRouter Modell</span>
              <input value={settings.openrouterModel} onChange={(event) => updateSetting("openrouterModel", event.target.value)} />
            </label>
            <label>
              <span>Lokal bevorzugen</span>
              <input type="checkbox" checked={settings.preferLocal} onChange={(event) => updateSetting("preferLocal", event.target.checked)} />
            </label>
            <label>
              <span>TTS aktiv</span>
              <input type="checkbox" checked={settings.ttsEnabled} onChange={(event) => updateSetting("ttsEnabled", event.target.checked)} />
            </label>
            <label>
              <span>TTS Stimme</span>
              <input value={settings.ttsVoice} onChange={(event) => updateSetting("ttsVoice", event.target.value)} />
            </label>
            <label>
              <span>TTS Tempo</span>
              <input
                type="number"
                min="0.5"
                max="2.0"
                step="0.1"
                value={settings.ttsSpeed}
                onChange={(event) => updateSetting("ttsSpeed", Number(event.target.value))}
              />
            </label>
            <div className="provider-row">
              <strong>OpenAI API-Key</strong>
              <span>{secretStatus["openai-compatible"]?.configured ? "gespeichert" : "nicht gesetzt"}</span>
              <input
                type="password"
                placeholder="sk-..."
                value={apiKeyInputs["openai-compatible"] ?? ""}
                onChange={(event) =>
                  setApiKeyInputs((current) => ({ ...current, "openai-compatible": event.target.value }))
                }
              />
              <div className="replacement-actions">
                <button type="button" onClick={() => saveProviderKey("openai-compatible")}>Speichern</button>
                <button type="button" onClick={() => removeProviderKey("openai-compatible")}>Loeschen</button>
              </div>
            </div>
            <div className="provider-row">
              <strong>OpenRouter API-Key</strong>
              <span>{secretStatus.openrouter?.configured ? "gespeichert" : "nicht gesetzt"}</span>
              <input
                type="password"
                placeholder="sk-or-..."
                value={apiKeyInputs.openrouter ?? ""}
                onChange={(event) => setApiKeyInputs((current) => ({ ...current, openrouter: event.target.value }))}
              />
              <div className="replacement-actions">
                <button type="button" onClick={() => saveProviderKey("openrouter")}>Speichern</button>
                <button type="button" onClick={() => removeProviderKey("openrouter")}>Loeschen</button>
              </div>
            </div>
            <div className="provider-list">
              {providers.map((provider) => (
                <div className="provider-row" key={provider.id}>
                  <strong>{provider.label}</strong>
                  <span>{provider.enabled ? "aktiv" : "geplant"} · {provider.kind}</span>
                  <small>{provider.note}</small>
                </div>
              ))}
            </div>
          </SettingsGroup>

          <SettingsGroup title="Datenschutz und Updates">
            <label>
              <span>Verlauf speichern</span>
              <input type="checkbox" checked={settings.saveHistory} onChange={(event) => updateSetting("saveHistory", event.target.checked)} />
            </label>
            <label>
              <span>API-Anfragen protokollieren</span>
              <input
                type="checkbox"
                checked={settings.logApiRequests}
                onChange={(event) => updateSetting("logApiRequests", event.target.checked)}
              />
            </label>
            <label>
              <span>Tray-Menue aktiv</span>
              <input type="checkbox" checked={settings.trayEnabled} onChange={(event) => updateSetting("trayEnabled", event.target.checked)} />
            </label>
            <label>
              <span>Update-Kanal</span>
              <select value={settings.updateChannel} onChange={(event) => updateSetting("updateChannel", event.target.value)}>
                <option value="stable">stable</option>
                <option value="beta">beta</option>
                <option value="dev">dev</option>
              </select>
            </label>
            <div className="update-check">
              <button
                type="button"
                onClick={() => {
                  setUpdateState("Pruefe...");
                  checkForUpdates()
                    .then((status) => setUpdateState(status.message))
                    .catch((error: unknown) => setUpdateState(error instanceof Error ? error.message : String(error)));
                }}
              >
                Nach Updates suchen
              </button>
              <span>{updateState}</span>
            </div>
            <div className="provider-list">
              {historyEntries.map((entry) => (
                <div className="provider-row" key={entry.id}>
                  <strong>
                    {entry.kind} {entry.success ? "ok" : "fehler"}
                  </strong>
                  <span>{entry.provider ?? "-"}</span>
                  <small>{new Date(entry.tsUnixMs).toLocaleString()}</small>
                </div>
              ))}
            </div>
          </SettingsGroup>
        </div>
      </section>
    </main>
  );
}

function SettingsGroup({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="setting-card">
      <h2>{title}</h2>
      <div className="setting-fields">{children}</div>
    </section>
  );
}
