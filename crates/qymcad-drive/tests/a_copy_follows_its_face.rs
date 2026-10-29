use qymcad_drive::{Angle, Area, BodyId, BoxLength, CopyFace, Corners, Cuboid, FaceId, FaceRole, From, Height, Kind, Radius, Rebuild, Session, SketchAxis, Sound, To, Volume, Width, WorldPlane};

/// The end cap of the 10 by 20 by 30 mm box leaves a sheet of 200 mm^2. The box stays.
#[test]
fn the_end_cap_is_copied() {
    let mut session = Session::empty();
    let made = box_of(&mut session);
    let doc = session.document();
    let cap = end_cap(&doc, made.body);
    let copied = session.copy_face(CopyFace { body: made.body, face: cap }).expect("a copy");
    let doc = session.document();
    near_area(sheet(&doc, copied), Area { mm2: 200.0 }, "the sheet");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.components.iter().filter(|item| item.kind == Kind::Part).count(), 1, "one part");
    assert_eq!(copies(&doc), 1, "the copy node");
    assert_ne!(copied, made.body, "the sheet is the box");
    near(solid(&doc, made.body), Volume { mm3: 6000.0 }, "the box");
}

/// The same cap copied again is a second sheet. The box was not taken.
#[test]
fn the_same_cap_is_copied_again() {
    let mut session = Session::empty();
    let made = box_of(&mut session);
    let doc = session.document();
    let cap = end_cap(&doc, made.body);
    let first = session.copy_face(CopyFace { body: made.body, face: cap }).expect("a copy");
    let second = session.copy_face(CopyFace { body: made.body, face: cap }).expect("a second copy");
    let doc = session.document();
    assert_ne!(second, first, "the second copy is the first");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(copies(&doc), 2, "one copy was rewritten");
    near_area(sheet(&doc, first), Area { mm2: 200.0 }, "the first sheet");
    near_area(sheet(&doc, second), Area { mm2: 200.0 }, "the second sheet");
    near(solid(&doc, made.body), Volume { mm3: 6000.0 }, "the box");
}

/// A face id of 0 writes no node. The real copy is still the sheet.
#[test]
fn a_face_id_of_zero_writes_no_copy() {
    let mut session = Session::empty();
    let made = box_of(&mut session);
    let doc = session.document();
    let cap = end_cap(&doc, made.body);
    let missing = session.copy_face(CopyFace { body: made.body, face: FaceId { id: 0 } });
    assert_eq!(missing.expect_err("the face").text, "error-face-not-found");
    let held = session.document();
    assert_eq!(copies(&held), 0, "a face id of 0 wrote a copy");
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
    let copied = session.copy_face(CopyFace { body: made.body, face: cap }).expect("a copy");
    let doc = session.document();
    near_area(sheet(&doc, copied), Area { mm2: 200.0 }, "the sheet");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// A face that is not on the box is replaced by the end cap.
#[test]
fn a_missing_face_is_replaced_by_the_cap() {
    let mut session = Session::empty();
    let made = box_of(&mut session);
    let doc = session.document();
    let cap = end_cap(&doc, made.body);
    let failed = session.copy_face(CopyFace { body: made.body, face: FaceId { id: 1 } }).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-faces-not-found", "the refusal was {}", refusal(&doc));
    near(solid(&doc, made.body), Volume { mm3: 6000.0 }, "the box");
    assert!(doc.bodies.iter().all(|body| body.id != failed.id), "the copy is among the bodies");
    let zero = session.copy_face(CopyFace { body: made.body, face: FaceId { id: 0 } });
    assert_eq!(zero.expect_err("the face").text, "error-face-not-found");
    let held = session.document();
    assert_eq!(refusal(&held), "error-faces-not-found", "the refusal was {}", refusal(&held));
    assert_eq!(copies(&held), 1, "the refused copy was rewritten");
    let copied = session.copy_face(CopyFace { body: made.body, face: cap }).expect("the cap is tried again");
    assert_eq!(copied, failed, "a second copy was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(copies(&doc), 1, "the refused copy was left beside the new one");
    near_area(sheet(&doc, copied), Area { mm2: 200.0 }, "the sheet");
}

/// The id a refused copy returned is that same copy.
#[test]
fn a_missing_copy_named_by_its_own_id_is_the_same_copy() {
    let mut session = Session::empty();
    let made = box_of(&mut session);
    let doc = session.document();
    let cap = end_cap(&doc, made.body);
    let failed = session.copy_face(CopyFace { body: made.body, face: FaceId { id: 1 } }).expect("the node is written");
    let _held = session.document();
    let copied = session.copy_face(CopyFace { body: failed, face: cap }).expect("the cap is tried again");
    assert_eq!(copied, failed, "a second copy was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(copies(&doc), 1, "the refused copy was left beside the new one");
    near_area(sheet(&doc, copied), Area { mm2: 200.0 }, "the sheet");
    near(solid(&doc, made.body), Volume { mm3: 6000.0 }, "the box");
}

/// An id of 0 and a sketch id write no node. The real copy is still the sheet.
#[test]
fn a_missing_body_is_refused_before_a_copy() {
    let mut session = Session::empty();
    let made = box_of(&mut session);
    let doc = session.document();
    let cap = end_cap(&doc, made.body);
    let missing = session.copy_face(CopyFace { body: BodyId { id: 0 }, face: cap });
    assert_eq!(missing.expect_err("id 0").text, "error-source-body-not-built");
    let sketch = session.copy_face(CopyFace { body: BodyId { id: made.drawn.sketch.id }, face: cap });
    assert_eq!(sketch.expect_err("a sketch").text, "error-source-body-deleted");
    let held = session.document();
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
    assert_eq!(copies(&held), 0, "a refused id wrote a copy");
    let copied = session.copy_face(CopyFace { body: made.body, face: cap }).expect("a copy");
    let doc = session.document();
    near_area(sheet(&doc, copied), Area { mm2: 200.0 }, "the sheet");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// A turn across its axis is not a copy source.
#[test]
fn a_turn_across_the_axis_is_not_a_copy_source() {
    let mut session = Session::empty();
    let made = box_of(&mut session);
    let _box = session.document();
    let across = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, -5.0) }, to: To { at: (10.0, 15.0) } }).expect("a rectangle across the axis");
    let turned = session.revolve(across, SketchAxis::X, Angle { deg: 360.0 }).expect("the node is written");
    let _held = session.document();
    let failed = session.copy_face(CopyFace { body: turned, face: FaceId { id: 1 } });
    assert_eq!(failed.expect_err("the turn").text, "error-source-body-not-built");
    let doc = session.document();
    assert_eq!(copies(&doc), 0, "a copy was written");
    assert_eq!(refusal(&doc), "error-revolve-profile-crosses-axis", "the refusal was {}", refusal(&doc));
    near(solid(&doc, made.body), Volume { mm3: 6000.0 }, "the box");
}

/// A fillet the rebuild has refused still holds its extrusion.
#[test]
fn a_refused_fillet_holds_the_extrusion() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = doc.bodies.iter().find(|body| body.id == block.id).expect("the extrusion").edges.iter().find(|item| (item.length.mm - 20.0).abs() < 0.1).expect("the 20 mm edge").id;
    let face = end_cap(&doc, block);
    let _rounded = session.fillet(block, edge, Radius { mm: 100.0 }).expect("the node is written");
    let _refused = session.document();
    let failed = session.copy_face(CopyFace { body: block, face });
    assert_eq!(failed.expect_err("the extrusion").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(copies(&doc), 0, "a copy was written");
    assert_eq!(refusal(&doc), "fillet R100.00 failed on 1 edges", "the refusal was {}", refusal(&doc));
    near(solid(&doc, block), Volume { mm3: 6000.0 }, "the extrusion");
}

/// A copy of a missing face that has not been read joins the box, and the copy stays out of the list.
#[test]
fn an_unread_missing_face_joins_the_box() {
    let mut session = Session::empty();
    let made = box_of(&mut session);
    let _read = session.document();
    let failed = session.copy_face(CopyFace { body: made.body, face: FaceId { id: 1 } }).expect("the node is written");
    let joined = session.cuboid(Cuboid { length: BoxLength { mm: 2.0 }, width: Width { mm: 2.0 }, height: Height { mm: 2.0 } }).expect("a box");
    let doc = session.document();
    assert_eq!(doc.nodes.iter().find(|node| node.id == joined.id).expect("the union").name, "feat-name-body-union");
    near(solid(&doc, joined), Volume { mm3: 6006.0 }, "the union");
    assert!(doc.bodies.iter().all(|body| body.id != failed.id), "the copy is among the bodies");
    assert_eq!(refusal(&doc), "error-faces-not-found", "the refusal was {}", refusal(&doc));
    near(solid(&doc, made.body), Volume { mm3: 6000.0 }, "the box");
    let cube = doc.bodies.iter().map(|item| BodyId { id: item.id }).find(|item| item.id != made.body.id && item.id != joined.id).expect("the cuboid");
    near(solid(&doc, cube), Volume { mm3: 8.0 }, "the cuboid");
}

/// A copy of the end cap that has not been read stays a sheet beside the union.
#[test]
fn an_unread_copy_leaves_the_sheet_beside_the_box() {
    let mut session = Session::empty();
    let made = box_of(&mut session);
    let doc = session.document();
    let cap = end_cap(&doc, made.body);
    let copied = session.copy_face(CopyFace { body: made.body, face: cap }).expect("a copy");
    let joined = session.cuboid(Cuboid { length: BoxLength { mm: 2.0 }, width: Width { mm: 2.0 }, height: Height { mm: 2.0 } }).expect("a box");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.nodes.iter().find(|node| node.id == joined.id).expect("the union").name, "feat-name-body-union");
    near(solid(&doc, joined), Volume { mm3: 6006.0 }, "the union");
    near_area(sheet(&doc, copied), Area { mm2: 200.0 }, "the sheet");
    near(solid(&doc, made.body), Volume { mm3: 6000.0 }, "the box");
}

/// The copied cap follows the rectangle. A 10 by 10 mm cap is 100 mm^2.
#[test]
fn the_copied_cap_follows_the_rectangle() {
    let mut session = Session::empty();
    let made = box_of(&mut session);
    let doc = session.document();
    let cap = end_cap(&doc, made.body);
    let copied = session.copy_face(CopyFace { body: made.body, face: cap }).expect("a copy");
    let _read = session.document();
    session.resize(made.drawn, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 10.0) } }).expect("a smaller rectangle");
    let smaller = session.document();
    assert_eq!(smaller.rebuild, Rebuild::Quiet, "{}", refusal(&smaller));
    near(solid(&smaller, made.body), Volume { mm3: 3000.0 }, "the box");
    near_area(sheet(&smaller, copied), Area { mm2: 100.0 }, "the sheet");
}

struct Drawn {
    drawn: qymcad_drive::Profile,
    body: BodyId,
}

fn box_of(session: &mut Session) -> Drawn {
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    let body = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    Drawn { drawn, body }
}

fn end_cap(doc: &qymcad_drive::Document, id: BodyId) -> FaceId {
    body(doc, id).faces.iter().find(|face| face.role == FaceRole::CapEnd).expect("the end cap").id
}

fn copies(doc: &qymcad_drive::Document) -> usize {
    doc.nodes.iter().filter(|node| node.name == "feat-name-face-copy").count()
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

fn sheet(doc: &qymcad_drive::Document, id: BodyId) -> f64 {
    let found = body(doc, id);
    assert_eq!(found.sound, Sound::Valid, "the shape is valid");
    match found.measure {
        qymcad_drive::Measure::Live { volume, solids } => {
            assert_eq!(solids, 0, "a sheet has no solid");
            assert!(volume.mm3.abs() < 1.0, "a sheet holds {} mm^3", volume.mm3);
        }
        qymcad_drive::Measure::Absent => panic!("body {} has no live shape", id.id),
    }
    assert_eq!(found.faces.len(), 1, "one face");
    assert_eq!(found.faces[0].role, FaceRole::Other, "the copied face");
    found.faces[0].area.mm2
}

fn near(got: f64, want: Volume, what: &str) {
    assert!((got - want.mm3).abs() < 1.0, "{what}: volume {got} mm^3, wanted {} within 1", want.mm3);
}

fn near_area(got: f64, want: Area, what: &str) {
    assert!((got - want.mm2).abs() < 1.0, "{what}: area {got} mm^2, wanted {} within 1", want.mm2);
}

fn refusal(doc: &qymcad_drive::Document) -> String {
    match &doc.rebuild {
        Rebuild::Refused { refusals } => refusals.iter().map(|refusal| refusal.text.clone()).collect::<Vec<_>>().join(" "),
        Rebuild::Quiet => "quiet".into(),
        Rebuild::Stopped => "stopped".into(),
    }
}
