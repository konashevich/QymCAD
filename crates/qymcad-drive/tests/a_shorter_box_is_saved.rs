use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"qymcad-drive","version":"0.1.1"}}}"#;
const NOTIFIED: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
const LIST: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;
const MISSING: &str = "the expression is not in the file";

/// The height becomes half of `span`, the file is saved, and opening it brings that shorter box back.
#[test]
fn the_shorter_box_is_saved() {
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
    let made_msg = call(&mut input, &lines, 4, &extrude_call(&structured(&drawn_msg)));
    same_text(&made_msg);
    let made = Made { sketch: number_at(&structured(&drawn_msg), "sketch"), contour: number_at(&structured(&drawn_msg), "contour"), body: number_at(&structured(&made_msg), "body") };
    let built = call(&mut input, &lines, 5, &read_call(5));
    same_text(&built);
    tall_box(&structured(&built), &made);
    let stored = call(&mut input, &lines, 6, &parameter_call());
    same_text(&stored);
    empty_reply(&stored);
    let bound = call(&mut input, &lines, 7, &bind_call(&made));
    same_text(&bound);
    empty_reply(&bound);
    let live = call(&mut input, &lines, 8, &read_call(8));
    same_text(&live);
    short_box(&structured(&live), &made);
    let wrote = call(&mut input, &lines, 9, &save_call(&filed));
    same_text(&wrote);
    empty_reply(&wrote);
    let extra = call(&mut input, &lines, 10, &second_rectangle());
    same_text(&extra);
    let diverged = call(&mut input, &lines, 11, &read_call(11));
    same_text(&diverged);
    two_sketches(&structured(&diverged), &made);
    let opened = call(&mut input, &lines, 12, &open_call(&filed));
    same_text(&opened);
    empty_reply(&opened);
    let read = call(&mut input, &lines, 13, &read_call(13));
    same_text(&read);
    short_box(&structured(&read), &made);
    let widened = call(&mut input, &lines, 14, &wider_parameter());
    same_text(&widened);
    empty_reply(&widened);
    let grown = call(&mut input, &lines, 15, &read_call(15));
    same_text(&grown);
    taller_box(&structured(&grown), &made);
    drop(input);
    finished(&mut running);
    let _ended = lines.reader.join();
    assert!(lines.rx.try_recv().is_err(), "the handshake");
}

struct Made {
    sketch: u64,
    contour: u64,
    body: u64,
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

fn extrude_call(drawn: &serde_json::Value) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "part.extrude",
            "arguments": { "sketch": number_at(drawn, "sketch"), "contour": number_at(drawn, "contour"), "height": 30 }
        }
    })
    .to_string()
}

fn second_rectangle() -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 10,
        "method": "tools/call",
        "params": {
            "name": "sketch.rectangle",
            "arguments": { "plane": "xz", "from": { "x": 0, "y": 0 }, "to": { "x": 1, "y": 1 } }
        }
    })
    .to_string()
}

fn parameter_call() -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "tools/call",
        "params": {
            "name": "document.parameter",
            "arguments": { "name": "span", "expression": "40" }
        }
    })
    .to_string()
}

fn wider_parameter() -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 14,
        "method": "tools/call",
        "params": {
            "name": "document.parameter",
            "arguments": { "name": "span", "expression": "80" }
        }
    })
    .to_string()
}

fn bind_call(made: &Made) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 7,
        "method": "tools/call",
        "params": {
            "name": "part.bind",
            "arguments": { "body": made.body, "dimension": "height", "expression": "span/2" }
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

struct Filed {
    path: std::path::PathBuf,
}

impl Filed {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("qymcad-drive-short-mcp-{}.qcad", std::process::id()));
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
        "id": 9,
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
        "id": 12,
        "method": "tools/call",
        "params": {
            "name": "document.open",
            "arguments": { "path": filed.path.display().to_string() }
        }
    })
    .to_string()
}

fn tall_box(structured: &serde_json::Value, made: &Made) {
    quiet(structured);
    one_sketch(structured, made);
    let body = one_body(structured, made);
    let volume = body.get("volume").and_then(serde_json::Value::as_f64).unwrap_or_else(|| panic!("{MISSING}"));
    if (volume - 10.0 * 20.0 * 30.0).abs() >= 1.0 {
        panic!("{MISSING}");
    }
}

fn two_sketches(structured: &serde_json::Value, made: &Made) {
    quiet(structured);
    let sketches = structured.get("sketches").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if sketches.len() != 2 || !sketches.iter().any(|sketch| sketch.get("id").and_then(serde_json::Value::as_u64) == Some(made.sketch)) {
        panic!("{MISSING}");
    }
    let body = one_body(structured, made);
    let volume = body.get("volume").and_then(serde_json::Value::as_f64).unwrap_or_else(|| panic!("{MISSING}"));
    if (volume - 10.0 * 20.0 * 20.0).abs() >= 1.0 {
        panic!("{MISSING}");
    }
}

fn short_box(structured: &serde_json::Value, made: &Made) {
    quiet(structured);
    one_sketch(structured, made);
    the_extrusion(structured, made);
    let body = one_body(structured, made);
    let volume = body.get("volume").and_then(serde_json::Value::as_f64).unwrap_or_else(|| panic!("{MISSING}"));
    let short = 10.0 * 20.0 * 20.0;
    let tall = 10.0 * 20.0 * 30.0;
    if (volume - short).abs() >= 1.0 || (volume - tall).abs() < 1.0 {
        panic!("{MISSING}");
    }
    short_edges(body);
    short_faces(body);
}

fn taller_box(structured: &serde_json::Value, made: &Made) {
    quiet(structured);
    one_sketch(structured, made);
    the_extrusion(structured, made);
    let body = one_body(structured, made);
    let volume = body.get("volume").and_then(serde_json::Value::as_f64).unwrap_or_else(|| panic!("{MISSING}"));
    let grown = 10.0 * 20.0 * 40.0;
    let short = 10.0 * 20.0 * 20.0;
    if (volume - grown).abs() >= 1.0 || (volume - short).abs() < 1.0 {
        panic!("{MISSING}");
    }
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

fn one_sketch(structured: &serde_json::Value, made: &Made) {
    let sketches = structured.get("sketches").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if sketches.len() != 1 {
        panic!("{MISSING}");
    }
    let sketch = &sketches[0];
    if sketch.get("id").and_then(serde_json::Value::as_u64) != Some(made.sketch) || sketch.get("plane").and_then(serde_json::Value::as_str) != Some("xz") {
        panic!("{MISSING}");
    }
    let contours = sketch.get("contours").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if contours.len() != 1 || contours[0].get("id").and_then(serde_json::Value::as_u64) != Some(made.contour) || contours[0].get("form").and_then(serde_json::Value::as_str) != Some("closed") {
        panic!("{MISSING}");
    }
}

fn the_extrusion(structured: &serde_json::Value, made: &Made) {
    let nodes = structured.get("nodes").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if nodes.len() != 2 {
        panic!("{MISSING}");
    }
    if nodes[0].get("name").and_then(serde_json::Value::as_str) != Some("name-sketch-n#1") || nodes[0].get("id").and_then(serde_json::Value::as_u64) != Some(made.sketch) {
        panic!("{MISSING}");
    }
    if nodes[1].get("name").and_then(serde_json::Value::as_str) != Some("feat-name-extrude") || nodes[1].get("id").and_then(serde_json::Value::as_u64) != Some(made.body) {
        panic!("{MISSING}");
    }
}

fn one_body<'a>(structured: &'a serde_json::Value, made: &Made) -> &'a serde_json::Value {
    let bodies = structured.get("bodies").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if bodies.len() != 1 {
        panic!("{MISSING}");
    }
    let body = bodies.iter().find(|body| body.get("id").and_then(serde_json::Value::as_u64) == Some(made.body)).unwrap_or_else(|| panic!("{MISSING}"));
    if body.get("sound").and_then(serde_json::Value::as_str) != Some("valid") || body.get("solids").and_then(serde_json::Value::as_u64) != Some(1) {
        panic!("{MISSING}");
    }
    body
}

fn short_edges(body: &serde_json::Value) {
    let edges = body.get("edges").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if edges.len() != 12 || edges.iter().any(|edge| length_near(edge, 30.0)) {
        panic!("{MISSING}");
    }
    let tens = edges.iter().filter(|edge| length_near(edge, 10.0)).count();
    let twenties = edges.iter().filter(|edge| length_near(edge, 20.0)).count();
    if tens != 4 || twenties != 8 {
        panic!("{MISSING}");
    }
}

fn short_faces(body: &serde_json::Value) {
    let faces = body.get("faces").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if faces.len() != 6 {
        panic!("{MISSING}");
    }
    if !faces.iter().any(|face| role_is(face, "cap-start") && area_near(face, 200.0)) || !faces.iter().any(|face| role_is(face, "cap-end") && area_near(face, 200.0)) {
        panic!("{MISSING}");
    }
    let walls: Vec<&serde_json::Value> = faces.iter().filter(|face| role_is(face, "wall")).collect();
    let narrow = walls.iter().filter(|face| area_near(face, 200.0)).count();
    let wide = walls.iter().filter(|face| area_near(face, 400.0)).count();
    if walls.len() != 4 || narrow != 2 || wide != 2 || walls.iter().any(|face| area_near(face, 300.0) || area_near(face, 600.0)) {
        panic!("{MISSING}");
    }
}

fn role_is(face: &serde_json::Value, role: &str) -> bool {
    face.get("role").and_then(serde_json::Value::as_str) == Some(role)
}

fn area_near(face: &serde_json::Value, mm2: f64) -> bool {
    let area = face.get("area").and_then(serde_json::Value::as_f64).unwrap_or(f64::NAN);
    (area - mm2).abs() < 1.0
}

fn length_near(edge: &serde_json::Value, mm: f64) -> bool {
    let length = edge.get("length").and_then(serde_json::Value::as_f64).unwrap_or(f64::NAN);
    (length - mm).abs() < 1.0
}

fn empty_reply(msg: &serde_json::Value) {
    let reply = structured(msg);
    if reply.as_object().is_none_or(|fields| !fields.is_empty()) {
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
