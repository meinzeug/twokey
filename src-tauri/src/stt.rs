use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use reqwest::blocking::multipart;
use serde::{Deserialize, Serialize};

use crate::{history, secrets, settings};

pub struct Transcript {
    pub text: String,
    pub provider: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalWhisperDiagnostics {
    pub whisper_available: bool,
    pub ffmpeg_available: bool,
    pub managed_whisper_path: String,
    pub managed_ffmpeg_path: String,
    pub message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SherpaOnnxDiagnostics {
    pub runtime_available: bool,
    pub model_available: bool,
    pub managed_venv_path: String,
    pub managed_model_path: String,
    pub message: String,
}

const SHERPA_MODEL_ARCHIVE_URL: &str = "https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8.tar.bz2";
const SHERPA_MODEL_DIR_NAME: &str = "sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8";

#[derive(Deserialize)]
struct OpenAITranscriptionResponse {
    text: String,
}

pub fn transcribe(audio_path: &Path) -> Result<Transcript, String> {
    let app_settings = settings::load().unwrap_or_default();
    let result = match app_settings.stt_provider.as_str() {
        "local-whisper" => transcribe_with_local_whisper(
            audio_path,
            &app_settings.default_language,
            &app_settings.whisper_model,
            app_settings.whisper_beam_size,
        ),
        "external-command" => {
            let command_template = std::env::var("TWOKEY_STT_COMMAND")
                .map_err(|_| "TWOKEY_STT_COMMAND ist nicht gesetzt".to_string());
            command_template.and_then(|template| transcribe_with_command(audio_path, &template))
        }
        "openai-compatible" => transcribe_with_openai_compatible(audio_path),
        "sherpa-onnx" => transcribe_with_sherpa_onnx(audio_path, &app_settings.default_language),
        _ => {
            if let Ok(command_template) = std::env::var("TWOKEY_STT_COMMAND") {
                transcribe_with_command(audio_path, &command_template)
            } else {
                mock_transcribe(audio_path)
            }
        }
    };

    let _ = history::record(history::AuditEvent {
        kind: "stt".to_string(),
        mode: None,
        provider: result.as_ref().ok().map(|transcript| transcript.provider.clone()),
        input_text: Some(audio_path.to_string_lossy().to_string()),
        output_text: result.as_ref().ok().map(|transcript| transcript.text.clone()),
        metadata_json: None,
        success: result.is_ok(),
    });

    result
}

pub fn ensure_sherpa_onnx_installed() -> Result<(), String> {
    if !command_exists("python3") {
        return Err("python3 wurde nicht gefunden. sherpa-onnx benoetigt eine Python-Runtime.".to_string());
    }

    install_sherpa_in_venv()
}

pub fn ensure_sherpa_onnx_runtime(sudo_password: Option<&str>) -> Result<String, String> {
    ensure_sherpa_system_dependencies(sudo_password)?;
    ensure_sherpa_onnx_installed()?;
    ensure_sherpa_model_downloaded()?;
    Ok("sherpa-onnx Runtime und Modell sind bereit.".to_string())
}

fn ensure_sherpa_system_dependencies(sudo_password: Option<&str>) -> Result<(), String> {
    if command_exists("python3") && command_exists("tar") {
        return Ok(());
    }

    if !command_exists("apt-get") {
        return Err("python3/tar fehlen und apt-get ist nicht verfuegbar. Bitte installiere python3, python3-venv, python3-pip und tar manuell.".to_string());
    }

    run_privileged_command(
        "apt-get",
        &["install", "-y", "python3", "python3-venv", "python3-pip", "tar", "bzip2"],
        sudo_password,
    )?;

    if !command_exists("python3") || !command_exists("tar") {
        return Err("python3/tar konnten nach der Installation nicht verifiziert werden.".to_string());
    }

    Ok(())
}

pub fn sherpa_onnx_diagnostics() -> SherpaOnnxDiagnostics {
    let venv_python = sherpa_venv_python_path();
    let model_dir = sherpa_model_dir_path();
    let runtime_available = venv_python.is_file();
    let model_available = sherpa_model_files(&model_dir).is_some();

    let message = if runtime_available && model_available {
        "sherpa-onnx ist bereit.".to_string()
    } else if !runtime_available && !model_available {
        "Weder sherpa-onnx Runtime noch Modell sind bereit. Fuehre Runtime-Setup aus.".to_string()
    } else if !runtime_available {
        "sherpa-onnx Runtime fehlt. Runtime-Setup erforderlich.".to_string()
    } else {
        "sherpa-onnx Modell fehlt. Runtime-Setup erforderlich.".to_string()
    };

    SherpaOnnxDiagnostics {
        runtime_available,
        model_available,
        managed_venv_path: sherpa_venv_path().to_string_lossy().to_string(),
        managed_model_path: model_dir.to_string_lossy().to_string(),
        message,
    }
}

pub fn ensure_local_whisper_installed() -> Result<(), String> {
    if command_exists("whisper-cli") || find_whisper_program().is_some() {
        return Ok(());
    }

    let mut last_error = String::new();
    if command_exists("pipx") {
        match run_command("pipx", &["install", "--force", "openai-whisper"]) {
            Ok(()) => {
                if command_exists("whisper-cli") || find_whisper_program().is_some() {
                    return Ok(());
                }
            }
            Err(error) => {
                last_error = error;
            }
        }
    }

    if command_exists("python3") {
        match install_whisper_in_venv() {
            Ok(()) => {
                if command_exists("whisper-cli") || find_whisper_program().is_some() {
                    return Ok(());
                }
            }
            Err(error) => {
                last_error = error;
            }
        }
    }

    Err(if last_error.is_empty() {
        "Lokal Whisper konnte nicht automatisch installiert werden. Installiere 'pipx' oder 'python3-venv', damit TwoKey Whisper in einer isolierten Umgebung einrichten kann.".to_string()
    } else {
        format!(
            "Lokal Whisper konnte nicht automatisch installiert werden: {}",
            last_error
        )
    })
}

pub fn ensure_local_whisper_runtime(sudo_password: Option<&str>) -> Result<String, String> {
    ensure_local_whisper_installed()?;
    ensure_ffmpeg_installed(sudo_password)?;
    Ok("Whisper und ffmpeg sind bereit.".to_string())
}

pub fn local_whisper_diagnostics() -> LocalWhisperDiagnostics {
    let whisper_path = whisper_venv_path().join("bin").join("whisper");
    let ffmpeg_path = twokey_managed_bin_path("ffmpeg");
    let whisper_available = command_exists("whisper-cli") || find_whisper_program().is_some();
    let ffmpeg_available = find_ffmpeg_program().is_some();

    let message = if whisper_available && ffmpeg_available {
        "Lokal Whisper ist bereit.".to_string()
    } else if !whisper_available && !ffmpeg_available {
        "Weder Whisper noch ffmpeg sind bereit. Fuehre Runtime-Setup aus oder speichere local-whisper erneut.".to_string()
    } else if !whisper_available {
        "Whisper CLI fehlt. Runtime-Setup erforderlich.".to_string()
    } else {
        "ffmpeg fehlt. Runtime-Setup erforderlich.".to_string()
    };

    LocalWhisperDiagnostics {
        whisper_available,
        ffmpeg_available,
        managed_whisper_path: whisper_path.to_string_lossy().to_string(),
        managed_ffmpeg_path: ffmpeg_path.to_string_lossy().to_string(),
        message,
    }
}

fn transcribe_with_local_whisper(
    audio_path: &Path,
    language: &str,
    whisper_model: &str,
    whisper_beam_size: u8,
) -> Result<Transcript, String> {
    ensure_local_whisper_installed()?;

    if !audio_path.is_file() {
        return Err(format!(
            "Audiodatei fuer Transkription wurde nicht gefunden: {}",
            audio_path.to_string_lossy()
        ));
    }

    let ffmpeg_program = find_ffmpeg_program();
    let clip_duration = estimate_audio_duration_secs(audio_path);
    if clip_duration.is_some_and(|duration| duration < 0.9) {
        return Err("Aufnahme zu kurz fuer verlaessliche Transkription. Bitte Hotkey laenger halten und den Satz komplett sprechen.".to_string());
    }

    let short_clip_fast_path = clip_duration.is_some_and(|duration| duration <= 2.5);

    let effective_model = if short_clip_fast_path {
        match whisper_model {
            "large-v3" => "medium",
            "medium" => "base",
            _ => whisper_model,
        }
    } else {
        whisper_model
    };

    let effective_beam_size: u8 = whisper_beam_size.clamp(1, 10);

    let mut commands: Vec<(Vec<String>, Option<PathBuf>)> = Vec::new();
    if command_exists("whisper-cli") {
        commands.push((
            vec![
                "whisper-cli".to_string(),
                "-f".to_string(),
                audio_path.to_string_lossy().to_string(),
                "-l".to_string(),
                language.to_string(),
                "-nt".to_string(),
            ],
            None,
        ));
    }

    if let Some(whisper_program) = find_whisper_program() {
        if ffmpeg_program.is_none() {
            return Err(
                "Lokal Whisper benoetigt ffmpeg, aber es wurde nicht gefunden. Installiere ffmpeg (z.B. 'sudo apt install ffmpeg')."
                    .to_string(),
            );
        }

        let beam_size = effective_beam_size.to_string();

        let output_dir = unique_whisper_output_dir();
        let _ = fs::create_dir_all(&output_dir);
        commands.push((
            vec![
                whisper_program.clone(),
                audio_path.to_string_lossy().to_string(),
                "--language".to_string(),
                language.to_string(),
                "--model".to_string(),
                effective_model.to_string(),
                "--beam_size".to_string(),
                beam_size,
                "--best_of".to_string(),
                "1".to_string(),
                "--temperature".to_string(),
                "0".to_string(),
                "--fp16".to_string(),
                "False".to_string(),
                "--condition_on_previous_text".to_string(),
                "False".to_string(),
                "--no_speech_threshold".to_string(),
                "0.6".to_string(),
                "--logprob_threshold".to_string(),
                "-1.0".to_string(),
                "--compression_ratio_threshold".to_string(),
                "2.4".to_string(),
                "--output_format".to_string(),
                "txt".to_string(),
                "--output_dir".to_string(),
                output_dir.to_string_lossy().to_string(),
            ],
            Some(output_dir),
        ));

        // Fallback pass: auto language + tiny model to avoid empty results on very short clips.
        let fallback_output_dir = unique_whisper_output_dir();
        let _ = fs::create_dir_all(&fallback_output_dir);
        commands.push((
            vec![
                whisper_program,
                audio_path.to_string_lossy().to_string(),
                "--language".to_string(),
                language.to_string(),
                "--model".to_string(),
                "base".to_string(),
                "--beam_size".to_string(),
                "2".to_string(),
                "--best_of".to_string(),
                "1".to_string(),
                "--temperature".to_string(),
                "0".to_string(),
                "--fp16".to_string(),
                "False".to_string(),
                "--condition_on_previous_text".to_string(),
                "False".to_string(),
                "--no_speech_threshold".to_string(),
                "0.6".to_string(),
                "--logprob_threshold".to_string(),
                "-1.0".to_string(),
                "--compression_ratio_threshold".to_string(),
                "2.4".to_string(),
                "--output_format".to_string(),
                "txt".to_string(),
                "--output_dir".to_string(),
                fallback_output_dir.to_string_lossy().to_string(),
            ],
            Some(fallback_output_dir),
        ));
    }

    let mut attempt_errors: Vec<String> = Vec::new();

    for (command, output_dir) in commands {
        let program = &command[0];
        let mut process = Command::new(program);
        process.args(&command[1..]);
        if let Some(ffmpeg) = ffmpeg_program.as_ref() {
            process.env("PATH", prepend_parent_to_path(ffmpeg));
        }

        let output = process
            .output()
            .map_err(|error| format!("{program} konnte nicht gestartet werden: {error}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

        if !output.status.success() {
            let detail = if !stderr.is_empty() {
                stderr
            } else if !stdout.is_empty() {
                stdout
            } else {
                format!("Exit-Code {}", output.status)
            };
            attempt_errors.push(format!("{program}: {detail}"));
            continue;
        }

        let mut text = stdout;
        if text.is_empty() {
            if let Some(ref dir) = output_dir {
                if let Ok(from_file) = read_whisper_txt_output(audio_path, dir) {
                    text = from_file;
                } else if let Ok(from_scan) = read_any_whisper_txt_output(dir) {
                    text = from_scan;
                }
            }
        }

        if text.is_empty() {
            if !stderr.is_empty() {
                attempt_errors.push(format!("{program}: {stderr}"));
            } else {
                attempt_errors.push(format!(
                    "{program}: Kein stdout und keine lesbare Whisper-Outputdatei"
                ));
            }
            continue;
        }

        if looks_like_whisper_error_text(&text) {
            attempt_errors.push(format!("{program}: {text}"));
            continue;
        }

        let cleaned = sanitize_transcript(&text);
        return Ok(Transcript {
            text: cleaned,
            provider: "local-whisper".to_string(),
        });
    }

    if attempt_errors.is_empty() {
        return Err("Lokal Whisper lieferte kein Transkript. Bitte pruefe Aufnahmepegel und Spracheinstellung.".to_string());
    }

    let only_empty_output_failures = attempt_errors
        .iter()
        .all(|error| error.contains("Kein stdout und keine lesbare Whisper-Outputdatei"));

    if only_empty_output_failures {
        return Ok(Transcript {
            text: "Keine Sprache erkannt.".to_string(),
            provider: "local-whisper".to_string(),
        });
    }

    Err(format!(
        "Lokale Whisper-Transkription fehlgeschlagen: {}",
        attempt_errors.join(" | ")
    ))
}

fn transcribe_with_sherpa_onnx(audio_path: &Path, language: &str) -> Result<Transcript, String> {
    ensure_sherpa_onnx_runtime(None)?;

    if !audio_path.is_file() {
        return Err(format!(
            "Audiodatei fuer Transkription wurde nicht gefunden: {}",
            audio_path.to_string_lossy()
        ));
    }

    let clip_duration = estimate_audio_duration_secs(audio_path);
    if clip_duration.is_some_and(|duration| duration < 0.9) {
        return Err("Aufnahme zu kurz fuer verlaessliche Transkription. Bitte Hotkey laenger halten und den Satz komplett sprechen.".to_string());
    }

    let model_dir = sherpa_model_dir_path();
    let model_files = sherpa_model_files(&model_dir)
        .ok_or_else(|| "sherpa-onnx Modell-Dateien wurden nicht gefunden. Fuehre Runtime-Setup erneut aus.".to_string())?;

    let python = sherpa_venv_python_path();
    if !python.is_file() {
        return Err("sherpa-onnx Python-Runtime fehlt. Fuehre Runtime-Setup erneut aus.".to_string());
    }

    let python_script = r#"
import sys
import wave
import numpy as np
import sherpa_onnx

mode = sys.argv[1]
tokens = sys.argv[2]
model = sys.argv[3]
encoder = sys.argv[4]
decoder = sys.argv[5]
joiner = sys.argv[6]
audio_path = sys.argv[7]
language = sys.argv[8]

if mode == "ctc":
    recognizer = sherpa_onnx.OfflineRecognizer.from_nemo_ctc(
        model=model,
        tokens=tokens,
        num_threads=1,
        provider="cpu",
        decoding_method="greedy_search",
        debug=False,
    )
else:
    recognizer = sherpa_onnx.OfflineRecognizer.from_transducer(
        encoder=encoder,
        decoder=decoder,
        joiner=joiner,
        tokens=tokens,
        num_threads=1,
        provider="cpu",
        decoding_method="greedy_search",
        debug=False,
        model_type="nemo_transducer",
    )

with wave.open(audio_path, "rb") as wf:
    sample_rate = wf.getframerate()
    channels = wf.getnchannels()
    sample_width = wf.getsampwidth()
    frames = wf.readframes(wf.getnframes())

if sample_width != 2:
    raise RuntimeError("Unsupported WAV sample width. Expected 16-bit PCM.")

audio = np.frombuffer(frames, dtype=np.int16).astype(np.float32) / 32768.0
if channels > 1:
    audio = audio.reshape(-1, channels)[:, 0]

stream = recognizer.create_stream()
if language:
    try:
        stream.set_option("language", language)
    except Exception:
        pass

stream.accept_waveform(sample_rate, audio)
recognizer.decode_stream(stream)
text = stream.result.text.strip()
print(text)
"#;

    let model_arg = model_files
        .model
        .as_ref()
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_default();
    let encoder_arg = model_files
        .encoder
        .as_ref()
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_default();
    let decoder_arg = model_files
        .decoder
        .as_ref()
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_default();
    let joiner_arg = model_files
        .joiner
        .as_ref()
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_default();

    let output = Command::new(&python)
        .arg("-c")
        .arg(python_script)
        .arg(if model_files.model.is_some() { "ctc" } else { "transducer" })
        .arg(&model_files.tokens)
        .arg(model_arg)
        .arg(encoder_arg)
        .arg(decoder_arg)
        .arg(joiner_arg)
        .arg(audio_path)
        .arg(language)
        .output()
        .map_err(|error| format!("sherpa-onnx konnte nicht gestartet werden: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let detail = if !stderr.is_empty() {
            stderr
        } else if !stdout.is_empty() {
            stdout
        } else {
            format!("Exit-Code {}", output.status)
        };

        return Err(format!("sherpa-onnx Transkription fehlgeschlagen: {detail}"));
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        return Ok(Transcript {
            text: "Keine Sprache erkannt.".to_string(),
            provider: "sherpa-onnx".to_string(),
        });
    }

    Ok(Transcript {
        text: sanitize_transcript(&text),
        provider: "sherpa-onnx".to_string(),
    })
}

fn unique_whisper_output_dir() -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    env::temp_dir().join(format!("twokey-whisper-{ts}"))
}

fn read_whisper_txt_output(audio_path: &Path, output_dir: &Path) -> Result<String, String> {
    let stem = audio_path
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "Dateiname der Audiodatei konnte nicht gelesen werden".to_string())?;

    let txt_path = output_dir.join(format!("{stem}.txt"));
    if !txt_path.is_file() {
        return Err(format!(
            "Whisper-Outputdatei wurde nicht erzeugt: {}",
            txt_path.to_string_lossy()
        ));
    }

    let text = fs::read_to_string(&txt_path)
        .map_err(|error| format!("Whisper-Output konnte nicht gelesen werden: {error}"))?
        .trim()
        .to_string();

    if text.is_empty() {
        return Err("Whisper-Outputdatei war leer".to_string());
    }

    Ok(text)
}

fn read_any_whisper_txt_output(output_dir: &Path) -> Result<String, String> {
    let mut txt_paths: Vec<PathBuf> = fs::read_dir(output_dir)
        .map_err(|error| format!("Whisper-Outputverzeichnis konnte nicht gelesen werden: {error}"))?
        .filter_map(|entry| entry.ok().map(|value| value.path()))
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("txt"))
        .collect();

    txt_paths.sort();

    for path in txt_paths {
        let text = fs::read_to_string(&path)
            .map_err(|error| format!("Whisper-Output konnte nicht gelesen werden: {error}"))?
            .trim()
            .to_string();
        if !text.is_empty() {
            return Ok(text);
        }
    }

    Err("Keine nutzbare Whisper-Outputdatei im Outputverzeichnis gefunden".to_string())
}

fn sanitize_transcript(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0;

    while index < chars.len() {
        if chars[index] == '[' {
            let mut marker = String::new();
            let mut cursor = index + 1;
            while cursor < chars.len() {
                let next = chars[cursor];
                marker.push(next);
                cursor += 1;
                if next == ']' || marker.len() > 40 {
                    break;
                }
            }

            let lower = marker.to_ascii_lowercase();
            if marker.ends_with(']') && lower.contains("-->") {
                index = cursor;
                continue;
            }

            out.push('[');
            out.push_str(&marker);
            index = cursor;
            continue;
        }

        out.push(chars[index]);
        index += 1;
    }

    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn estimate_audio_duration_secs(audio_path: &Path) -> Option<f32> {
    let metadata = fs::metadata(audio_path).ok()?;
    let len = metadata.len();
    if len <= 44 {
        return None;
    }

    // Recorder writes WAV mono 16kHz s16: bytes_per_second = 16000 * 2 = 32000
    let pcm_bytes = len.saturating_sub(44) as f32;
    Some(pcm_bytes / 32000.0)
}

fn find_ffmpeg_program() -> Option<PathBuf> {
    if command_exists("ffmpeg") {
        return Some(PathBuf::from("ffmpeg"));
    }

    let managed = twokey_managed_bin_path("ffmpeg");
    if managed.is_file() {
        return Some(managed);
    }

    None
}

fn prepend_parent_to_path(program: &Path) -> String {
    let mut paths: Vec<PathBuf> = Vec::new();
    if let Some(parent) = program.parent() {
        paths.push(parent.to_path_buf());
    }

    if let Some(existing) = env::var_os("PATH") {
        paths.extend(env::split_paths(&existing));
    }

    env::join_paths(paths)
        .ok()
        .and_then(|value| value.into_string().ok())
        .unwrap_or_else(|| env::var("PATH").unwrap_or_default())
}

fn looks_like_whisper_error_text(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("filenotfounderror")
        || lower.contains("no such file or directory: 'ffmpeg'")
        || (lower.starts_with("skipping ") && lower.contains(" due to "))
}

fn find_whisper_program() -> Option<String> {
    if command_exists("whisper") {
        return Some("whisper".to_string());
    }

    let venv_whisper = whisper_venv_path().join("bin").join("whisper");
    if venv_whisper.is_file() {
        return Some(venv_whisper.to_string_lossy().to_string());
    }

    let local = local_bin_path("whisper");
    if local.is_file() {
        return Some(local.to_string_lossy().to_string());
    }

    None
}

fn local_bin_path(name: &str) -> PathBuf {
    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home).join(".local").join("bin").join(name);
    }

    PathBuf::from(".").join(name)
}

fn whisper_venv_path() -> PathBuf {
    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("twokey")
            .join("whisper-venv");
    }

    PathBuf::from(".twokey-whisper-venv")
}

fn sherpa_venv_path() -> PathBuf {
    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("twokey")
            .join("sherpa-venv");
    }

    PathBuf::from(".twokey-sherpa-venv")
}

fn sherpa_venv_python_path() -> PathBuf {
    sherpa_venv_path().join("bin").join("python")
}

fn sherpa_models_root_path() -> PathBuf {
    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("twokey")
            .join("models");
    }

    PathBuf::from(".twokey-models")
}

fn sherpa_model_dir_path() -> PathBuf {
    sherpa_models_root_path().join(SHERPA_MODEL_DIR_NAME)
}

struct SherpaModelFiles {
    tokens: PathBuf,
    model: Option<PathBuf>,
    encoder: Option<PathBuf>,
    decoder: Option<PathBuf>,
    joiner: Option<PathBuf>,
}

fn sherpa_model_files(model_dir: &Path) -> Option<SherpaModelFiles> {
    let tokens = model_dir.join("tokens.txt");
    if !tokens.is_file() {
        return None;
    }

    let model = [
        model_dir.join("model.int8.onnx"),
        model_dir.join("model.onnx"),
    ]
    .iter()
    .find(|path| path.is_file())
    .cloned();

    if model.is_some() {
        return Some(SherpaModelFiles {
            tokens,
            model,
            encoder: None,
            decoder: None,
            joiner: None,
        });
    }

    let encoder = [
        model_dir.join("encoder.int8.onnx"),
        model_dir.join("encoder.onnx"),
    ]
    .iter()
    .find(|path| path.is_file())
    .cloned();
    let decoder = [
        model_dir.join("decoder.int8.onnx"),
        model_dir.join("decoder.onnx"),
    ]
    .iter()
    .find(|path| path.is_file())
    .cloned();
    let joiner = [
        model_dir.join("joiner.int8.onnx"),
        model_dir.join("joiner.onnx"),
    ]
    .iter()
    .find(|path| path.is_file())
    .cloned();

    if encoder.is_some() && decoder.is_some() && joiner.is_some() {
        return Some(SherpaModelFiles {
            tokens,
            model: None,
            encoder,
            decoder,
            joiner,
        });
    }

    None
}

fn twokey_managed_bin_path(name: &str) -> PathBuf {
    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("twokey")
            .join("bin")
            .join(name);
    }

    PathBuf::from(".").join(name)
}

fn install_whisper_in_venv() -> Result<(), String> {
    let venv_path = whisper_venv_path();
    if let Some(parent) = venv_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Whisper-Verzeichnis konnte nicht angelegt werden: {error}"))?;
    }

    if !venv_path.join("bin").join("python").is_file() {
        let venv = venv_path.to_string_lossy().to_string();
        run_command("python3", &["-m", "venv", &venv])?;
    }

    let venv_python = venv_path.join("bin").join("python");
    let python = venv_python.to_string_lossy().to_string();
    run_command(&python, &["-m", "pip", "install", "--upgrade", "pip", "setuptools", "wheel"])?;
    run_command(&python, &["-m", "pip", "install", "--upgrade", "openai-whisper"])?;

    let whisper = venv_path.join("bin").join("whisper");
    if !whisper.is_file() {
        return Err("Whisper wurde installiert, aber das CLI-Binary fehlt im venv".to_string());
    }

    Ok(())
}

fn install_sherpa_in_venv() -> Result<(), String> {
    let venv_path = sherpa_venv_path();
    if let Some(parent) = venv_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("sherpa-onnx Verzeichnis konnte nicht angelegt werden: {error}"))?;
    }

    if !venv_path.join("bin").join("python").is_file() {
        let venv = venv_path.to_string_lossy().to_string();
        run_command("python3", &["-m", "venv", &venv])?;
    }

    let python = sherpa_venv_python_path().to_string_lossy().to_string();
    run_command(&python, &["-m", "pip", "install", "--upgrade", "pip", "setuptools", "wheel"])?;
    run_command(&python, &["-m", "pip", "install", "--upgrade", "numpy", "click", "sherpa-onnx"])?;
    Ok(())
}

fn ensure_sherpa_model_downloaded() -> Result<(), String> {
    let model_dir = sherpa_model_dir_path();
    if sherpa_model_files(&model_dir).is_some() {
        return Ok(());
    }

    let models_root = sherpa_models_root_path();
    fs::create_dir_all(&models_root)
        .map_err(|error| format!("Modell-Verzeichnis konnte nicht erstellt werden: {error}"))?;

    let archive_name = SHERPA_MODEL_ARCHIVE_URL
        .rsplit('/')
        .next()
        .ok_or_else(|| "Ungueltige sherpa-onnx Modell-URL".to_string())?;
    let archive_path = models_root.join(archive_name);

    if !archive_path.is_file() {
        let mut response = reqwest::blocking::get(SHERPA_MODEL_ARCHIVE_URL)
            .map_err(|error| format!("sherpa-onnx Modell konnte nicht heruntergeladen werden: {error}"))?;

        if !response.status().is_success() {
            return Err(format!(
                "sherpa-onnx Modell konnte nicht heruntergeladen werden (HTTP {})",
                response.status()
            ));
        }

        let mut file = fs::File::create(&archive_path)
            .map_err(|error| format!("Modell-Archiv konnte nicht gespeichert werden: {error}"))?;
        std::io::copy(&mut response, &mut file)
            .map_err(|error| format!("Modell-Archiv konnte nicht geschrieben werden: {error}"))?;
    }

    let status = Command::new("tar")
        .arg("-xjf")
        .arg(&archive_path)
        .arg("-C")
        .arg(&models_root)
        .status()
        .map_err(|error| format!("tar konnte nicht gestartet werden: {error}"))?;

    if !status.success() {
        return Err(format!("tar konnte das sherpa-onnx Modell nicht entpacken (Status {status})"));
    }

    if sherpa_model_files(&model_dir).is_none() {
        return Err("sherpa-onnx Modell wurde entpackt, aber benoetigte Dateien fehlen (tokens.txt plus model.onnx oder encoder/decoder/joiner.onnx).".to_string());
    }

    Ok(())
}

fn ensure_ffmpeg_installed(sudo_password: Option<&str>) -> Result<(), String> {
    if find_ffmpeg_program().is_some() {
        return Ok(());
    }

    if !command_exists("apt-get") {
        return Err("ffmpeg fehlt und apt-get ist nicht verfuegbar. Bitte ffmpeg manuell installieren oder Runtime-Setup ueber die CLI ausfuehren.".to_string());
    }

    run_privileged_command("apt-get", &["install", "-y", "ffmpeg"], sudo_password)?;

    if find_ffmpeg_program().is_some() {
        Ok(())
    } else {
        Err("ffmpeg konnte nicht verifiziert werden. Bitte Installation manuell pruefen.".to_string())
    }
}

fn run_privileged_command(program: &str, args: &[&str], sudo_password: Option<&str>) -> Result<(), String> {
    if is_root() {
        return run_command(program, args);
    }

    if let Some(password) = sudo_password.map(str::trim).filter(|value| !value.is_empty()) {
        if !command_exists("sudo") {
            return Err("sudo wurde nicht gefunden. Bitte ffmpeg manuell installieren oder TwoKey als root starten.".to_string());
        }

        let mut child = Command::new("sudo")
            .args(["-S", "-p", "", program])
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("sudo konnte nicht gestartet werden: {error}"))?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(format!("{password}\n").as_bytes())
                .map_err(|error| format!("sudo-Passwort konnte nicht uebergeben werden: {error}"))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|error| format!("sudo-Installation konnte nicht abgeschlossen werden: {error}"))?;

        if output.status.success() {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let detail = if !stderr.is_empty() {
            stderr
        } else if !stdout.is_empty() {
            stdout
        } else {
            format!("Exit-Code {}", output.status)
        };

        return Err(detail);
    }

    if command_exists("pkexec") {
        let mut pkexec_args = vec![program];
        pkexec_args.extend(args.iter().copied());
        return run_command("pkexec", &pkexec_args);
    }

    Err("ffmpeg fehlt. Gib im Dialog ein sudo-Passwort ein oder installiere ffmpeg manuell.".to_string())
}

fn is_root() -> bool {
    Command::new("sh")
        .arg("-c")
        .arg("id -u")
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim() == "0")
        .unwrap_or(false)
}

fn run_command(program: &str, args: &[&str]) -> Result<(), String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|error| format!("{program} konnte nicht gestartet werden: {error}"))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let detail = if !stderr.is_empty() {
        stderr
    } else if !stdout.is_empty() {
        stdout
    } else {
        format!("Exit-Code {}", output.status)
    };

    Err(detail)
}

fn transcribe_with_openai_compatible(audio_path: &Path) -> Result<Transcript, String> {
    let app_settings = settings::load().unwrap_or_default();
    let api_key = secrets::get_provider_api_key("openai-compatible")?;
    let endpoint = format!("{}/audio/transcriptions", app_settings.openai_base_url.trim_end_matches('/'));
    let audio_bytes = fs::read(audio_path).map_err(|error| format!("Audiodatei konnte nicht gelesen werden: {error}"))?;

    let file_part = multipart::Part::bytes(audio_bytes)
        .file_name("recording.wav")
        .mime_str("audio/wav")
        .map_err(|error| format!("Audio-MIME konnte nicht gesetzt werden: {error}"))?;

    let form = multipart::Form::new().text("model", "whisper-1").part("file", file_part);

    let response = reqwest::blocking::Client::new()
        .post(endpoint)
        .header("Authorization", format!("Bearer {api_key}"))
        .multipart(form)
        .send()
        .map_err(|error| format!("OpenAI-kompatibles STT konnte nicht erreicht werden: {error}"))?;

    if !response.status().is_success() {
        return Err(format!("OpenAI-kompatibles STT antwortete mit HTTP {}", response.status()));
    }

    let payload: OpenAITranscriptionResponse = response
        .json()
        .map_err(|error| format!("STT-Antwort konnte nicht gelesen werden: {error}"))?;
    let text = payload.text.trim().to_string();
    if text.is_empty() {
        return Err("OpenAI-kompatibles STT lieferte kein Transkript".to_string());
    }

    Ok(Transcript {
        text,
        provider: "openai-compatible".to_string(),
    })
}

fn transcribe_with_command(audio_path: &Path, command_template: &str) -> Result<Transcript, String> {
    if !command_template.contains("{audio}") {
        return Err("TWOKEY_STT_COMMAND muss den Platzhalter {audio} enthalten".to_string());
    }

    let audio_arg = shell_escape(audio_path.to_string_lossy().as_ref());
    let command = command_template.replace("{audio}", &audio_arg);
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .map_err(|error| format!("STT-Befehl konnte nicht gestartet werden: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("STT-Befehl fehlgeschlagen: {}", stderr.trim()));
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        return Err("STT-Befehl lieferte kein Transkript".to_string());
    }

    Ok(Transcript {
        text,
        provider: "external-command".to_string(),
    })
}

fn mock_transcribe(audio_path: &Path) -> Result<Transcript, String> {
    let bytes = fs::metadata(audio_path)
        .map_err(|error| format!("Audiodatei konnte nicht gelesen werden: {error}"))?
        .len();

    Ok(Transcript {
        text: format!(
            "Mock-Transkript fuer {} Audio. Echtes STT kann mit TWOKEY_STT_COMMAND aktiviert werden.",
            human_bytes(bytes)
        ),
        provider: "mock".to_string(),
    })
}

fn human_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        return format!("{bytes} B");
    }

    let kib = bytes as f64 / 1024.0;
    if kib < 1024.0 {
        return format!("{kib:.1} KiB");
    }

    format!("{:.1} MiB", kib / 1024.0)
}

fn shell_escape(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn command_exists(name: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {name} >/dev/null 2>&1"))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
