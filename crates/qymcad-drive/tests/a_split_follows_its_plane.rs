use qymcad_drive::{Angle, BodyId, BoxLength, Corners, Cuboid, EdgeId, From, Height, Kind, Measure, Offset, Radius, Rebuild, Session, SketchAxis, Sound, Split, To, Width, WorldPlane};

/// The 10 by 20 by 30 mm box, cut by the table at 10 mm, is 2000 mm^3 below and 4000 mm^3 above.
#[test]
fn a_box_split_ten_millimetres_up_is_two_thousand_and_four_thousand() {
    let mut session = Session::empty();
    let block = session.cuboid(standing_box()).expect("a box");
    let made = session.split(cut(block, WorldPlane::Xy, 10.0)).expect("a split");
    let doc = session.document();
    near(live(&doc, made.bodies[0]), 2000.0, "the lower piece");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.components.iter().filter(|item| item.kind == Kind::Part).count(), 1, "one part");
    assert_ne!(made.bodies[0], block, "the lower piece is the box");
    assert_ne!(made.bodies[1], block, "the upper piece is the box");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-split-body").count(), 1, "the split node");
    near(live(&doc, made.bodies[1]), 4000.0, "the upper piece");
}

/// A 10 mm square standing 20 mm off the origin, grown 10 mm and cut 4 mm up, is 400 mm^3 below and 600 mm^3 above.
#[test]
fn a_square_standing_away_from_the_origin_splits() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xy, corners(20.0, 0.0, 30.0, 10.0)).expect("a square");
    let block = session.extrude(drawn, Height { mm: 10.0 }).expect("an extrusion");
    let _read = session.document();
    let made = session.split(cut(block, WorldPlane::Xy, 4.0)).expect("a split");
    let doc = session.document();
    near(live(&doc, made.bodies[0]), 400.0, "the lower piece");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-split-body").count(), 1, "the split node");
    near(live(&doc, made.bodies[1]), 600.0, "the upper piece");
}

/// The same box cut by the front plane at offset 0 is two pieces of 3000 mm^3.
#[test]
fn a_box_split_through_the_front_plane_is_two_pieces_of_three_thousand() {
    let mut session = Session::empty();
    let block = session.cuboid(standing_box()).expect("a box");
    let made = session.split(cut(block, WorldPlane::Xz, 0.0)).expect("a split");
    let doc = session.document();
    near(live(&doc, made.bodies[0]), 3000.0, "one half");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.components.iter().filter(|item| item.kind == Kind::Part).count(), 1, "one part");
    assert_ne!(made.bodies[0], block, "the first piece is the box");
    assert_ne!(made.bodies[1], block, "the second piece is the box");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-split-body").count(), 1, "the split node");
    near(live(&doc, made.bodies[1]), 3000.0, "the other half");
}

/// A plane on a face, or clear of the box, writes no split. The real cut is still 2000 mm^3 and 4000 mm^3.
#[test]
fn a_plane_that_misses_leaves_the_box() {
    let mut session = Session::empty();
    let block = session.cuboid(standing_box()).expect("a box");
    for mm in [0.0, 30.0, -10.0, 100.0] {
        let failed = session.split(cut(block, WorldPlane::Xy, mm));
        assert_eq!(failed.expect_err("the plane").text, "error-op-failed-split-body", "an offset of {mm}");
    }
    let held = session.document();
    assert_eq!(held.nodes.iter().filter(|node| node.name == "feat-name-split-body").count(), 0, "a plane that misses wrote a split");
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
    near(live(&held, block), 6000.0, "the box after the refused planes");
    let made = session.split(cut(block, WorldPlane::Xy, 10.0)).expect("a split");
    let doc = session.document();
    near(live(&doc, made.bodies[0]), 2000.0, "the lower piece");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-split-body").count(), 1, "the split node");
    near(live(&doc, made.bodies[1]), 4000.0, "the upper piece");
}

/// An offset that is not finite writes no split. The real cut is still 2000 mm^3 and 4000 mm^3.
#[test]
fn an_offset_that_is_not_finite_leaves_the_box() {
    let mut session = Session::empty();
    let block = session.cuboid(standing_box()).expect("a box");
    for mm in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let failed = session.split(cut(block, WorldPlane::Xy, mm));
        assert_eq!(failed.expect_err("the offset").text, "sketch-plane-offset", "an offset of {mm}");
    }
    let held = session.document();
    assert_eq!(held.nodes.iter().filter(|node| node.name == "feat-name-split-body").count(), 0, "an offset that is not finite wrote a split");
    let made = session.split(cut(block, WorldPlane::Xy, 10.0)).expect("a split");
    let doc = session.document();
    near(live(&doc, made.bodies[0]), 2000.0, "the lower piece");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-split-body").count(), 1, "the split node");
    near(live(&doc, made.bodies[1]), 4000.0, "the upper piece");
}

/// An id of 0 and a sketch id write no split. The real cut of the box is still 2000 mm^3 and 4000 mm^3.
#[test]
fn a_missing_body_is_refused_before_a_split() {
    let mut session = Session::empty();
    let block = session.cuboid(standing_box()).expect("a box");
    let missing = session.split(cut(BodyId { id: 0 }, WorldPlane::Xy, 10.0));
    assert_eq!(missing.expect_err("id 0").text, "error-source-body-not-built");
    let drawn = session.rectangle(WorldPlane::Xy, corners(0.0, 0.0, 10.0, 10.0)).expect("a rectangle");
    let sketch = session.split(cut(BodyId { id: drawn.sketch.id }, WorldPlane::Xy, 10.0));
    assert_eq!(sketch.expect_err("a sketch").text, "error-source-body-deleted");
    let held = session.document();
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
    assert_eq!(held.nodes.iter().filter(|node| node.name == "feat-name-split-body").count(), 0, "a refused id wrote a split");
    let made = session.split(cut(block, WorldPlane::Xy, 10.0)).expect("a split");
    let doc = session.document();
    near(live(&doc, made.bodies[0]), 2000.0, "the lower piece");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-split-body").count(), 1, "the split node");
    near(live(&doc, made.bodies[1]), 4000.0, "the upper piece");
}

/// The box is consumed as soon as the split is written, so a second split of it writes nothing.
#[test]
fn a_second_split_of_the_box_is_refused() {
    let mut session = Session::empty();
    let block = session.cuboid(standing_box()).expect("a box");
    let made = session.split(cut(block, WorldPlane::Xy, 10.0)).expect("a split");
    let again = session.split(cut(block, WorldPlane::Xy, 10.0));
    assert_eq!(again.expect_err("the box").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-split-body").count(), 1, "a second split was written");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, made.bodies[0]), 2000.0, "the lower piece");
    near(live(&doc, made.bodies[1]), 4000.0, "the upper piece");
}

/// A turn across its axis is not a split source. The box stays 6000 mm^3 and the crossing refusal stays alone.
#[test]
fn a_turn_across_the_axis_is_not_a_split_source() {
    let mut session = Session::empty();
    let block = session.cuboid(standing_box()).expect("a box");
    let across = turn_across(&mut session);
    let failed = session.split(cut(across, WorldPlane::Xy, 10.0));
    assert_eq!(failed.expect_err("the turn").text, "error-source-body-not-built");
    assert_ne!(across, block, "the refused turn is the box");
    let doc = session.document();
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-split-body").count(), 0, "the refused turn wrote a split");
    assert_eq!(refusal(&doc), "error-revolve-profile-crosses-axis", "the refusal was {}", refusal(&doc));
    near(live(&doc, block), 6000.0, "the box beside the refusal");
}

/// A fillet the rebuild has refused still holds the extrusion. A split of it must not be written beside that refusal.
#[test]
fn a_refused_fillet_holds_the_body_a_split_would_take() {
    let mut session = Session::empty();
    let block = extrusion_held_by_a_refused_fillet(&mut session);
    let failed = session.split(cut(block, WorldPlane::Xy, 10.0));
    assert_eq!(failed.expect_err("the extrusion").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(refusal(&doc), "fillet R100.00 failed on 1 edges", "the refusal was {}", refusal(&doc));
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-split-body").count(), 0, "the held extrusion wrote a split");
    near(live(&doc, block), 6000.0, "the extrusion beside the refusal");
}

fn standing_box() -> Cuboid {
    Cuboid { length: BoxLength { mm: 10.0 }, width: Width { mm: 20.0 }, height: Height { mm: 30.0 } }
}

fn cut(body: BodyId, plane: WorldPlane, mm: f64) -> Split {
    Split { body, plane, offset: Offset { mm } }
}

fn extrusion_held_by_a_refused_fillet(session: &mut Session) -> BodyId {
    let drawn = session.rectangle(WorldPlane::Xz, corners(0.0, 0.0, 10.0, 20.0)).expect("a rectangle");
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
    let across = session.rectangle(WorldPlane::Xz, corners(0.0, -5.0, 10.0, 15.0)).expect("a rectangle across the axis");
    session.revolve(across, SketchAxis::X, Angle { deg: 360.0 }).expect("the node is written")
}

fn corners(x0: f64, y0: f64, x1: f64, y1: f64) -> Corners {
    Corners { from: From { at: (x0, y0) }, to: To { at: (x1, y1) } }
}

fn live(doc: &qymcad_drive::Document, id: BodyId) -> f64 {
    let body = doc.bodies.iter().find(|body| body.id == id.id).unwrap_or_else(|| panic!("body {} is gone", id.id));
    assert_eq!(body.sound, Sound::Valid, "the shape is valid");
    match body.measure {
        Measure::Live { volume, solids } => {
            assert_eq!(solids, 1, "one solid");
            volume.mm3
        }
        Measure::Absent => panic!("body {} has no live shape", id.id),
    }
}

fn near(got: f64, want: f64, what: &str) {
    assert!((got - want).abs() < 1.0, "{what}: volume {got} mm^3, wanted {want} within 1");
}

fn refusal(doc: &qymcad_drive::Document) -> String {
    match &doc.rebuild {
        Rebuild::Refused { refusals } => refusals.iter().map(|refusal| refusal.text.clone()).collect::<Vec<_>>().join(" "),
        Rebuild::Quiet => "quiet".into(),
        Rebuild::Stopped => "stopped".into(),
    }
}
