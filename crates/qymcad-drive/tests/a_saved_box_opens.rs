use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"qymcad-drive","version":"0.1.1"}}}"#;
const NOTIFIED: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
const LIST: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;
const MISSING: &str = "the saved box is not the document";

/// A 15 mm box is saved, a second rectangle is drawn, and opening the file brings the saved box back.
#[test]
fn the_saved_box_opens() {
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

    let drawn_msg = call(&mut input, &lines, 3, &first_rectangle());
    same_text(&drawn_msg);
    let drawn = Drawn { profile: profile_of(&structured(&drawn_msg)) };
    let made_msg = call(&mut input, &lines, 4, &extrude_call(&drawn.profile));
    same_text(&made_msg);
    let saved_read = call(&mut input, &lines, 5, &read_call(5));
    same_text(&saved_read);
    let wrote_msg = call(&mut input, &lines, 6, &save_call(&filed));
    same_text(&wrote_msg);
    let wrote = structured(&wrote_msg);
    if wrote.as_object().is_none_or(|fields| !fields.is_empty()) {
        panic!("{MISSING}");
    }
    let extra_msg = call(&mut input, &lines, 7, &second_rectangle());
    same_text(&extra_msg);
    let diverged_msg = call(&mut input, &lines, 8, &read_call(8));
    same_text(&diverged_msg);
    two_sketches(&structured(&diverged_msg));
    let opened_msg = call(&mut input, &lines, 9, &open_call(&filed));
    same_text(&opened_msg);
    let opened = structured(&opened_msg);
    if opened.as_object().is_none_or(|fields| !fields.is_empty()) {
        panic!("{MISSING}");
    }
    let read = call(&mut input, &lines, 10, &read_call(10));
    same_text(&read);
    the_saved_box(&structured(&read));
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

fn first_rectangle() -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "sketch.rectangle",
            "arguments": { "plane": "xz", "from": { "x": 0, "y": 0 }, "to": { "x": 10, "y": 20 } }
        }
    })
    .to_string()
}

fn second_rectangle() -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 7,
        "method": "tools/call",
        "params": {
            "name": "sketch.rectangle",
            "arguments": { "plane": "xz", "from": { "x": 0, "y": 0 }, "to": { "x": 1, "y": 1 } }
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
            "arguments": { "sketch": drawn.sketch.id, "contour": drawn.contour.id, "height": 15 }
        }
    })
    .to_string()
}

struct Drawn {
    profile: qymcad_drive::Profile,
}

struct Filed {
    path: std::path::PathBuf,
}

impl Filed {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("qymcad-drive-open-mcp-{}.qcad", std::process::id()));
        remove(&path);
        Self { path }
    }
}

impl Drop for Filed {
    fn drop(&mut self) {
        remove(&self.path);
    }
}

fn remove(path: &std::path::Path) {
    let text = path.display().to_string();
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(format!("{text}.bak"));
    let _ = std::fs::remove_file(format!("{text}.tmp~"));
}

fn save_call(filed: &Filed) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "tools/call",
        "params": {
            "name": "document.save",
            "arguments": { "path": filed.path.display().to_string() }
        }
    })
    .to_string()
}

fn open_call(filed: &Filed) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 9,
        "method": "tools/call",
        "params": {
            "name": "document.open",
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

fn two_sketches(structured: &serde_json::Value) {
    quiet(structured);
    let sketches = structured.get("sketches").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if sketches.len() != 2 {
        panic!("{MISSING}");
    }
    let bodies = structured.get("bodies").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if bodies.len() != 1 {
        panic!("{MISSING}");
    }
    one_box(&bodies[0]);
}

fn the_saved_box(structured: &serde_json::Value) {
    quiet(structured);
    let owners = Owners { sketch: one_sketch(structured), part: two_components(structured) };
    two_nodes(structured, &owners);
    let bodies = structured.get("bodies").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if bodies.len() != 1 {
        panic!("{MISSING}");
    }
    one_box(&bodies[0]);
}

fn quiet(structured: &serde_json::Value) {
    if structured.get("rebuild").and_then(serde_json::Value::as_str) != Some("quiet") {
        panic!("{MISSING}");
    }
    let refusals = structured.get("refusals").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if !refusals.is_empty() {
        panic!("{MISSING}");
    }
}

fn two_components(structured: &serde_json::Value) -> u64 {
    let components = structured.get("components").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if components.len() != 2 {
        panic!("{MISSING}");
    }
    let root = components[0].get("id").and_then(serde_json::Value::as_u64).unwrap_or_else(|| panic!("{MISSING}"));
    if components[0].get("kind").and_then(serde_json::Value::as_str) != Some("assembly")
        || components[0].get("name").and_then(serde_json::Value::as_str) != Some("name-assembly")
        || !components[0].get("parent").is_some_and(serde_json::Value::is_null)
    {
        panic!("{MISSING}");
    }
    if components[1].get("kind").and_then(serde_json::Value::as_str) != Some("part")
        || components[1].get("name").and_then(serde_json::Value::as_str) != Some("name-part-n#1")
        || components[1].get("parent").and_then(serde_json::Value::as_u64) != Some(root)
    {
        panic!("{MISSING}");
    }
    components[1].get("id").and_then(serde_json::Value::as_u64).unwrap_or_else(|| panic!("{MISSING}"))
}

fn one_sketch(structured: &serde_json::Value) -> u64 {
    let sketches = structured.get("sketches").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if sketches.len() != 1 {
        panic!("{MISSING}");
    }
    let sketch = &sketches[0];
    if sketch.get("name").and_then(serde_json::Value::as_str) != Some("name-sketch-n#1") || sketch.get("plane").and_then(serde_json::Value::as_str) != Some("xz") {
        panic!("{MISSING}");
    }
    let contours = sketch.get("contours").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if contours.len() != 1 || contours[0].get("form").and_then(serde_json::Value::as_str) != Some("closed") {
        panic!("{MISSING}");
    }
    sketch.get("id").and_then(serde_json::Value::as_u64).unwrap_or_else(|| panic!("{MISSING}"))
}

struct Owners {
    sketch: u64,
    part: u64,
}

fn two_nodes(structured: &serde_json::Value, owners: &Owners) {
    let nodes = structured.get("nodes").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if nodes.len() != 2 || nodes.iter().any(|node| node.get("name").and_then(serde_json::Value::as_str) == Some("name-sketch-n#2")) {
        panic!("{MISSING}");
    }
    let sketch_node = &nodes[0];
    let extrude = &nodes[1];
    if sketch_node.get("name").and_then(serde_json::Value::as_str) != Some("name-sketch-n#1")
        || sketch_node.get("id").and_then(serde_json::Value::as_u64) != Some(owners.sketch)
        || sketch_node.get("parent").and_then(serde_json::Value::as_u64) != Some(owners.part)
    {
        panic!("{MISSING}");
    }
    if extrude.get("name").and_then(serde_json::Value::as_str) != Some("feat-name-extrude") || extrude.get("parent").and_then(serde_json::Value::as_u64) != Some(owners.part) {
        panic!("{MISSING}");
    }
}

fn one_box(body: &serde_json::Value) {
    if body.get("sound").and_then(serde_json::Value::as_str) != Some("valid") || body.get("solids").and_then(serde_json::Value::as_u64) != Some(1) {
        panic!("{MISSING}");
    }
    let volume = body.get("volume").and_then(serde_json::Value::as_f64).unwrap_or_else(|| panic!("{MISSING}"));
    let box_mm3 = 10.0 * 20.0 * 15.0;
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
