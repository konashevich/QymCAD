use qymcad_drive::{Angle, Area, BodyId, BoxLength, Corners, Cuboid, FaceId, FaceRole, From, Height, Kind, OffsetFace, Outset, Radius, Rebuild, Session, SketchAxis, Sound, To, Volume, Width, WorldPlane};

/// The side of the diameter 10 mm rod, moved 2 mm outward, is a sheet of 1098.56 mm^2. The rod stays.
#[test]
fn the_side_moves_out() {
    let mut session = Session::empty();
    let made = rod(&mut session);
    let doc = session.document();
    let face = side(&doc, made.body);
    let moved = session.offset_face(outward(made.body, face)).expect("an offset");
    let doc = session.document();
    near_area(sheet(&doc, moved), outward_area(), "the sheet");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.components.iter().filter(|item| item.kind == Kind::Part).count(), 1, "one part");
    assert_eq!(offsets(&doc), 1, "the offset node");
    assert_ne!(moved, made.body, "the sheet is the rod");
    near(solid(&doc, made.body), cylinder(Radius { mm: 5.0 }, Height { mm: 25.0 }), "the rod");
}

/// The same side moved 2 mm inward is the smaller sheet.
#[test]
fn the_side_moves_in() {
    let mut session = Session::empty();
    let made = rod(&mut session);
    let doc = session.document();
    let face = side(&doc, made.body);
    let moved = session.offset_face(OffsetFace { body: made.body, face, by: Outset { mm: -2.0 } }).expect("an offset");
    let doc = session.document();
    near_area(sheet(&doc, moved), inward_area(), "the sheet");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(solid(&doc, made.body), cylinder(Radius { mm: 5.0 }, Height { mm: 25.0 }), "the rod");
}

/// A distance of 0 leaves the side where it is.
#[test]
fn a_distance_of_zero_is_the_side() {
    let mut session = Session::empty();
    let made = rod(&mut session);
    let doc = session.document();
    let face = side(&doc, made.body);
    let moved = session.offset_face(OffsetFace { body: made.body, face, by: Outset { mm: 0.0 } }).expect("an offset");
    let doc = session.document();
    near_area(sheet(&doc, moved), side_area(), "the sheet");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// Moving the side in by its radius is replaced by the inward offset.
#[test]
fn a_distance_of_the_radius_is_replaced_by_the_inward_offset() {
    let mut session = Session::empty();
    let made = rod(&mut session);
    let doc = session.document();
    let face = side(&doc, made.body);
    let failed = session.offset_face(OffsetFace { body: made.body, face, by: Outset { mm: -5.0 } }).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "a round face moved in by its radius 5.00 or more", "the refusal was {}", refusal(&doc));
    near(solid(&doc, made.body), cylinder(Radius { mm: 5.0 }, Height { mm: 25.0 }), "the rod");
    assert!(doc.bodies.iter().all(|body| body.id != failed.id), "the offset is among the bodies");
    let missing = session.offset_face(OffsetFace { body: made.body, face: FaceId { id: 0 }, by: Outset { mm: -2.0 } });
    assert_eq!(missing.expect_err("the face").text, "error-face-not-found");
    held_back(&mut session, OffsetFace { body: made.body, face, by: Outset { mm: f64::NAN } }, "error-offset-distance");
    held_back(&mut session, OffsetFace { body: made.body, face, by: Outset { mm: f64::INFINITY } }, "error-offset-distance");
    let held = session.document();
    assert_eq!(refusal(&held), "a round face moved in by its radius 5.00 or more", "the refusal was {}", refusal(&held));
    assert_eq!(offsets(&held), 1, "the refused offset was rewritten");
    let moved = session.offset_face(OffsetFace { body: made.body, face, by: Outset { mm: -2.0 } }).expect("the side is tried again");
    assert_eq!(moved, failed, "a second offset was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(offsets(&doc), 1, "the refused offset was left beside the new one");
    near_area(sheet(&doc, moved), inward_area(), "the sheet");
}

/// The id a refused offset returned is that same offset.
#[test]
fn a_refused_offset_named_by_its_own_id_is_the_same_offset() {
    let mut session = Session::empty();
    let made = rod(&mut session);
    let doc = session.document();
    let face = side(&doc, made.body);
    let failed = session.offset_face(OffsetFace { body: made.body, face, by: Outset { mm: -5.0 } }).expect("the node is written");
    let _held = session.document();
    let moved = session.offset_face(OffsetFace { body: failed, face, by: Outset { mm: -2.0 } }).expect("the side is tried again");
    assert_eq!(moved, failed, "a second offset was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(offsets(&doc), 1, "the refused offset was left beside the new one");
    near_area(sheet(&doc, moved), inward_area(), "the sheet");
    near(solid(&doc, made.body), cylinder(Radius { mm: 5.0 }, Height { mm: 25.0 }), "the rod");
}

/// The same side offset again is a second sheet. The rod was not taken.
#[test]
fn the_same_side_is_offset_again() {
    let mut session = Session::empty();
    let made = rod(&mut session);
    let doc = session.document();
    let face = side(&doc, made.body);
    let first = session.offset_face(outward(made.body, face)).expect("an offset");
    let second = session.offset_face(outward(made.body, face)).expect("a second offset");
    let doc = session.document();
    assert_ne!(second, first, "the second offset is the first");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(offsets(&doc), 2, "one offset was rewritten");
    near_area(sheet(&doc, first), outward_area(), "the first sheet");
    near_area(sheet(&doc, second), outward_area(), "the second sheet");
    near(solid(&doc, made.body), cylinder(Radius { mm: 5.0 }, Height { mm: 25.0 }), "the rod");
}

/// A flat cap moved 5 mm outward keeps its area. The box stays.
#[test]
fn a_flat_cap_keeps_its_area() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let cap = doc.bodies.iter().find(|body| body.id == block.id).expect("the box").faces.iter().find(|face| face.role == FaceRole::CapEnd).expect("the end cap").id;
    let moved = session.offset_face(OffsetFace { body: block, face: cap, by: Outset { mm: 5.0 } }).expect("an offset");
    let doc = session.document();
    near_area(sheet(&doc, moved), Area { mm2: 200.0 }, "the sheet");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(solid(&doc, block), Volume { mm3: 6000.0 }, "the box");
}

/// A face id of 0 writes no node. The real offset is still the outward sheet.
#[test]
fn a_face_id_of_zero_writes_no_offset() {
    let mut session = Session::empty();
    let made = rod(&mut session);
    let doc = session.document();
    let face = side(&doc, made.body);
    let missing = session.offset_face(OffsetFace { body: made.body, face: FaceId { id: 0 }, by: Outset { mm: 2.0 } });
    assert_eq!(missing.expect_err("the face").text, "error-face-not-found");
    let held = session.document();
    assert_eq!(offsets(&held), 0, "a face id of 0 wrote an offset");
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
    let moved = session.offset_face(outward(made.body, face)).expect("an offset");
    let doc = session.document();
    near_area(sheet(&doc, moved), outward_area(), "the sheet");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// An id of 0 and a sketch id write no node. The real offset is still the outward sheet.
#[test]
fn a_missing_body_is_refused_before_an_offset() {
    let mut session = Session::empty();
    let made = rod(&mut session);
    let doc = session.document();
    let face = side(&doc, made.body);
    let missing = session.offset_face(outward(BodyId { id: 0 }, face));
    assert_eq!(missing.expect_err("id 0").text, "error-source-body-not-built");
    let sketch = session.offset_face(outward(BodyId { id: made.drawn.sketch.id }, face));
    assert_eq!(sketch.expect_err("a sketch").text, "error-source-body-deleted");
    let held = session.document();
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
    assert_eq!(offsets(&held), 0, "a refused id wrote an offset");
    let moved = session.offset_face(outward(made.body, face)).expect("an offset");
    let doc = session.document();
    near_area(sheet(&doc, moved), outward_area(), "the sheet");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// A turn across its axis is not an offset source.
#[test]
fn a_turn_across_the_axis_is_not_an_offset_source() {
    let mut session = Session::empty();
    let made = rod(&mut session);
    let _rod = session.document();
    let across = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, -5.0) }, to: To { at: (10.0, 15.0) } }).expect("a rectangle across the axis");
    let turned = session.revolve(across, SketchAxis::X, Angle { deg: 360.0 }).expect("the node is written");
    let _held = session.document();
    let failed = session.offset_face(OffsetFace { body: turned, face: FaceId { id: 1 }, by: Outset { mm: 2.0 } });
    assert_eq!(failed.expect_err("the turn").text, "error-source-body-not-built");
    let doc = session.document();
    assert_eq!(offsets(&doc), 0, "an offset was written");
    assert_eq!(refusal(&doc), "error-revolve-profile-crosses-axis", "the refusal was {}", refusal(&doc));
    near(solid(&doc, made.body), cylinder(Radius { mm: 5.0 }, Height { mm: 25.0 }), "the rod");
}

/// A fillet the rebuild has refused still holds its extrusion.
#[test]
fn a_refused_fillet_holds_the_extrusion() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = doc.bodies.iter().find(|body| body.id == block.id).expect("the extrusion").edges.iter().find(|item| (item.length.mm - 20.0).abs() < 0.1).expect("the 20 mm edge").id;
    let face = doc.bodies.iter().find(|body| body.id == block.id).expect("the extrusion").faces.iter().find(|item| item.role == FaceRole::CapEnd).expect("the end cap").id;
    let _rounded = session.fillet(block, edge, Radius { mm: 100.0 }).expect("the node is written");
    let _refused = session.document();
    let failed = session.offset_face(OffsetFace { body: block, face, by: Outset { mm: 2.0 } });
    assert_eq!(failed.expect_err("the extrusion").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(offsets(&doc), 0, "an offset was written");
    assert_eq!(refusal(&doc), "fillet R100.00 failed on 1 edges", "the refusal was {}", refusal(&doc));
    near(solid(&doc, block), Volume { mm3: 6000.0 }, "the extrusion");
}

/// An offset of a missing face that has not been read joins the box, and the offset stays out of the list.
#[test]
fn an_unread_missing_face_joins_the_box() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let _read = session.document();
    let failed = session.offset_face(OffsetFace { body: block, face: FaceId { id: 1 }, by: Outset { mm: 5.0 } }).expect("the node is written");
    let joined = session.cuboid(Cuboid { length: BoxLength { mm: 2.0 }, width: Width { mm: 2.0 }, height: Height { mm: 2.0 } }).expect("a box");
    let doc = session.document();
    assert_eq!(doc.nodes.iter().find(|node| node.id == joined.id).expect("the union").name, "feat-name-body-union");
    near(solid(&doc, joined), Volume { mm3: 6006.0 }, "the union");
    assert!(doc.bodies.iter().all(|body| body.id != failed.id), "the offset is among the bodies");
    assert_eq!(refusal(&doc), "error-faces-not-found", "the refusal was {}", refusal(&doc));
    near(solid(&doc, block), Volume { mm3: 6000.0 }, "the box");
    let cube = doc.bodies.iter().map(|item| BodyId { id: item.id }).find(|item| item.id != block.id && item.id != joined.id).expect("the cuboid");
    near(solid(&doc, cube), Volume { mm3: 8.0 }, "the cuboid");
}

/// An offset of the end cap that has not been read stays a sheet beside the union.
#[test]
fn an_unread_offset_leaves_the_sheet_beside_the_box() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let cap = doc.bodies.iter().find(|body| body.id == block.id).expect("the box").faces.iter().find(|face| face.role == FaceRole::CapEnd).expect("the end cap").id;
    let moved = session.offset_face(OffsetFace { body: block, face: cap, by: Outset { mm: 5.0 } }).expect("an offset");
    let joined = session.cuboid(Cuboid { length: BoxLength { mm: 2.0 }, width: Width { mm: 2.0 }, height: Height { mm: 2.0 } }).expect("a box");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.nodes.iter().find(|node| node.id == joined.id).expect("the union").name, "feat-name-body-union");
    near(solid(&doc, joined), Volume { mm3: 6006.0 }, "the union");
    near_area(sheet(&doc, moved), Area { mm2: 200.0 }, "the sheet");
    near(solid(&doc, block), Volume { mm3: 6000.0 }, "the box");
}

/// The offset side follows the rod. A radius of 6 mm with 2 mm outward is 1255.57 mm^2.
#[test]
fn the_offset_side_follows_the_rod() {
    let mut session = Session::empty();
    let made = rod(&mut session);
    let doc = session.document();
    let face = side(&doc, made.body);
    let moved = session.offset_face(outward(made.body, face)).expect("an offset");
    let _read = session.document();
    session.resize(made.drawn, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (25.0, 6.0) } }).expect("a thicker rod");
    let thicker = session.document();
    assert_eq!(thicker.rebuild, Rebuild::Quiet, "{}", refusal(&thicker));
    near(solid(&thicker, made.body), cylinder(Radius { mm: 6.0 }, Height { mm: 25.0 }), "the rod");
    near_area(sheet(&thicker, moved), Area { mm2: 1255.5692845963902 }, "the sheet");
}

struct Stock {
    drawn: qymcad_drive::Profile,
    body: BodyId,
}

fn rod(session: &mut Session) -> Stock {
    let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (25.0, 5.0) } }).expect("a rectangle");
    let body = session.revolve(drawn, SketchAxis::X, Angle { deg: 360.0 }).expect("a rod");
    Stock { drawn, body }
}

fn side(doc: &qymcad_drive::Document, id: BodyId) -> FaceId {
    body(doc, id).faces.iter().max_by(|a, b| a.area.mm2.partial_cmp(&b.area.mm2).expect("an area")).expect("the side").id
}

fn outward(body: BodyId, face: FaceId) -> OffsetFace {
    OffsetFace { body, face, by: Outset { mm: 2.0 } }
}

fn held_back(session: &mut Session, asked: OffsetFace, text: &str) {
    let err = session.offset_face(asked).expect_err(text);
    assert_eq!(err.text, text, "the refusal was {}", err.text);
    let held = session.document();
    assert_eq!(offsets(&held), 1, "the refused offset was rewritten");
}

fn side_area() -> Area {
    Area { mm2: 784.665986932697 }
}

fn outward_area() -> Area {
    Area { mm2: 1098.5616899323497 }
}

fn inward_area() -> Area {
    Area { mm2: 470.98205682716025 }
}

fn cylinder(radius: Radius, height: Height) -> Volume {
    Volume { mm3: std::f64::consts::PI * radius.mm * radius.mm * height.mm }
}

fn offsets(doc: &qymcad_drive::Document) -> usize {
    doc.nodes.iter().filter(|node| node.name == "feat-name-offset-surface").count()
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
