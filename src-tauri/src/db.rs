use rusqlite::{Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;
use uuid::Uuid;

// Data structures matching the TypeScript schema
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Favorite {
    pub id: String,
    pub app_id: String,
    pub display_name: String,
    pub pinned_order: Option<i32>,
    pub icon_hint: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InsertFavorite {
    pub app_id: String,
    pub display_name: String,
    pub pinned_order: Option<i32>,
    pub icon_hint: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockRule {
    pub id: String,
    pub app_id: String,
    pub match_kind: String,
    pub mode: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InsertBlockRule {
    pub app_id: String,
    pub match_kind: String,
    pub mode: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    pub id: String,
    pub start_utc: i64,
    pub end_utc: i64,
    pub status: String,
    pub duration_secs: i32,
    pub remaining_secs: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InsertSession {
    pub start_utc: i64,
    pub end_utc: i64,
    pub status: String,
    pub duration_secs: i32,
    pub remaining_secs: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSession {
    pub start_utc: Option<i64>,
    pub end_utc: Option<i64>,
    pub status: Option<String>,
    pub duration_secs: Option<i32>,
    pub remaining_secs: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

pub struct DbState {
    pub conn: Mutex<Connection>,
}

impl DbState {
    pub fn new(db_path: &str) -> SqliteResult<Self> {
        let conn = Connection::open(db_path)?;
        
        // Create tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS favorites (
                id TEXT PRIMARY KEY,
                app_id TEXT NOT NULL,
                display_name TEXT NOT NULL,
                pinned_order INTEGER,
                icon_hint TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS block_rules (
                id TEXT PRIMARY KEY,
                app_id TEXT NOT NULL,
                match_kind TEXT NOT NULL,
                mode TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                start_utc INTEGER NOT NULL,
                end_utc INTEGER NOT NULL,
                status TEXT NOT NULL,
                duration_secs INTEGER NOT NULL,
                remaining_secs INTEGER
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        Ok(DbState {
            conn: Mutex::new(conn),
        })
    }
}

// Tauri Commands - Favorites
#[tauri::command]
pub fn get_favorites(db: State<DbState>) -> Result<Vec<Favorite>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, app_id, display_name, pinned_order, icon_hint FROM favorites ORDER BY pinned_order")
        .map_err(|e| e.to_string())?;

    let favorites = stmt
        .query_map([], |row| {
            Ok(Favorite {
                id: row.get(0)?,
                app_id: row.get(1)?,
                display_name: row.get(2)?,
                pinned_order: row.get(3)?,
                icon_hint: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(favorites)
}

#[tauri::command]
pub fn create_favorite(db: State<DbState>, favorite: InsertFavorite) -> Result<Favorite, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO favorites (id, app_id, display_name, pinned_order, icon_hint) VALUES (?1, ?2, ?3, ?4, ?5)",
        (
            &id,
            &favorite.app_id,
            &favorite.display_name,
            &favorite.pinned_order,
            &favorite.icon_hint,
        ),
    )
    .map_err(|e| e.to_string())?;

    Ok(Favorite {
        id,
        app_id: favorite.app_id,
        display_name: favorite.display_name,
        pinned_order: favorite.pinned_order,
        icon_hint: favorite.icon_hint,
    })
}

#[tauri::command]
pub fn delete_favorite(db: State<DbState>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM favorites WHERE id = ?1", [&id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// Tauri Commands - Block Rules
#[tauri::command]
pub fn get_block_rules(db: State<DbState>) -> Result<Vec<BlockRule>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, app_id, match_kind, mode FROM block_rules")
        .map_err(|e| e.to_string())?;

    let rules = stmt
        .query_map([], |row| {
            Ok(BlockRule {
                id: row.get(0)?,
                app_id: row.get(1)?,
                match_kind: row.get(2)?,
                mode: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rules)
}

#[tauri::command]
pub fn create_block_rule(db: State<DbState>, rule: InsertBlockRule) -> Result<BlockRule, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO block_rules (id, app_id, match_kind, mode) VALUES (?1, ?2, ?3, ?4)",
        (&id, &rule.app_id, &rule.match_kind, &rule.mode),
    )
    .map_err(|e| e.to_string())?;

    Ok(BlockRule {
        id,
        app_id: rule.app_id,
        match_kind: rule.match_kind,
        mode: rule.mode,
    })
}

#[tauri::command]
pub fn delete_block_rule(db: State<DbState>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM block_rules WHERE id = ?1", [&id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// Tauri Commands - Sessions
#[tauri::command]
pub fn get_sessions(db: State<DbState>) -> Result<Vec<Session>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, start_utc, end_utc, status, duration_secs, remaining_secs FROM sessions ORDER BY start_utc DESC")
        .map_err(|e| e.to_string())?;

    let sessions = stmt
        .query_map([], |row| {
            Ok(Session {
                id: row.get(0)?,
                start_utc: row.get(1)?,
                end_utc: row.get(2)?,
                status: row.get(3)?,
                duration_secs: row.get(4)?,
                remaining_secs: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(sessions)
}

#[tauri::command]
pub fn create_session(db: State<DbState>, session: InsertSession) -> Result<Session, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO sessions (id, start_utc, end_utc, status, duration_secs, remaining_secs) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            &id,
            &session.start_utc,
            &session.end_utc,
            &session.status,
            &session.duration_secs,
            &session.remaining_secs,
        ),
    )
    .map_err(|e| e.to_string())?;

    Ok(Session {
        id,
        start_utc: session.start_utc,
        end_utc: session.end_utc,
        status: session.status,
        duration_secs: session.duration_secs,
        remaining_secs: session.remaining_secs,
    })
}

#[tauri::command]
pub fn update_session(db: State<DbState>, id: String, updates: UpdateSession) -> Result<Session, String> {
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;

        // Build UPDATE statements for each field separately to handle Options
        if let Some(start_utc) = updates.start_utc {
            conn.execute("UPDATE sessions SET start_utc = ?1 WHERE id = ?2", (start_utc, &id))
                .map_err(|e| e.to_string())?;
        }
        if let Some(end_utc) = updates.end_utc {
            conn.execute("UPDATE sessions SET end_utc = ?1 WHERE id = ?2", (end_utc, &id))
                .map_err(|e| e.to_string())?;
        }
        if let Some(status) = updates.status {
            conn.execute("UPDATE sessions SET status = ?1 WHERE id = ?2", (status, &id))
                .map_err(|e| e.to_string())?;
        }
        if let Some(duration_secs) = updates.duration_secs {
            conn.execute("UPDATE sessions SET duration_secs = ?1 WHERE id = ?2", (duration_secs, &id))
                .map_err(|e| e.to_string())?;
        }
        if let Some(remaining_secs) = updates.remaining_secs {
            conn.execute("UPDATE sessions SET remaining_secs = ?1 WHERE id = ?2", (remaining_secs, &id))
                .map_err(|e| e.to_string())?;
        }
    } // Release lock here

    // Fetch and return updated session with fresh lock
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let session = conn
        .query_row(
            "SELECT id, start_utc, end_utc, status, duration_secs, remaining_secs FROM sessions WHERE id = ?1",
            [&id],
            |row| {
                Ok(Session {
                    id: row.get(0)?,
                    start_utc: row.get(1)?,
                    end_utc: row.get(2)?,
                    status: row.get(3)?,
                    duration_secs: row.get(4)?,
                    remaining_secs: row.get(5)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(session)
}

// Tauri Commands - Settings
#[tauri::command]
pub fn get_settings(db: State<DbState>) -> Result<Vec<Setting>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| e.to_string())?;

    let settings = stmt
        .query_map([], |row| {
            Ok(Setting {
                key: row.get(0)?,
                value: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(settings)
}

#[tauri::command]
pub fn upsert_setting(db: State<DbState>, key: String, value: String) -> Result<Setting, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = ?2",
        (&key, &value),
    )
    .map_err(|e| e.to_string())?;

    Ok(Setting { key, value })
}
