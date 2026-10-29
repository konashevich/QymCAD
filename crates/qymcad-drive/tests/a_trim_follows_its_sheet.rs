use qymcad_drive::{
    Area, BodyId, BoxLength, CopyFace, Corners, Cuboid, FaceId, FaceRole, From, Height, Keep, KeepX, KeepY, KeepZ, Kind, Radius, Rebuild, Session, Sound, To, Trim, Volume, Width, WorldPlane,
};

/// The point on the small side of the cut keeps a sheet of 80 mm^2.
#[test]
fn the_point_keeps_the_smaller_piece() {
    let mut session = Session::empty();
    let made = crossing(&mut session);
    let piece = session.trim(cut(made.sheet, left(), made.tool)).expect("a trim");
    let doc = session.document();
    one_face(sheet(&doc, piece), Area { mm2: 80.0 }, "the piece");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.components.iter().filter(|item| item.kind == Kind::Part).count(), 1, "one part");
    assert_eq!(trims(&doc), 1, "the trim node");
    assert_eq!(doc.nodes.iter().find(|node| node.id == piece.id).expect("the node").name, "feat-name-trim");
    assert_ne!(piece, made.block, "the piece is the box");
    near(solid(&doc, made.block), Volume { mm3: 6000.0 }, "the box");
    near(solid(&doc, made.tool), Volume { mm3: 7040.0 }, "the tool");
}

/// The point on the large side of the cut keeps a sheet of 120 mm^2.
#[test]
fn the_other_point_keeps_the_larger_piece() {
    let mut session = Session::empty();
    let made = crossing(&mut session);
    let piece = session.trim(cut(made.sheet, right(), made.tool)).expect("a trim");
    let doc = session.document();
    one_face(sheet(&doc, piece), Area { mm2: 120.0 }, "the piece");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(solid(&doc, made.block), Volume { mm3: 6000.0 }, "the box");
}

/// The origin is a legal point, and it keeps the 80 mm^2 piece.
#[test]
fn the_origin_keeps_the_smaller_piece() {
    let mut session = Session::empty();
    let made = crossing(&mut session);
    let piece = session.trim(cut(made.sheet, origin(), made.tool)).expect("a trim");
    let doc = session.document();
    one_face(sheet(&doc, piece), Area { mm2: 80.0 }, "the piece");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// A tool that misses leaves the cap. The crossing tool replaces that trim.
#[test]
fn a_tool_that_misses_is_replaced_by_the_cut() {
    let mut session = Session::empty();
    let made = crossing(&mut session);
    let far = far_tool(&mut session);
    let failed = session.trim(cut(made.sheet, left(), far)).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "operation failed: Trim", "the refusal was {}", refusal(&doc));
    one_face(sheet(&doc, failed), Area { mm2: 200.0 }, "the passed sheet");
    near(solid(&doc, made.block), Volume { mm3: 6000.0 }, "the box");
    let gap = session.trim(cut(made.sheet, point(KeepX { mm: f64::NAN }, KeepY { mm: 10.0 }, KeepZ { mm: 30.0 }), far));
    assert_eq!(gap.expect_err("the point").text, "error-trim-keep");
    let solid = session.trim(cut(made.block, left(), made.tool));
    assert_eq!(solid.expect_err("the box").text, "error-trim-sheet");
    let held = session.document();
    assert_eq!(refusal(&held), "operation failed: Trim", "the refusal was {}", refusal(&held));
    assert_eq!(trims(&held), 1, "the refused trim was rewritten");
    let piece = session.trim(cut(made.sheet, left(), made.tool)).expect("the cut is tried");
    assert_eq!(piece, failed, "a second trim was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(trims(&doc), 1, "the refused trim was left beside the new one");
    one_face(sheet(&doc, piece), Area { mm2: 80.0 }, "the piece");
}

/// The id a refused trim returned is that same trim.
#[test]
fn a_refused_trim_named_by_its_own_id_is_the_same_trim() {
    let mut session = Session::empty();
    let made = crossing(&mut session);
    let far = far_tool(&mut session);
    let failed = session.trim(cut(made.sheet, left(), far)).expect("the node is written");
    let _held = session.document();
    let piece = session.trim(cut(failed, left(), made.tool)).expect("the cut is tried");
    assert_eq!(piece, failed, "a second trim was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(trims(&doc), 1, "the refused trim was left beside the new one");
    one_face(sheet(&doc, piece), Area { mm2: 80.0 }, "the piece");
    near(solid(&doc, made.block), Volume { mm3: 6000.0 }, "the box");
}

/// The cut is not a tool for itself. Opened again, that tool is not built yet.
#[test]
fn the_cut_is_not_its_own_tool() {
    let mut session = Session::empty();
    let made = crossing(&mut session);
    let far = far_tool(&mut session);
    let failed = session.trim(cut(made.sheet, left(), far)).expect("the node is written");
    let _held = session.document();
    let named = session.trim(cut(made.sheet, left(), failed));
    assert_eq!(named.expect_err("the tool").text, "error-trim-tool");
    let itself = session.trim(cut(failed, left(), failed));
    assert_eq!(itself.expect_err("the tool").text, "error-trim-tool");
    let doc = session.document();
    assert_eq!(refusal(&doc), "operation failed: Trim", "the refusal was {}", refusal(&doc));
    assert_eq!(trims(&doc), 1, "the refused trim was rewritten");
    one_face(sheet(&doc, failed), Area { mm2: 200.0 }, "the passed sheet");
    let opened = opened(&session, "itself");
    assert_eq!(refusal(&opened), "operation failed: Trim", "the refusal was {}", refusal(&opened));
    one_face(sheet(&opened, failed), Area { mm2: 200.0 }, "the opened sheet");
    let piece = session.trim(cut(failed, left(), made.tool)).expect("the cut is tried");
    assert_eq!(piece, failed, "a second trim was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    one_face(sheet(&doc, piece), Area { mm2: 80.0 }, "the piece");
}

/// A body that stands later is not a tool for this cut. Opened again, that body is not built yet.
#[test]
fn a_later_body_is_not_the_tool() {
    let mut session = Session::empty();
    let made = crossing(&mut session);
    let far = far_tool(&mut session);
    let first = session.trim(cut(made.sheet, left(), far)).expect("the first node");
    let _held = session.document();
    let another = session.copy_face(CopyFace { body: made.block, face: made.cap }).expect("another cap");
    let _sheet = session.document();
    let second = session.trim(cut(another, left(), far)).expect("the second node");
    let _read = session.document();
    let forward = session.trim(cut(made.sheet, left(), second));
    assert_eq!(forward.expect_err("the later cut").text, "error-trim-tool");
    let doc = session.document();
    assert_eq!(refusal(&doc), "operation failed: Trim operation failed: Trim", "the refusal was {}", refusal(&doc));
    assert_eq!(trims(&doc), 2, "the first trim was rewritten");
    one_face(sheet(&doc, first), Area { mm2: 200.0 }, "the first sheet");
    one_face(sheet(&doc, second), Area { mm2: 200.0 }, "the second sheet");
    let opened = opened(&session, "later");
    assert_eq!(refusal(&opened), "operation failed: Trim operation failed: Trim", "the refusal was {}", refusal(&opened));
    one_face(sheet(&opened, first), Area { mm2: 200.0 }, "the opened first sheet");
    one_face(sheet(&opened, second), Area { mm2: 200.0 }, "the opened second sheet");
}

/// The cap already trimmed is not trimmed again.
#[test]
fn the_trimmed_cap_is_not_trimmed_again() {
    let mut session = Session::empty();
    let made = crossing(&mut session);
    let piece = session.trim(cut(made.sheet, left(), made.tool)).expect("a trim");
    let _read = session.document();
    let again = session.trim(cut(made.sheet, left(), made.tool));
    assert_eq!(again.expect_err("the cap").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(trims(&doc), 1, "a second trim was added");
    one_face(sheet(&doc, piece), Area { mm2: 80.0 }, "the piece");
}

/// A missing sheet, a missing tool, an unread body, and the box asked as the sheet write no node.
#[test]
fn a_missing_sheet_or_tool_writes_no_trim() {
    let mut session = Session::empty();
    let made = crossing(&mut session);
    let missing_sheet = session.trim(cut(BodyId { id: 0 }, left(), made.tool));
    assert_eq!(missing_sheet.expect_err("id 0").text, "error-source-body-not-built");
    let missing_tool = session.trim(cut(made.sheet, left(), BodyId { id: 0 }));
    assert_eq!(missing_tool.expect_err("id 0").text, "error-source-body-not-built");
    let sketch = session.trim(cut(BodyId { id: made.drawn.sketch.id }, left(), made.tool));
    assert_eq!(sketch.expect_err("a sketch").text, "error-source-body-deleted");
    let unread_sheet = session.copy_face(CopyFace { body: made.block, face: made.cap }).expect("a copy");
    let unread = session.trim(cut(unread_sheet, left(), made.tool));
    assert_eq!(unread.expect_err("the copy").text, "error-source-body-not-built");
    let raw = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (40.0, 40.0) }, to: To { at: (50.0, 50.0) } }).expect("a square");
    let unread_tool = session.extrude(raw, Height { mm: 10.0 }).expect("a tool");
    let no_tool = session.trim(cut(made.sheet, left(), unread_tool));
    assert_eq!(no_tool.expect_err("the tool").text, "error-source-body-not-built");
    let solid = session.trim(cut(made.block, left(), made.tool));
    assert_eq!(solid.expect_err("the box").text, "error-trim-sheet");
    let held = session.document();
    assert_eq!(trims(&held), 0, "a refused call wrote a trim");
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
    let piece = session.trim(cut(made.sheet, left(), made.tool)).expect("a trim");
    let doc = session.document();
    one_face(sheet(&doc, piece), Area { mm2: 80.0 }, "the piece");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// The cap trimmed by the box it was copied from stays the whole cap.
#[test]
fn a_cap_trimmed_by_its_own_box_stays_the_cap() {
    let mut session = Session::empty();
    let made = crossing(&mut session);
    let failed = session.trim(cut(made.sheet, left(), made.block)).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "operation failed: Trim", "the refusal was {}", refusal(&doc));
    one_face(sheet(&doc, failed), Area { mm2: 200.0 }, "the passed sheet");
    near(solid(&doc, made.block), Volume { mm3: 6000.0 }, "the box");
    let copied = session.copy_face(CopyFace { body: made.block, face: made.cap });
    assert!(copied.is_ok(), "the box was taken");
}

/// A fillet the rebuild has refused still holds its extrusion.
#[test]
fn a_refused_fillet_holds_the_extrusion() {
    let mut session = Session::empty();
    let tool = session.cuboid(Cuboid { length: BoxLength { mm: 2.0 }, width: Width { mm: 2.0 }, height: Height { mm: 2.0 } }).expect("a tool");
    let _tool = session.document();
    let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = doc.bodies.iter().find(|body| body.id == block.id).expect("the extrusion").edges.iter().find(|item| (item.length.mm - 20.0).abs() < 0.1).expect("the 20 mm edge").id;
    let cap = doc.bodies.iter().find(|body| body.id == block.id).expect("the extrusion").faces.iter().find(|face| face.role == FaceRole::CapEnd).expect("the cap").id;
    let sheet = session.copy_face(CopyFace { body: block, face: cap }).expect("a copy");
    let _sheet = session.document();
    let _rounded = session.fillet(block, edge, Radius { mm: 100.0 }).expect("the node is written");
    let _refused = session.document();
    let failed = session.trim(cut(sheet, left(), tool));
    assert_eq!(failed.expect_err("the sheet").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(trims(&doc), 0, "a trim was written");
    assert_eq!(refusal(&doc), "fillet R100.00 failed on 1 edges", "the refusal was {}", refusal(&doc));
    near(solid(&doc, block), Volume { mm3: 6000.0 }, "the extrusion");
}

/// An unread trim of the cap stays a sheet beside the union.
#[test]
fn an_unread_trim_leaves_the_piece_beside_the_box() {
    let mut session = Session::empty();
    let made = crossing(&mut session);
    let piece = session.trim(cut(made.sheet, left(), made.tool)).expect("a trim");
    let joined = session.cuboid(Cuboid { length: BoxLength { mm: 2.0 }, width: Width { mm: 2.0 }, height: Height { mm: 2.0 } }).expect("a box");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.nodes.iter().find(|node| node.id == joined.id).expect("the union").name, "feat-name-body-union");
    near(solid(&doc, joined), Volume { mm3: 6006.0 }, "the union");
    one_face(sheet(&doc, piece), Area { mm2: 80.0 }, "the piece");
    near(solid(&doc, made.block), Volume { mm3: 6000.0 }, "the box");
    near(solid(&doc, made.tool), Volume { mm3: 7040.0 }, "the tool");
    let wall = doc.bodies.iter().find(|body| body.id == made.tool.id).expect("the tool").faces.iter().find(|face| (face.area.mm2 - 320.0).abs() < 1.0).expect("a wall").id;
    let kept = doc.bodies.iter().find(|body| body.id == piece.id).expect("the piece").faces[0].id;
    let held = session.copy_face(CopyFace { body: made.block, face: made.cap });
    assert_eq!(held.expect_err("the box").text, "msg-body-consumed");
    let copied = session.copy_face(CopyFace { body: made.tool, face: wall });
    assert!(copied.is_ok(), "the tool was taken");
    let sheet = session.copy_face(CopyFace { body: piece, face: kept });
    assert!(sheet.is_ok(), "the trim was taken");
}

/// The trimmed sheet follows the rectangle. A 10 by 10 mm cap cut at x = 4 leaves 40 mm^2.
#[test]
fn the_trimmed_sheet_follows_the_rectangle() {
    let mut session = Session::empty();
    let made = crossing(&mut session);
    let piece = session.trim(cut(made.sheet, left(), made.tool)).expect("a trim");
    let _read = session.document();
    session.resize(made.drawn, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 10.0) } }).expect("a smaller rectangle");
    let smaller = session.document();
    assert_eq!(smaller.rebuild, Rebuild::Quiet, "{}", refusal(&smaller));
    near(solid(&smaller, made.block), Volume { mm3: 3000.0 }, "the box");
    one_face(sheet(&smaller, piece), Area { mm2: 40.0 }, "the piece");
    near(solid(&smaller, made.tool), Volume { mm3: 7040.0 }, "the tool");
}

struct Opened {
    tool: BodyId,
    drawn: qymcad_drive::Profile,
    block: BodyId,
    cap: FaceId,
    sheet: BodyId,
}

fn crossing(session: &mut Session) -> Opened {
    let tool_drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (4.0, -1.0) }, to: To { at: (12.0, 21.0) } }).expect("the tool");
    let tool = session.extrude(tool_drawn, Height { mm: 40.0 }).expect("the tool body");
    let _tool = session.document();
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("the box");
    let block = session.extrude(drawn, Height { mm: 30.0 }).expect("the box body");
    let doc = session.document();
    let cap = body(&doc, block).faces.iter().find(|face| face.role == FaceRole::CapEnd).expect("the cap").id;
    let sheet = session.copy_face(CopyFace { body: block, face: cap }).expect("a copy");
    let _sheet = session.document();
    Opened { tool, drawn, block, cap, sheet }
}

fn opened(session: &Session, name: &str) -> qymcad_drive::Document {
    let path = std::env::temp_dir().join(format!("qymcad-drive-trim-{name}-{}.qcad", std::process::id()));
    session.save(&path).expect("save");
    let mut opened = Session::open(&path).expect("open");
    let doc = opened.document();
    let _gone = std::fs::remove_file(&path);
    doc
}

fn far_tool(session: &mut Session) -> BodyId {
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (100.0, 100.0) }, to: To { at: (110.0, 110.0) } }).expect("a square");
    let tool = session.extrude(drawn, Height { mm: 10.0 }).expect("a tool");
    let _read = session.document();
    tool
}

fn point(x: KeepX, y: KeepY, z: KeepZ) -> Keep {
    Keep { x, y, z }
}

fn left() -> Keep {
    point(KeepX { mm: 2.0 }, KeepY { mm: 10.0 }, KeepZ { mm: 30.0 })
}

fn right() -> Keep {
    point(KeepX { mm: 7.0 }, KeepY { mm: 10.0 }, KeepZ { mm: 30.0 })
}

fn origin() -> Keep {
    point(KeepX { mm: 0.0 }, KeepY { mm: 0.0 }, KeepZ { mm: 0.0 })
}

fn cut(sheet: BodyId, keep: Keep, tool: BodyId) -> Trim {
    Trim { sheet, keep, tool }
}

fn trims(doc: &qymcad_drive::Document) -> usize {
    doc.nodes.iter().filter(|node| node.name == "feat-name-trim").count()
}

fn body(doc: &qymcad_drive::Document, id: BodyId) -> &qymcad_drive::Body {
    doc.bodies.iter().find(|body| body.id == id.id).unwrap_or_else(|| panic!("body {} is gone", id.id))
}

fn solid(doc: &qymcad_drive::Document, id: BodyId) -> f64 {
    let found = body(doc, id);
    assert_eq!(found.sound, Sound::Valid, "the shape is valid");
    match found.measure {
        qymcad_drive::Measure::Live { volume, solids } => {
            assert_eq!(solids, 1, "one solid");
            volume.mm3
        }
        qymcad_drive::Measure::Absent => panic!("body {} has no live shape", id.id),
    }
}

fn sheet(doc: &qymcad_drive::Document, id: BodyId) -> Vec<f64> {
    let found = body(doc, id);
    assert_eq!(found.sound, Sound::Valid, "the shape is valid");
    match found.measure {
        qymcad_drive::Measure::Live { volume, solids } => {
            assert_eq!(solids, 0, "a sheet has no solid");
            assert!(volume.mm3.abs() < 1.0, "a sheet holds {} mm^3", volume.mm3);
        }
        qymcad_drive::Measure::Absent => panic!("body {} has no live shape", id.id),
    }
    assert!(found.faces.iter().all(|face| face.role == FaceRole::Other), "a face kept a solid role");
    found.faces.iter().map(|face| face.area.mm2).collect()
}

fn one_face(got: Vec<f64>, want: Area, what: &str) {
    assert_eq!(got.len(), 1, "{what}: {} faces", got.len());
    assert!((got[0] - want.mm2).abs() < 1.0, "{what}: area {} mm^2, wanted {} within 1", got[0], want.mm2);
}

fn near(got: f64, want: Volume, what: &str) {
    assert!((got - want.mm3).abs() < 1.0, "{what}: volume {got} mm^3, wanted {} within 1", want.mm3);
}

fn refusal(doc: &qymcad_drive::Document) -> String {
    match &doc.rebuild {
        Rebuild::Refused { refusals } => refusals.iter().map(|refusal| refusal.text.clone()).collect::<Vec<_>>().join(" "),
        Rebuild::Quiet => "quiet".into(),
        Rebuild::Stopped => "stopped".into(),
    }
}
