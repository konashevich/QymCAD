use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"qymcad-drive","version":"0.1.1"}}}"#;
const NOTIFIED: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
const LIST: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;
const MISSING: &str = "the cap is not trimmed";

/// The end cap of a 10 by 20 by 30 mm extrusion, cut by a tool that crosses it at x = 4, keeps the 80 mm^2 piece.
#[test]
fn the_cap_is_trimmed() {
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

    let tool_msg = call(&mut input, &lines, 3, &tool_rectangle());
    same_text(&tool_msg);
    let tool_drawn = ToolSketch { drawn: profile_of(&structured(&tool_msg)) };
    let tool_body = call(&mut input, &lines, 4, &extrude_call(&tool_drawn.drawn, 4, 40.0));
    same_text(&tool_body);
    let tool = Tool { body: qymcad_drive::BodyId { id: number_at(&structured(&tool_body), "body") } };
    let block_msg = call(&mut input, &lines, 5, &block_rectangle());
    same_text(&block_msg);
    let block_drawn = BlockSketch { drawn: profile_of(&structured(&block_msg)) };
    let block_body = call(&mut input, &lines, 6, &extrude_call(&block_drawn.drawn, 6, 30.0));
    same_text(&block_body);
    let block = Block { body: qymcad_drive::BodyId { id: number_at(&structured(&block_body), "body") } };
    let looked = call(&mut input, &lines, 7, &read_call(7));
    same_text(&looked);
    let end = end_cap(&structured(&looked), &block);
    let copied_msg = call(&mut input, &lines, 8, &copy_call(&block, &end));
    same_text(&copied_msg);
    let copy = CapCopy { body: one_body(&structured(&copied_msg)) };
    if copy.body.id == tool.body.id || copy.body.id == block.body.id {
        panic!("{MISSING}");
    }
    let built = call(&mut input, &lines, 9, &read_call(9));
    same_text(&built);
    let trimmed_msg = call(&mut input, &lines, 10, &trim_call(&copy, &tool));
    same_text(&trimmed_msg);
    let reply = structured(&trimmed_msg);
    if reply.as_object().is_none_or(|fields| fields.len() != 1 || !fields.contains_key("body")) {
        panic!("{MISSING}");
    }
    let piece = Piece { body: qymcad_drive::BodyId { id: number_at(&reply, "body") } };
    if piece.body.id == tool.body.id || piece.body.id == block.body.id || piece.body.id == copy.body.id {
        panic!("{MISSING}");
    }
    let read = call(&mut input, &lines, 11, &read_call(11));
    same_text(&read);
    trimmed_cap(&structured(&read), &tool_drawn, &block_drawn, &tool, &block, &copy, &piece);
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

fn tool_rectangle() -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "sketch.rectangle",
            "arguments": { "plane": "xy", "from": { "x": 4, "y": -1 }, "to": { "x": 12, "y": 21 } }
        }
    })
    .to_string()
}

fn block_rectangle() -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 5,
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

fn extrude_call(drawn: &qymcad_drive::Profile, id: i64, height: f64) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "tools/call",
        "params": {
            "name": "part.extrude",
            "arguments": { "sketch": drawn.sketch.id, "contour": drawn.contour.id, "height": height }
        }
    })
    .to_string()
}

struct ToolSketch {
    drawn: qymcad_drive::Profile,
}

struct BlockSketch {
    drawn: qymcad_drive::Profile,
}

struct Tool {
    body: qymcad_drive::BodyId,
}

struct Block {
    body: qymcad_drive::BodyId,
}

struct CapCopy {
    body: qymcad_drive::BodyId,
}

struct Piece {
    body: qymcad_drive::BodyId,
}

struct End {
    face: qymcad_drive::FaceId,
}

fn end_cap(read: &serde_json::Value, block: &Block) -> End {
    let body = body_of(read, block.body.id);
    let faces = body.get("faces").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    let caps: Vec<&serde_json::Value> = faces.iter().filter(|face| face.get("role").and_then(serde_json::Value::as_str) == Some("cap-end")).collect();
    if caps.len() != 1 {
        panic!("{MISSING}");
    }
    End { face: qymcad_drive::FaceId { id: face_id(caps[0]) } }
}

fn face_id(face: &serde_json::Value) -> u32 {
    let id = face.get("id").and_then(serde_json::Value::as_u64).unwrap_or_else(|| panic!("{MISSING}"));
    u32::try_from(id).unwrap_or_else(|_| panic!("{MISSING}"))
}

fn body_of(read: &serde_json::Value, id: u64) -> &serde_json::Value {
    let bodies = read.get("bodies").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    bodies.iter().find(|body| body.get("id").and_then(serde_json::Value::as_u64) == Some(id)).unwrap_or_else(|| panic!("{MISSING}"))
}

fn one_body(reply: &serde_json::Value) -> qymcad_drive::BodyId {
    if reply.as_object().is_none_or(|fields| fields.len() != 1 || !fields.contains_key("body")) {
        panic!("{MISSING}");
    }
    qymcad_drive::BodyId { id: number_at(reply, "body") }
}

fn copy_call(block: &Block, end: &End) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 8,
        "method": "tools/call",
        "params": {
            "name": "part.copy_face",
            "arguments": { "body": block.body.id, "face": end.face.id }
        }
    })
    .to_string()
}

fn trim_call(copy: &CapCopy, tool: &Tool) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 10,
        "method": "tools/call",
        "params": {
            "name": "part.trim",
            "arguments": {
                "sheet": copy.body.id,
                "tool": tool.body.id,
                "keep": { "x": 2, "y": 10, "z": 30 }
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

fn trimmed_cap(structured: &serde_json::Value, tool_drawn: &ToolSketch, block_drawn: &BlockSketch, tool: &Tool, block: &Block, copy: &CapCopy, piece: &Piece) {
    if structured.get("rebuild").and_then(serde_json::Value::as_str) != Some("quiet") {
        panic!("{MISSING}");
    }
    let refusals = structured.get("refusals").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if !refusals.is_empty() {
        panic!("{MISSING}");
    }
    two_sketches(structured, tool_drawn, block_drawn);
    six_nodes(structured, tool_drawn, block_drawn, tool, block, copy, piece);
    four_bodies(structured, tool, block, copy, piece);
}

fn two_sketches(structured: &serde_json::Value, tool_drawn: &ToolSketch, block_drawn: &BlockSketch) {
    let sketches = structured.get("sketches").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if sketches.len() != 2 {
        panic!("{MISSING}");
    }
    one_sketch(sketches, &tool_drawn.drawn, "name-sketch-n#1");
    one_sketch(sketches, &block_drawn.drawn, "name-sketch-n#2");
}

fn one_sketch(sketches: &[serde_json::Value], drawn: &qymcad_drive::Profile, name: &str) {
    let found = sketches.iter().filter(|sketch| sketch.get("id").and_then(serde_json::Value::as_u64) == Some(drawn.sketch.id)).collect::<Vec<_>>();
    if found.len() != 1 || found[0].get("name").and_then(serde_json::Value::as_str) != Some(name) || found[0].get("plane").and_then(serde_json::Value::as_str) != Some("xy") {
        panic!("{MISSING}");
    }
    let contours = found[0].get("contours").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if contours.len() != 1 || contours[0].get("id").and_then(serde_json::Value::as_u64) != Some(drawn.contour.id) || contours[0].get("form").and_then(serde_json::Value::as_str) != Some("closed") {
        panic!("{MISSING}");
    }
}

fn six_nodes(structured: &serde_json::Value, tool_drawn: &ToolSketch, block_drawn: &BlockSketch, tool: &Tool, block: &Block, copy: &CapCopy, piece: &Piece) {
    let part = the_part(structured);
    let nodes = structured.get("nodes").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if nodes.len() != 6 {
        panic!("{MISSING}");
    }
    let parents = [
        node_at(nodes, 0, "name-sketch-n#1", tool_drawn.drawn.sketch.id),
        node_at(nodes, 1, "feat-name-extrude", tool.body.id),
        node_at(nodes, 2, "name-sketch-n#2", block_drawn.drawn.sketch.id),
        node_at(nodes, 3, "feat-name-extrude", block.body.id),
        node_at(nodes, 4, "feat-name-face-copy", copy.body.id),
        node_at(nodes, 5, "feat-name-trim", piece.body.id),
    ];
    if parents.iter().any(|parent| *parent != part) {
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

fn four_bodies(structured: &serde_json::Value, tool: &Tool, block: &Block, copy: &CapCopy, piece: &Piece) {
    let bodies = structured.get("bodies").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if bodies.len() != 4 {
        panic!("{MISSING}");
    }
    held_tool(&bodies[0], tool.body.id);
    held_block(&bodies[1], block.body.id);
    held_sheet(&bodies[2], copy.body.id);
    held_sheet(&bodies[3], piece.body.id);
    tool_edges(&bodies[0]);
    tool_faces(&bodies[0]);
    block_edges(&bodies[1]);
    block_faces(&bodies[1]);
    copy_edges(&bodies[2]);
    copy_faces(&bodies[2]);
    piece_edges(&bodies[3]);
    piece_faces(&bodies[3]);
}

fn held_tool(body: &serde_json::Value, id: u64) {
    if body.get("id").and_then(serde_json::Value::as_u64) != Some(id) {
        panic!("{MISSING}");
    }
    one_solid(body);
    // The volume prints just under 7040. The mesh prints the same number. The read's volume is the kernel volume.
    let volume = body.get("volume").and_then(serde_json::Value::as_f64).unwrap_or_else(|| panic!("{MISSING}"));
    let tool = (12.0 - 4.0) * (21.0 - -1.0) * 40.0;
    if (volume - tool).abs() >= 1.0 {
        panic!("{MISSING}");
    }
}

fn held_block(body: &serde_json::Value, id: u64) {
    if body.get("id").and_then(serde_json::Value::as_u64) != Some(id) {
        panic!("{MISSING}");
    }
    one_solid(body);
    let volume = body.get("volume").and_then(serde_json::Value::as_f64).unwrap_or_else(|| panic!("{MISSING}"));
    let block = 10.0 * 20.0 * 30.0;
    if (volume - block).abs() >= 1.0 {
        panic!("{MISSING}");
    }
}

fn held_sheet(body: &serde_json::Value, id: u64) {
    if body.get("id").and_then(serde_json::Value::as_u64) != Some(id) {
        panic!("{MISSING}");
    }
    // The copy's mesh is 2000 mm^3. The piece's mesh is 800 mm^3. The read's volume is the kernel volume.
    if body.get("sound").and_then(serde_json::Value::as_str) != Some("valid") || body.get("solids").and_then(serde_json::Value::as_u64) != Some(0) {
        panic!("{MISSING}");
    }
    let volume = body.get("volume").and_then(serde_json::Value::as_f64).unwrap_or_else(|| panic!("{MISSING}"));
    if volume.abs() >= 1.0 {
        panic!("{MISSING}");
    }
}

fn tool_edges(body: &serde_json::Value) {
    let edges = body.get("edges").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    // 0, 4, 8, 10, 20, 22, 30, and 40 do not share a 0.1 mm band.
    if edges.len() != 12 || count_length(edges, 8.0) != 4 || count_length(edges, 22.0) != 4 || count_length(edges, 40.0) != 4 || count_length(edges, 0.0) != 0 {
        panic!("{MISSING}");
    }
    for edge in edges {
        if !length_near(edge, 8.0) && !length_near(edge, 22.0) && !length_near(edge, 40.0) {
            panic!("{MISSING}");
        }
    }
}

fn block_edges(body: &serde_json::Value) {
    let edges = body.get("edges").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if edges.len() != 12 || count_length(edges, 10.0) != 4 || count_length(edges, 20.0) != 4 || count_length(edges, 30.0) != 4 || count_length(edges, 0.0) != 0 {
        panic!("{MISSING}");
    }
    for edge in edges {
        if !length_near(edge, 10.0) && !length_near(edge, 20.0) && !length_near(edge, 30.0) {
            panic!("{MISSING}");
        }
    }
}

fn copy_edges(body: &serde_json::Value) {
    let edges = body.get("edges").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if edges.len() != 4 || count_length(edges, 10.0) != 2 || count_length(edges, 20.0) != 2 || count_length(edges, 4.0) != 0 || count_length(edges, 0.0) != 0 {
        panic!("{MISSING}");
    }
    for edge in edges {
        if !length_near(edge, 10.0) && !length_near(edge, 20.0) {
            panic!("{MISSING}");
        }
    }
}

fn piece_edges(body: &serde_json::Value) {
    let edges = body.get("edges").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if edges.len() != 4 || count_length(edges, 4.0) != 2 || count_length(edges, 20.0) != 2 || count_length(edges, 10.0) != 0 || count_length(edges, 0.0) != 0 {
        panic!("{MISSING}");
    }
    for edge in edges {
        if !length_near(edge, 4.0) && !length_near(edge, 20.0) {
            panic!("{MISSING}");
        }
    }
}

fn count_length(edges: &[serde_json::Value], mm: f64) -> usize {
    edges.iter().filter(|edge| length_near(edge, mm)).count()
}

fn tool_faces(body: &serde_json::Value) {
    let faces = body.get("faces").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if faces.len() != 6 || count_role(faces, "cap-start") != 1 || count_role(faces, "cap-end") != 1 || count_role(faces, "wall") != 4 || count_role(faces, "other") != 0 {
        panic!("{MISSING}");
    }
    for face in faces {
        match face.get("role").and_then(serde_json::Value::as_str) {
            Some("cap-start" | "cap-end") => {
                if !area_near(face, 176.0) || face.get("source").is_some() {
                    panic!("{MISSING}");
                }
            }
            Some("wall") => {}
            _ => panic!("{MISSING}"),
        }
    }
    wall_once(faces, 8, 320.0);
    wall_once(faces, 9, 880.0);
    wall_once(faces, 10, 320.0);
    wall_once(faces, 11, 880.0);
}

fn block_faces(body: &serde_json::Value) {
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
    wall_once(faces, 21, 300.0);
    wall_once(faces, 22, 600.0);
    wall_once(faces, 23, 300.0);
    wall_once(faces, 24, 600.0);
}

fn copy_faces(body: &serde_json::Value) {
    let faces = body.get("faces").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    if faces.len() != 1 || count_role(faces, "other") != 1 || count_role(faces, "cap-start") != 0 || count_role(faces, "cap-end") != 0 || count_role(faces, "wall") != 0 {
        panic!("{MISSING}");
    }
    for face in faces {
        if face.get("role").and_then(serde_json::Value::as_str) != Some("other") || !area_near(face, 200.0) || face.get("source").is_some() {
            panic!("{MISSING}");
        }
    }
}

fn piece_faces(body: &serde_json::Value) {
    let faces = body.get("faces").and_then(serde_json::Value::as_array).unwrap_or_else(|| panic!("{MISSING}"));
    // The block's caps and the copy are 200 mm^2. This face is the piece.
    if faces.len() != 1 || count_role(faces, "other") != 1 || count_role(faces, "cap-start") != 0 || count_role(faces, "cap-end") != 0 || count_role(faces, "wall") != 0 {
        panic!("{MISSING}");
    }
    for face in faces {
        if face.get("role").and_then(serde_json::Value::as_str) != Some("other") || !area_near(face, 80.0) || face.get("source").is_some() {
            panic!("{MISSING}");
        }
    }
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
