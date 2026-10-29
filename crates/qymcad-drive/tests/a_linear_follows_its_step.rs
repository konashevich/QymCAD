use qymcad_drive::{
    AlongX, AlongY, AlongZ, Angle, BodyId, BoxLength, Copies, Corners, Cuboid, From, Height, Kind, Linear, Measure, Radius, Rebuild, Session, SketchAxis, Sound, Step, To, Width, WorldPlane,
};

/// The 10 by 20 by 30 mm box, stepped 10 mm along X, three copies, is one solid of 18000 mm^3.
#[test]
fn a_row_of_three_touching_boxes_is_eighteen_thousand() {
    let mut session = Session::empty();
    let block = session.cuboid(standing_box()).expect("a box");
    let made = session.linear(row(block, 10.0, 3)).expect("a row");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.components.iter().filter(|item| item.kind == Kind::Part).count(), 1, "one part");
    assert_ne!(made, block, "the row is the box");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-linear-array").count(), 1, "the row node");
    near(live(&doc, made), 18000.0, "the row");
}

/// A step of -10 mm along X is the same row the other way, still 18000 mm^3.
#[test]
fn a_row_stepped_the_other_way_is_eighteen_thousand() {
    let mut session = Session::empty();
    let block = session.cuboid(standing_box()).expect("a box");
    let made = session.linear(row(block, -10.0, 3)).expect("a row");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_ne!(made, block, "the row is the box");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-linear-array").count(), 1, "the row node");
    near(live(&doc, made), 18000.0, "the row the other way");
}

/// A count of 0 and a count of 1 write no row. The real row of the same box is still 18000 mm^3.
#[test]
fn a_count_below_two_leaves_the_box() {
    let mut session = Session::empty();
    let block = session.cuboid(standing_box()).expect("a box");
    let none = session.linear(row(block, 10.0, 0));
    assert_eq!(none.expect_err("no copies").text, "error-array-of-one");
    let one = session.linear(row(block, 10.0, 1));
    assert_eq!(one.expect_err("one copy").text, "error-array-of-one");
    let doc = session.document();
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-linear-array").count(), 0, "a count below two wrote a row");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, block), 6000.0, "the box after the refused counts");
    let made = session.linear(row(block, 10.0, 3)).expect("a row");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_ne!(made, block, "the row is the box");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-linear-array").count(), 1, "the row node");
    near(live(&doc, made), 18000.0, "the row after the refused counts");
}

/// A step that is not finite, and a step no longer than 1e-6 mm, write no row. The real row is still 18000 mm^3.
#[test]
fn a_step_that_does_not_move_leaves_the_box() {
    let mut session = Session::empty();
    let block = session.cuboid(standing_box()).expect("a box");
    for along in [f64::NAN, f64::INFINITY, 0.0, 1e-6] {
        let failed = session.linear(row(block, along, 3));
        assert_eq!(failed.expect_err("the step").text, "sketch-array-step", "a step of {along}");
    }
    let held = session.document();
    assert_eq!(held.nodes.iter().filter(|node| node.name == "feat-name-linear-array").count(), 0, "a step that does not move wrote a row");
    let made = session.linear(row(block, 10.0, 3)).expect("a row");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_ne!(made, block, "the row is the box");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-linear-array").count(), 1, "the row node");
    near(live(&doc, made), 18000.0, "the row after the refused steps");
}

/// A turn across its axis is not a row. The box stays 6000 mm^3 and the crossing refusal stays alone.
#[test]
fn a_turn_across_the_axis_is_not_a_row() {
    let mut session = Session::empty();
    let block = session.cuboid(standing_box()).expect("a box");
    let across = turn_across(&mut session);
    let failed = session.linear(row(across, 10.0, 3));
    assert_eq!(failed.expect_err("the turn").text, "error-source-body-not-built");
    let doc = session.document();
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-linear-array").count(), 0, "the refused turn wrote a row");
    assert_eq!(refusal(&doc), "error-revolve-profile-crosses-axis", "the refusal was {}", refusal(&doc));
    near(live(&doc, block), 6000.0, "the box beside the refusal");
}

/// A fillet the rebuild has refused still holds the extrusion. A row of it must not be written beside that refusal.
#[test]
fn a_refused_fillet_holds_the_body_a_row_would_take() {
    let mut session = Session::empty();
    let block = extrusion_held_by_a_refused_fillet(&mut session);
    let failed = session.linear(row(block, 10.0, 3));
    assert_eq!(failed.expect_err("the extrusion").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(refusal(&doc), "fillet R100.00 failed on 1 edges", "the refusal was {}", refusal(&doc));
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-linear-array").count(), 0, "the held extrusion wrote a row");
    near(live(&doc, block), 6000.0, "the extrusion beside the refusal");
}

fn standing_box() -> Cuboid {
    Cuboid { length: BoxLength { mm: 10.0 }, width: Width { mm: 20.0 }, height: Height { mm: 30.0 } }
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

fn row(body: BodyId, along: f64, n: u32) -> Linear {
    Linear { body, step: Step { x: AlongX { mm: along }, y: AlongY { mm: 0.0 }, z: AlongZ { mm: 0.0 } }, copies: Copies { n } }
}

fn turn_across(session: &mut Session) -> BodyId {
    let across = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, -5.0) }, to: To { at: (10.0, 15.0) } }).expect("a rectangle across the axis");
    session.revolve(across, SketchAxis::X, Angle { deg: 360.0 }).expect("the node is written")
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
