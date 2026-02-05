use rusqlite::Connection;
use crate::core::Project;

pub struct ProjectRepo<'a> {
    conn: &'a Connection,
}

impl<'a> ProjectRepo<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn list(&self) -> Result<Vec<Project>, String> {
        let mut stmt = self.conn
            .prepare("SELECT id, name, path, created_at, updated_at FROM projects ORDER BY updated_at DESC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    tools: vec![],
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }
}
