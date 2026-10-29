use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"qymcad-drive","version":"0.1.1"}}}"#;
const NOTIFIED: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
const LIST: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;

/// A rectangle from (-5, -5) to (15, 15) sits on the plane 30 mm off XY, and the read has no body.
#[test]
fn the_rectangle_sits_on_the_plane() {
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

    let raised_msg = call(&mut input, &lines, 3, &plane_call());
    same_text(&raised_msg);
    let plane = qymcad_drive::PlaneId { id: number_at(&structured(&raised_msg), "plane") };
    let drawn_msg = call(&mut input, &lines, 4, &datum_call(&plane));
    same_text(&drawn_msg);
    let drawn = drawn_of(&structured(&drawn_msg));
    let read = call(&mut input, &lines, 5, &read_call(5));
    same_text(&read);
    on_the_plane(&structured(&read), &plane, &drawn);
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

fn plane_call() -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "sketch.plane",
            "arguments": { "plane": "xy", "offset": 30 }
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

fn datum_call(plane: &qymcad_drive::PlaneId) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "sketch.datum",
            "arguments": {
                "plane": plane.id,
                "from": { "x": -5, "y": -5 },
                "to": { "x": 15, "y": 15 }
            }
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

fn on_the_plane(structured: &serde_json::Value, plane: &qymcad_drive::PlaneId, drawn: &Drawn) {
    if structured.get("rebuild").and_then(serde_json::Value::as_str) != Some("quiet") {
        panic!("the rectangle is not on the plane");
    }
    let refusals = structured.get("refusals").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the rectangle is not on the plane"));
    if !refusals.is_empty() {
        panic!("the rectangle is not on the plane");
    }
    let bodies = structured.get("bodies").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the rectangle is not on the plane"));
    if !bodies.is_empty() {
        panic!("the rectangle is not on the plane");
    }
    let part = the_part(structured);
    one_sketch(structured, plane, drawn);
    two_nodes(structured, plane, drawn, part);
}

fn one_sketch(structured: &serde_json::Value, plane: &qymcad_drive::PlaneId, drawn: &Drawn) {
    let sketches = structured.get("sketches").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the rectangle is not on the plane"));
    if sketches.len() != 1 {
        panic!("the rectangle is not on the plane");
    }
    let sketch = &sketches[0];
    let datum = sketch.get("plane").and_then(|place| place.get("datum")).and_then(serde_json::Value::as_u64);
    if sketch.get("id").and_then(serde_json::Value::as_u64) != Some(drawn.sketch.id)
        || sketch.get("name").and_then(serde_json::Value::as_str) != Some("name-sketch-n#1")
        || sketch.get("plane").and_then(serde_json::Value::as_str).is_some()
        || datum != Some(plane.id)
    {
        panic!("the rectangle is not on the plane");
    }
    let contours = sketch.get("contours").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the rectangle is not on the plane"));
    if contours.len() != 1 || contours[0].get("id").and_then(serde_json::Value::as_u64) != Some(drawn.contour.id) || contours[0].get("form").and_then(serde_json::Value::as_str) != Some("closed") {
        panic!("the rectangle is not on the plane");
    }
}

fn two_nodes(structured: &serde_json::Value, plane: &qymcad_drive::PlaneId, drawn: &Drawn, part: u64) {
    let nodes = structured.get("nodes").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the rectangle is not on the plane"));
    if nodes.len() != 2 {
        panic!("the rectangle is not on the plane");
    }
    let raised = nodes.iter().filter(|node| node.get("name").and_then(serde_json::Value::as_str) == Some("name-plane")).collect::<Vec<_>>();
    let sketch = nodes.iter().filter(|node| node.get("name").and_then(serde_json::Value::as_str) == Some("name-sketch-n#1")).collect::<Vec<_>>();
    if raised.len() != 1 || raised[0].get("id").and_then(serde_json::Value::as_u64) != Some(plane.id) || raised[0].get("parent").and_then(serde_json::Value::as_u64) != Some(part) {
        panic!("the rectangle is not on the plane");
    }
    if sketch.len() != 1 || sketch[0].get("id").and_then(serde_json::Value::as_u64) != Some(drawn.sketch.id) || sketch[0].get("parent").and_then(serde_json::Value::as_u64) != Some(part) {
        panic!("the rectangle is not on the plane");
    }
}

fn the_part(structured: &serde_json::Value) -> u64 {
    let components = structured.get("components").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the rectangle is not on the plane"));
    if components.len() != 2 {
        panic!("the rectangle is not on the plane");
    }
    let roots: Vec<&serde_json::Value> = components.iter().filter(|component| component.get("kind").and_then(serde_json::Value::as_str) == Some("assembly")).collect();
    if roots.len() != 1 || roots[0].get("name").and_then(serde_json::Value::as_str) != Some("name-assembly") || !roots[0].get("parent").is_some_and(serde_json::Value::is_null) {
        panic!("the rectangle is not on the plane");
    }
    let root = roots[0].get("id").and_then(serde_json::Value::as_u64).unwrap_or_else(|| panic!("the rectangle is not on the plane"));
    let parts: Vec<&serde_json::Value> = components.iter().filter(|component| component.get("kind").and_then(serde_json::Value::as_str) == Some("part")).collect();
    if parts.len() != 1 || parts[0].get("name").and_then(serde_json::Value::as_str) != Some("name-part-n#1") || parts[0].get("parent").and_then(serde_json::Value::as_u64) != Some(root) {
        panic!("the rectangle is not on the plane");
    }
    parts[0].get("id").and_then(serde_json::Value::as_u64).unwrap_or_else(|| panic!("the rectangle is not on the plane"))
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
