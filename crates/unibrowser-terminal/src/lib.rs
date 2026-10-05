//! Pseudo-terminal sessions for the integrated terminal panel.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TerminalError {
    #[error("PTY: {0}")]
    Pty(String),
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct SpawnOptions {
    pub cwd: Option<PathBuf>,
    pub cols: u16,
    pub rows: u16,
}

impl Default for SpawnOptions {
    fn default() -> Self {
        Self {
            cwd: None,
            cols: 120,
            rows: 32,
        }
    }
}

pub struct TerminalSession {
    master: Box<dyn portable_pty::MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    output_rx: Receiver<Vec<u8>>,
    _reader_thread: thread::JoinHandle<()>,
}

impl TerminalSession {
    pub fn spawn(opts: SpawnOptions) -> Result<Self, TerminalError> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: opts.rows,
                cols: opts.cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| TerminalError::Pty(e.to_string()))?;
        let shell = default_shell();
        let mut cmd = CommandBuilder::new(&shell.program);
        for arg in &shell.args {
            cmd.arg(arg);
        }
        if let Some(cwd) = &opts.cwd {
            cmd.cwd(cwd);
        }
        let _child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| TerminalError::Pty(e.to_string()))?;
        let master = pair.master;
        let mut reader = master
            .try_clone_reader()
            .map_err(|e| TerminalError::Pty(e.to_string()))?;
        let writer = master
            .take_writer()
            .map_err(|e| TerminalError::Pty(e.to_string()))?;
        let (tx, output_rx) = mpsc::channel();
        let reader_thread = thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(_) => {
                        thread::sleep(Duration::from_millis(10));
                    }
                }
            }
        });
        Ok(Self {
            master,
            writer,
            output_rx,
            _reader_thread: reader_thread,
        })
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<(), TerminalError> {
        self.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| TerminalError::Pty(e.to_string()))
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), TerminalError> {
        self.writer.write_all(data)?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn drain_output(&mut self) -> Vec<Vec<u8>> {
        let mut chunks = Vec::new();
        while let Ok(chunk) = self.output_rx.try_recv() {
            chunks.push(chunk);
        }
        chunks
    }

    pub fn try_recv_output(&mut self) -> Option<Vec<u8>> {
        match self.output_rx.try_recv() {
            Ok(v) => Some(v),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => None,
        }
    }
}

struct ShellSpec {
    program: String,
    args: Vec<String>,
}

fn default_shell() -> ShellSpec {
    #[cfg(windows)]
    {
        ShellSpec {
            program: "powershell.exe".into(),
            args: vec!["-NoLogo".into()],
        }
    }
    #[cfg(not(windows))]
    {
        let program = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into());
        ShellSpec {
            program,
            args: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn spawn_shell_emits_output() {
        let mut session = TerminalSession::spawn(SpawnOptions {
            cwd: None,
            cols: 80,
            rows: 24,
        })
        .expect("spawn pty");
        #[cfg(windows)]
        let script = "Write-Output PTY_OK\r\n";
        #[cfg(not(windows))]
        let script = "echo PTY_OK\n";
        session.write(script.as_bytes()).expect("write");
        let deadline = Instant::now() + Duration::from_secs(8);
        let mut collected = String::new();
        while Instant::now() < deadline {
            for chunk in session.drain_output() {
                collected.push_str(&String::from_utf8_lossy(&chunk));
            }
            if collected.contains("PTY_OK") {
                return;
            }
            thread::sleep(Duration::from_millis(50));
        }
        panic!("expected PTY_OK in output, got: {}", collected);
    }
}
