use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"qymcad-drive","version":"0.1.1"}}}"#;
const NOTIFIED: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
const LIST: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;

/// A full turn of the 10 by 20 mm rectangle about sketch X is a cylinder of 4000 pi mm^3.
#[test]
fn the_rectangle_is_turned() {
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

    let drawn_msg = call(&mut input, &lines, 3, &rectangle_call());
    same_text(&drawn_msg);
    let drawn = drawn_of(&structured(&drawn_msg));
    let built = call(&mut input, &lines, 4, &revolve_call(&drawn));
    same_text(&built);
    let body = qymcad_drive::BodyId { id: number_at(&structured(&built), "body") };
    let read = call(&mut input, &lines, 5, &read_call(5));
    same_text(&read);
    let fields = structured(&read);
    turned_cylinder(&fields, &drawn, &body);
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

fn drawn_of(reply: &serde_json::Value) -> Drawn {
    Drawn { sketch: qymcad_drive::SketchId { id: number_at(reply, "sketch") }, contour: qymcad_drive::ContourId { id: number_at(reply, "contour") } }
}

fn revolve_call(drawn: &Drawn) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "part.revolve",
            "arguments": { "sketch": drawn.sketch.id, "contour": drawn.contour.id, "axis": "x", "angle": 360 }
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

fn turned_cylinder(structured: &serde_json::Value, drawn: &Drawn, made: &qymcad_drive::BodyId) {
    if structured.get("rebuild").and_then(serde_json::Value::as_str) != Some("quiet") {
        panic!("the turn is not the cylinder");
    }
    let refusals = structured.get("refusals").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the turn is not the cylinder"));
    if !refusals.is_empty() {
        panic!("the turn is not the cylinder");
    }
    one_rectangle(structured, drawn);
    two_nodes(structured, drawn);
    let bodies = structured.get("bodies").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the turn is not the cylinder"));
    if bodies.len() != 1 {
        panic!("the turn is not the cylinder");
    }
    let body = body_named(structured, made.id, "the turn is not the cylinder");
    one_solid(body, "the turn is not the cylinder");
    let volume = body.get("volume").and_then(serde_json::Value::as_f64).unwrap_or_else(|| panic!("the turn is not the cylinder"));
    if (volume - 4000.0 * std::f64::consts::PI).abs() >= 1.0 {
        panic!("the turn is not the cylinder");
    }
    cylinder_edges(body);
    cylinder_faces(body);
}

fn one_rectangle(structured: &serde_json::Value, drawn: &Drawn) {
    let sketches = structured.get("sketches").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the turn is not the cylinder"));
    if sketches.len() != 1 {
        panic!("the turn is not the cylinder");
    }
    let sketch = &sketches[0];
    if sketch.get("id").and_then(serde_json::Value::as_u64) != Some(drawn.sketch.id)
        || sketch.get("name").and_then(serde_json::Value::as_str) != Some("name-sketch-n#1")
        || sketch.get("plane").and_then(serde_json::Value::as_str) != Some("xz")
    {
        panic!("the turn is not the cylinder");
    }
    let contours = sketch.get("contours").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the turn is not the cylinder"));
    if contours.len() != 1 || contours[0].get("id").and_then(serde_json::Value::as_u64) != Some(drawn.contour.id) || contours[0].get("form").and_then(serde_json::Value::as_str) != Some("closed") {
        panic!("the turn is not the cylinder");
    }
}

fn two_nodes(structured: &serde_json::Value, drawn: &Drawn) {
    let nodes = structured.get("nodes").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the turn is not the cylinder"));
    if nodes.len() != 2 {
        panic!("the turn is not the cylinder");
    }
    let sketch = nodes.iter().filter(|node| node.get("name").and_then(serde_json::Value::as_str) == Some("name-sketch-n#1")).collect::<Vec<_>>();
    let turned = nodes.iter().filter(|node| node.get("name").and_then(serde_json::Value::as_str) == Some("feat-name-revolve")).collect::<Vec<_>>();
    if sketch.len() != 1 || sketch[0].get("id").and_then(serde_json::Value::as_u64) != Some(drawn.sketch.id) || turned.len() != 1 {
        panic!("the turn is not the cylinder");
    }
}

fn cylinder_edges(body: &serde_json::Value) {
    let edges = body.get("edges").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the turn is not the cylinder"));
    // Two rims are chords near zero. The third edge is the 10 mm side, not a circumference.
    if edges.len() != 3 || count_length(edges, 0.0) != 2 || count_length(edges, 10.0) != 1 {
        panic!("the turn is not the cylinder");
    }
    for edge in edges {
        if !length_near(edge, 0.0) && !length_near(edge, 10.0) {
            panic!("the turn is not the cylinder");
        }
    }
}

fn count_length(edges: &[serde_json::Value], mm: f64) -> usize {
    edges.iter().filter(|edge| length_near(edge, mm)).count()
}

fn cylinder_faces(body: &serde_json::Value) {
    let faces = body.get("faces").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the turn is not the cylinder"));
    if faces.len() != 3 || faces.iter().any(|face| role_is(face, "wall") || role_is(face, "cap-start") || role_is(face, "cap-end") || face.get("source").is_some()) {
        panic!("the turn is not the cylinder");
    }
    // Both ends are about 3.59 mm^2 under 400 pi, so the disk formula is not the check.
    if count_area(faces, 1253.0514399185558) != 2 || count_area(faces, 1255.740078516056) != 1 {
        panic!("the turn is not the cylinder");
    }
    for face in faces {
        if !role_is(face, "other") || (!area_near(face, 1253.0514399185558) && !area_near(face, 1255.740078516056)) {
            panic!("the turn is not the cylinder");
        }
    }
}

fn count_area(faces: &[serde_json::Value], mm2: f64) -> usize {
    faces.iter().filter(|face| area_near(face, mm2)).count()
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
