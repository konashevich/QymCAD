use qymcad_drive::{Area, BodyId, BoxLength, CopyFace, Corners, Cuboid, FaceId, FaceRole, From, Height, Kind, Move, Place, Profile, Radius, Rebuild, Session, Sound, To, Volume, Width, WorldPlane};

/// The 10 by 20 by 30 mm box, shifted 100 mm along X, stands clear of a 2 mm cube.
#[test]
fn the_box_moves_clear_of_the_cube() {
    let mut session = Session::empty();
    let block = stock(&mut session);
    let cap = cap_of(&session.document(), block);
    let moved = session.move_body(placed(block, shift_x(100.0))).expect("a move");
    let doc = session.document();
    near(solid(&doc, moved), Volume { mm3: 6000.0 }, "the moved box");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.components.iter().filter(|item| item.kind == Kind::Part).count(), 1, "one part");
    assert_eq!(moves(&doc), 1, "the move node");
    assert_eq!(doc.nodes.iter().find(|node| node.id == moved.id).expect("the node").name, "feat-name-move");
    assert_ne!(moved, block, "the move is the box");
    near(solid(&doc, block), Volume { mm3: 6000.0 }, "the box");
    let result = cap_of(&doc, moved);
    held(&mut session, block, cap);
    let joined = cube(&mut session);
    let doc = session.document();
    assert_eq!(doc.nodes.iter().find(|node| node.id == joined.id).expect("the union").name, "feat-name-body-union");
    near(pieces(&doc, joined), Volume { mm3: 6008.0 }, "the union");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    held(&mut session, moved, result);
}

/// The same box, moved by nothing, still meets the 2 mm cube.
#[test]
fn an_identity_move_still_meets_the_cube() {
    let mut session = Session::empty();
    let block = stock(&mut session);
    let moved = session.move_body(placed(block, shift_x(0.0))).expect("a move");
    let _read = session.document();
    let joined = cube(&mut session);
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(solid(&doc, moved), Volume { mm3: 6000.0 }, "the moved box");
    near(solid(&doc, joined), Volume { mm3: 6006.000000000001 }, "the union");
}

/// A shift that has not been read is the solid the cube joins.
#[test]
fn an_unread_move_joins_the_cube() {
    let mut session = Session::empty();
    let block = stock(&mut session);
    let moved = session.move_body(placed(block, shift_x(100.0))).expect("a move");
    let joined = cube(&mut session);
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.nodes.iter().find(|node| node.id == joined.id).expect("the union").name, "feat-name-body-union");
    near(pieces(&doc, joined), Volume { mm3: 6008.0 }, "the union");
    near(solid(&doc, moved), Volume { mm3: 6000.0 }, "the move");
    near(solid(&doc, block), Volume { mm3: 6000.0 }, "the box");
}

/// A second shift of the result is a new node. The first move is consumed. The second is not.
#[test]
fn a_second_move_of_the_result_is_a_new_node() {
    let mut session = Session::empty();
    let block = stock(&mut session);
    let first = session.move_body(placed(block, shift_x(100.0))).expect("a move");
    let doc = session.document();
    let first_cap = cap_of(&doc, first);
    let second = session.move_body(placed(first, shift_x(100.0))).expect("a second move");
    assert_ne!(second, first, "the second move rewrote the first");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(moves(&doc), 2, "the second move was not a node");
    let second_cap = cap_of(&doc, second);
    held(&mut session, first, first_cap);
    free(&mut session, second, second_cap);
}

/// The box already moved is not moved again.
#[test]
fn the_moved_box_is_not_moved_again() {
    let mut session = Session::empty();
    let block = stock(&mut session);
    let moved = session.move_body(placed(block, shift_x(100.0))).expect("a move");
    let _read = session.document();
    let again = session.move_body(placed(block, shift_x(100.0)));
    assert_eq!(again.expect_err("the box").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(moves(&doc), 1, "a second move was added");
    near(solid(&doc, moved), Volume { mm3: 6000.0 }, "the moved box");
    near(solid(&doc, block), Volume { mm3: 6000.0 }, "the box");
}

/// A copy of the end cap, shifted 100 mm along X, stays a sheet of 200 mm^2.
#[test]
fn a_moved_cap_stays_a_sheet() {
    let mut session = Session::empty();
    let block = stock(&mut session);
    let doc = session.document();
    let cap = cap_of(&doc, block);
    let sheet = session.copy_face(CopyFace { body: block, face: cap }).expect("a copy");
    let _sheet = session.document();
    let moved = session.move_body(placed(sheet, shift_x(100.0))).expect("a move");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near_area(sheet_area(&doc, moved), Area { mm2: 200.0 }, "the cap");
    free(&mut session, block, cap);
    let again = session.move_body(placed(sheet, shift_x(100.0)));
    assert_eq!(again.expect_err("the sheet").text, "msg-body-consumed");
    let face = body(&doc, moved).faces[0].id;
    free(&mut session, moved, face);
}

/// A matrix of all zeros is a node, and the 100 mm shift writes that same node again.
#[test]
fn a_zero_placement_is_rewritten_by_the_shift() {
    let mut session = Session::empty();
    let block = stock(&mut session);
    let failed = session.move_body(placed(block, nowhere())).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "operation failed: Move", "the refusal was {}", refusal(&doc));
    assert_eq!(moves(&doc), 1, "the refused move was not one node");
    near(solid(&doc, failed), Volume { mm3: 6000.0 }, "the move");
    near(solid(&doc, block), Volume { mm3: 6000.0 }, "the box");
    let fixed = session.move_body(placed(block, shift_x(100.0))).expect("the shift");
    assert_eq!(fixed, failed, "a second move was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(moves(&doc), 1, "the refused move was left beside the new one");
    near(solid(&doc, fixed), Volume { mm3: 6000.0 }, "the moved box");
    let joined = cube(&mut session);
    let doc = session.document();
    near(pieces(&doc, joined), Volume { mm3: 6008.0 }, "the union");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// The id a refused move returned is that same move.
#[test]
fn a_refused_move_named_by_its_own_id_is_the_same_move() {
    let mut session = Session::empty();
    let block = stock(&mut session);
    let failed = session.move_body(placed(block, nowhere())).expect("the node is written");
    let _held = session.document();
    let fixed = session.move_body(placed(failed, shift_x(100.0))).expect("the shift");
    assert_eq!(fixed, failed, "a second move was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(moves(&doc), 1, "the refused move was left beside the new one");
    near(solid(&doc, fixed), Volume { mm3: 6000.0 }, "the moved box");
    let joined = cube(&mut session);
    let doc = session.document();
    near(pieces(&doc, joined), Volume { mm3: 6008.0 }, "the union");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// A placement that is not finite does not rewrite a refused move, and it writes no node of its own.
#[test]
fn a_bad_placement_writes_no_move() {
    let mut session = Session::empty();
    let block = stock(&mut session);
    let bad = session.move_body(placed(block, unfixed()));
    assert_eq!(bad.expect_err("the placement").text, "error-move-place");
    let quiet = session.document();
    assert_eq!(moves(&quiet), 0, "a refused call wrote a move");
    assert_eq!(quiet.rebuild, Rebuild::Quiet, "{}", refusal(&quiet));
    let failed = session.move_body(placed(block, nowhere())).expect("the node is written");
    let _held = session.document();
    let again = session.move_body(placed(block, unfixed()));
    assert_eq!(again.expect_err("the placement").text, "error-move-place");
    let by_id = session.move_body(placed(failed, unfixed()));
    assert_eq!(by_id.expect_err("the placement").text, "error-move-place");
    let held = session.document();
    assert_eq!(refusal(&held), "operation failed: Move", "the refusal was {}", refusal(&held));
    assert_eq!(moves(&held), 1, "the refused move was rewritten");
    near(solid(&held, failed), Volume { mm3: 6000.0 }, "the box");
    let moved = session.move_body(placed(block, shift_x(100.0))).expect("a move");
    assert_eq!(moved, failed, "a second move was added");
    let _read = session.document();
    let joined = cube(&mut session);
    let doc = session.document();
    near(pieces(&doc, joined), Volume { mm3: 6008.0 }, "the union");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// A missing body, a sketch, and an unread body write no node.
#[test]
fn a_missing_body_writes_no_move() {
    let mut session = Session::empty();
    let made = box_sketch(&mut session);
    let missing = session.move_body(placed(BodyId { id: 0 }, shift_x(100.0)));
    assert_eq!(missing.expect_err("id 0").text, "error-source-body-not-built");
    let sketch = session.move_body(placed(BodyId { id: made.drawn.sketch.id }, shift_x(100.0)));
    assert_eq!(sketch.expect_err("a sketch").text, "error-source-body-deleted");
    let raw = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (40.0, 40.0) }, to: To { at: (50.0, 50.0) } }).expect("a square");
    let unread = session.extrude(raw, Height { mm: 10.0 }).expect("a body");
    let no_body = session.move_body(placed(unread, shift_x(100.0)));
    assert_eq!(no_body.expect_err("the body").text, "error-source-body-not-built");
    let held = session.document();
    assert_eq!(moves(&held), 0, "a refused call wrote a move");
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
    let moved = session.move_body(placed(made.block, shift_x(100.0))).expect("a move");
    let _read = session.document();
    let joined = cube(&mut session);
    let doc = session.document();
    near(solid(&doc, moved), Volume { mm3: 6000.0 }, "the moved box");
    near(pieces(&doc, joined), Volume { mm3: 6008.0 }, "the union");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// A fillet the rebuild has refused still holds its extrusion.
#[test]
fn a_refused_fillet_holds_the_extrusion() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = body(&doc, block).edges.iter().find(|item| (item.length.mm - 20.0).abs() < 0.1).expect("the 20 mm edge").id;
    let _rounded = session.fillet(block, edge, Radius { mm: 100.0 }).expect("the node is written");
    let _refused = session.document();
    let failed = session.move_body(placed(block, shift_x(100.0)));
    assert_eq!(failed.expect_err("the extrusion").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(moves(&doc), 0, "a move was written");
    assert_eq!(refusal(&doc), "fillet R100.00 failed on 1 edges", "the refusal was {}", refusal(&doc));
    near(solid(&doc, block), Volume { mm3: 6000.0 }, "the extrusion");
}

/// The moved box follows the rectangle. A 10 by 10 mm cap shifted 100 mm is 3000 mm^3.
#[test]
fn the_moved_box_follows_the_rectangle() {
    let mut session = Session::empty();
    let made = box_sketch(&mut session);
    let moved = session.move_body(placed(made.block, shift_x(100.0))).expect("a move");
    let _read = session.document();
    session.resize(made.drawn, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 10.0) } }).expect("a smaller rectangle");
    let smaller = session.document();
    assert_eq!(smaller.rebuild, Rebuild::Quiet, "{}", refusal(&smaller));
    near(solid(&smaller, moved), Volume { mm3: 3000.0 }, "the moved box");
    near(solid(&smaller, made.block), Volume { mm3: 3000.0 }, "the box");
    let joined = cube(&mut session);
    let doc = session.document();
    near(pieces(&doc, joined), Volume { mm3: 3008.0 }, "the union");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

struct BoxSketch {
    drawn: Profile,
    block: BodyId,
}

fn box_sketch(session: &mut Session) -> BoxSketch {
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("the box");
    let block = session.extrude(drawn, Height { mm: 30.0 }).expect("the box body");
    let _read = session.document();
    BoxSketch { drawn, block }
}

fn stock(session: &mut Session) -> BodyId {
    box_sketch(session).block
}

fn shift_x(mm: f64) -> Place {
    Place { mat: [1.0, 0.0, 0.0, mm, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0] }
}

fn nowhere() -> Place {
    Place { mat: [0.0; 12] }
}

fn unfixed() -> Place {
    let mut by = shift_x(100.0);
    by.mat[3] = f64::NAN;
    by
}

fn placed(body: BodyId, by: Place) -> Move {
    Move { body, by }
}

fn cube(session: &mut Session) -> BodyId {
    session.cuboid(Cuboid { length: BoxLength { mm: 2.0 }, width: Width { mm: 2.0 }, height: Height { mm: 2.0 } }).expect("a cube")
}

fn cap_of(doc: &qymcad_drive::Document, id: BodyId) -> FaceId {
    body(doc, id).faces.iter().find(|face| face.role == FaceRole::CapEnd).expect("the cap").id
}

fn moves(doc: &qymcad_drive::Document) -> usize {
    doc.nodes.iter().filter(|node| node.name == "feat-name-move").count()
}

fn body(doc: &qymcad_drive::Document, id: BodyId) -> &qymcad_drive::Body {
    doc.bodies.iter().find(|body| body.id == id.id).unwrap_or_else(|| panic!("body {} is gone", id.id))
}

struct Read {
    volume: f64,
    solids: u32,
}

fn read(doc: &qymcad_drive::Document, id: BodyId) -> Read {
    let found = body(doc, id);
    assert_eq!(found.sound, Sound::Valid, "the shape is valid");
    match found.measure {
        qymcad_drive::Measure::Live { volume, solids } => Read { volume: volume.mm3, solids },
        qymcad_drive::Measure::Absent => panic!("body {} has no live shape", id.id),
    }
}

fn solid(doc: &qymcad_drive::Document, id: BodyId) -> f64 {
    let found = read(doc, id);
    assert_eq!(found.solids, 1, "one solid");
    found.volume
}

fn pieces(doc: &qymcad_drive::Document, id: BodyId) -> f64 {
    let found = read(doc, id);
    assert_eq!(found.solids, 2, "two solids");
    found.volume
}

fn sheet_area(doc: &qymcad_drive::Document, id: BodyId) -> f64 {
    let found = body(doc, id);
    assert_eq!(found.sound, Sound::Valid, "the shape is valid");
    match found.measure {
        qymcad_drive::Measure::Live { volume, solids } => {
            assert_eq!(solids, 0, "a sheet has no solid");
            assert!(volume.mm3.abs() < 1.0, "a sheet holds {} mm^3", volume.mm3);
        }
        qymcad_drive::Measure::Absent => panic!("body {} has no live shape", id.id),
    }
    assert_eq!(found.faces.len(), 1, "the sheet has {} faces", found.faces.len());
    found.faces.iter().map(|face| face.area.mm2).sum()
}

fn near(got: f64, want: Volume, what: &str) {
    assert!((got - want.mm3).abs() < 1.0, "{what}: volume {got} mm^3, wanted {} within 1", want.mm3);
}

fn near_area(got: f64, want: Area, what: &str) {
    assert!((got - want.mm2).abs() < 1.0, "{what}: area {got} mm^2, wanted {} within 1", want.mm2);
}

fn free(session: &mut Session, body: BodyId, face: FaceId) {
    session.copy_face(CopyFace { body, face }).expect("the body was taken");
}

fn held(session: &mut Session, body: BodyId, face: FaceId) {
    let copied = session.copy_face(CopyFace { body, face });
    assert_eq!(copied.expect_err("the body").text, "msg-body-consumed");
}

fn refusal(doc: &qymcad_drive::Document) -> String {
    match &doc.rebuild {
        Rebuild::Refused { refusals } => refusals.iter().map(|refusal| refusal.text.clone()).collect::<Vec<_>>().join(" "),
        Rebuild::Quiet => "quiet".into(),
        Rebuild::Stopped => "stopped".into(),
    }
}
