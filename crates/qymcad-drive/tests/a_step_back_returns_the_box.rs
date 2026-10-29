use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"qymcad-drive","version":"0.1.1"}}}"#;
const NOTIFIED: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
const LIST: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;

/// The 10 by 20 mm rectangle is shrunk and then put back, and the same extrusion is 10 by 20 by 30 mm again.
#[test]
fn the_step_back_returns_the_box() {
    let path = process_path();
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

    let drawn = call(&mut input, &lines, 3, &rectangle_call());
    same_text(&drawn);
    let drawn_ids = Drawn { sketch: qymcad_drive::SketchId { id: number_at(&structured(&drawn), "sketch") }, contour: qymcad_drive::ContourId { id: number_at(&structured(&drawn), "contour") } };
    let made = call(&mut input, &lines, 4, &extrude_call(&drawn_ids));
    same_text(&made);
    let extrusion = qymcad_drive::BodyId { id: number_at(&structured(&made), "body") };
    let moved = call(&mut input, &lines, 5, &resize_call(&drawn_ids));
    same_text(&moved);
    empty_reply(&moved);
    let returned = call(&mut input, &lines, 6, &back_call());
    same_text(&returned);
    empty_reply(&returned);
    let read = call(&mut input, &lines, 7, &read_call(7));
    same_text(&read);
    let fields = structured(&read);
    same_sketch(&fields, &drawn_ids);
    restored_box(&fields, &extrusion);
    drop(input);
    finished(&mut running);
    let _ended = lines.reader.join();
    assert!(lines.rx.try_recv().is_err(), "the handshake");
}

struct Drawn {
    sketch: qymcad_drive::SketchId,
    contour: qymcad_drive::ContourId,
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

fn read_call(id: i64) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "tools/call",
        "params": { "name": "document.read" }
    })
    .to_string()
}

fn resize_call(drawn: &Drawn) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "tools/call",
        "params": {
            "name": "sketch.resize",
            "arguments": {
                "sketch": drawn.sketch.id,
                "contour": drawn.contour.id,
                "from": { "x": 0, "y": 0 },
                "to": { "x": 10, "y": 10 }
            }
        }
    })
    .to_string()
}

fn back_call() -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "tools/call",
        "params": { "name": "document.back" }
    })
    .to_string()
}

fn same_sketch(structured: &serde_json::Value, drawn: &Drawn) {
    let sketches = structured.get("sketches").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the step did not return"));
    if sketches.len() != 1 {
        panic!("the step did not return");
    }
    let sketch = &sketches[0];
    if sketch.get("id").and_then(serde_json::Value::as_u64) != Some(drawn.sketch.id) || sketch.get("plane").and_then(serde_json::Value::as_str) != Some("xz") {
        panic!("the step did not return");
    }
    let contours = sketch.get("contours").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the step did not return"));
    if contours.len() != 1 {
        panic!("the step did not return");
    }
    let contour = &contours[0];
    if contour.get("id").and_then(serde_json::Value::as_u64) != Some(drawn.contour.id) || contour.get("form").and_then(serde_json::Value::as_str) != Some("closed") {
        panic!("the step did not return");
    }
}

fn restored_box(structured: &serde_json::Value, extrusion: &qymcad_drive::BodyId) {
    if structured.get("rebuild").and_then(serde_json::Value::as_str) != Some("quiet") {
        panic!("the step did not return");
    }
    let refusals = structured.get("refusals").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the step did not return"));
    if !refusals.is_empty() {
        panic!("the step did not return");
    }
    let bodies = structured.get("bodies").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the step did not return"));
    if bodies.len() != 1 {
        panic!("the step did not return");
    }
    let body = body_named(structured, extrusion.id, "the step did not return");
    one_solid(body, "the step did not return");
    let volume = body.get("volume").and_then(serde_json::Value::as_f64).unwrap_or_else(|| panic!("the step did not return"));
    if (volume - 10.0 * 10.0 * 30.0).abs() < 1.0 || (volume - 10.0 * 20.0 * 30.0).abs() >= 1.0 {
        panic!("the step did not return");
    }
    restored_edges(body);
    restored_faces(body);
}

fn empty_reply(msg: &serde_json::Value) {
    let reply = structured(msg);
    let fields = reply.as_object().unwrap_or_else(|| panic!("the step did not return"));
    if !fields.is_empty() {
        panic!("the step did not return");
    }
}

fn restored_edges(body: &serde_json::Value) {
    let edges = body.get("edges").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the step did not return"));
    // 10 x 30 x 20 mm has the same three lengths and the same volume, so each length has to be present and no other length may appear.
    if !edges.iter().any(|edge| length_near(edge, 10.0)) || !edges.iter().any(|edge| length_near(edge, 20.0)) || !edges.iter().any(|edge| length_near(edge, 30.0)) {
        panic!("the step did not return");
    }
    for edge in edges {
        if !length_near(edge, 10.0) && !length_near(edge, 20.0) && !length_near(edge, 30.0) {
            panic!("the step did not return");
        }
    }
}

fn restored_faces(body: &serde_json::Value) {
    let faces = body.get("faces").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the step did not return"));
    if !faces.iter().any(|face| role_is(face, "cap-start") && area_near(face, 200.0)) || !faces.iter().any(|face| role_is(face, "cap-end") && area_near(face, 200.0)) {
        panic!("the step did not return");
    }
    let walls: Vec<&serde_json::Value> = faces.iter().filter(|face| role_is(face, "wall")).collect();
    if !walls.iter().any(|face| area_near(face, 300.0)) || !walls.iter().any(|face| area_near(face, 600.0)) {
        panic!("the step did not return");
    }
    for wall in walls {
        if !area_near(wall, 300.0) && !area_near(wall, 600.0) {
            panic!("the step did not return");
        }
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
    (length - mm).abs() < 0.1
}

fn one_solid(body: &serde_json::Value, missing: &str) {
    if body.get("sound").and_then(serde_json::Value::as_str) != Some("valid") || body.get("solids").and_then(serde_json::Value::as_u64) != Some(1) {
        panic!("{missing}");
    }
}

fn body_named<'a>(structured: &'a serde_json::Value, id: u64, missing: &str) -> &'a serde_json::Value {
    let bodies = structured.get("bodies").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{missing}"));
    bodies.iter().find(|body| body.get("id").and_then(serde_json::Value::as_u64) == Some(id)).unwrap_or_else(|| panic!("{missing}"))
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
