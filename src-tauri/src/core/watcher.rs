use crate::core::ToolType;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::Emitter;

const DEBOUNCE_MS: u64 = 500;
const CHANNEL_BOUND: usize = 1000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChangeEvent {
    pub tool: ToolType,
    pub path: String,
    pub scope: String,
}

pub struct WriteGuard {
    path: PathBuf,
    writing: Arc<Mutex<HashSet<PathBuf>>>,
}

impl Drop for WriteGuard {
    fn drop(&mut self) {
        self.writing.lock().unwrap().remove(&self.path);
    }
}

struct WatchRoot {
    path: PathBuf,
    tool: ToolType,
    scope: String,
}

enum Msg {
    Event(notify::Result<notify::Event>),
    RemoveRoot(PathBuf),
    Stop,
}

pub struct FileWatcher {
    watcher: Mutex<Option<RecommendedWatcher>>,
    roots: Arc<Mutex<Vec<WatchRoot>>>,
    projects: Mutex<HashMap<PathBuf, Vec<PathBuf>>>,
    writing: Arc<Mutex<HashSet<PathBuf>>>,
    tx: Mutex<Option<mpsc::SyncSender<Msg>>>,
    handle: Mutex<Option<std::thread::JoinHandle<()>>>,
}

impl FileWatcher {
    pub fn new(app: tauri::AppHandle) -> Result<Self, String> {
        let (tx, rx) = mpsc::sync_channel(CHANNEL_BOUND);
        let roots: Arc<Mutex<Vec<WatchRoot>>> = Arc::default();
        let writing: Arc<Mutex<HashSet<PathBuf>>> = Arc::default();

        let notify_tx = tx.clone();
        let watcher = notify::recommended_watcher(move |res| {
            let _ = notify_tx.try_send(Msg::Event(res));
        })
        .map_err(|e| e.to_string())?;

        let (r, w) = (Arc::clone(&roots), Arc::clone(&writing));
        let handle = std::thread::spawn(move || run_loop(app, rx, r, w));

        Ok(Self {
            watcher: Mutex::new(Some(watcher)),
            roots,
            projects: Mutex::new(HashMap::new()),
            writing,
            tx: Mutex::new(Some(tx)),
            handle: Mutex::new(Some(handle)),
        })
    }

    pub fn start_global_watch(&self) -> Result<(), String> {
        let home = dirs::home_dir().ok_or("No home directory")?;
        let dirs = [
            (home.join(".claude"), ToolType::ClaudeCode),
            (home.join(".codex"), ToolType::Codex),
            (home.join(".gemini"), ToolType::Gemini),
            (home.join(".config/opencode"), ToolType::OpenCode),
        ];
        for (path, tool) in dirs {
            self.add_root(path, tool, "global".into())?;
        }
        Ok(())
    }

    pub fn watch_project(&self, project: &Path) -> Result<(), String> {
        let scope = project.to_string_lossy().into_owned();
        let dirs = [
            (project.join(".claude"), ToolType::ClaudeCode),
            (project.join(".codex"), ToolType::Codex),
            (project.join(".gemini"), ToolType::Gemini),
            (project.join(".opencode"), ToolType::OpenCode),
        ];
        let mut added = Vec::new();
        for (path, tool) in dirs {
            if self.add_root(path.clone(), tool, scope.clone())? {
                added.push(path);
            }
        }
        if !added.is_empty() {
            self.projects.lock().unwrap().insert(project.to_path_buf(), added);
        }
        Ok(())
    }

    pub fn unwatch_project(&self, project: &Path) {
        let paths = self.projects.lock().unwrap().remove(project).unwrap_or_default();
        for p in paths {
            self.remove_root(&p);
        }
    }

    pub fn begin_write(&self, path: &Path) -> WriteGuard {
        self.writing.lock().unwrap().insert(path.to_path_buf());
        WriteGuard {
            path: path.to_path_buf(),
            writing: Arc::clone(&self.writing),
        }
    }

    pub fn end_write(&self, path: &Path) {
        self.writing.lock().unwrap().remove(path);
    }

    fn add_root(&self, path: PathBuf, tool: ToolType, scope: String) -> Result<bool, String> {
        if !path.is_dir() {
            return Ok(false);
        }
        // Lock watcher first, then roots (consistent order)
        let mut watcher_guard = self.watcher.lock().unwrap();
        let mut roots = self.roots.lock().unwrap();
        if roots.iter().any(|r| r.path == path) {
            return Ok(false);
        }
        if let Some(w) = watcher_guard.as_mut() {
            w.watch(&path, RecursiveMode::NonRecursive)
                .map_err(|e| e.to_string())?;
        }
        roots.push(WatchRoot { path, tool, scope });
        Ok(true)
    }

    fn remove_root(&self, path: &PathBuf) {
        // Lock watcher first, then roots (consistent order)
        let mut watcher_guard = self.watcher.lock().unwrap();
        if let Some(w) = watcher_guard.as_mut() {
            let _ = w.unwatch(path);
        }
        self.roots.lock().unwrap().retain(|r| &r.path != path);
        if let Some(tx) = self.tx.lock().unwrap().as_ref() {
            let _ = tx.try_send(Msg::RemoveRoot(path.clone()));
        }
    }
}

impl Drop for FileWatcher {
    fn drop(&mut self) {
        // Stop watcher first to prevent new events
        self.watcher.lock().unwrap().take();
        // Drop sender to trigger Disconnected in run_loop
        self.tx.lock().unwrap().take();
        if let Some(h) = self.handle.lock().unwrap().take() {
            let _ = h.join();
        }
    }
}

fn run_loop(
    app: tauri::AppHandle,
    rx: mpsc::Receiver<Msg>,
    roots: Arc<Mutex<Vec<WatchRoot>>>,
    writing: Arc<Mutex<HashSet<PathBuf>>>,
) {
    let debounce = Duration::from_millis(DEBOUNCE_MS);
    let mut pending: HashMap<PathBuf, (ToolType, String, Instant)> = HashMap::new();

    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Msg::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Ok(Msg::RemoveRoot(root)) => {
                pending.retain(|p, _| !p.starts_with(&root));
            }
            Ok(Msg::Event(Ok(ev))) => {
                use notify::event::EventKind;
                // Only skip read-only access events, keep modify/create/remove
                if matches!(ev.kind, EventKind::Access(notify::event::AccessKind::Read)) {
                    continue;
                }
                for path in ev.paths {
                    if let Some((tool, scope)) = find_root(&roots, &path) {
                        pending.insert(path, (tool, scope, Instant::now()));
                    }
                }
            }
            _ => {}
        }

        let now = Instant::now();
        pending.retain(|path, (tool, scope, ts)| {
            if now.duration_since(*ts) < debounce {
                return true;
            }
            // Skip if currently writing, but keep in pending for next tick
            if writing.lock().unwrap().contains(path) {
                return true;
            }
            let _ = app.emit(
                "config-changed",
                ConfigChangeEvent {
                    tool: *tool,
                    path: path.to_string_lossy().into_owned(),
                    scope: scope.clone(),
                },
            );
            false
        });
    }
}

fn find_root(roots: &Arc<Mutex<Vec<WatchRoot>>>, path: &Path) -> Option<(ToolType, String)> {
    roots
        .lock()
        .unwrap()
        .iter()
        .filter(|r| path.starts_with(&r.path))
        .max_by_key(|r| r.path.components().count())
        .map(|r| (r.tool, r.scope.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_roots(entries: &[(&str, ToolType, &str)]) -> Arc<Mutex<Vec<WatchRoot>>> {
        Arc::new(Mutex::new(
            entries.iter().map(|(p, t, s)| WatchRoot {
                path: PathBuf::from(p),
                tool: *t,
                scope: s.to_string(),
            }).collect()
        ))
    }

    #[test]
    fn find_root_matches_exact() {
        let roots = make_roots(&[
            ("/home/user/.claude", ToolType::ClaudeCode, "global"),
        ]);
        let result = find_root(&roots, Path::new("/home/user/.claude/.mcp.json"));
        assert!(result.is_some());
        let (tool, scope) = result.unwrap();
        assert_eq!(tool, ToolType::ClaudeCode);
        assert_eq!(scope, "global");
    }

    #[test]
    fn find_root_returns_none_for_unmatched() {
        let roots = make_roots(&[
            ("/home/user/.claude", ToolType::ClaudeCode, "global"),
        ]);
        let result = find_root(&roots, Path::new("/home/user/.codex/config.toml"));
        assert!(result.is_none());
    }

    #[test]
    fn find_root_prefers_longer_match() {
        let roots = make_roots(&[
            ("/home/user/.claude", ToolType::ClaudeCode, "global"),
            ("/home/user/project/.claude", ToolType::ClaudeCode, "/home/user/project"),
        ]);
        let result = find_root(&roots, Path::new("/home/user/project/.claude/.mcp.json"));
        assert!(result.is_some());
        let (_tool, scope) = result.unwrap();
        assert_eq!(scope, "/home/user/project");
    }

    #[test]
    fn find_root_matches_multiple_tools() {
        let roots = make_roots(&[
            ("/home/user/.claude", ToolType::ClaudeCode, "global"),
            ("/home/user/.codex", ToolType::Codex, "global"),
            ("/home/user/.gemini", ToolType::Gemini, "global"),
        ]);

        let r1 = find_root(&roots, Path::new("/home/user/.codex/config.toml"));
        assert_eq!(r1.unwrap().0, ToolType::Codex);

        let r2 = find_root(&roots, Path::new("/home/user/.gemini/settings.json"));
        assert_eq!(r2.unwrap().0, ToolType::Gemini);
    }

    #[test]
    fn config_change_event_serialization() {
        let event = ConfigChangeEvent {
            tool: ToolType::ClaudeCode,
            path: "/home/user/.claude/.mcp.json".into(),
            scope: "global".into(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("ClaudeCode"));
        assert!(json.contains(".mcp.json"));
    }

    #[test]
    fn write_guard_removes_path_on_drop() {
        let writing: Arc<Mutex<HashSet<PathBuf>>> = Arc::default();
        let path = PathBuf::from("/test/path");

        {
            writing.lock().unwrap().insert(path.clone());
            let _guard = WriteGuard {
                path: path.clone(),
                writing: Arc::clone(&writing),
            };
            assert!(writing.lock().unwrap().contains(&path));
        }

        assert!(!writing.lock().unwrap().contains(&path));
    }
}