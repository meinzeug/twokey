use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection};
use serde::Serialize;

#[derive(Clone)]
pub struct AuditEvent {
    pub kind: String,
    pub mode: Option<String>,
    pub provider: Option<String>,
    pub input_text: Option<String>,
    pub output_text: Option<String>,
    pub metadata_json: Option<String>,
    pub success: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: i64,
    pub ts_unix_ms: i64,
    pub kind: String,
    pub mode: Option<String>,
    pub provider: Option<String>,
    pub input_text: Option<String>,
    pub output_text: Option<String>,
    pub metadata_json: Option<String>,
    pub success: bool,
}

pub fn init() -> Result<(), String> {
    let connection = open_connection()?;
    connection
        .execute_batch(
            "
            CREATE TABLE IF NOT EXISTS history_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ts_unix_ms INTEGER NOT NULL,
                kind TEXT NOT NULL,
                mode TEXT,
                provider TEXT,
                input_text TEXT,
                output_text TEXT,
                metadata_json TEXT,
                success INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_history_events_ts ON history_events(ts_unix_ms DESC);
            CREATE INDEX IF NOT EXISTS idx_history_events_kind ON history_events(kind);
            ",
        )
        .map_err(|error| format!("SQLite-Schema konnte nicht initialisiert werden: {error}"))?;

    Ok(())
}

pub fn record(event: AuditEvent) -> Result<(), String> {
    let connection = open_connection()?;
    let ts_unix_ms = now_unix_ms();

    connection
        .execute(
            "
            INSERT INTO history_events (
                ts_unix_ms,
                kind,
                mode,
                provider,
                input_text,
                output_text,
                metadata_json,
                success
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ",
            params![
                ts_unix_ms,
                event.kind,
                event.mode,
                event.provider,
                event.input_text,
                event.output_text,
                event.metadata_json,
                if event.success { 1 } else { 0 }
            ],
        )
        .map_err(|error| format!("SQLite-Audit konnte nicht geschrieben werden: {error}"))?;

    Ok(())
}

pub fn list_recent(limit: u32) -> Result<Vec<HistoryEntry>, String> {
    let connection = open_connection()?;
    let mut statement = connection
        .prepare(
            "
            SELECT id, ts_unix_ms, kind, mode, provider, input_text, output_text, metadata_json, success
            FROM history_events
            ORDER BY ts_unix_ms DESC
            LIMIT ?1
            ",
        )
        .map_err(|error| format!("SQLite-Abfrage konnte nicht vorbereitet werden: {error}"))?;

    let rows = statement
        .query_map(params![limit], |row| {
            Ok(HistoryEntry {
                id: row.get(0)?,
                ts_unix_ms: row.get(1)?,
                kind: row.get(2)?,
                mode: row.get(3)?,
                provider: row.get(4)?,
                input_text: row.get(5)?,
                output_text: row.get(6)?,
                metadata_json: row.get(7)?,
                success: row.get::<_, i32>(8)? == 1,
            })
        })
        .map_err(|error| format!("SQLite-Abfrage fehlgeschlagen: {error}"))?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row.map_err(|error| format!("SQLite-Zeile konnte nicht gelesen werden: {error}"))?);
    }

    Ok(entries)
}

fn open_connection() -> Result<Connection, String> {
    let db_path = history_db_path()?;
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("SQLite-Datenverzeichnis konnte nicht erstellt werden: {error}"))?;
    }

    Connection::open(db_path).map_err(|error| format!("SQLite-Datenbank konnte nicht geoeffnet werden: {error}"))
}

fn history_db_path() -> Result<PathBuf, String> {
    let base = if let Ok(value) = std::env::var("XDG_DATA_HOME") {
        PathBuf::from(value)
    } else {
        let home = std::env::var("HOME").map_err(|_| "HOME ist nicht gesetzt".to_string())?;
        PathBuf::from(home).join(".local").join("share")
    };

    Ok(base.join("twokey-ai").join("history.db"))
}

fn now_unix_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}
