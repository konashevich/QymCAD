use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"qymcad-drive","version":"0.1.1"}}}"#;
const NOTIFIED: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
const LIST: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;

/// The process draws the 10 by 20 by 30 mm box. A second session reads the saved file and does not draw.
#[test]
fn the_box_comes_through() {
    let path = process_path();
    let kept = Kept::new();
    let mut child = Command::new(&path).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().unwrap_or_else(|_| panic!("the process did not answer"));
    let stdout = child.stdout.take().unwrap_or_else(|| panic!("the process did not answer"));
    let mut input = child.stdin.take().unwrap_or_else(|| panic!("the process did not answer"));
    let mut running = Running { child };
    let lines = spawn_reader(stdout);
    write_line(&mut input, INITIALIZE);
    let greeted = next_line(&lines);
    if id_of(&greeted) != 1 {
        panic!("the process did not answer");
    }
    write_line(&mut input, NOTIFIED);
    write_line(&mut input, LIST);
    let listed = next_line(&lines);
    if id_of(&listed) != 2 {
        panic!("the process did not answer");
    }
    let tools = tools_of(&listed);
    // A missing sketch tool means the box was never drawn. Stop before a call the process would ignore.
    let _sketch = tool_named(&tools, "sketch.rectangle");
    if listed.get("result").and_then(|result| result.get("nextCursor")).is_some() || tools.len() != 52 {
        panic!("the box is not in the document");
    }
    prepared(tool_named(&tools, "sketch.rectangle"));
    prepared(tool_named(&tools, "sketch.datum"));
    prepared(tool_named(&tools, "sketch.line"));
    prepared(tool_named(&tools, "sketch.plane"));
    prepared(tool_named(&tools, "sketch.resize"));
    prepared(tool_named(&tools, "document.back"));
    prepared(tool_named(&tools, "document.mate"));
    prepared(tool_named(&tools, "document.parameter"));
    prepared(tool_named(&tools, "document.part"));
    prepared(tool_named(&tools, "part.extrude"));
    prepared(tool_named(&tools, "part.cuboid"));
    prepared(tool_named(&tools, "part.cylinder"));
    prepared(tool_named(&tools, "part.draft"));
    prepared(tool_named(&tools, "part.sphere"));
    prepared(tool_named(&tools, "part.split"));
    prepared(tool_named(&tools, "part.split_faces"));
    prepared(tool_named(&tools, "part.stitch"));
    prepared(tool_named(&tools, "part.sweep"));
    prepared(tool_named(&tools, "part.thicken"));
    prepared(tool_named(&tools, "part.thread"));
    prepared(tool_named(&tools, "part.cone"));
    prepared(tool_named(&tools, "part.copy_face"));
    prepared(tool_named(&tools, "part.torus"));
    prepared(tool_named(&tools, "part.trim"));
    prepared(tool_named(&tools, "part.prism"));
    prepared(tool_named(&tools, "part.push"));
    prepared(tool_named(&tools, "part.remove_face"));
    prepared(tool_named(&tools, "part.replace_face"));
    prepared(tool_named(&tools, "part.revolve"));
    prepared(tool_named(&tools, "part.bind"));
    prepared(tool_named(&tools, "part.boolean"));
    prepared(tool_named(&tools, "part.fillet"));
    prepared(tool_named(&tools, "part.chamfer"));
    prepared(tool_named(&tools, "part.circular"));
    prepared(tool_named(&tools, "part.shell"));
    prepared(tool_named(&tools, "part.hole"));
    prepared(tool_named(&tools, "part.linear"));
    prepared(tool_named(&tools, "part.loft"));
    prepared(tool_named(&tools, "part.mirror"));
    prepared(tool_named(&tools, "part.move"));
    prepared(tool_named(&tools, "part.offset_face"));
    prepared(tool_named(&tools, "part.patch"));
    prepared(tool_named(&tools, "document.read"));
    prepared(tool_named(&tools, "document.save"));
    prepared(tool_named(&tools, "document.save_step"));
    prepared(tool_named(&tools, "document.save_stl"));
    prepared(tool_named(&tools, "document.show"));
    prepared(tool_named(&tools, "document.open_stl"));
    prepared(tool_named(&tools, "document.open"));
    prepared(tool_named(&tools, "document.open_step"));
    prepared(tool_named(&tools, "document.slide"));
    prepared(tool_named(&tools, "part.auger"));

    let drawn = call(&mut input, &lines, 3, &rectangle_call());
    same_text(&drawn);
    let drawn_ids = Drawn { sketch: qymcad_drive::SketchId { id: number_at(&structured(&drawn), "sketch") }, contour: qymcad_drive::ContourId { id: number_at(&structured(&drawn), "contour") } };
    let made = call(&mut input, &lines, 4, &extrude_call(&drawn_ids));
    same_text(&made);
    let body = number_at(&structured(&made), "body");
    let read = call(&mut input, &lines, 5, &read_call());
    same_text(&read);
    let read_fields = structured(&read);
    read_box(&read_fields);
    tree_holds(&read_fields, body);
    let saved = call(&mut input, &lines, 6, &save_call(&kept));
    same_text(&saved);
    opened_box(&kept);
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
            "arguments": { "plane": "xz", "from": { "x": 0, "y": 0 }, "to": { "x": 10, "y": 20 } }
        }
    })
    .to_string()
}

struct Drawn {
    sketch: qymcad_drive::SketchId,
    contour: qymcad_drive::ContourId,
}

fn extrude_call(drawn: &Drawn) -> String {
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

fn read_call() -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "tools/call",
        "params": { "name": "document.read" }
    })
    .to_string()
}

fn save_call(kept: &Kept) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "tools/call",
        "params": {
            "name": "document.save",
            "arguments": { "path": kept.path.display().to_string() }
        }
    })
    .to_string()
}

fn tools_of(msg: &serde_json::Value) -> Vec<serde_json::Value> {
    msg.get("result").and_then(|result| result.get("tools")).and_then(serde_json::Value::as_array).cloned().unwrap_or_else(|| panic!("the process did not answer"))
}

fn tool_named<'a>(tools: &'a [serde_json::Value], name: &str) -> &'a serde_json::Value {
    tools.iter().find(|tool| tool.get("name").and_then(serde_json::Value::as_str) == Some(name)).unwrap_or_else(|| panic!("the box is not in the document"))
}

fn prepared(tool: &serde_json::Value) {
    if tool.get("description").and_then(serde_json::Value::as_str).is_none() {
        panic!("the box is not in the document");
    }
    if tool.get("inputSchema").and_then(serde_json::Value::as_object).is_none() {
        panic!("the box is not in the document");
    }
}

fn structured(msg: &serde_json::Value) -> serde_json::Value {
    msg.get("result").and_then(|result| result.get("structuredContent")).cloned().unwrap_or_else(|| panic!("the box is not in the document"))
}

fn same_text(msg: &serde_json::Value) {
    let result = msg.get("result").unwrap_or_else(|| panic!("the box is not in the document"));
    if result.get("isError").and_then(serde_json::Value::as_bool) == Some(true) {
        panic!("the box is not in the document");
    }
    let structured = result.get("structuredContent").unwrap_or_else(|| panic!("the box is not in the document"));
    let text = result
        .get("content")
        .and_then(serde_json::Value::as_array)
        .and_then(|content| content.first())
        .and_then(|item| item.get("text"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("the box is not in the document"));
    if text.contains('\n') {
        panic!("the box is not in the document");
    }
    let parsed: serde_json::Value = serde_json::from_str(text).unwrap_or_else(|_| panic!("the box is not in the document"));
    if parsed != *structured {
        panic!("the box is not in the document");
    }
}

fn number_at(value: &serde_json::Value, key: &str) -> u64 {
    value.get(key).and_then(serde_json::Value::as_u64).unwrap_or_else(|| panic!("the box is not in the document"))
}

fn read_box(structured: &serde_json::Value) {
    if structured.get("rebuild").and_then(serde_json::Value::as_str) != Some("quiet") {
        panic!("the box is not in the document");
    }
    let bodies = structured.get("bodies").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the box is not in the document"));
    let Some(body) = bodies.first() else {
        panic!("the box is not in the document");
    };
    if bodies.len() != 1 || body.get("sound").and_then(serde_json::Value::as_str) != Some("valid") || body.get("solids").and_then(serde_json::Value::as_u64) != Some(1) {
        panic!("the box is not in the document");
    }
    let volume = body.get("volume").and_then(serde_json::Value::as_f64).unwrap_or_else(|| panic!("the box is not in the document"));
    if (volume - 6000.0).abs() >= 1.0 {
        panic!("the box is not in the document");
    }
}

fn opened_box(kept: &Kept) {
    let mut opened = qymcad_drive::Session::open(&kept.path).unwrap_or_else(|_| panic!("the box is not in the document"));
    let doc = opened.document();
    if doc.rebuild != qymcad_drive::Rebuild::Quiet || doc.bodies.len() != 1 {
        panic!("the box is not in the document");
    }
    let body = &doc.bodies[0];
    if body.sound != qymcad_drive::Sound::Valid {
        panic!("the box is not in the document");
    }
    match body.measure {
        qymcad_drive::Measure::Live { volume, solids } if solids == 1 && (volume.mm3 - 6000.0).abs() < 1.0 => {}
        _ => panic!("the box is not in the document"),
    }
}

fn tree_holds(structured: &serde_json::Value, body: u64) {
    let sketches = structured.get("sketches").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the box is not in the document"));
    if sketches.len() != 1 {
        panic!("the box is not in the document");
    }
    let nodes = structured.get("nodes").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the box is not in the document"));
    if !nodes.iter().any(|node| node.get("id").and_then(serde_json::Value::as_u64) == Some(body)) {
        panic!("the box is not in the document");
    }
    let components = structured.get("components").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the box is not in the document"));
    if !components.iter().any(|component| component.get("kind").and_then(serde_json::Value::as_str) == Some("part")) {
        panic!("the box is not in the document");
    }
}

fn call(input: &mut impl Write, lines: &Lines, id: i64, line: &str) -> serde_json::Value {
    write_line(input, line);
    let msg = next_line(lines);
    if id_of(&msg) != id || msg.get("jsonrpc").and_then(serde_json::Value::as_str) != Some("2.0") {
        panic!("the process did not answer");
    }
    msg
}

struct Kept {
    path: std::path::PathBuf,
}

impl Kept {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("qymcad-drive-mcp-{}.qcad", std::process::id()));
        remove(&path);
        Self { path }
    }
}

impl Drop for Kept {
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
