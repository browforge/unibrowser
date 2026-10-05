//! Integrated terminal PTY ([docs/terminal-panel.md](../../../docs/terminal-panel.md)).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex};
use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use unibrowser_terminal::{SpawnOptions, TerminalSession};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalOutputEvent {
    pub session_id: u32,
    pub data: String,
}

struct TerminalState {
    next_id: u32,
    sessions: HashMap<u32, Arc<Mutex<TerminalSession>>>,
}

static TERMINALS: LazyLock<Mutex<TerminalState>> = LazyLock::new(|| {
    Mutex::new(TerminalState {
        next_id: 1,
        sessions: HashMap::new(),
    })
});

pub fn spawn_terminal(app: AppHandle, cwd: Option<String>) -> Result<u32, String> {
    let cwd_path = cwd.map(PathBuf::from);
    let session = TerminalSession::spawn(SpawnOptions {
        cwd: cwd_path,
        cols: 120,
        rows: 32,
    })
    .map_err(|e| e.to_string())?;
    let id;
    let arc;
    {
        let mut state = TERMINALS.lock().unwrap_or_else(|e| e.into_inner());
        id = state.next_id;
        state.next_id += 1;
        arc = Arc::new(Mutex::new(session));
        state.sessions.insert(id, arc.clone());
    }
    start_output_pump(app, id, arc);
    Ok(id)
}

fn start_output_pump(app: AppHandle, id: u32, session: Arc<Mutex<TerminalSession>>) {
    thread::spawn(move || {
        while TERMINALS
            .lock()
            .map(|s| s.sessions.contains_key(&id))
            .unwrap_or(false)
        {
            let chunk = session
                .lock()
                .ok()
                .and_then(|mut s| s.try_recv_output());
            if let Some(bytes) = chunk {
                let data = String::from_utf8_lossy(&bytes).into_owned();
                let _ = app.emit(
                    "terminal-output",
                    TerminalOutputEvent {
                        session_id: id,
                        data,
                    },
                );
            }
            thread::sleep(Duration::from_millis(25));
        }
    });
}

fn session_arc(session_id: u32) -> Result<Arc<Mutex<TerminalSession>>, String> {
    TERMINALS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .sessions
        .get(&session_id)
        .cloned()
        .ok_or_else(|| "terminal session not found".to_string())
}

pub fn terminal_write(session_id: u32, data: String) -> Result<(), String> {
    let session = session_arc(session_id)?;
    let mut guard = session.lock().map_err(|_| "terminal lock poisoned")?;
    guard.write(data.as_bytes()).map_err(|e| e.to_string())
}

pub fn terminal_resize(session_id: u32, cols: u16, rows: u16) -> Result<(), String> {
    let session = session_arc(session_id)?;
    let mut guard = session.lock().map_err(|_| "terminal lock poisoned")?;
    guard.resize(cols, rows).map_err(|e| e.to_string())
}
