use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"qymcad-drive","version":"0.1.1"}}}"#;
const NOTIFIED: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
const LIST: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;
const MISSING: &str = "the frustum is not lofted";

/// A 10 mm square on the table, lofted to a 20 mm square 30 mm above it, is a frustum of 7000 mm^3.
#[test]
fn the_frustum_is_lofted() {
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

    let lower_msg = call(&mut input, &lines, 3, &rectangle_call());
    same_text(&lower_msg);
    let lower = qymcad_drive::Lower { profile: profile_of(&structured(&lower_msg)) };
    let raised_msg = call(&mut input, &lines, 4, &plane_call());
    same_text(&raised_msg);
    let plane = qymcad_drive::PlaneId { id: number_at(&structured(&raised_msg), "plane") };
    let upper_msg = call(&mut input, &lines, 5, &datum_call(&plane));
    same_text(&upper_msg);
    let upper = qymcad_drive::Upper { profile: profile_of(&structured(&upper_msg)) };
    let built = call(&mut input, &lines, 6, &loft_call(&lower, &upper));
    same_text(&built);
    let reply = structured(&built);
    if reply.as_object().is_none_or(|fields| fields.len() != 1 || !fields.contains_key("body")) {
        panic!("{MISSING}");
    }
    let body = qymcad_drive::BodyId { id: number_at(&reply, "body") };
    let read = call(&mut input, &lines, 7, &read_call(7));
    same_text(&read);
    lofted(&structured(&read), &lower, &upper, &plane, &body);
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
            "arguments": { "plane": "xy", "from": { "x": 0, "y": 0 }, "to": { "x": 10, "y": 10 } }
        }
    })
    .to_string()
}

fn plane_call() -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "sketch.plane",
            "arguments": { "plane": "xy", "offset": 30 }
        }
    })
    .to_string()
}

fn profile_of(reply: &serde_json::Value) -> qymcad_drive::Profile {
    qymcad_drive::Profile { sketch: qymcad_drive::SketchId { id: number_at(reply, "sketch") }, contour: qymcad_drive::ContourId { id: number_at(reply, "contour") } }
}

fn datum_call(plane: &qymcad_drive::PlaneId) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 5,
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

fn loft_call(lower: &qymcad_drive::Lower, upper: &qymcad_drive::Upper) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "tools/call",
        "params": {
            "name": "part.loft",
            "arguments": {
                "lower": { "sketch": lower.profile.sketch.id, "contour": lower.profile.contour.id },
                "upper": { "sketch": upper.profile.sketch.id, "contour": upper.profile.contour.id }
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

fn lofted(structured: &serde_json::Value, lower: &qymcad_drive::Lower, upper: &qymcad_drive::Upper, plane: &qymcad_drive::PlaneId, made: &qymcad_drive::BodyId) {
    if structured.get("rebuild").and_then(serde_json::Value::as_str) != Some("quiet") {
        panic!("{MISSING}");
    }
    let refusals = structured.get("refusals").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if !refusals.is_empty() {
        panic!("{MISSING}");
    }
    let part = the_part(structured);
    two_squares(structured, lower, upper, plane);
    four_nodes(structured, lower, upper, plane, made, part);
    let bodies = structured.get("bodies").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if bodies.len() != 1 {
        panic!("{MISSING}");
    }
    let body = body_named(structured, made.id);
    one_solid(body);
    let volume = body.get("volume").and_then(serde_json::Value::as_f64).unwrap_or_else(|| panic!("{MISSING}"));
    if (volume - 7000.0).abs() >= 1.0 {
        panic!("{MISSING}");
    }
    frustum_edges(body);
    frustum_faces(body);
}

fn two_squares(structured: &serde_json::Value, lower: &qymcad_drive::Lower, upper: &qymcad_drive::Upper, plane: &qymcad_drive::PlaneId) {
    let sketches = structured.get("sketches").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if sketches.len() != 2 {
        panic!("{MISSING}");
    }
    let table = sketches.iter().filter(|sketch| sketch.get("name").and_then(serde_json::Value::as_str) == Some("name-sketch-n#1")).collect::<Vec<_>>();
    let raised = sketches.iter().filter(|sketch| sketch.get("name").and_then(serde_json::Value::as_str) == Some("name-sketch-n#2")).collect::<Vec<_>>();
    if table.len() != 1 || raised.len() != 1 {
        panic!("{MISSING}");
    }
    on_table(table[0], &lower.profile);
    on_datum(raised[0], &upper.profile, plane);
}

fn on_table(sketch: &serde_json::Value, drawn: &qymcad_drive::Profile) {
    if sketch.get("id").and_then(serde_json::Value::as_u64) != Some(drawn.sketch.id) || sketch.get("plane").and_then(serde_json::Value::as_str) != Some("xy") {
        panic!("{MISSING}");
    }
    closed(sketch, drawn.contour.id);
}

fn on_datum(sketch: &serde_json::Value, drawn: &qymcad_drive::Profile, plane: &qymcad_drive::PlaneId) {
    let datum = sketch.get("plane").and_then(|place| place.get("datum")).and_then(serde_json::Value::as_u64);
    if sketch.get("id").and_then(serde_json::Value::as_u64) != Some(drawn.sketch.id) || sketch.get("plane").and_then(serde_json::Value::as_str).is_some() || datum != Some(plane.id) {
        panic!("{MISSING}");
    }
    closed(sketch, drawn.contour.id);
}

fn closed(sketch: &serde_json::Value, id: u64) {
    let contours = sketch.get("contours").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if contours.len() != 1 || contours[0].get("id").and_then(serde_json::Value::as_u64) != Some(id) || contours[0].get("form").and_then(serde_json::Value::as_str) != Some("closed") {
        panic!("{MISSING}");
    }
}

fn four_nodes(structured: &serde_json::Value, lower: &qymcad_drive::Lower, upper: &qymcad_drive::Upper, plane: &qymcad_drive::PlaneId, made: &qymcad_drive::BodyId, part: u64) {
    let nodes = structured.get("nodes").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if nodes.len() != 4 || nodes.iter().any(|node| node.get("name").and_then(serde_json::Value::as_str) == Some("feat-name-body-union")) {
        panic!("{MISSING}");
    }
    let table = node_parent(nodes, "name-sketch-n#1", lower.profile.sketch.id);
    let raised = node_parent(nodes, "name-plane", plane.id);
    let square = node_parent(nodes, "name-sketch-n#2", upper.profile.sketch.id);
    let lofted = node_parent(nodes, "feat-name-loft", made.id);
    if table != part || raised != part || square != part || lofted != part {
        panic!("{MISSING}");
    }
}

fn node_parent(nodes: &[serde_json::Value], name: &str, id: u64) -> u64 {
    let found = nodes.iter().filter(|node| node.get("name").and_then(serde_json::Value::as_str) == Some(name)).collect::<Vec<_>>();
    if found.len() != 1 || found[0].get("id").and_then(serde_json::Value::as_u64) != Some(id) {
        panic!("{MISSING}");
    }
    found[0].get("parent").and_then(serde_json::Value::as_u64).unwrap_or_else(|| panic!("{MISSING}"))
}

fn the_part(structured: &serde_json::Value) -> u64 {
    let components = structured.get("components").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if components.len() != 2 {
        panic!("{MISSING}");
    }
    let roots: Vec<&serde_json::Value> = components.iter().filter(|component| component.get("kind").and_then(serde_json::Value::as_str) == Some("assembly")).collect();
    if roots.len() != 1 || roots[0].get("name").and_then(serde_json::Value::as_str) != Some("name-assembly") || !roots[0].get("parent").is_some_and(serde_json::Value::is_null) {
        panic!("{MISSING}");
    }
    let root = roots[0].get("id").and_then(serde_json::Value::as_u64).unwrap_or_else(|| panic!("{MISSING}"));
    let parts: Vec<&serde_json::Value> = components.iter().filter(|component| component.get("kind").and_then(serde_json::Value::as_str) == Some("part")).collect();
    if parts.len() != 1 || parts[0].get("name").and_then(serde_json::Value::as_str) != Some("name-part-n#1") || parts[0].get("parent").and_then(serde_json::Value::as_u64) != Some(root) {
        panic!("{MISSING}");
    }
    parts[0].get("id").and_then(serde_json::Value::as_u64).unwrap_or_else(|| panic!("{MISSING}"))
}

fn frustum_edges(body: &serde_json::Value) {
    let edges = body.get("edges").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    // Four edges of the lower square, four of the upper square, and four corner-to-corner slants.
    if edges.len() != 12 || count_length(edges, 10.0) != 4 || count_length(edges, 20.0) != 4 || count_length(edges, 30.822070014844883) != 4 {
        panic!("{MISSING}");
    }
    for edge in edges {
        if !length_near(edge, 10.0) && !length_near(edge, 20.0) && !length_near(edge, 30.822070014844883) {
            panic!("{MISSING}");
        }
    }
}

fn count_length(edges: &[serde_json::Value], mm: f64) -> usize {
    edges.iter().filter(|edge| length_near(edge, mm)).count()
}

fn frustum_faces(body: &serde_json::Value) {
    let faces = body.get("faces").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    // The ends are the two squares. The four sides are trapezoids and carry no wall source.
    if faces.len() != 6 || count_role(faces, "cap-start") != 1 || count_role(faces, "cap-end") != 1 || count_role(faces, "other") != 4 || count_role(faces, "wall") != 0 {
        panic!("{MISSING}");
    }
    for face in faces {
        let area = match face.get("role").and_then(serde_json::Value::as_str) {
            Some("cap-start") => 100.0,
            Some("cap-end") => 400.0,
            Some("other") => 456.2071897723665,
            _ => panic!("{MISSING}"),
        };
        if !area_near(face, area) || face.get("source").is_some() {
            panic!("{MISSING}");
        }
    }
}

fn count_role(faces: &[serde_json::Value], role: &str) -> usize {
    faces.iter().filter(|face| face.get("role").and_then(serde_json::Value::as_str) == Some(role)).count()
}

fn area_near(face: &serde_json::Value, mm2: f64) -> bool {
    let area = face.get("area").and_then(serde_json::Value::as_f64).unwrap_or(f64::NAN);
    (area - mm2).abs() < 1.0
}

fn length_near(edge: &serde_json::Value, mm: f64) -> bool {
    let length = edge.get("length").and_then(serde_json::Value::as_f64).unwrap_or(f64::NAN);
    (length - mm).abs() < 0.1
}

fn one_solid(body: &serde_json::Value) {
    if body.get("sound").and_then(serde_json::Value::as_str) != Some("valid") || body.get("solids").and_then(serde_json::Value::as_u64) != Some(1) {
        panic!("{MISSING}");
    }
}

fn body_named(structured: &serde_json::Value, id: u64) -> &serde_json::Value {
    let bodies = structured.get("bodies").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    bodies.iter().find(|body| body.get("id").and_then(serde_json::Value::as_u64) == Some(id)).unwrap_or_else(|| panic!("{MISSING}"))
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
