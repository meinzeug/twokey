use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::{audio::AudioRecorder, stt};

const HOLD_DELAY: Duration = Duration::from_millis(180);
const DOUBLE_TAP_WINDOW: Duration = Duration::from_millis(420);
const POLL_INTERVAL: Duration = Duration::from_millis(24);

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopCapabilities {
    pub session_type: String,
    pub hotkeys_supported: bool,
    pub audio_supported: bool,
    pub automation_backend: String,
    pub warning: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct HotkeyEvent {
    kind: &'static str,
    status: &'static str,
    message: String,
    audio_path: Option<String>,
    transcript: Option<String>,
    provider: Option<String>,
}

pub fn capabilities() -> DesktopCapabilities {
    let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "unknown".to_string());
    let audio_supported = has_command("pw-record") || has_command("parec") || has_command("arecord");

    match session_type.as_str() {
        "x11" => DesktopCapabilities {
            session_type,
            hotkeys_supported: true,
            audio_supported,
            automation_backend: "x11-query-keymap".to_string(),
            warning: None,
        },
        "wayland" => DesktopCapabilities {
            session_type,
            hotkeys_supported: false,
            audio_supported,
            automation_backend: if has_command("wtype") || has_command("ydotool") {
                "wayland-typed-insert".to_string()
            } else {
                "wayland-limited".to_string()
            },
            warning: Some(
                "Wayland blockiert generische globale Hold-Hotkeys. Einfuegen kann ueber wtype/ydotool funktionieren, globale Hold-Hotkeys bleiben eingeschraenkt."
                    .to_string(),
            ),
        },
        _ => DesktopCapabilities {
            session_type,
            hotkeys_supported: false,
            audio_supported,
            automation_backend: "unknown".to_string(),
            warning: Some("Desktop-Sitzung konnte nicht sicher erkannt werden.".to_string()),
        },
    }
}

pub fn start_hotkey_service(app: &tauri::App) {
    let caps = capabilities();
    let handle = app.handle().clone();

    let _ = handle.emit(
        "twokey://hotkey-event",
        HotkeyEvent {
            kind: "capabilities",
            status: "ready",
            message: capability_message(&caps),
            audio_path: None,
            transcript: None,
            provider: None,
        },
    );

    if !caps.hotkeys_supported {
        return;
    }

    let recorder = app.state::<Arc<Mutex<AudioRecorder>>>().inner().clone();

    thread::spawn(move || {
        if let Err(error) = run_x11_loop(handle.clone(), recorder) {
            emit_event(
                &handle,
                HotkeyEvent {
                    kind: "error",
                    status: "error",
                    message: error,
                    audio_path: None,
                    transcript: None,
                    provider: None,
                },
            );
        }
    });
}

fn run_x11_loop(app: AppHandle, recorder: Arc<Mutex<AudioRecorder>>) -> Result<(), String> {
    let keymap = X11Keymap::open()?;
    let mut was_down = false;
    let mut recording = false;
    let mut press_started: Option<Instant> = None;
    let mut last_tap: Option<Instant> = None;

    loop {
        let combo_down = keymap.ctrl_space_down();
        let escape_down = keymap.escape_down();
        let now = Instant::now();

        if escape_down && recording {
            if let Ok(mut recorder) = recorder.lock() {
                let _ = recorder.cancel();
            }
            recording = false;
            press_started = None;
            emit_event(
                &app,
                HotkeyEvent {
                    kind: "cancelled",
                    status: "ready",
                    message: "Audioaufnahme abgebrochen".to_string(),
                    audio_path: None,
                    transcript: None,
                    provider: None,
                },
            );
        }

        if combo_down && !was_down {
            press_started = Some(now);
        }

        if combo_down && !recording && press_started.is_some_and(|started| now.duration_since(started) >= HOLD_DELAY) {
            match recorder.lock().map_err(|_| "Recorder-State ist gesperrt".to_string())?.start() {
                Ok(path) => {
                    recording = true;
                    emit_event(
                        &app,
                        HotkeyEvent {
                            kind: "recording-started",
                            status: "listening",
                            message: "Höre zu... Loslassen verarbeitet die Aufnahme.".to_string(),
                            audio_path: Some(path.to_string_lossy().to_string()),
                            transcript: None,
                            provider: None,
                        },
                    );
                }
                Err(error) => {
                    press_started = None;
                    emit_event(
                        &app,
                        HotkeyEvent {
                            kind: "error",
                            status: "error",
                            message: error,
                            audio_path: None,
                            transcript: None,
                            provider: None,
                        },
                    );
                }
            }
        }

        if !combo_down && was_down {
            if recording {
                let stopped_path = recorder
                    .lock()
                    .map_err(|_| "Recorder-State ist gesperrt".to_string())?
                    .stop()?;
                recording = false;

                if let Some(path) = stopped_path {
                    emit_event(
                        &app,
                        HotkeyEvent {
                            kind: "recording-stopped",
                            status: "transcribing",
                            message: "Audioaufnahme gespeichert. Transkribiere...".to_string(),
                            audio_path: Some(path.to_string_lossy().to_string()),
                            transcript: None,
                            provider: None,
                        },
                    );

                    start_transcription(app.clone(), path);
                }
            } else if press_started.is_some_and(|started| now.duration_since(started) < HOLD_DELAY) {
                if last_tap.is_some_and(|tap| now.duration_since(tap) <= DOUBLE_TAP_WINDOW) {
                    last_tap = None;
                    emit_event(
                        &app,
                        HotkeyEvent {
                            kind: "mode-cycle",
                            status: "ready",
                            message: "Doppeltipp erkannt: nächster Modus".to_string(),
                            audio_path: None,
                            transcript: None,
                            provider: None,
                        },
                    );
                } else {
                    last_tap = Some(now);
                }
            }

            press_started = None;
        }

        was_down = combo_down;
        thread::sleep(POLL_INTERVAL);
    }
}

fn start_transcription(app: AppHandle, path: std::path::PathBuf) {
    thread::spawn(move || match stt::transcribe(&path) {
        Ok(transcript) => emit_event(
            &app,
            HotkeyEvent {
                kind: "transcript-ready",
                status: "ready",
                message: "Transkription abgeschlossen. KI-Verarbeitung folgt in Phase 4.".to_string(),
                audio_path: Some(path.to_string_lossy().to_string()),
                transcript: Some(transcript.text),
                provider: Some(transcript.provider),
            },
        ),
        Err(error) => emit_event(
            &app,
            HotkeyEvent {
                kind: "error",
                status: "error",
                message: error,
                audio_path: Some(path.to_string_lossy().to_string()),
                transcript: None,
                provider: None,
            },
        ),
    });
}

fn capability_message(caps: &DesktopCapabilities) -> String {
    if let Some(warning) = &caps.warning {
        return warning.clone();
    }

    format!(
        "Hotkey bereit: Ctrl+Space halten zum Aufnehmen, doppelt tippen zum Moduswechsel. Backend: {}.",
        caps.automation_backend
    )
}

fn emit_event(app: &AppHandle, event: HotkeyEvent) {
    eprintln!("twokey event: {} - {}", event.kind, event.message);
    let _ = app.emit("twokey://hotkey-event", event);
}

fn has_command(name: &str) -> bool {
    std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {name} >/dev/null 2>&1"))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

struct X11Keymap {
    display: *mut x11::xlib::Display,
    ctrl_left: u32,
    ctrl_right: u32,
    space: u32,
    escape: u32,
}

unsafe impl Send for X11Keymap {}

impl X11Keymap {
    fn open() -> Result<Self, String> {
        unsafe {
            let display = x11::xlib::XOpenDisplay(std::ptr::null());
            if display.is_null() {
                return Err("X11-Display konnte nicht geöffnet werden".to_string());
            }

            Ok(Self {
                display,
                ctrl_left: keycode(display, x11::keysym::XK_Control_L),
                ctrl_right: keycode(display, x11::keysym::XK_Control_R),
                space: keycode(display, x11::keysym::XK_space),
                escape: keycode(display, x11::keysym::XK_Escape),
            })
        }
    }

    fn ctrl_space_down(&self) -> bool {
        let map = self.query_keymap();
        (key_is_down(&map, self.ctrl_left) || key_is_down(&map, self.ctrl_right)) && key_is_down(&map, self.space)
    }

    fn escape_down(&self) -> bool {
        let map = self.query_keymap();
        key_is_down(&map, self.escape)
    }

    fn query_keymap(&self) -> [i8; 32] {
        let mut keys = [0_i8; 32];
        unsafe {
            x11::xlib::XQueryKeymap(self.display, keys.as_mut_ptr());
        }
        keys
    }
}

impl Drop for X11Keymap {
    fn drop(&mut self) {
        unsafe {
            x11::xlib::XCloseDisplay(self.display);
        }
    }
}

unsafe fn keycode(display: *mut x11::xlib::Display, keysym: u32) -> u32 {
    x11::xlib::XKeysymToKeycode(display, keysym.into()).into()
}

fn key_is_down(map: &[i8; 32], keycode: u32) -> bool {
    if keycode == 0 {
        return false;
    }

    let byte_index = (keycode / 8) as usize;
    let bit_index = keycode % 8;

    map.get(byte_index)
        .map(|byte| (*byte as u8 & (1 << bit_index)) != 0)
        .unwrap_or(false)
}
