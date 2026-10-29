//! The process starts another copy of the graphical program on a saved file.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

const MISSING: &str = "the program was not asked";

static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// The copy is started with the path, marked as a side copy, and without the open program's socket.
#[test]
fn the_running_program_is_asked_to_open_the_file() {
    let held = Held::new();
    let mut running = started(&held);
    let call = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": { "name": "document.show", "arguments": { "path": held.file } }
    });
    write_line(&mut running.input, &call.to_string());
    let reply = next_line(&running.lines);
    delivered(&reply);
    let asked = held.record();
    assert_eq!(asked, format!("{}\n1\nno-queue\n", held.file), "{MISSING}");
}

/// The executable is not there, so the process refuses and does not start a copy.
#[test]
fn a_stopped_program_is_refused() {
    let held = Held::new();
    let gone = held.program.with_extension("missing");
    let _ = std::fs::remove_file(&gone);
    let mut running = started_with(&held, &gone);
    let call = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": { "name": "document.show", "arguments": { "path": held.file } }
    });
    write_line(&mut running.input, &call.to_string());
    let reply = next_line(&running.lines);
    refused(&reply, "The graphical application is missing.");
    assert!(!held.note.exists(), "{MISSING}");
}

/// A path that is not a file is refused before a copy is started.
#[test]
fn a_missing_file_is_refused() {
    let held = Held::new();
    let mut running = started(&held);
    let call = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": { "name": "document.show", "arguments": { "path": format!("{}-gone", held.file) } }
    });
    write_line(&mut running.input, &call.to_string());
    let reply = next_line(&running.lines);
    refused(&reply, "The file is missing.");
    assert!(!held.note.exists(), "{MISSING}");
}

struct Held {
    program: std::path::PathBuf,
    file: String,
    note: std::path::PathBuf,
}

impl Held {
    fn new() -> Self {
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("qymcad-show-{}-{n}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap_or_else(|_| panic!("{MISSING}"));
        let program = dir.join("gui");
        let note = dir.join("asked");
        let script = "#!/bin/sh\n{\nprintf '%s\\n' \"$1\"\nprintf '%s\\n' \"${QYMCAD_SIDE-}\"\nif [ -n \"${QYMCAD_COMMAND_QUEUE+x}\" ]; then\nprintf 'queue\\n'\nelse\nprintf 'no-queue\\n'\nfi\n} > \"$QYMCAD_SHOW_RECORD\"\n";
        std::fs::write(&program, script).unwrap_or_else(|_| panic!("{MISSING}"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).unwrap_or_else(|_| panic!("{MISSING}"));
        }
        let path = dir.join("cap.qcad");
        std::fs::write(&path, b"").unwrap_or_else(|_| panic!("{MISSING}"));
        let file = path.display().to_string();
        Self { program, file, note }
    }

    fn record(&self) -> String {
        let started = std::time::Instant::now();
        loop {
            if let Ok(text) = std::fs::read_to_string(&self.note) {
                if text.contains("no-queue") || text.contains("queue") {
                    return text;
                }
            }
            if started.elapsed() > Duration::from_secs(5) {
                panic!("{MISSING}");
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

impl Drop for Held {
    fn drop(&mut self) {
        if let Some(dir) = self.program.parent() {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

struct Started {
    child: Child,
    input: std::process::ChildStdin,
    lines: Lines,
}

impl Drop for Started {
    fn drop(&mut self) {
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        if let Some(reader) = self.lines.reader.take() {
            let _ = reader.join();
        }
    }
}

fn started(held: &Held) -> Started {
    started_with(held, &held.program)
}

fn started_with(held: &Held, program: &std::path::Path) -> Started {
    let mut child = Command::new(process_path())
        .env("QYMCAD_GUI", program)
        .env("QYMCAD_SHOW_RECORD", &held.note)
        .env("QYMCAD_COMMAND_QUEUE", "/tmp/qymcad-show-must-not-inherit.sock")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|_| panic!("{MISSING}"));
    let stdout = child.stdout.take().unwrap_or_else(|| panic!("{MISSING}"));
    let input = child.stdin.take().unwrap_or_else(|| panic!("{MISSING}"));
    Started { child, input, lines: spawn_reader(stdout) }
}

fn process_path() -> std::ffi::OsString {
    if let Some(path) = std::env::var_os("CARGO_BIN_EXE_qymcad-mcp") {
        return path;
    }
    if let Some(path) = option_env!("CARGO_BIN_EXE_qymcad-mcp") {
        return path.into();
    }
    panic!("{MISSING}");
}

struct Lines {
    rx: Receiver<String>,
    reader: Option<std::thread::JoinHandle<()>>,
}

fn spawn_reader(stdout: std::process::ChildStdout) -> Lines {
    let (tx, rx) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        let mut incoming = BufReader::new(stdout);
        let mut line = String::new();
        loop {
            line.clear();
            match incoming.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    if tx.send(line.trim_end_matches(['\r', '\n']).to_string()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
    Lines { rx, reader: Some(reader) }
}

fn write_line(input: &mut impl Write, line: &str) {
    writeln!(input, "{line}").unwrap_or_else(|_| panic!("{MISSING}"));
    input.flush().unwrap_or_else(|_| panic!("{MISSING}"));
}

fn next_line(lines: &Lines) -> serde_json::Value {
    let line = lines.rx.recv_timeout(Duration::from_secs(5)).unwrap_or_else(|_| panic!("{MISSING}"));
    serde_json::from_str(&line).unwrap_or_else(|_| panic!("{MISSING}"))
}

fn delivered(msg: &serde_json::Value) {
    let result = msg.get("result").unwrap_or_else(|| panic!("{MISSING}"));
    assert_ne!(result.get("isError").and_then(serde_json::Value::as_bool), Some(true), "{MISSING}");
    assert_eq!(result.get("structuredContent").and_then(serde_json::Value::as_object).map(serde_json::Map::len), Some(0), "{MISSING}");
}

fn refused(msg: &serde_json::Value, text: &str) {
    let result = msg.get("result").unwrap_or_else(|| panic!("{MISSING}"));
    assert_eq!(result.get("isError").and_then(serde_json::Value::as_bool), Some(true), "{MISSING}");
    let said = result.get("content").and_then(serde_json::Value::as_array).and_then(|content| content.first()).and_then(|item| item.get("text")).and_then(serde_json::Value::as_str);
    assert_eq!(said, Some(text), "{MISSING}");
}
