use std::{
    ffi::CString,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::{audio::AudioRecorder, settings, stt};

const HOLD_DELAY: Duration = Duration::from_millis(180);
const DOUBLE_TAP_WINDOW: Duration = Duration::from_millis(420);
const POLL_INTERVAL: Duration = Duration::from_millis(24);
const HOTKEY_RELOAD_INTERVAL: Duration = Duration::from_millis(1200);

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

#[derive(Clone, Debug, PartialEq)]
enum HotkeyTrigger {
    Key(u32),
    MouseButton(u32),
}

#[derive(Clone, Debug, PartialEq)]
struct HotkeySpec {
    ctrl: bool,
    alt: bool,
    shift: bool,
    meta: bool,
    trigger: HotkeyTrigger,
    label: String,
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
    let mut spec = load_hotkey_spec(&keymap);
    let mut last_reload = Instant::now();
    let mut was_down = false;
    let mut recording = false;
    let mut press_started: Option<Instant> = None;
    let mut last_tap: Option<Instant> = None;

    loop {
        let now = Instant::now();
        if last_tap.is_some_and(|tap| now.duration_since(tap) > DOUBLE_TAP_WINDOW) {
            last_tap = None;
            emit_event(
                &app,
                HotkeyEvent {
                    kind: "file-context-pick",
                    status: "ready",
                    message: "Einzeltipp erkannt: Dateikontext auswaehlen".to_string(),
                    audio_path: None,
                    transcript: None,
                    provider: None,
                },
            );
        }

        if now.duration_since(last_reload) >= HOTKEY_RELOAD_INTERVAL {
            last_reload = now;
            let reloaded = load_hotkey_spec(&keymap);
            if reloaded != spec {
                spec = reloaded;
                emit_event(
                    &app,
                    HotkeyEvent {
                        kind: "hotkey-updated",
                        status: "ready",
                        message: format!("Hotkey aktualisiert: {}", spec.label),
                        audio_path: None,
                        transcript: None,
                        provider: None,
                    },
                );
            }
        }

        let state = keymap.query_state();
        let combo_down = keymap.hotkey_down(&state, &spec);
        let escape_down = keymap.escape_down(&state);

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
                message: "Transkription abgeschlossen. Starte KI-Verarbeitung.".to_string(),
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

    let hotkey_label = settings::load()
        .map(|loaded| loaded.main_hotkey)
        .unwrap_or_else(|_| "Ctrl+Space".to_string());

    format!(
        "Hotkey bereit: {} halten zum Aufnehmen, doppelt tippen zum Moduswechsel. Backend: {}.",
        hotkey_label, caps.automation_backend
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

fn load_hotkey_spec(keymap: &X11Keymap) -> HotkeySpec {
    let from_settings = settings::load().ok().map(|loaded| loaded.main_hotkey);
    let raw = from_settings.unwrap_or_else(|| "Ctrl+Space".to_string());
    parse_hotkey_spec(&raw, keymap).unwrap_or_else(|| default_hotkey_spec(keymap))
}

fn parse_hotkey_spec(raw: &str, keymap: &X11Keymap) -> Option<HotkeySpec> {
    let parts: Vec<&str> = raw
        .split('+')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();

    let mut ctrl = false;
    let mut alt = false;
    let mut shift = false;
    let mut meta = false;
    let mut trigger: Option<HotkeyTrigger> = None;

    for part in &parts {
        if part.eq_ignore_ascii_case("ctrl") || part.eq_ignore_ascii_case("control") {
            ctrl = true;
            continue;
        }
        if part.eq_ignore_ascii_case("alt") {
            alt = true;
            continue;
        }
        if part.eq_ignore_ascii_case("shift") {
            shift = true;
            continue;
        }
        if part.eq_ignore_ascii_case("meta") || part.eq_ignore_ascii_case("super") || part.eq_ignore_ascii_case("win") {
            meta = true;
            continue;
        }

        let lowered = part.to_ascii_lowercase();
        if lowered == "mousemiddle" || lowered == "mouse2" {
            trigger = Some(HotkeyTrigger::MouseButton(2));
            continue;
        }
        if lowered == "mouseback" || lowered == "mouse4" || lowered == "mousex1" {
            trigger = Some(HotkeyTrigger::MouseButton(4));
            continue;
        }
        if lowered == "mouseforward" || lowered == "mouse5" || lowered == "mousex2" {
            trigger = Some(HotkeyTrigger::MouseButton(5));
            continue;
        }

        if let Some(keycode) = keymap.resolve_key(part) {
            trigger = Some(HotkeyTrigger::Key(keycode));
        }
    }

    // Allow combos like Ctrl+Super by using the last modifier as trigger key.
    if trigger.is_none() {
        if let Some(last) = parts.last() {
            if last.eq_ignore_ascii_case("meta") || last.eq_ignore_ascii_case("super") || last.eq_ignore_ascii_case("win") {
                trigger = Some(HotkeyTrigger::Key(keymap.meta_left));
                meta = false;
            } else if last.eq_ignore_ascii_case("ctrl") || last.eq_ignore_ascii_case("control") {
                trigger = Some(HotkeyTrigger::Key(keymap.ctrl_left));
                ctrl = false;
            } else if last.eq_ignore_ascii_case("alt") {
                trigger = Some(HotkeyTrigger::Key(keymap.alt_left));
                alt = false;
            } else if last.eq_ignore_ascii_case("shift") {
                trigger = Some(HotkeyTrigger::Key(keymap.shift_left));
                shift = false;
            }
        }
    }

    Some(HotkeySpec {
        ctrl,
        alt,
        shift,
        meta,
        trigger: trigger?,
        label: raw.to_string(),
    })
}

fn default_hotkey_spec(keymap: &X11Keymap) -> HotkeySpec {
    HotkeySpec {
        ctrl: true,
        alt: false,
        shift: false,
        meta: false,
        trigger: HotkeyTrigger::Key(keymap.resolve_key("space").unwrap_or(65)),
        label: "Ctrl+Space".to_string(),
    }
}

struct X11Keymap {
    display: *mut x11::xlib::Display,
    root_window: x11::xlib::Window,
    ctrl_left: u32,
    ctrl_right: u32,
    alt_left: u32,
    alt_right: u32,
    shift_left: u32,
    shift_right: u32,
    meta_left: u32,
    meta_right: u32,
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
                root_window: x11::xlib::XDefaultRootWindow(display),
                ctrl_left: keycode(display, x11::keysym::XK_Control_L),
                ctrl_right: keycode(display, x11::keysym::XK_Control_R),
                alt_left: keycode(display, x11::keysym::XK_Alt_L),
                alt_right: keycode(display, x11::keysym::XK_Alt_R),
                shift_left: keycode(display, x11::keysym::XK_Shift_L),
                shift_right: keycode(display, x11::keysym::XK_Shift_R),
                meta_left: keycode(display, x11::keysym::XK_Super_L),
                meta_right: keycode(display, x11::keysym::XK_Super_R),
                escape: keycode(display, x11::keysym::XK_Escape),
            })
        }
    }

    fn resolve_key(&self, raw: &str) -> Option<u32> {
        let normalized = if raw.eq_ignore_ascii_case("space") {
            "space".to_string()
        } else if raw.eq_ignore_ascii_case("esc") {
            "Escape".to_string()
        } else if raw.eq_ignore_ascii_case("enter") {
            "Return".to_string()
        } else {
            raw.to_string()
        };

        let c_string = CString::new(normalized).ok()?;
        let keysym = unsafe { x11::xlib::XStringToKeysym(c_string.as_ptr()) };
        if keysym == 0 {
            return None;
        }

        let keycode = unsafe { x11::xlib::XKeysymToKeycode(self.display, keysym) };
        if keycode == 0 {
            None
        } else {
            Some(keycode.into())
        }
    }

    fn hotkey_down(&self, state: &InputState, spec: &HotkeySpec) -> bool {
        if !self.modifiers_match(state, spec) {
            return false;
        }

        match spec.trigger {
            HotkeyTrigger::Key(keycode) => key_is_down(&state.keys, keycode),
            HotkeyTrigger::MouseButton(button) => mouse_button_is_down(state.pointer_mask, button),
        }
    }

    fn escape_down(&self, state: &InputState) -> bool {
        key_is_down(&state.keys, self.escape)
    }

    fn modifiers_match(&self, state: &InputState, spec: &HotkeySpec) -> bool {
        let ctrl = key_is_down(&state.keys, self.ctrl_left) || key_is_down(&state.keys, self.ctrl_right);
        let alt = key_is_down(&state.keys, self.alt_left) || key_is_down(&state.keys, self.alt_right);
        let shift = key_is_down(&state.keys, self.shift_left) || key_is_down(&state.keys, self.shift_right);
        let meta = key_is_down(&state.keys, self.meta_left) || key_is_down(&state.keys, self.meta_right);

        ctrl == spec.ctrl && alt == spec.alt && shift == spec.shift && meta == spec.meta
    }

    fn query_state(&self) -> InputState {
        let mut keys = [0_i8; 32];
        let mut root_return: x11::xlib::Window = 0;
        let mut child_return: x11::xlib::Window = 0;
        let mut root_x = 0;
        let mut root_y = 0;
        let mut win_x = 0;
        let mut win_y = 0;
        let mut pointer_mask: u32 = 0;

        unsafe {
            x11::xlib::XQueryKeymap(self.display, keys.as_mut_ptr());
            let _ = x11::xlib::XQueryPointer(
                self.display,
                self.root_window,
                &mut root_return,
                &mut child_return,
                &mut root_x,
                &mut root_y,
                &mut win_x,
                &mut win_y,
                &mut pointer_mask,
            );
        }

        InputState { keys, pointer_mask }
    }
}

struct InputState {
    keys: [i8; 32],
    pointer_mask: u32,
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

fn mouse_button_is_down(pointer_mask: u32, button: u32) -> bool {
    let mask = match button {
        1 => x11::xlib::Button1Mask,
        2 => x11::xlib::Button2Mask,
        3 => x11::xlib::Button3Mask,
        4 => x11::xlib::Button4Mask,
        5 => x11::xlib::Button5Mask,
        _ => 0,
    };

    mask != 0 && (pointer_mask & mask) != 0
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
