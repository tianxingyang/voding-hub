use crate::db::{init_db, ProjectRepo};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static DB_COUNTER: AtomicU64 = AtomicU64::new(0);

struct TempDbDir(PathBuf);

impl TempDbDir {
    fn new() -> Self {
        let id = DB_COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "voding-db-test-{}-{}-{:?}",
            std::process::id(),
            id,
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &PathBuf {
        &self.0
    }
}

impl Drop for TempDbDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn init_creates_db_and_table() {
    let tmp = TempDbDir::new();
    let conn = init_db(tmp.path()).unwrap();

    let count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='projects'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);

    let db_file = tmp.path().join("voding-hub.db");
    assert!(db_file.exists());
}

#[test]
fn project_add_and_list() {
    let tmp = TempDbDir::new();
    let conn = init_db(tmp.path()).unwrap();
    let repo = ProjectRepo::new(&conn);

    let p1 = repo.add("/home/user/project1").unwrap();
    assert_eq!(p1.name, "project1");
    assert_eq!(p1.path, "/home/user/project1");
    assert!(p1.id > 0);

    let p2 = repo.add("/home/user/project2").unwrap();

    let projects = repo.list().unwrap();
    assert_eq!(projects.len(), 2);
}

#[test]
fn project_add_duplicate_fails() {
    let tmp = TempDbDir::new();
    let conn = init_db(tmp.path()).unwrap();
    let repo = ProjectRepo::new(&conn);

    repo.add("/home/user/dup").unwrap();
    let result = repo.add("/home/user/dup");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already exists"));
}

#[test]
fn project_remove() {
    let tmp = TempDbDir::new();
    let conn = init_db(tmp.path()).unwrap();
    let repo = ProjectRepo::new(&conn);

    let p = repo.add("/home/user/to-remove").unwrap();
    assert_eq!(repo.list().unwrap().len(), 1);

    repo.remove(p.id).unwrap();
    assert!(repo.list().unwrap().is_empty());
}

#[test]
fn project_get_by_id() {
    let tmp = TempDbDir::new();
    let conn = init_db(tmp.path()).unwrap();
    let repo = ProjectRepo::new(&conn);

    let p = repo.add("/home/user/project").unwrap();

    let found = repo.get_by_id(p.id).unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().path, "/home/user/project");

    let not_found = repo.get_by_id(99999).unwrap();
    assert!(not_found.is_none());
}

#[test]
fn project_exists_by_path() {
    let tmp = TempDbDir::new();
    let conn = init_db(tmp.path()).unwrap();
    let repo = ProjectRepo::new(&conn);

    repo.add("/home/user/exists").unwrap();

    assert!(repo.exists_by_path("/home/user/exists").unwrap());
    assert!(!repo.exists_by_path("/home/user/not-exists").unwrap());
}

#[test]
fn project_persistence_across_reconnect() {
    let tmp = TempDbDir::new();

    {
        let conn = init_db(tmp.path()).unwrap();
        let repo = ProjectRepo::new(&conn);
        repo.add("/home/user/persistent1").unwrap();
        repo.add("/home/user/persistent2").unwrap();
    }

    {
        let conn = init_db(tmp.path()).unwrap();
        let repo = ProjectRepo::new(&conn);
        let projects = repo.list().unwrap();
        assert_eq!(projects.len(), 2);

        let paths: Vec<_> = projects.iter().map(|p| p.path.as_str()).collect();
        assert!(paths.contains(&"/home/user/persistent1"));
        assert!(paths.contains(&"/home/user/persistent2"));
    }
}

#[test]
fn project_list_with_validation() {
    let tmp = TempDbDir::new();
    let conn = init_db(tmp.path()).unwrap();
    let repo = ProjectRepo::new(&conn);

    let existing_path = tmp.path().join("existing");
    fs::create_dir_all(&existing_path).unwrap();

    repo.add(&existing_path.to_string_lossy()).unwrap();
    repo.add("/nonexistent/path/project").unwrap();

    let validated = repo.list_with_validation().unwrap();
    assert_eq!(validated.len(), 2);

    let (existing, exists) = validated.iter().find(|(p, _)| p.path.contains("existing")).unwrap();
    assert!(exists);
    assert!(existing.path.contains("existing"));

    let (_, nonexistent_exists) = validated.iter().find(|(p, _)| p.path.contains("nonexistent")).unwrap();
    assert!(!nonexistent_exists);
}

#[test]
fn project_add_empty_path_fails() {
    let tmp = TempDbDir::new();
    let conn = init_db(tmp.path()).unwrap();
    let repo = ProjectRepo::new(&conn);

    let result = repo.add("");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("empty"));

    let result2 = repo.add("   ");
    assert!(result2.is_err());
}

#[test]
fn project_timestamps() {
    let tmp = TempDbDir::new();
    let conn = init_db(tmp.path()).unwrap();
    let repo = ProjectRepo::new(&conn);

    let p = repo.add("/home/user/timestamped").unwrap();
    assert!(p.created_at > 0);
    assert!(p.updated_at > 0);
    assert_eq!(p.created_at, p.updated_at);
}
