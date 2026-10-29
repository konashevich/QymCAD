use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"qymcad-drive","version":"0.1.1"}}}"#;
const NOTIFIED: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
const LIST: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;
const MISSING: &str = "the plate is not grown";

/// A 2 mm plate grows on the end cap of a 10 by 20 by 30 mm extrusion.
#[test]
fn the_plate_is_grown() {
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
    let drawn = profile_of(&structured(&drawn_msg));
    let grown_msg = call(&mut input, &lines, 4, &extrude_call(&drawn));
    same_text(&grown_msg);
    let grown = Grown { body: qymcad_drive::BodyId { id: number_at(&structured(&grown_msg), "body") } };
    let looked = call(&mut input, &lines, 5, &read_call(5));
    same_text(&looked);
    let cap = end_cap(&structured(&looked), &grown);
    let built = call(&mut input, &lines, 6, &thicken_call(&grown, &cap));
    same_text(&built);
    let reply = structured(&built);
    if reply.as_object().is_none_or(|fields| fields.len() != 1 || !fields.contains_key("body")) {
        panic!("{MISSING}");
    }
    let made = Thickened { body: qymcad_drive::BodyId { id: number_at(&reply, "body") } };
    if made.body.id == grown.body.id {
        panic!("{MISSING}");
    }
    let read = call(&mut input, &lines, 7, &read_call(7));
    same_text(&read);
    grown_plate(&structured(&read), &drawn, &grown, &made);
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

struct Grown {
    body: qymcad_drive::BodyId,
}

struct Thickened {
    body: qymcad_drive::BodyId,
}

struct Cap {
    face: qymcad_drive::FaceId,
}

fn end_cap(read: &serde_json::Value, grown: &Grown) -> Cap {
    let body = body_of(read, grown.body.id);
    let faces = body.get("faces").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    let caps: Vec<&serde_json::Value> = faces.iter().filter(|face| face.get("role").and_then(serde_json::Value::as_str) == Some("cap-end")).collect();
    if caps.len() != 1 {
        panic!("{MISSING}");
    }
    Cap { face: qymcad_drive::FaceId { id: face_id(caps[0]) } }
}

fn face_id(face: &serde_json::Value) -> u32 {
    let id = face.get("id").and_then(serde_json::Value::as_u64).unwrap_or_else(|| panic!("{MISSING}"));
    u32::try_from(id).unwrap_or_else(|_| panic!("{MISSING}"))
}

fn body_of(read: &serde_json::Value, id: u64) -> &serde_json::Value {
    let bodies = read.get("bodies").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    bodies.iter().find(|body| body.get("id").and_then(serde_json::Value::as_u64) == Some(id)).unwrap_or_else(|| panic!("{MISSING}"))
}

fn thicken_call(grown: &Grown, cap: &Cap) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "tools/call",
        "params": {
            "name": "part.thicken",
            "arguments": { "body": grown.body.id, "face": cap.face.id, "thickness": 2 }
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

fn grown_plate(structured: &serde_json::Value, drawn: &qymcad_drive::Profile, grown: &Grown, made: &Thickened) {
    if structured.get("rebuild").and_then(serde_json::Value::as_str) != Some("quiet") {
        panic!("{MISSING}");
    }
    let refusals = structured.get("refusals").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if !refusals.is_empty() {
        panic!("{MISSING}");
    }
    one_rectangle(structured, drawn);
    let part = the_part(structured);
    three_nodes(structured, drawn, grown, made, part);
    two_bodies(structured, grown, made);
}

fn one_rectangle(structured: &serde_json::Value, drawn: &qymcad_drive::Profile) {
    let sketches = structured.get("sketches").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if sketches.len() != 1 {
        panic!("{MISSING}");
    }
    let sketch = &sketches[0];
    if sketch.get("id").and_then(serde_json::Value::as_u64) != Some(drawn.sketch.id)
        || sketch.get("name").and_then(serde_json::Value::as_str) != Some("name-sketch-n#1")
        || sketch.get("plane").and_then(serde_json::Value::as_str) != Some("xy")
    {
        panic!("{MISSING}");
    }
    let contours = sketch.get("contours").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if contours.len() != 1 || contours[0].get("id").and_then(serde_json::Value::as_u64) != Some(drawn.contour.id) || contours[0].get("form").and_then(serde_json::Value::as_str) != Some("closed") {
        panic!("{MISSING}");
    }
}

fn three_nodes(structured: &serde_json::Value, drawn: &qymcad_drive::Profile, grown: &Grown, made: &Thickened, part: u64) {
    let nodes = structured.get("nodes").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if nodes.len() != 3 {
        panic!("{MISSING}");
    }
    let sketch = node_parent(nodes, "name-sketch-n#1", drawn.sketch.id);
    let extruded = node_parent(nodes, "feat-name-extrude", grown.body.id);
    let thickened = node_parent(nodes, "feat-name-thicken", made.body.id);
    if sketch != part || extruded != part || thickened != part {
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

fn two_bodies(structured: &serde_json::Value, grown: &Grown, made: &Thickened) {
    let bodies = structured.get("bodies").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if bodies.len() != 2 {
        panic!("{MISSING}");
    }
    held(&bodies[0], grown.body.id, 6000.0);
    held_plate(&bodies[1], made.body.id);
    grown_edges(&bodies[0]);
    grown_faces(&bodies[0]);
    plated_edges(&bodies[1]);
    plated_faces(&bodies[1]);
}

fn held(body: &serde_json::Value, id: u64, mm3: f64) {
    if body.get("id").and_then(serde_json::Value::as_u64) != Some(id) {
        panic!("{MISSING}");
    }
    one_solid(body);
    let volume = body.get("volume").and_then(serde_json::Value::as_f64).unwrap_or_else(|| panic!("{MISSING}"));
    if (volume - mm3).abs() >= 1.0 {
        panic!("{MISSING}");
    }
}

fn held_plate(body: &serde_json::Value, id: u64) {
    if body.get("id").and_then(serde_json::Value::as_u64) != Some(id) {
        panic!("{MISSING}");
    }
    one_solid(body);
    let volume = body.get("volume").and_then(serde_json::Value::as_f64).unwrap_or_else(|| panic!("{MISSING}"));
    let plate = 200.0 * (30.0 + 2.0);
    if (volume - plate).abs() >= 1.0 {
        panic!("{MISSING}");
    }
}

fn grown_edges(body: &serde_json::Value) {
    let edges = body.get("edges").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    // 10, 20, and 30 do not share a 0.1 mm band. A 32 mm edge is the plated height.
    if edges.len() != 12 || count_length(edges, 10.0) != 4 || count_length(edges, 20.0) != 4 || count_length(edges, 30.0) != 4 || count_length(edges, 32.0) != 0 {
        panic!("{MISSING}");
    }
    for edge in edges {
        if !length_near(edge, 10.0) && !length_near(edge, 20.0) && !length_near(edge, 30.0) {
            panic!("{MISSING}");
        }
    }
}

fn plated_edges(body: &serde_json::Value) {
    let edges = body.get("edges").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    // 10, 20, and 32 do not share a 0.1 mm band. A 30 mm edge is the extrusion height.
    if edges.len() != 12 || count_length(edges, 10.0) != 4 || count_length(edges, 20.0) != 4 || count_length(edges, 32.0) != 4 || count_length(edges, 30.0) != 0 {
        panic!("{MISSING}");
    }
    for edge in edges {
        if !length_near(edge, 10.0) && !length_near(edge, 20.0) && !length_near(edge, 32.0) {
            panic!("{MISSING}");
        }
    }
}

fn count_length(edges: &[serde_json::Value], mm: f64) -> usize {
    edges.iter().filter(|edge| length_near(edge, mm)).count()
}

fn grown_faces(body: &serde_json::Value) {
    let faces = body.get("faces").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if faces.len() != 6 || count_role(faces, "cap-start") != 1 || count_role(faces, "cap-end") != 1 || count_role(faces, "wall") != 4 || count_role(faces, "other") != 0 {
        panic!("{MISSING}");
    }
    for face in faces {
        match face.get("role").and_then(serde_json::Value::as_str) {
            Some("cap-start" | "cap-end") => {
                if !area_near(face, 200.0) || face.get("source").is_some() {
                    panic!("{MISSING}");
                }
            }
            Some("wall") => {}
            _ => panic!("{MISSING}"),
        }
    }
    wall_once(faces, 8, 300.0);
    wall_once(faces, 9, 600.0);
    wall_once(faces, 10, 300.0);
    wall_once(faces, 11, 600.0);
}

fn plated_faces(body: &serde_json::Value) {
    let faces = body.get("faces").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    // The plate is welded on. The outer face is unnamed. The start cap is also 200 mm^2.
    if faces.len() != 6 || count_role(faces, "cap-start") != 1 || count_role(faces, "cap-end") != 0 || count_role(faces, "other") != 1 || count_role(faces, "wall") != 4 {
        panic!("{MISSING}");
    }
    for face in faces {
        match face.get("role").and_then(serde_json::Value::as_str) {
            Some("cap-start") => {
                if !area_near(face, 200.0) || face.get("source").is_some() {
                    panic!("{MISSING}");
                }
            }
            Some("other") => {
                if !area_near(face, 200.0) || face.get("source").is_some() {
                    panic!("{MISSING}");
                }
            }
            Some("wall") => {}
            _ => panic!("{MISSING}"),
        }
    }
    wall_once(faces, 8, 320.0);
    wall_once(faces, 10, 320.0);
    wall_once(faces, 9, 640.0);
    wall_once(faces, 11, 640.0);
}

fn wall_once(faces: &[serde_json::Value], source: u64, area: f64) {
    let count = faces
        .iter()
        .filter(|face| face.get("role").and_then(serde_json::Value::as_str) == Some("wall") && face.get("source").and_then(serde_json::Value::as_u64) == Some(source) && area_near(face, area))
        .count();
    if count != 1 {
        panic!("{MISSING}");
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
