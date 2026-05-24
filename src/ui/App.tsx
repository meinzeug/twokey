import { useEffect, useMemo, useRef, useState } from "react";
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
  askOllama,
  getDesktopCapabilities,
  insertText,
  listenForHotkeyEvents,
  openSettingsWindow,
  readSelectedText,
  replaceSelectedText,
  type DesktopCapabilities,
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

          askOllama(event.transcript)
            .then((answer) => {
              if (!mounted) {
                return;
              }

              setStatus("ready");
              setEventMessage("Ollama-Antwort bereit.");
              setAssistantAnswer(answer);
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
              askOllama(
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
            <button type="button" disabled>
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

function SettingsWindow() {
  const settingsSections = [
    "Allgemein",
    "Hotkeys",
    "Sprache",
    "Sprachausgabe",
    "KI-Provider",
    "Datenschutz",
    "Updates",
  ];

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
            <p className="eyebrow">Phase 4 Platzhalter</p>
            <h1>Ollama-Gespräch ist vorbereitet</h1>
          </div>
          <span>v0.1.0</span>
        </div>

        <div className="settings-grid">
          <SettingCard title="Overlay" value="Pille, dunkles Theme, Modusmenü" />
          <SettingCard title="Hotkeys" value="Ctrl+Space ist der erste X11-Hold-Hotkey. Wayland wird explizit begrenzt gemeldet." />
          <SettingCard title="Diktat" value="Diktiermodus fuegt Transkripte unter X11 per Clipboard und xdotool ein." />
          <SettingCard title="Text bearbeiten" value="Markierten Text lesen, mit Ollama umformulieren und erst nach Vorschau ersetzen." />
          <SettingCard title="Provider" value="Ollama laeuft lokal mit qwen2.5:3b. OpenAI-kompatible APIs folgen spaeter." />
          <SettingCard title="Datenschutz" value="XDG-Pfade, lokale Defaults und externe Warnungen geplant" />
        </div>
      </section>
    </main>
  );
}

function SettingCard({ title, value }: { title: string; value: string }) {
  return (
    <article className="setting-card">
      <h2>{title}</h2>
      <p>{value}</p>
    </article>
  );
}
