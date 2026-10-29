use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"qymcad-drive","version":"0.1.1"}}}"#;
const NOTIFIED: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
const LIST: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;
const MISSING: &str = "the second part does not stand";

/// A second part, shifted 100 mm along X, keeps its own 10 by 20 by 30 mm box. The first box stays.
#[test]
fn the_second_part_stands() {
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

    let first_msg = call(&mut input, &lines, 3, &rectangle_call(3));
    same_text(&first_msg);
    let first = FirstDrawn { drawn: profile_of(&structured(&first_msg)) };
    let extruded_msg = call(&mut input, &lines, 4, &extrude_call(4, &first.drawn));
    same_text(&extruded_msg);
    let block = FirstBox { body: qymcad_drive::BodyId { id: number_at(&structured(&extruded_msg), "body") } };
    let built = call(&mut input, &lines, 5, &read_call(5));
    same_text(&built);
    let root = assembly_of(&structured(&built));
    let earlier = part_named(&structured(&built), "name-part-n#1");
    let part_msg = call(&mut input, &lines, 6, &part_call());
    same_text(&part_msg);
    let reply = structured(&part_msg);
    if reply.as_object().is_none_or(|fields| fields.len() != 1 || !fields.contains_key("part")) {
        panic!("{MISSING}");
    }
    let added = Added { part: qymcad_drive::PartId { id: number_at(&reply, "part") } };
    if added.part.id == root || added.part.id == earlier {
        panic!("{MISSING}");
    }
    let placed = call(&mut input, &lines, 7, &read_call(7));
    same_text(&placed);
    let second_msg = call(&mut input, &lines, 8, &rectangle_call(8));
    same_text(&second_msg);
    let second = SecondDrawn { drawn: profile_of(&structured(&second_msg)) };
    let next_msg = call(&mut input, &lines, 9, &extrude_call(9, &second.drawn));
    same_text(&next_msg);
    let next = SecondBox { body: qymcad_drive::BodyId { id: number_at(&structured(&next_msg), "body") } };
    if next.body.id == block.body.id {
        panic!("{MISSING}");
    }
    let read = call(&mut input, &lines, 10, &read_call(10));
    same_text(&read);
    stood(&structured(&read), &first, &block, &second, &next, &added);
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

fn rectangle_call(id: i64) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
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

fn extrude_call(id: i64, drawn: &qymcad_drive::Profile) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "tools/call",
        "params": {
            "name": "part.extrude",
            "arguments": { "sketch": drawn.sketch.id, "contour": drawn.contour.id, "height": 30 }
        }
    })
    .to_string()
}

struct FirstDrawn {
    drawn: qymcad_drive::Profile,
}

struct SecondDrawn {
    drawn: qymcad_drive::Profile,
}

struct FirstBox {
    body: qymcad_drive::BodyId,
}

struct SecondBox {
    body: qymcad_drive::BodyId,
}

struct Added {
    part: qymcad_drive::PartId,
}

fn part_call() -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "tools/call",
        "params": {
            "name": "document.part",
            "arguments": { "place": [1, 0, 0, 100, 0, 1, 0, 0, 0, 0, 1, 0] }
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

fn stood(structured: &serde_json::Value, first: &FirstDrawn, block: &FirstBox, second: &SecondDrawn, next: &SecondBox, added: &Added) {
    if structured.get("rebuild").and_then(serde_json::Value::as_str) != Some("quiet") {
        panic!("{MISSING}");
    }
    let refusals = structured.get("refusals").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if !refusals.is_empty() {
        panic!("{MISSING}");
    }
    three_components(structured, added);
    two_sketches(structured, first, second);
    four_nodes(structured, first, block, second, next, added);
    two_boxes(structured, block, next);
}

fn three_components(structured: &serde_json::Value, added: &Added) {
    let components = structured.get("components").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if components.len() != 3 {
        panic!("{MISSING}");
    }
    let root = assembly_row(&components[0]);
    identity(&components[0]);
    if components[1].get("kind").and_then(serde_json::Value::as_str) != Some("part")
        || components[1].get("name").and_then(serde_json::Value::as_str) != Some("name-part-n#1")
        || components[1].get("parent").and_then(serde_json::Value::as_u64) != Some(root)
    {
        panic!("{MISSING}");
    }
    identity(&components[1]);
    if components[2].get("id").and_then(serde_json::Value::as_u64) != Some(added.part.id)
        || components[2].get("kind").and_then(serde_json::Value::as_str) != Some("part")
        || components[2].get("name").and_then(serde_json::Value::as_str) != Some("name-part-n#2")
        || components[2].get("parent").and_then(serde_json::Value::as_u64) != Some(root)
        || components[1].get("id").and_then(serde_json::Value::as_u64) == Some(added.part.id)
    {
        panic!("{MISSING}");
    }
    shifted(&components[2]);
}

fn assembly_row(component: &serde_json::Value) -> u64 {
    if component.get("kind").and_then(serde_json::Value::as_str) != Some("assembly")
        || component.get("name").and_then(serde_json::Value::as_str) != Some("name-assembly")
        || !component.get("parent").is_some_and(serde_json::Value::is_null)
    {
        panic!("{MISSING}");
    }
    component.get("id").and_then(serde_json::Value::as_u64).unwrap_or_else(|| panic!("{MISSING}"))
}

fn identity(component: &serde_json::Value) {
    let want = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0];
    place_near(component, &want);
}

fn shifted(component: &serde_json::Value) {
    let want = [1.0, 0.0, 0.0, 100.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0];
    place_near(component, &want);
}

fn place_near(component: &serde_json::Value, want: &[f64; 12]) {
    let place = component.get("place").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if place.len() != 12 {
        panic!("{MISSING}");
    }
    for index in 0..12 {
        let got = place[index].as_f64().unwrap_or(f64::NAN);
        if (got - want[index]).abs() >= 1e-9 {
            panic!("{MISSING}");
        }
    }
}

fn two_sketches(structured: &serde_json::Value, first: &FirstDrawn, second: &SecondDrawn) {
    let sketches = structured.get("sketches").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if sketches.len() != 2 {
        panic!("{MISSING}");
    }
    one_sketch(&sketches[0], &first.drawn, "name-sketch-n#1");
    one_sketch(&sketches[1], &second.drawn, "name-sketch-n#2");
}

fn one_sketch(sketch: &serde_json::Value, drawn: &qymcad_drive::Profile, name: &str) {
    if sketch.get("id").and_then(serde_json::Value::as_u64) != Some(drawn.sketch.id)
        || sketch.get("name").and_then(serde_json::Value::as_str) != Some(name)
        || sketch.get("plane").and_then(serde_json::Value::as_str) != Some("xy")
    {
        panic!("{MISSING}");
    }
    let contours = sketch.get("contours").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if contours.len() != 1 || contours[0].get("id").and_then(serde_json::Value::as_u64) != Some(drawn.contour.id) || contours[0].get("form").and_then(serde_json::Value::as_str) != Some("closed") {
        panic!("{MISSING}");
    }
}

fn four_nodes(structured: &serde_json::Value, first: &FirstDrawn, block: &FirstBox, second: &SecondDrawn, next: &SecondBox, added: &Added) {
    let earlier = part_named(structured, "name-part-n#1");
    let nodes = structured.get("nodes").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if nodes.len() != 4 || nodes.iter().any(|node| node.get("name").and_then(serde_json::Value::as_str) == Some("feat-name-body-union")) {
        panic!("{MISSING}");
    }
    if node_at(nodes, 0, "name-sketch-n#1", first.drawn.sketch.id) != earlier || node_at(nodes, 1, "feat-name-extrude", block.body.id) != earlier {
        panic!("{MISSING}");
    }
    if node_at(nodes, 2, "name-sketch-n#2", second.drawn.sketch.id) != added.part.id || node_at(nodes, 3, "feat-name-extrude", next.body.id) != added.part.id {
        panic!("{MISSING}");
    }
}

fn node_at(nodes: &[serde_json::Value], place: usize, name: &str, id: u64) -> u64 {
    let node = nodes.get(place).unwrap_or_else(|| panic!("{MISSING}"));
    if node.get("name").and_then(serde_json::Value::as_str) != Some(name) || node.get("id").and_then(serde_json::Value::as_u64) != Some(id) {
        panic!("{MISSING}");
    }
    node.get("parent").and_then(serde_json::Value::as_u64).unwrap_or_else(|| panic!("{MISSING}"))
}

fn assembly_of(structured: &serde_json::Value) -> u64 {
    let components = structured.get("components").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    let roots: Vec<&serde_json::Value> = components.iter().filter(|component| component.get("kind").and_then(serde_json::Value::as_str) == Some("assembly")).collect();
    if roots.len() != 1 {
        panic!("{MISSING}");
    }
    roots[0].get("id").and_then(serde_json::Value::as_u64).unwrap_or_else(|| panic!("{MISSING}"))
}

fn part_named(structured: &serde_json::Value, name: &str) -> u64 {
    let components = structured.get("components").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    let found: Vec<&serde_json::Value> = components.iter().filter(|component| component.get("name").and_then(serde_json::Value::as_str) == Some(name)).collect();
    if found.len() != 1 {
        panic!("{MISSING}");
    }
    found[0].get("id").and_then(serde_json::Value::as_u64).unwrap_or_else(|| panic!("{MISSING}"))
}

fn two_boxes(structured: &serde_json::Value, block: &FirstBox, next: &SecondBox) {
    let bodies = structured.get("bodies").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if bodies.len() != 2 {
        panic!("{MISSING}");
    }
    held(&bodies[0], block.body.id);
    box_faces(&bodies[0], 8);
    held(&bodies[1], next.body.id);
    box_faces(&bodies[1], 22);
}

fn held(body: &serde_json::Value, id: u64) {
    if body.get("id").and_then(serde_json::Value::as_u64) != Some(id) {
        panic!("{MISSING}");
    }
    // The mesh of each box prints the same number as its volume. The read's volume is the kernel volume.
    one_solid(body);
    let volume = body.get("volume").and_then(serde_json::Value::as_f64).unwrap_or_else(|| panic!("{MISSING}"));
    let block = 10.0 * 20.0 * 30.0;
    if (volume - block).abs() >= 1.0 {
        panic!("{MISSING}");
    }
    box_edges(body);
}

fn box_edges(body: &serde_json::Value) {
    let edges = body.get("edges").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    // 0, 10, 20, and 30 do not share a 0.1 mm band.
    if edges.len() != 12 || count_length(edges, 10.0) != 4 || count_length(edges, 20.0) != 4 || count_length(edges, 30.0) != 4 || count_length(edges, 0.0) != 0 {
        panic!("{MISSING}");
    }
    for edge in edges {
        if !length_near(edge, 10.0) && !length_near(edge, 20.0) && !length_near(edge, 30.0) {
            panic!("{MISSING}");
        }
    }
}

fn count_length(edges: &[serde_json::Value], mm: f64) -> usize {
    edges.iter().filter(|edge| length_near(edge, mm)).count()
}

fn box_faces(body: &serde_json::Value, first_source: u64) {
    let faces = body.get("faces").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    // A wall of the first box is not a wall of the second. This body is the one passed in.
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
    wall_once(faces, first_source, 300.0);
    wall_once(faces, first_source + 1, 600.0);
    wall_once(faces, first_source + 2, 300.0);
    wall_once(faces, first_source + 3, 600.0);
}

fn wall_once(faces: &[serde_json::Value], source: u64, area: f64) {
    let found = faces
        .iter()
        .filter(|face| face.get("role").and_then(serde_json::Value::as_str) == Some("wall") && face.get("source").and_then(serde_json::Value::as_u64) == Some(source) && area_near(face, area))
        .count();
    if found != 1 {
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
