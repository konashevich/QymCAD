use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"qymcad-drive","version":"0.1.1"}}}"#;
const NOTIFIED: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
const LIST: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;

/// The process answers `initialize`, names the fifty-two tools, and exits when its input closes.
#[test]
fn the_process_answers() {
    let path = process_path();
    let mut child = Command::new(path).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().unwrap_or_else(|_| panic!("the process did not answer"));
    let stdout = child.stdout.take().unwrap_or_else(|| panic!("the process did not answer"));
    let mut input = child.stdin.take().unwrap_or_else(|| panic!("the process did not answer"));
    let mut running = Running { child };
    let lines = spawn_reader(stdout);
    write_line(&mut input, INITIALIZE);
    opened(&next_line(&lines));
    write_line(&mut input, NOTIFIED);
    write_line(&mut input, LIST);
    listed(&next_line(&lines));
    drop(input);
    finished(&mut running);
    let _ended = lines.reader.join();
    assert!(lines.rx.try_recv().is_err(), "the handshake");
}

fn process_path() -> std::ffi::OsString {
    // Cargo names this variable with the binary's name, hyphen included.
    if let Some(path) = std::env::var_os("CARGO_BIN_EXE_qymcad-mcp") {
        return path;
    }
    if let Some(path) = option_env!("CARGO_BIN_EXE_qymcad-mcp") {
        return path.into();
    }
    panic!("the process did not answer");
}

struct Running {
    child: Child,
}

impl Drop for Running {
    fn drop(&mut self) {
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

struct Lines {
    rx: Receiver<String>,
    reader: std::thread::JoinHandle<()>,
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
    Lines { rx, reader }
}

fn write_line(input: &mut impl Write, line: &str) {
    writeln!(input, "{line}").unwrap_or_else(|_| panic!("the process did not answer"));
    input.flush().unwrap_or_else(|_| panic!("the process did not answer"));
}

fn next_line(lines: &Lines) -> serde_json::Value {
    let line = lines.rx.recv_timeout(Duration::from_secs(5)).unwrap_or_else(|_| panic!("the process did not answer"));
    serde_json::from_str(&line).unwrap_or_else(|_| panic!("the process did not answer"))
}

fn opened(msg: &serde_json::Value) {
    assert_eq!(id_of(msg), 1, "the id");
    protocol(msg);
    let result = msg.get("result").unwrap_or_else(|| panic!("the process did not answer"));
    assert_eq!(result.get("protocolVersion").and_then(serde_json::Value::as_str), Some("2025-11-25"), "the revision");
    let info = result.get("serverInfo").unwrap_or_else(|| panic!("the process did not answer"));
    assert_eq!(info.get("name").and_then(serde_json::Value::as_str), Some("qymcad"), "the name");
    assert_eq!(info.get("version").and_then(serde_json::Value::as_str), Some("0.1.1"), "the version");
    let tools = result.get("capabilities").and_then(|caps| caps.get("tools")).and_then(serde_json::Value::as_object).unwrap_or_else(|| panic!("the process did not answer"));
    assert!(tools.get("listChanged").is_none(), "the list will change");
}

fn listed(msg: &serde_json::Value) {
    assert_eq!(id_of(msg), 2, "the id");
    protocol(msg);
    let tools = msg.get("result").and_then(|result| result.get("tools")).and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the process did not answer"));
    let mut names = Vec::new();
    for tool in tools {
        let name = tool.get("name").and_then(serde_json::Value::as_str).unwrap_or_else(|| panic!("the tools"));
        assert!(tool.get("description").and_then(serde_json::Value::as_str).is_some(), "the tools");
        assert!(tool.get("inputSchema").and_then(serde_json::Value::as_object).is_some(), "the tools");
        names.push(name);
    }
    names.sort_unstable();
    assert_eq!(
        names,
        [
            "document.back", "document.mate", "document.open", "document.open_step", "document.open_stl", "document.parameter", "document.part", "document.read", "document.save",
            "document.save_step", "document.save_stl", "document.show", "document.slide", "part.auger", "part.bind", "part.boolean", "part.chamfer", "part.circular", "part.cone", "part.copy_face",
            "part.cuboid", "part.cylinder", "part.draft", "part.extrude", "part.fillet", "part.hole", "part.linear", "part.loft", "part.mirror", "part.move", "part.offset_face", "part.patch",
            "part.prism", "part.push", "part.remove_face", "part.replace_face", "part.revolve", "part.shell", "part.sphere", "part.split", "part.split_faces", "part.stitch", "part.sweep",
            "part.thicken", "part.thread", "part.torus", "part.trim", "sketch.datum", "sketch.line", "sketch.plane", "sketch.rectangle", "sketch.resize"
        ],
        "the tools"
    );
}

fn protocol(msg: &serde_json::Value) {
    assert_eq!(msg.get("jsonrpc").and_then(serde_json::Value::as_str), Some("2.0"), "the protocol");
}

fn id_of(msg: &serde_json::Value) -> i64 {
    msg.get("id").and_then(serde_json::Value::as_i64).unwrap_or_else(|| panic!("the process did not answer"))
}

fn finished(running: &mut Running) {
    let started = Instant::now();
    loop {
        match running.child.try_wait() {
            Ok(Some(status)) => {
                assert!(status.success(), "the process did not finish");
                return;
            }
            Ok(None) if started.elapsed() < Duration::from_secs(5) => std::thread::sleep(Duration::from_millis(20)),
            _ => {
                let _ = running.child.kill();
                let _ = running.child.wait();
                panic!("the process did not finish");
            }
        }
    }
}
