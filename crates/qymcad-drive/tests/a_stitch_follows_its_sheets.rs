use qymcad_drive::{Area, BodyId, BoxLength, CopyFace, Corners, Cuboid, FaceId, FaceRole, From, Gap, Height, Kind, Radius, Rebuild, Session, Sound, Stitch, To, Volume, Width, WorldPlane};

/// The end cap and the 300 mm^2 wall, sewn at 0.0001 mm, are one sheet of those two areas.
#[test]
fn the_cap_and_the_short_wall() {
    let mut session = Session::empty();
    let made = box_of(&mut session);
    let doc = session.document();
    let cap = copied(&mut session, made.body, end_cap(&doc, made.body));
    let wall = copied(&mut session, made.body, wall_of(&doc, made.body, Area { mm2: 300.0 }));
    let sewn = session.stitch(pair(vec![cap, wall], Gap { mm: 0.0001 })).expect("a stitch");
    let doc = session.document();
    areas(sheet(&doc, sewn), &[200.0, 300.0], "the sheet");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.components.iter().filter(|item| item.kind == Kind::Part).count(), 1, "one part");
    assert_eq!(stitches(&doc), 1, "the stitch node");
    assert_ne!(sewn, made.body, "the sheet is the box");
    near(solid(&doc, made.body), Volume { mm3: 6000.0 }, "the box");
}

/// A gap of 0 mm sews the same two faces.
#[test]
fn a_gap_of_zero_sews_the_same_sheet() {
    let mut session = Session::empty();
    let made = box_of(&mut session);
    let doc = session.document();
    let cap = copied(&mut session, made.body, end_cap(&doc, made.body));
    let wall = copied(&mut session, made.body, wall_of(&doc, made.body, Area { mm2: 300.0 }));
    let sewn = session.stitch(pair(vec![cap, wall], Gap { mm: 0.0 })).expect("a stitch");
    let doc = session.document();
    areas(sheet(&doc, sewn), &[200.0, 300.0], "the sheet");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// A gap of -1 mm sews the same two faces.
#[test]
fn a_negative_gap_sews_the_same_sheet() {
    let mut session = Session::empty();
    let made = box_of(&mut session);
    let doc = session.document();
    let cap = copied(&mut session, made.body, end_cap(&doc, made.body));
    let wall = copied(&mut session, made.body, wall_of(&doc, made.body, Area { mm2: 300.0 }));
    let sewn = session.stitch(pair(vec![cap, wall], Gap { mm: -1.0 })).expect("a stitch");
    let doc = session.document();
    areas(sheet(&doc, sewn), &[200.0, 300.0], "the sheet");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// The end cap and a 600 mm^2 wall are those two areas.
#[test]
fn the_cap_and_the_long_wall() {
    let mut session = Session::empty();
    let made = box_of(&mut session);
    let doc = session.document();
    let cap = copied(&mut session, made.body, end_cap(&doc, made.body));
    let wall = copied(&mut session, made.body, wall_of(&doc, made.body, Area { mm2: 600.0 }));
    let sewn = session.stitch(pair(vec![cap, wall], Gap { mm: 0.0001 })).expect("a stitch");
    let doc = session.document();
    areas(sheet(&doc, sewn), &[200.0, 600.0], "the sheet");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(solid(&doc, made.body), Volume { mm3: 6000.0 }, "the box");
}

/// The two end caps share no edge. The cap and the short wall replace that stitch.
#[test]
fn the_two_caps_are_replaced_by_the_wall() {
    let mut session = Session::empty();
    let made = box_of(&mut session);
    let doc = session.document();
    let end = copied(&mut session, made.body, end_cap(&doc, made.body));
    let start = copied(&mut session, made.body, start_cap(&doc, made.body));
    let failed = session.stitch(pair(vec![end, start], Gap { mm: 0.0001 })).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "surfaces share no edges — nothing to stitch", "the refusal was {}", refusal(&doc));
    areas(passed(&doc, failed), &[200.0], "the passed sheet");
    near(solid(&doc, made.body), Volume { mm3: 6000.0 }, "the box");
    let alone = session.stitch(Stitch { sheets: vec![end], gap: Gap { mm: 0.0001 } });
    assert_eq!(alone.expect_err("one sheet").text, "msg-stitch-needs-two");
    let twice = session.stitch(pair(vec![end, end], Gap { mm: 0.0001 }));
    assert_eq!(twice.expect_err("one sheet").text, "msg-stitch-needs-two");
    let gap = session.stitch(pair(vec![end, start], Gap { mm: f64::NAN }));
    assert_eq!(gap.expect_err("the gap").text, "error-stitch-gap");
    let held = session.document();
    assert_eq!(refusal(&held), "surfaces share no edges — nothing to stitch", "the refusal was {}", refusal(&held));
    assert_eq!(stitches(&held), 1, "the refused stitch was rewritten");
    let wall = copied(&mut session, made.body, wall_of(&doc, made.body, Area { mm2: 300.0 }));
    let sewn = session.stitch(pair(vec![end, wall], Gap { mm: 0.0001 })).expect("the wall is tried");
    assert_eq!(sewn, failed, "a second stitch was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(stitches(&doc), 1, "the refused stitch was left beside the new one");
    areas(sheet(&doc, sewn), &[200.0, 300.0], "the sheet");
}

/// The id a refused stitch returned is that same stitch.
#[test]
fn a_refused_stitch_named_by_its_own_id_is_the_same_stitch() {
    let mut session = Session::empty();
    let made = box_of(&mut session);
    let doc = session.document();
    let end = copied(&mut session, made.body, end_cap(&doc, made.body));
    let start = copied(&mut session, made.body, start_cap(&doc, made.body));
    let failed = session.stitch(pair(vec![end, start], Gap { mm: 0.0001 })).expect("the node is written");
    let _held = session.document();
    let named = session.stitch(Stitch { sheets: vec![failed, end], gap: Gap { mm: 0.0001 } });
    assert_eq!(named.expect_err("one sheet").text, "msg-stitch-needs-two");
    let held = session.document();
    assert_eq!(refusal(&held), "surfaces share no edges — nothing to stitch", "the refusal was {}", refusal(&held));
    assert_eq!(stitches(&held), 1, "the refused stitch was rewritten");
    areas(passed(&held, failed), &[200.0], "the passed sheet");
    let wall = copied(&mut session, made.body, wall_of(&doc, made.body, Area { mm2: 300.0 }));
    let sewn = session.stitch(Stitch { sheets: vec![failed, end, wall], gap: Gap { mm: 0.0001 } }).expect("the wall is tried");
    assert_eq!(sewn, failed, "a second stitch was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(stitches(&doc), 1, "the refused stitch was left beside the new one");
    areas(sheet(&doc, sewn), &[200.0, 300.0], "the sheet");
    near(solid(&doc, made.body), Volume { mm3: 6000.0 }, "the box");
}

/// The sheets already sewn are not sewn again.
#[test]
fn the_sewn_sheets_are_not_sewn_again() {
    let mut session = Session::empty();
    let made = box_of(&mut session);
    let doc = session.document();
    let cap = copied(&mut session, made.body, end_cap(&doc, made.body));
    let wall = copied(&mut session, made.body, wall_of(&doc, made.body, Area { mm2: 300.0 }));
    let sewn = session.stitch(pair(vec![cap, wall], Gap { mm: 0.0001 })).expect("a stitch");
    let _read = session.document();
    let again = session.stitch(pair(vec![cap, wall], Gap { mm: 0.0001 }));
    assert_eq!(again.expect_err("the sheets").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(stitches(&doc), 1, "a second stitch was added");
    areas(sheet(&doc, sewn), &[200.0, 300.0], "the sheet");
}

/// Fewer than two sheets, a solid, an id of 0, and a sketch id write no node.
#[test]
fn a_short_list_a_solid_and_a_missing_id_write_no_stitch() {
    let mut session = Session::empty();
    let made = box_of(&mut session);
    let doc = session.document();
    let cap = copied(&mut session, made.body, end_cap(&doc, made.body));
    let wall = copied(&mut session, made.body, wall_of(&doc, made.body, Area { mm2: 300.0 }));
    let alone = session.stitch(Stitch { sheets: vec![cap], gap: Gap { mm: 0.0001 } });
    assert_eq!(alone.expect_err("one sheet").text, "msg-stitch-needs-two");
    let twice = session.stitch(pair(vec![cap, cap], Gap { mm: 0.0001 }));
    assert_eq!(twice.expect_err("one sheet").text, "msg-stitch-needs-two");
    let solid = session.stitch(pair(vec![made.body, made.body], Gap { mm: 0.0001 }));
    assert_eq!(solid.expect_err("the box").text, "msg-stitch-only-sheets");
    let missing = session.stitch(pair(vec![BodyId { id: 0 }, cap], Gap { mm: 0.0001 }));
    assert_eq!(missing.expect_err("id 0").text, "error-source-body-not-built");
    let sketch = session.stitch(pair(vec![BodyId { id: made.drawn.sketch.id }, cap], Gap { mm: 0.0001 }));
    assert_eq!(sketch.expect_err("a sketch").text, "error-source-body-deleted");
    let held = session.document();
    assert_eq!(stitches(&held), 0, "a refused call wrote a stitch");
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
    let sewn = session.stitch(pair(vec![cap, wall], Gap { mm: 0.0001 })).expect("a stitch");
    let doc = session.document();
    areas(sheet(&doc, sewn), &[200.0, 300.0], "the sheet");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// A fillet the rebuild has refused still holds its extrusion.
#[test]
fn a_refused_fillet_holds_the_extrusion() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = doc.bodies.iter().find(|body| body.id == block.id).expect("the extrusion").edges.iter().find(|item| (item.length.mm - 20.0).abs() < 0.1).expect("the 20 mm edge").id;
    let cap = copied(&mut session, block, end_cap(&doc, block));
    let wall = copied(&mut session, block, wall_of(&doc, block, Area { mm2: 300.0 }));
    let _rounded = session.fillet(block, edge, Radius { mm: 100.0 }).expect("the node is written");
    let _refused = session.document();
    let failed = session.stitch(pair(vec![cap, wall], Gap { mm: 0.0001 }));
    assert_eq!(failed.expect_err("the sheets").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(stitches(&doc), 0, "a stitch was written");
    assert_eq!(refusal(&doc), "fillet R100.00 failed on 1 edges", "the refusal was {}", refusal(&doc));
    near(solid(&doc, block), Volume { mm3: 6000.0 }, "the extrusion");
}

/// An unread stitch of the cap and the short wall stays a sheet beside the union.
#[test]
fn an_unread_stitch_leaves_the_sheet_beside_the_box() {
    let mut session = Session::empty();
    let made = box_of(&mut session);
    let doc = session.document();
    let cap = copied(&mut session, made.body, end_cap(&doc, made.body));
    let wall = copied(&mut session, made.body, wall_of(&doc, made.body, Area { mm2: 300.0 }));
    let sewn = session.stitch(pair(vec![cap, wall], Gap { mm: 0.0001 })).expect("a stitch");
    let joined = session.cuboid(Cuboid { length: BoxLength { mm: 2.0 }, width: Width { mm: 2.0 }, height: Height { mm: 2.0 } }).expect("a box");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.nodes.iter().find(|node| node.id == joined.id).expect("the union").name, "feat-name-body-union");
    near(solid(&doc, joined), Volume { mm3: 6006.0 }, "the union");
    areas(sheet(&doc, sewn), &[200.0, 300.0], "the sheet");
    near(solid(&doc, made.body), Volume { mm3: 6000.0 }, "the box");
}

/// The sewn sheet follows the rectangle. A 10 by 10 mm cap is 100 mm^2 beside the 300 mm^2 wall.
#[test]
fn the_sewn_sheet_follows_the_rectangle() {
    let mut session = Session::empty();
    let made = box_of(&mut session);
    let doc = session.document();
    let cap = copied(&mut session, made.body, end_cap(&doc, made.body));
    let wall = copied(&mut session, made.body, wall_of(&doc, made.body, Area { mm2: 300.0 }));
    let sewn = session.stitch(pair(vec![cap, wall], Gap { mm: 0.0001 })).expect("a stitch");
    let _read = session.document();
    session.resize(made.drawn, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 10.0) } }).expect("a smaller rectangle");
    let smaller = session.document();
    assert_eq!(smaller.rebuild, Rebuild::Quiet, "{}", refusal(&smaller));
    near(solid(&smaller, made.body), Volume { mm3: 3000.0 }, "the box");
    areas(sheet(&smaller, sewn), &[100.0, 300.0], "the sheet");
}

/// The six faces of the box, sewn, are the box again.
#[test]
fn the_six_faces_close_into_the_box() {
    let mut session = Session::empty();
    let made = box_of(&mut session);
    let faces: Vec<FaceId> = session.document().bodies.iter().find(|body| body.id == made.body.id).expect("the box").faces.iter().map(|face| face.id).collect();
    assert_eq!(faces.len(), 6, "the box has six faces");
    let sheets: Vec<BodyId> = faces.iter().map(|face| copied(&mut session, made.body, *face)).collect();
    let sewn = session.stitch(Stitch { sheets, gap: Gap { mm: 0.0001 } }).expect("a stitch");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(solid(&doc, sewn), Volume { mm3: 6000.0 }, "the solid");
    let faces = body(&doc, sewn).faces.iter().map(|face| face.area.mm2).collect();
    areas(faces, &[300.0, 600.0, 300.0, 600.0, 200.0, 200.0], "the solid");
    assert!(body(&doc, sewn).faces.iter().all(|face| face.role == FaceRole::Other), "a face kept a solid role");
    near(solid(&doc, made.body), Volume { mm3: 6000.0 }, "the box");
}

/// An unread stitch of the six faces is the solid the cube joins.
#[test]
fn an_unread_closed_stitch_joins_the_cube() {
    let mut session = Session::empty();
    let made = box_of(&mut session);
    let faces: Vec<FaceId> = session.document().bodies.iter().find(|body| body.id == made.body.id).expect("the box").faces.iter().map(|face| face.id).collect();
    let sheets: Vec<BodyId> = faces.iter().map(|face| copied(&mut session, made.body, *face)).collect();
    let sewn = session.stitch(Stitch { sheets, gap: Gap { mm: 0.0001 } }).expect("a stitch");
    let joined = session.cuboid(Cuboid { length: BoxLength { mm: 2.0 }, width: Width { mm: 2.0 }, height: Height { mm: 2.0 } }).expect("a box");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.nodes.iter().find(|node| node.id == joined.id).expect("the union").name, "feat-name-body-union");
    near(solid(&doc, joined), Volume { mm3: 6006.0 }, "the union");
    near(solid(&doc, sewn), Volume { mm3: 6000.0 }, "the stitched solid");
    near(solid(&doc, made.body), Volume { mm3: 6000.0 }, "the box");
    let face = doc.bodies.iter().find(|body| body.id == sewn.id).expect("the stitch").faces[0].id;
    let held = session.copy_face(CopyFace { body: sewn, face });
    assert_eq!(held.expect_err("the stitch").text, "msg-body-consumed");
    let cap = end_cap(&session.document(), made.body);
    let copied_cap = session.copy_face(CopyFace { body: made.body, face: cap });
    assert!(copied_cap.is_ok(), "the box was taken");
}

struct Opened {
    drawn: qymcad_drive::Profile,
    body: BodyId,
}

fn box_of(session: &mut Session) -> Opened {
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    let body = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    Opened { drawn, body }
}

fn end_cap(doc: &qymcad_drive::Document, id: BodyId) -> FaceId {
    body(doc, id).faces.iter().find(|face| face.role == FaceRole::CapEnd).expect("the end cap").id
}

fn start_cap(doc: &qymcad_drive::Document, id: BodyId) -> FaceId {
    body(doc, id).faces.iter().find(|face| face.role == FaceRole::CapStart).expect("the start cap").id
}

fn wall_of(doc: &qymcad_drive::Document, id: BodyId, area: Area) -> FaceId {
    body(doc, id).faces.iter().find(|face| (face.area.mm2 - area.mm2).abs() < 1.0).expect("the wall").id
}

fn copied(session: &mut Session, block: BodyId, face: FaceId) -> BodyId {
    let made = session.copy_face(CopyFace { body: block, face }).expect("a copy");
    let _read = session.document();
    made
}

fn pair(sheets: Vec<BodyId>, gap: Gap) -> Stitch {
    Stitch { sheets, gap }
}

fn stitches(doc: &qymcad_drive::Document) -> usize {
    doc.nodes.iter().filter(|node| node.name == "feat-name-stitch").count()
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

fn passed(doc: &qymcad_drive::Document, id: BodyId) -> Vec<f64> {
    let found = sheet(doc, id);
    assert_eq!(found.len(), 1, "the passed sheet has one face");
    found
}

fn areas(got: Vec<f64>, want: &[f64], what: &str) {
    assert_eq!(got.len(), want.len(), "{what}: {} faces, wanted {}", got.len(), want.len());
    let mut left = got;
    for area in want {
        let place = left.iter().position(|got| (got - area).abs() < 1.0);
        assert!(place.is_some(), "{what}: {left:?} has no face within 1 mm^2 of {area}");
        left.swap_remove(place.expect("the face"));
    }
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
