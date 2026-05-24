import { useEffect, useMemo, useState } from "react";
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
import { getDesktopSessionType, openSettingsWindow } from "../utils/tauri";

type AssistantMode = "conversation" | "edit" | "dictation" | "feedback";
type AssistantStatus = "ready" | "listening" | "transcribing" | "thinking" | "writing" | "error";

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
  const [sessionType, setSessionType] = useState("unknown");
  const activeMode = modes[mode];
  const ActiveIcon = activeMode.icon;

  useEffect(() => {
    let mounted = true;

    getDesktopSessionType()
      .then((value) => {
        if (mounted) {
          setSessionType(value);
        }
      })
      .catch(() => {
        if (mounted) {
          setSessionType("browser-preview");
        }
      });

    return () => {
      mounted = false;
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
              Session: <strong>{sessionType}</strong>. Echte Hotkeys, Audio und Desktop-Automation folgen in Phase 2.
            </span>
          </div>

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
            <p className="eyebrow">Phase 1 Platzhalter</p>
            <h1>Grundlagen sind vorbereitet</h1>
          </div>
          <span>v0.1.0</span>
        </div>

        <div className="settings-grid">
          <SettingCard title="Overlay" value="Pille, dunkles Theme, Modusmenü" />
          <SettingCard title="Hotkeys" value="Geplant für Phase 2 mit X11/Wayland-Prüfung" />
          <SettingCard title="Provider" value="Ollama und OpenAI-kompatible APIs ab späteren Phasen" />
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
