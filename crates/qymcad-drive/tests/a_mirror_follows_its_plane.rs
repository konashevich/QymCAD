use qymcad_drive::{Angle, BodyId, BoxLength, Corners, Cuboid, EdgeId, From, Height, Kind, Measure, Mirror, Radius, Rebuild, Session, SketchAxis, Sound, To, Width, WorldPlane};

/// The 10 by 20 by 30 mm box mirrored through the table is one solid of 12000 mm^3.
#[test]
fn a_box_mirrored_through_the_table_is_twelve_thousand() {
    let mut session = Session::empty();
    let block = session.cuboid(standing_box()).expect("a box");
    let made = session.mirror(Mirror { body: block, plane: WorldPlane::Xy }).expect("a mirror");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.components.iter().filter(|item| item.kind == Kind::Part).count(), 1, "one part");
    assert_ne!(made, block, "the mirror is the box");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-mirror").count(), 1, "the mirror node");
    near(live(&doc, made), 12000.0, "the mirror");
}

/// An id of 0 and a sketch id write no mirror. The real mirror of the box is still 12000 mm^3.
#[test]
fn a_missing_body_is_refused_before_a_mirror() {
    let mut session = Session::empty();
    let block = session.cuboid(standing_box()).expect("a box");
    let missing = session.mirror(Mirror { body: BodyId { id: 0 }, plane: WorldPlane::Xy });
    assert_eq!(missing.expect_err("id 0").text, "error-source-body-not-built");
    let drawn = session.rectangle(WorldPlane::Xy, corners(0.0, 0.0, 10.0, 10.0)).expect("a rectangle");
    let sketch = session.mirror(Mirror { body: BodyId { id: drawn.sketch.id }, plane: WorldPlane::Xy });
    assert_eq!(sketch.expect_err("a sketch").text, "error-source-body-deleted");
    let held = session.document();
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
    assert_eq!(held.nodes.iter().filter(|node| node.name == "feat-name-mirror").count(), 0, "a refused id wrote a mirror");
    let made = session.mirror(Mirror { body: block, plane: WorldPlane::Xy }).expect("a mirror");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_ne!(made, block, "the mirror is the box");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-mirror").count(), 1, "the mirror node");
    near(live(&doc, made), 12000.0, "the mirror after the refused ids");
}

/// The box is consumed as soon as the mirror is written, so a second mirror of it writes nothing.
#[test]
fn a_second_mirror_of_the_box_is_refused() {
    let mut session = Session::empty();
    let block = session.cuboid(standing_box()).expect("a box");
    let made = session.mirror(Mirror { body: block, plane: WorldPlane::Xy }).expect("a mirror");
    let again = session.mirror(Mirror { body: block, plane: WorldPlane::Xy });
    assert_eq!(again.expect_err("the box").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-mirror").count(), 1, "a second mirror was written");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_ne!(made, block, "the mirror is the box");
    near(live(&doc, made), 12000.0, "the mirror after the second ask");
}

/// A turn across its axis is not a mirror source. The box stays 6000 mm^3 and the crossing refusal stays alone.
#[test]
fn a_turn_across_the_axis_is_not_a_mirror_source() {
    let mut session = Session::empty();
    let block = session.cuboid(standing_box()).expect("a box");
    let across = turn_across(&mut session);
    let failed = session.mirror(Mirror { body: across, plane: WorldPlane::Xy });
    assert_eq!(failed.expect_err("the turn").text, "error-source-body-not-built");
    assert_ne!(across, block, "the refused turn is the box");
    let doc = session.document();
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-mirror").count(), 0, "the refused turn wrote a mirror");
    assert_eq!(refusal(&doc), "error-revolve-profile-crosses-axis", "the refusal was {}", refusal(&doc));
    near(live(&doc, block), 6000.0, "the box beside the refusal");
}

/// A fillet the rebuild has refused still holds the extrusion. A mirror of it must not be written beside that refusal.
#[test]
fn a_refused_fillet_holds_the_body_a_mirror_would_take() {
    let mut session = Session::empty();
    let block = extrusion_held_by_a_refused_fillet(&mut session);
    let failed = session.mirror(Mirror { body: block, plane: WorldPlane::Xy });
    assert_eq!(failed.expect_err("the extrusion").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(refusal(&doc), "fillet R100.00 failed on 1 edges", "the refusal was {}", refusal(&doc));
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-mirror").count(), 0, "the held extrusion wrote a mirror");
    near(live(&doc, block), 6000.0, "the extrusion beside the refusal");
}

fn standing_box() -> Cuboid {
    Cuboid { length: BoxLength { mm: 10.0 }, width: Width { mm: 20.0 }, height: Height { mm: 30.0 } }
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
