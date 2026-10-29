use qymcad_drive::{
    Angle, Area, BodyId, Corners, Dimension, EdgeId, Expression, FaceRole, From, Height, Kind, Measure, Offset, ParameterName, Radius, Rebuild, Session, SketchAxis, Sound, Split, To, Volume,
    WorldPlane,
};

/// A 10 by 20 mm rectangle grown 30 mm, marked by the table at 10 mm, stays 6000 mm^3 and has 10 named faces.
#[test]
fn a_rectangle_marked_on_the_table_keeps_ten_faces() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    let before = session.document();
    assert_eq!(face_count(&before, block), 6, "six faces before the mark");
    near_area(cap(&before, block), Area { mm2: 200.0 }, "the end cap before the mark");
    let made = session.split_faces(mark(block, WorldPlane::Xy, 10.0)).expect("a mark");
    let doc = session.document();
    near(live(&doc, made), Volume { mm3: 6000.0 }, "the body");
    assert_eq!(face_count(&doc, made), 10, "ten faces");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.components.iter().filter(|item| item.kind == Kind::Part).count(), 1, "one part");
    assert_ne!(made, block, "the marked body is the extrusion");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-split-face").count(), 1, "the split-face node");
    near_area(cap(&doc, made), Area { mm2: 200.0 }, "the end cap");
}

/// The same extrusion marked by the front plane at 10 mm splits the end cap into halves of 100 mm^2.
#[test]
fn a_rectangle_marked_on_the_front_plane_splits_the_end_cap() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    let made = session.split_faces(mark(block, WorldPlane::Xz, 10.0)).expect("a mark");
    let doc = session.document();
    near(live(&doc, made), Volume { mm3: 6000.0 }, "the body");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(face_count(&doc, made), 10, "ten faces");
    near_area(cap(&doc, made), Area { mm2: 100.0 }, "the end cap");
}

/// A plane clear of the extrusion writes no mark. The real cut is still 6000 mm^3 and 10 faces.
#[test]
fn a_plane_clear_of_the_body_leaves_the_extrusion() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    for mm in [-10.0, 100.0] {
        let failed = session.split_faces(mark(block, WorldPlane::Xy, mm));
        assert_eq!(failed.expect_err("the plane").text, "error-op-failed-split-faces", "an offset of {mm}");
    }
    let held = session.document();
    assert_eq!(held.nodes.iter().filter(|node| node.name == "feat-name-split-face").count(), 0, "a plane that misses wrote a mark");
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
    near(live(&held, block), Volume { mm3: 6000.0 }, "the extrusion after the refused planes");
    assert_eq!(face_count(&held, block), 6, "six faces after the refused planes");
    let made = session.split_faces(mark(block, WorldPlane::Xy, 10.0)).expect("a mark");
    let doc = session.document();
    near(live(&doc, made), Volume { mm3: 6000.0 }, "the body");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(face_count(&doc, made), 10, "ten faces");
}

/// An offset that is not finite writes no mark. The real cut is still 6000 mm^3 and 10 faces.
#[test]
fn an_offset_that_is_not_finite_leaves_the_extrusion() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    for mm in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let failed = session.split_faces(mark(block, WorldPlane::Xy, mm));
        assert_eq!(failed.expect_err("the offset").text, "sketch-plane-offset", "an offset of {mm}");
    }
    let held = session.document();
    assert_eq!(held.nodes.iter().filter(|node| node.name == "feat-name-split-face").count(), 0, "an offset that is not finite wrote a mark");
    let made = session.split_faces(mark(block, WorldPlane::Xy, 10.0)).expect("a mark");
    let doc = session.document();
    near(live(&doc, made), Volume { mm3: 6000.0 }, "the body");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(face_count(&doc, made), 10, "ten faces");
}

/// An id of 0 and a sketch id write no mark. The real cut of the extrusion is still 6000 mm^3 and 10 faces.
#[test]
fn a_missing_body_is_refused_before_a_mark() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    let missing = session.split_faces(mark(BodyId { id: 0 }, WorldPlane::Xy, 10.0));
    assert_eq!(missing.expect_err("id 0").text, "error-source-body-not-built");
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 10.0) } }).expect("a rectangle");
    let sketch = session.split_faces(mark(BodyId { id: drawn.sketch.id }, WorldPlane::Xy, 10.0));
    assert_eq!(sketch.expect_err("a sketch").text, "error-source-body-deleted");
    let held = session.document();
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
    assert_eq!(held.nodes.iter().filter(|node| node.name == "feat-name-split-face").count(), 0, "a refused id wrote a mark");
    let made = session.split_faces(mark(block, WorldPlane::Xy, 10.0)).expect("a mark");
    let doc = session.document();
    near(live(&doc, made), Volume { mm3: 6000.0 }, "the body");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(face_count(&doc, made), 10, "ten faces");
}

/// The extrusion is consumed as soon as the mark is written, so a second mark of it writes nothing.
#[test]
fn a_second_mark_of_the_extrusion_is_refused() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    let made = session.split_faces(mark(block, WorldPlane::Xy, 10.0)).expect("a mark");
    let again = session.split_faces(mark(block, WorldPlane::Xy, 10.0));
    assert_eq!(again.expect_err("the extrusion").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-split-face").count(), 1, "a second mark was written");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, made), Volume { mm3: 6000.0 }, "the body");
    assert_eq!(face_count(&doc, made), 10, "ten faces");
}

/// A turn across its axis is not a face source. The extrusion stays 6000 mm^3 and the crossing refusal stays alone.
#[test]
fn a_turn_across_the_axis_is_not_a_face_source() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    let across = turn_across(&mut session);
    let failed = session.split_faces(mark(across, WorldPlane::Xy, 10.0));
    assert_eq!(failed.expect_err("the turn").text, "error-source-body-not-built");
    assert_ne!(across, block, "the refused turn is the extrusion");
    let doc = session.document();
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-split-face").count(), 0, "the refused turn wrote a mark");
    assert_eq!(refusal(&doc), "error-revolve-profile-crosses-axis", "the refusal was {}", refusal(&doc));
    near(live(&doc, block), Volume { mm3: 6000.0 }, "the extrusion beside the refusal");
}

/// A fillet the rebuild has refused still holds the extrusion. A face mark of it must not be written beside that refusal.
#[test]
fn a_refused_fillet_holds_the_body_a_face_mark_would_take() {
    let mut session = Session::empty();
    let block = extrusion_held_by_a_refused_fillet(&mut session);
    let failed = session.split_faces(mark(block, WorldPlane::Xy, 10.0));
    assert_eq!(failed.expect_err("the extrusion").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(refusal(&doc), "fillet R100.00 failed on 1 edges", "the refusal was {}", refusal(&doc));
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-split-face").count(), 0, "the held extrusion wrote a mark");
    near(live(&doc, block), Volume { mm3: 6000.0 }, "the extrusion beside the refusal");
}

/// The marked body follows the extrusion height. At 30 mm it is 6000 mm^3, and at 20 mm it is 4000 mm^3, still with 10 faces.
#[test]
fn the_marked_body_follows_the_height() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    session.parameter(ParameterName { text: "span".into() }, Expression { text: "30".into() }).expect("a parameter");
    session.bind(block, Dimension::Height, Expression { text: "span".into() }).expect("the height");
    let made = session.split_faces(mark(block, WorldPlane::Xy, 10.0)).expect("a mark");
    let at_thirty = session.document();
    near(live(&at_thirty, made), Volume { mm3: 6000.0 }, "the body at 30 mm");
    assert_eq!(face_count(&at_thirty, made), 10, "ten faces at 30 mm");
    near_area(cap(&at_thirty, made), Area { mm2: 200.0 }, "the end cap at 30 mm");
    assert_eq!(at_thirty.rebuild, Rebuild::Quiet, "{}", refusal(&at_thirty));
    session.parameter(ParameterName { text: "span".into() }, Expression { text: "20".into() }).expect("the height");
    let at_twenty = session.document();
    near(live(&at_twenty, made), Volume { mm3: 4000.0 }, "the body at 20 mm");
    assert_eq!(at_twenty.rebuild, Rebuild::Quiet, "{}", refusal(&at_twenty));
    assert_eq!(face_count(&at_twenty, made), 10, "ten faces at 20 mm");
    near_area(cap(&at_twenty, made), Area { mm2: 200.0 }, "the end cap at 20 mm");
}

fn grown(session: &mut Session) -> BodyId {
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion")
}

fn mark(body: BodyId, plane: WorldPlane, mm: f64) -> Split {
    Split { body, plane, offset: Offset { mm } }
}

fn extrusion_held_by_a_refused_fillet(session: &mut Session) -> BodyId {
    let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = long_edge(&doc, block);
    let _rounded = session.fillet(block, edge, Radius { mm: 100.0 }).expect("the node is written");
    let _refused = session.document();
    block
}

fn long_edge(doc: &qymcad_drive::Document, block: BodyId) -> EdgeId {
    doc.bodies.iter().find(|body| body.id == block.id).expect("the extrusion").edges.iter().find(|edge| (edge.length.mm - 20.0).abs() < 0.1).expect("the 20 mm edge").id
}

fn turn_across(session: &mut Session) -> BodyId {
    let across = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, -5.0) }, to: To { at: (10.0, 15.0) } }).expect("a rectangle across the axis");
    session.revolve(across, SketchAxis::X, Angle { deg: 360.0 }).expect("the node is written")
}

fn body(doc: &qymcad_drive::Document, id: BodyId) -> &qymcad_drive::Body {
    doc.bodies.iter().find(|body| body.id == id.id).unwrap_or_else(|| panic!("body {} is gone", id.id))
}

fn face_count(doc: &qymcad_drive::Document, id: BodyId) -> usize {
    body(doc, id).faces.len()
}

fn cap(doc: &qymcad_drive::Document, id: BodyId) -> f64 {
    body(doc, id).faces.iter().find(|face| face.role == FaceRole::CapEnd).expect("the end cap").area.mm2
}

fn live(doc: &qymcad_drive::Document, id: BodyId) -> f64 {
    let found = body(doc, id);
    assert_eq!(found.sound, Sound::Valid, "the shape is valid");
    match found.measure {
        Measure::Live { volume, solids } => {
            assert_eq!(solids, 1, "one solid");
            volume.mm3
        }
        Measure::Absent => panic!("body {} has no live shape", id.id),
    }
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
