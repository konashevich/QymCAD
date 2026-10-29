use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"qymcad-drive","version":"0.1.1"}}}"#;
const NOTIFIED: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
const LIST: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;
const MISSING: &str = "the rod is not threaded";

/// An M10x1.5 thread, 20 mm long, is cut on the first short rim of the 10 mm rod. The mesh is the proof.
#[test]
fn the_rod_is_threaded() {
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
    let drawn = Drawn { profile: profile_of(&structured(&drawn_msg)) };
    let turned_msg = call(&mut input, &lines, 4, &revolve_call(&drawn.profile));
    same_text(&turned_msg);
    let rod = Rod { body: qymcad_drive::BodyId { id: number_at(&structured(&turned_msg), "body") } };
    let built = call(&mut input, &lines, 5, &read_call(5));
    same_text(&built);
    let rim = Rim { edge: first_rim(&body_named(&structured(&built), rod.body.id)) };
    let cut_msg = call(&mut input, &lines, 6, &thread_call(&rod, &rim));
    same_text(&cut_msg);
    let cut_reply = structured(&cut_msg);
    if cut_reply.as_object().is_none_or(|fields| fields.len() != 1 || !fields.contains_key("body")) {
        panic!("{MISSING}");
    }
    let cut = Cut { body: qymcad_drive::BodyId { id: number_at(&cut_reply, "body") } };
    if cut.body.id == rod.body.id {
        panic!("{MISSING}");
    }
    let read = call(&mut input, &lines, 7, &read_call(7));
    same_text(&read);
    threaded(&structured(&read), &drawn, &rod, &cut);
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
            "arguments": { "plane": "xz", "from": { "x": 0, "y": 0 }, "to": { "x": 25, "y": 5 } }
        }
    })
    .to_string()
}

fn profile_of(reply: &serde_json::Value) -> qymcad_drive::Profile {
    qymcad_drive::Profile { sketch: qymcad_drive::SketchId { id: number_at(reply, "sketch") }, contour: qymcad_drive::ContourId { id: number_at(reply, "contour") } }
}

fn revolve_call(drawn: &qymcad_drive::Profile) -> String {
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

struct Drawn {
    profile: qymcad_drive::Profile,
}

struct Rod {
    body: qymcad_drive::BodyId,
}

struct Cut {
    body: qymcad_drive::BodyId,
}

struct Rim {
    edge: qymcad_drive::EdgeId,
}

fn thread_call(rod: &Rod, rim: &Rim) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "tools/call",
        "params": {
            "name": "part.thread",
            "arguments": { "body": rod.body.id, "rim": rim.edge.id, "diameter": 10, "pitch": 1.5, "length": 20 }
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

fn first_rim(body: &serde_json::Value) -> qymcad_drive::EdgeId {
    let edges = body.get("edges").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    for edge in edges {
        let length = edge.get("length").and_then(serde_json::Value::as_f64).unwrap_or(f64::NAN);
        if length < 1.0 {
            let id = edge.get("id").and_then(serde_json::Value::as_u64).unwrap_or_else(|| panic!("{MISSING}"));
            let id = u32::try_from(id).unwrap_or_else(|_| panic!("{MISSING}"));
            return qymcad_drive::EdgeId { id };
        }
    }
    panic!("{MISSING}");
}

fn threaded(structured: &serde_json::Value, drawn: &Drawn, rod: &Rod, cut: &Cut) {
    if structured.get("rebuild").and_then(serde_json::Value::as_str) != Some("quiet") {
        panic!("{MISSING}");
    }
    let refusals = structured.get("refusals").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if !refusals.is_empty() {
        panic!("{MISSING}");
    }
    two_components(structured);
    one_sketch(structured, drawn);
    three_nodes(structured, drawn, rod, cut);
    two_bodies(structured, rod, cut);
}

fn two_components(structured: &serde_json::Value) {
    let components = structured.get("components").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if components.len() != 2 {
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
    let place = component.get("place").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if place.len() != 12 {
        panic!("{MISSING}");
    }
    for index in 0..12 {
        let got = place[index].as_f64().unwrap_or(f64::NAN);
        if (got - want[index]).abs() >= 1e-6 {
            panic!("{MISSING}");
        }
    }
}

fn one_sketch(structured: &serde_json::Value, drawn: &Drawn) {
    let sketches = structured.get("sketches").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if sketches.len() != 1 {
        panic!("{MISSING}");
    }
    let sketch = &sketches[0];
    if sketch.get("id").and_then(serde_json::Value::as_u64) != Some(drawn.profile.sketch.id)
        || sketch.get("name").and_then(serde_json::Value::as_str) != Some("name-sketch-n#1")
        || sketch.get("plane").and_then(serde_json::Value::as_str) != Some("xz")
    {
        panic!("{MISSING}");
    }
    let contours = sketch.get("contours").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if contours.len() != 1
        || contours[0].get("id").and_then(serde_json::Value::as_u64) != Some(drawn.profile.contour.id)
        || contours[0].get("form").and_then(serde_json::Value::as_str) != Some("closed")
    {
        panic!("{MISSING}");
    }
}

fn three_nodes(structured: &serde_json::Value, drawn: &Drawn, rod: &Rod, cut: &Cut) {
    let part = part_named(structured, "name-part-n#1");
    let nodes = structured.get("nodes").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if nodes.len() != 3 || nodes.iter().any(|node| node.get("name").and_then(serde_json::Value::as_str) == Some("feat-name-body-union")) {
        panic!("{MISSING}");
    }
    if node_at(nodes, 0, "name-sketch-n#1", drawn.profile.sketch.id) != part
        || node_at(nodes, 1, "feat-name-revolve", rod.body.id) != part
        || node_at(nodes, 2, "feat-name-thread", cut.body.id) != part
    {
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

fn part_named(structured: &serde_json::Value, name: &str) -> u64 {
    let components = structured.get("components").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    let found: Vec<&serde_json::Value> = components.iter().filter(|component| component.get("name").and_then(serde_json::Value::as_str) == Some(name)).collect();
    if found.len() != 1 {
        panic!("{MISSING}");
    }
    found[0].get("id").and_then(serde_json::Value::as_u64).unwrap_or_else(|| panic!("{MISSING}"))
}

fn two_bodies(structured: &serde_json::Value, rod: &Rod, cut: &Cut) {
    let bodies = structured.get("bodies").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if bodies.len() != 2 {
        panic!("{MISSING}");
    }
    plain(&bodies[0], rod.body.id);
    helix(&bodies[1], cut.body.id);
}

fn plain(body: &serde_json::Value, id: u64) {
    if body.get("id").and_then(serde_json::Value::as_u64) != Some(id) {
        panic!("{MISSING}");
    }
    one_solid(body);
    let faces = body.get("faces").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    let edges = body.get("edges").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if faces.len() != 3 || edges.len() != 3 || count_short(edges) != 2 || count_length(edges, 25.0) != 1 {
        panic!("{MISSING}");
    }
}

fn helix(body: &serde_json::Value, id: u64) {
    if body.get("id").and_then(serde_json::Value::as_u64) != Some(id) {
        panic!("{MISSING}");
    }
    one_solid(body);
    let faces = body.get("faces").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if faces.len() != 63 {
        panic!("{MISSING}");
    }
    // The kernel volume prints about 1686.45 and does not share this band. The mesh debug display is the proof.
    let mesh = body.get("mesh").and_then(serde_json::Value::as_f64).unwrap_or_else(|| panic!("{MISSING}"));
    let cut = 1681.3954982007497;
    if (mesh - cut).abs() >= 1.0 {
        panic!("{MISSING}");
    }
}

fn count_short(edges: &[serde_json::Value]) -> usize {
    edges.iter().filter(|edge| edge.get("length").and_then(serde_json::Value::as_f64).unwrap_or(f64::NAN) < 1.0).count()
}

fn count_length(edges: &[serde_json::Value], mm: f64) -> usize {
    edges.iter().filter(|edge| length_near(edge, mm)).count()
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

fn body_named(structured: &serde_json::Value, id: u64) -> serde_json::Value {
    let bodies = structured.get("bodies").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    bodies.iter().find(|body| body.get("id").and_then(serde_json::Value::as_u64) == Some(id)).cloned().unwrap_or_else(|| panic!("{MISSING}"))
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
