use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"qymcad-drive","version":"0.1.1"}}}"#;
const NOTIFIED: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
const LIST: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;

/// A height of 0 comes back as the session's text, and a radius the edge cannot take comes back on the next read.
#[test]
fn the_process_stays_alive() {
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

    // The height reply is read before the tool list is judged, so a process that dies on the height fails here.
    write_line(&mut input, &extrude_zero(&drawn_ids));
    height_was_refused(&refusal_of(&lines));
    five_tools(&listed);

    let made = call(&mut input, &lines, 5, &extrude_thirty(&drawn_ids));
    same_text(&made);
    let extrusion = qymcad_drive::BodyId { id: number_at(&structured(&made), "body") };
    let quiet = call(&mut input, &lines, 6, &read_call(6));
    same_text(&quiet);
    let edge = quiet_box(&structured(&quiet), &extrusion);
    let round = Round { body: extrusion, edge };
    let rounded = call(&mut input, &lines, 7, &fillet_call(&round));
    same_text(&rounded);
    let fillet = qymcad_drive::BodyId { id: number_at(&structured(&rounded), "body") };
    let refused = call(&mut input, &lines, 8, &read_call(8));
    same_text(&refused);
    let fields = structured(&refused);
    refused_box(&fields, &fillet);
    let bodies = fields.get("bodies").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the box is not in the document"));
    if !bodies.iter().any(|body| body.get("id").and_then(serde_json::Value::as_u64) == Some(extrusion.id)) {
        panic!("the box is not in the document");
    }
    drop(input);
    finished(&mut running);
    let _ended = lines.reader.join();
    assert!(lines.rx.try_recv().is_err(), "the handshake");
}

struct Drawn {
    sketch: qymcad_drive::SketchId,
    contour: qymcad_drive::ContourId,
}

struct Round {
    body: qymcad_drive::BodyId,
    edge: qymcad_drive::EdgeId,
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

fn extrude_zero(drawn: &Drawn) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "part.extrude",
            "arguments": { "sketch": drawn.sketch.id, "contour": drawn.contour.id, "height": 0 }
        }
    })
    .to_string()
}

fn extrude_thirty(drawn: &Drawn) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 5,
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

fn fillet_call(round: &Round) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 7,
        "method": "tools/call",
        "params": {
            "name": "part.fillet",
            "arguments": { "body": round.body.id, "edge": round.edge.id, "radius": 100 }
        }
    })
    .to_string()
}

fn five_tools(msg: &serde_json::Value) {
    if msg.get("result").and_then(|result| result.get("nextCursor")).is_some() {
        panic!("the tools");
    }
    let tools = msg.get("result").and_then(|result| result.get("tools")).and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the tools"));
    let mut names = Vec::new();
    for tool in tools {
        let name = tool.get("name").and_then(serde_json::Value::as_str).unwrap_or_else(|| panic!("the tools"));
        if tool.get("description").and_then(serde_json::Value::as_str).is_none() || tool.get("inputSchema").and_then(serde_json::Value::as_object).is_none() {
            panic!("the tools");
        }
        names.push(name);
    }
    names.sort_unstable();
    if names
        != [
            "document.back", "document.mate", "document.open", "document.open_step", "document.open_stl", "document.parameter", "document.part", "document.read", "document.save",
            "document.save_step", "document.save_stl", "document.show", "document.slide", "part.auger", "part.bind", "part.boolean", "part.chamfer", "part.circular", "part.cone", "part.copy_face",
            "part.cuboid", "part.cylinder", "part.draft", "part.extrude", "part.fillet", "part.hole", "part.linear", "part.loft", "part.mirror", "part.move", "part.offset_face", "part.patch",
            "part.prism", "part.push", "part.remove_face", "part.replace_face", "part.revolve", "part.shell", "part.sphere", "part.split", "part.split_faces", "part.stitch", "part.sweep",
            "part.thicken", "part.thread", "part.torus", "part.trim", "sketch.datum", "sketch.line", "sketch.plane", "sketch.rectangle", "sketch.resize",
        ]
    {
        panic!("the tools");
    }
}

fn height_was_refused(msg: &serde_json::Value) {
    if msg.get("error").is_some() || id_of_refusal(msg) != 4 || msg.get("jsonrpc").and_then(serde_json::Value::as_str) != Some("2.0") {
        panic!("the refusal is not in the reply");
    }
    let result = msg.get("result").unwrap_or_else(|| panic!("the refusal is not in the reply"));
    if result.get("isError").and_then(serde_json::Value::as_bool) != Some(true) || result.get("structuredContent").is_some() {
        panic!("the refusal is not in the reply");
    }
    let text = result
        .get("content")
        .and_then(serde_json::Value::as_array)
        .and_then(|content| content.first())
        .and_then(|item| item.get("text"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("the refusal is not in the reply"));
    if text.contains('\n') || text != "sketch-extrude-height" {
        panic!("the refusal is not in the reply");
    }
}

fn quiet_box(structured: &serde_json::Value, extrusion: &qymcad_drive::BodyId) -> qymcad_drive::EdgeId {
    if structured.get("rebuild").and_then(serde_json::Value::as_str) != Some("quiet") {
        panic!("the box is not in the document");
    }
    let refusals = structured.get("refusals").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the box is not in the document"));
    if !refusals.is_empty() {
        panic!("the box is not in the document");
    }
    let bodies = structured.get("bodies").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the box is not in the document"));
    let Some(body) = bodies.first() else {
        panic!("the box is not in the document");
    };
    if bodies.len() != 1 || body.get("id").and_then(serde_json::Value::as_u64) != Some(extrusion.id) {
        panic!("the box is not in the document");
    }
    live_box(body);
    long_edge(body)
}

fn refused_box(structured: &serde_json::Value, fillet: &qymcad_drive::BodyId) {
    if structured.get("rebuild").and_then(serde_json::Value::as_str) != Some("refused") {
        panic!("the refusal is not in the reply");
    }
    let refusals = structured.get("refusals").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the refusal is not in the reply"));
    let Some(refusal) = refusals.first() else {
        panic!("the refusal is not in the reply");
    };
    if refusals.len() != 1 || refusal.get("node").and_then(serde_json::Value::as_u64) != Some(fillet.id) {
        panic!("the refusal is not in the reply");
    }
    if refusal.get("text").and_then(serde_json::Value::as_str) != Some("fillet R100.00 failed on 1 edges") {
        panic!("the refusal is not in the reply");
    }
    let bodies = structured.get("bodies").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the box is not in the document"));
    if bodies.len() != 2 || !bodies.iter().any(|body| body.get("id").and_then(serde_json::Value::as_u64) == Some(fillet.id)) {
        panic!("the box is not in the document");
    }
    for body in bodies {
        live_box(body);
    }
}

fn live_box(body: &serde_json::Value) {
    if body.get("sound").and_then(serde_json::Value::as_str) != Some("valid") || body.get("solids").and_then(serde_json::Value::as_u64) != Some(1) {
        panic!("the box is not in the document");
    }
    let volume = body.get("volume").and_then(serde_json::Value::as_f64).unwrap_or_else(|| panic!("the box is not in the document"));
    if (volume - 6000.0).abs() >= 1.0 {
        panic!("the box is not in the document");
    }
}

fn long_edge(body: &serde_json::Value) -> qymcad_drive::EdgeId {
    let edges = body.get("edges").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("the 20 mm edge"));
    for edge in edges {
        let length = edge.get("length").and_then(serde_json::Value::as_f64).unwrap_or(f64::NAN);
        if (length - 20.0).abs() < 0.1 {
            let id = edge.get("id").and_then(serde_json::Value::as_u64).unwrap_or_else(|| panic!("the 20 mm edge"));
            let id = u32::try_from(id).unwrap_or_else(|_| panic!("the 20 mm edge"));
            return qymcad_drive::EdgeId { id };
        }
    }
    panic!("the 20 mm edge");
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

fn refusal_of(lines: &Lines) -> serde_json::Value {
    let line = match lines.rx.recv_timeout(Duration::from_secs(5)) {
        Ok(line) => line,
        Err(_) => panic!("the refusal is not in the reply"),
    };
    serde_json::from_str(&line).unwrap_or_else(|_| panic!("the refusal is not in the reply"))
}

fn id_of_refusal(msg: &serde_json::Value) -> i64 {
    msg.get("id").and_then(serde_json::Value::as_i64).unwrap_or_else(|| panic!("the refusal is not in the reply"))
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
