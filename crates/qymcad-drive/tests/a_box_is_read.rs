use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"qymcad-drive","version":"0.1.1"}}}"#;
const NOTIFIED: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
const LIST: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;
const MISSING: &str = "the box is not in the file";

/// The STEP file of the 10 by 20 by 30 mm box is read back as one valid solid. The document still holds the box.
#[test]
fn the_box_is_read() {
    let path = process_path();
    let filed = Filed::new();
    let mut child = Command::new(path).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().unwrap_or_else(|_| panic!("the process did not answer"));
    let stdout = child.stdout.take().unwrap_or_else(|| panic!("the process did not answer"));
    let mut input = child.stdin.take().unwrap_or_else(|| panic!("the process did not answer"));
    let mut running = Running { child };
    let lines = spawn_reader(stdout);
    write_line(&mut input, INITIALIZE);
    let greeted = next_line(&lines);
    if id_of(&greeted) != 1 || greeted.get("jsonrpc").and_then(serde_json::Value::as_str) != Some("2.0") {
        panic!("the process did not answer");
    }
    write_line(&mut input, NOTIFIED);
    write_line(&mut input, LIST);
    let listed = next_line(&lines);
    if id_of(&listed) != 2 {
        panic!("the process did not answer");
    }

    let drawn_msg = call(&mut input, &lines, 3, &rectangle_call());
    same_text(&drawn_msg);
    let drawn = Drawn { profile: profile_of(&structured(&drawn_msg)) };
    let made_msg = call(&mut input, &lines, 4, &extrude_call(&drawn.profile));
    same_text(&made_msg);
    let block = Block { body: qymcad_drive::BodyId { id: number_at(&structured(&made_msg), "body") } };
    let built = call(&mut input, &lines, 5, &read_call(5));
    same_text(&built);
    let wrote_msg = call(&mut input, &lines, 6, &step_call(&filed));
    same_text(&wrote_msg);
    let wrote = structured(&wrote_msg);
    if wrote.as_object().is_none_or(|fields| !fields.is_empty()) {
        panic!("{MISSING}");
    }
    let opened_msg = call(&mut input, &lines, 7, &open_call(&filed));
    same_text(&opened_msg);
    the_solids(&structured(&opened_msg));
    let read = call(&mut input, &lines, 8, &read_call(8));
    same_text(&read);
    still_the_box(&structured(&read), &block);
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

fn rectangle_call() -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "sketch.rectangle",
            "arguments": { "plane": "xy", "from": { "x": 0, "y": 0 }, "to": { "x": 10, "y": 20 } }
        }
    })
    .to_string()
}

fn profile_of(reply: &serde_json::Value) -> qymcad_drive::Profile {
    qymcad_drive::Profile { sketch: qymcad_drive::SketchId { id: number_at(reply, "sketch") }, contour: qymcad_drive::ContourId { id: number_at(reply, "contour") } }
}

fn extrude_call(drawn: &qymcad_drive::Profile) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "part.extrude",
            "arguments": { "sketch": drawn.sketch.id, "contour": drawn.contour.id, "height": 30 }
        }
    })
    .to_string()
}

struct Drawn {
    profile: qymcad_drive::Profile,
}

struct Block {
    body: qymcad_drive::BodyId,
}

struct Filed {
    path: std::path::PathBuf,
}

impl Filed {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("qymcad-drive-step-read-mcp-{}.step", std::process::id()));
        let _ = std::fs::remove_file(&path);
        Self { path }
    }
}

impl Drop for Filed {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn step_call(filed: &Filed) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "tools/call",
        "params": {
            "name": "document.save_step",
            "arguments": { "path": filed.path.display().to_string() }
        }
    })
    .to_string()
}

fn open_call(filed: &Filed) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 7,
        "method": "tools/call",
        "params": {
            "name": "document.open_step",
            "arguments": { "path": filed.path.display().to_string() }
        }
    })
    .to_string()
}

fn read_call(id: i64) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "tools/call",
        "params": { "name": "document.read" }
    })
    .to_string()
}

fn the_solids(reply: &serde_json::Value) {
    if reply.as_object().is_none_or(|fields| fields.len() != 1 || !fields.contains_key("solids")) {
        panic!("{MISSING}");
    }
    let solids = reply.get("solids").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if solids.len() != 1 {
        panic!("{MISSING}");
    }
    let solid = &solids[0];
    if solid.get("sound").and_then(serde_json::Value::as_str) != Some("valid") || solid.get("solids").and_then(serde_json::Value::as_u64) != Some(1) {
        panic!("{MISSING}");
    }
    let volume = solid.get("volume").and_then(serde_json::Value::as_f64).unwrap_or_else(|| panic!("{MISSING}"));
    let box_mm3 = 10.0 * 20.0 * 30.0;
    if (volume - box_mm3).abs() >= 1.0 {
        panic!("{MISSING}");
    }
    near_point(solid.get("low").unwrap_or_else(|| panic!("{MISSING}")), [0.0, 0.0, 0.0]);
    near_point(solid.get("high").unwrap_or_else(|| panic!("{MISSING}")), [10.0, 20.0, 30.0]);
}

fn near_point(corner: &serde_json::Value, want: [f64; 3]) {
    let got = [axis(corner, "x"), axis(corner, "y"), axis(corner, "z")];
    for index in 0..3 {
        if (got[index] - want[index]).abs() >= 1e-6 {
            panic!("{MISSING}");
        }
    }
}

fn axis(corner: &serde_json::Value, name: &str) -> f64 {
    corner.get(name).and_then(serde_json::Value::as_f64).unwrap_or_else(|| panic!("{MISSING}"))
}

fn still_the_box(structured: &serde_json::Value, block: &Block) {
    if structured.get("rebuild").and_then(serde_json::Value::as_str) != Some("quiet") {
        panic!("{MISSING}");
    }
    let refusals = structured.get("refusals").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if !refusals.is_empty() {
        panic!("{MISSING}");
    }
    let bodies = structured.get("bodies").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if bodies.len() != 1 {
        panic!("{MISSING}");
    }
    let body = &bodies[0];
    if body.get("id").and_then(serde_json::Value::as_u64) != Some(block.body.id) {
        panic!("{MISSING}");
    }
    if body.get("sound").and_then(serde_json::Value::as_str) != Some("valid") || body.get("solids").and_then(serde_json::Value::as_u64) != Some(1) {
        panic!("{MISSING}");
    }
    let volume = body.get("volume").and_then(serde_json::Value::as_f64).unwrap_or_else(|| panic!("{MISSING}"));
    let box_mm3 = 10.0 * 20.0 * 30.0;
    if (volume - box_mm3).abs() >= 1.0 {
        panic!("{MISSING}");
    }
    let faces = body.get("faces").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    let edges = body.get("edges").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if faces.len() != 6 || edges.len() != 12 {
        panic!("{MISSING}");
    }
}

fn structured(msg: &serde_json::Value) -> serde_json::Value {
    msg.get("result").and_then(|result| result.get("structuredContent")).cloned().unwrap_or_else(|| panic!("the process did not answer"))
}

fn same_text(msg: &serde_json::Value) {
    let result = msg.get("result").unwrap_or_else(|| panic!("the process did not answer"));
    if result.get("isError").and_then(serde_json::Value::as_bool) == Some(true) {
        panic!("the process did not answer");
    }
    let structured = result.get("structuredContent").unwrap_or_else(|| panic!("the process did not answer"));
    let text = result
        .get("content")
        .and_then(serde_json::Value::as_array)
        .and_then(|content| content.first())
        .and_then(|item| item.get("text"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("the process did not answer"));
    if text.contains('\n') {
        panic!("the process did not answer");
    }
    let parsed: serde_json::Value = serde_json::from_str(text).unwrap_or_else(|_| panic!("the process did not answer"));
    if parsed != *structured {
        panic!("the process did not answer");
    }
}

fn number_at(value: &serde_json::Value, key: &str) -> u64 {
    value.get(key).and_then(serde_json::Value::as_u64).unwrap_or_else(|| panic!("the process did not answer"))
}

fn call(input: &mut impl Write, lines: &Lines, id: i64, line: &str) -> serde_json::Value {
    write_line(input, line);
    let msg = next_line(lines);
    if id_of(&msg) != id || msg.get("jsonrpc").and_then(serde_json::Value::as_str) != Some("2.0") {
        panic!("the process did not answer");
    }
    msg
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
