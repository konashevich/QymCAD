use qymcad_drive::{Angle, BodyId, Circular, Copies, Corners, From, Height, Kind, Measure, Radius, Rebuild, Session, SketchAxis, Sound, To, WorldPlane};

/// Four copies of the first-quadrant square, over a full turn, are one solid of 4000 mm^3.
#[test]
fn a_ring_of_four_squares_is_four_thousand() {
    let mut session = Session::empty();
    let grown = quadrant(&mut session);
    let made = session.circular(ring(grown, 4, 360.0)).expect("a ring");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.components.iter().filter(|item| item.kind == Kind::Part).count(), 1, "one part");
    assert_ne!(made, grown, "the ring is the extrusion");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-circular-array").count(), 1, "the ring node");
    near(live(&doc, made), 4000.0, "the ring");
}

/// Two copies over 180 degrees stand 90 degrees apart, so the two squares are one solid of 2000 mm^3.
#[test]
fn a_half_turn_of_two_squares_is_two_thousand() {
    let mut session = Session::empty();
    let grown = quadrant(&mut session);
    let made = session.circular(ring(grown, 2, 180.0)).expect("a half turn");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_ne!(made, grown, "the ring is the extrusion");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-circular-array").count(), 1, "the ring node");
    near(live(&doc, made), 2000.0, "the half turn");
}

/// A negative full turn is passed through and still fills the four quadrants, 4000 mm^3.
#[test]
fn a_turn_the_other_way_is_four_thousand() {
    let mut session = Session::empty();
    let grown = quadrant(&mut session);
    let made = session.circular(ring(grown, 4, -360.0)).expect("a ring");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_ne!(made, grown, "the ring is the extrusion");
    near(live(&doc, made), 4000.0, "the ring the other way");
}

/// An angle past a full turn is passed through and spaced as a full turn, still 4000 mm^3.
#[test]
fn a_turn_past_a_full_circle_is_four_thousand() {
    let mut session = Session::empty();
    let grown = quadrant(&mut session);
    let made = session.circular(ring(grown, 4, 720.0)).expect("a ring");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_ne!(made, grown, "the ring is the extrusion");
    near(live(&doc, made), 4000.0, "the ring past a full turn");
}

/// A count of 0 and a count of 1 write no ring. The real ring of the same extrusion is still 4000 mm^3.
#[test]
fn a_count_below_two_leaves_the_square() {
    let mut session = Session::empty();
    let grown = quadrant(&mut session);
    let none = session.circular(ring(grown, 0, 360.0));
    assert_eq!(none.expect_err("no copies").text, "error-array-of-one");
    let one = session.circular(ring(grown, 1, 360.0));
    assert_eq!(one.expect_err("one copy").text, "error-array-of-one");
    let doc = session.document();
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-circular-array").count(), 0, "a count below two wrote a ring");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, grown), 1000.0, "the square after the refused counts");
    let made = session.circular(ring(grown, 4, 360.0)).expect("a ring");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_ne!(made, grown, "the ring is the extrusion");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-circular-array").count(), 1, "the ring node");
    near(live(&doc, made), 4000.0, "the ring after the refused counts");
}

/// An angle of zero, and an angle that is not finite, write no ring. The real ring is still 4000 mm^3.
#[test]
fn a_turn_of_nothing_leaves_the_square() {
    let mut session = Session::empty();
    let grown = quadrant(&mut session);
    for deg in [0.0, f64::NAN, f64::INFINITY] {
        let failed = session.circular(ring(grown, 4, deg));
        assert_eq!(failed.expect_err("the angle").text, "sketch-array-angle", "an angle of {deg}");
    }
    let held = session.document();
    assert_eq!(held.nodes.iter().filter(|node| node.name == "feat-name-circular-array").count(), 0, "an angle of nothing wrote a ring");
    let made = session.circular(ring(grown, 4, 360.0)).expect("a ring");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_ne!(made, grown, "the ring is the extrusion");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-circular-array").count(), 1, "the ring node");
    near(live(&doc, made), 4000.0, "the ring after the refused angles");
}

/// A turn across its axis is not a ring. The box stays 6000 mm^3 and the crossing refusal stays alone.
#[test]
fn a_turn_across_the_axis_is_not_a_ring() {
    let mut session = Session::empty();
    let block =
        session.cuboid(qymcad_drive::Cuboid { length: qymcad_drive::BoxLength { mm: 10.0 }, width: qymcad_drive::Width { mm: 20.0 }, height: qymcad_drive::Height { mm: 30.0 } }).expect("a box");
    let across = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, -5.0) }, to: To { at: (10.0, 15.0) } }).expect("a rectangle across the axis");
    let turned = session.revolve(across, SketchAxis::X, Angle { deg: 360.0 }).expect("the node is written");
    let failed = session.circular(ring(turned, 4, 360.0));
    assert_eq!(failed.expect_err("the turn").text, "error-source-body-not-built");
    let doc = session.document();
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-circular-array").count(), 0, "the refused turn wrote a ring");
    assert_eq!(refusal(&doc), "error-revolve-profile-crosses-axis", "the refusal was {}", refusal(&doc));
    near(live(&doc, block), 6000.0, "the box beside the refusal");
}

/// A fillet the rebuild has refused still holds the extrusion. A ring of it must not be written beside that refusal.
#[test]
fn a_refused_fillet_holds_the_body_a_ring_would_take() {
    let mut session = Session::empty();
    let block = extrusion_held_by_a_refused_fillet(&mut session);
    let failed = session.circular(ring(block, 4, 360.0));
    assert_eq!(failed.expect_err("the extrusion").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(refusal(&doc), "fillet R100.00 failed on 1 edges", "the refusal was {}", refusal(&doc));
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-circular-array").count(), 0, "the held extrusion wrote a ring");
    near(live(&doc, block), 6000.0, "the extrusion beside the refusal");
}

fn quadrant(session: &mut Session) -> BodyId {
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 10.0) } }).expect("a square");
    session.extrude(drawn, Height { mm: 10.0 }).expect("an extrusion")
}

fn extrusion_held_by_a_refused_fillet(session: &mut Session) -> BodyId {
    let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = doc.bodies.iter().find(|body| body.id == block.id).expect("the extrusion").edges.iter().find(|edge| (edge.length.mm - 20.0).abs() < 0.1).expect("the 20 mm edge").id;
    let _rounded = session.fillet(block, edge, Radius { mm: 100.0 }).expect("the node is written");
    let _refused = session.document();
    block
}

fn ring(body: BodyId, n: u32, deg: f64) -> Circular {
    Circular { body, copies: Copies { n }, turn: Angle { deg } }
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
