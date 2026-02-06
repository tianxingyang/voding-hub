use crate::core::Project;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ProjectRepo<'a> {
    conn: &'a Connection,
}

impl<'a> ProjectRepo<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    fn now_ms() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    }

    fn map_project(row: &rusqlite::Row) -> rusqlite::Result<Project> {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            tools: vec![],
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    }

    pub fn list(&self) -> Result<Vec<Project>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, path, created_at, updated_at FROM projects ORDER BY updated_at DESC")
            .map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], Self::map_project).map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    pub fn add(&self, path: &str) -> Result<Project, String> {
        let path = path.trim();
        if path.is_empty() {
            return Err("Path cannot be empty".into());
        }
        let name = Path::new(path)
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| path.to_string());
        let now = Self::now_ms();
        self.conn
            .execute(
                "INSERT INTO projects (name, path, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
                params![&name, path, now, now],
            )
            .map_err(|e| {
                if e.to_string().contains("UNIQUE constraint failed") {
                    "Project path already exists".to_string()
                } else {
                    e.to_string()
                }
            })?;
        Ok(Project {
            id: self.conn.last_insert_rowid(),
            name,
            path: path.to_string(),
            tools: vec![],
            created_at: now,
            updated_at: now,
        })
    }

    pub fn remove(&self, id: i64) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM projects WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_by_id(&self, id: i64) -> Result<Option<Project>, String> {
        self.conn
            .query_row(
                "SELECT id, name, path, created_at, updated_at FROM projects WHERE id = ?1",
                params![id],
                Self::map_project,
            )
            .optional()
            .map_err(|e| e.to_string())
    }

    pub fn exists_by_path(&self, path: &str) -> Result<bool, String> {
        self.conn
            .query_row::<i32, _, _>(
                "SELECT 1 FROM projects WHERE path = ?1 LIMIT 1",
                params![path],
                |row| row.get(0),
            )
            .optional()
            .map(|v| v.is_some())
            .map_err(|e| e.to_string())
    }

    pub fn list_with_validation(&self) -> Result<Vec<(Project, bool)>, String> {
        Ok(self
            .list()?
            .into_iter()
            .map(|p| {
                let exists = Path::new(&p.path).exists();
                (p, exists)
            })
            .collect())
    }
}
