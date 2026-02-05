use rusqlite::Connection;
use std::path::PathBuf;

pub fn init_db(app_dir: &PathBuf) -> Result<Connection, String> {
    let db_path = app_dir.join("voding-hub.db");
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            path TEXT NOT NULL UNIQUE,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn)
}
