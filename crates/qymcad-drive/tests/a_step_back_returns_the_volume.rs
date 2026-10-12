use qymcad_drive::{BodyId, ContourId, Corners, Dimension, Expression, Height, Kind, Measure, ParameterName, Plane, Profile, Rebuild, Session, SketchId, Sound, WorldPlane};

/// Changing the height expression changes the volume, a step back returns the previous volume, and
/// resizing the sketch above the body changes the volume again.
#[test]
fn a_step_back_returns_the_previous_volume() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    assert_one_box(&session.document(), made, drawn, 6000.0);

    session.parameter(ParameterName { text: "span".into() }, Expression { text: "40".into() }).expect("a parameter");
    session.bind(made, Dimension::Height, Expression { text: "span/2".into() }).expect("the height expression");
    assert_one_box(&session.document(), made, drawn, 4000.0);

    session.bind(made, Dimension::Height, Expression { text: "span/4".into() }).expect("the height expression");
    assert_one_box(&session.document(), made, drawn, 2000.0);
    let refused = session.parameter(ParameterName { text: "span".into() }, Expression { text: "nope".into() }).expect_err("the name is not in the document");
    assert_eq!(refused.text, "error-expr-unknown-name");
    session.back().expect("a step back");
    assert_one_box(&session.document(), made, drawn, 4000.0);

    session.resize(drawn, rectangle_10_by_10()).expect("the rectangle moves");
    let shrunk = session.document();
    assert_one_box(&shrunk, made, drawn, 2000.0);
    assert_eq!(shrunk.sketches.len(), 1, "the resize does not add a sketch");
    session.back().expect("a step back");
    assert_one_box(&session.document(), made, drawn, 4000.0);
}

/// A step back before anything was kept is refused, and the box stays.
#[test]
fn a_step_back_before_any_change_is_refused() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    assert_one_box(&session.document(), made, drawn, 6000.0);
    let err = session.back().expect_err("nothing has been kept");
    assert_eq!(err.text, "sketch-undo-empty");
    assert_one_box(&session.document(), made, drawn, 6000.0);
}

/// A resize that is refused keeps no step. One step back then returns the box from before the
/// height expression, 6000 mm³. A kept refusal would leave the read at 4000 mm³.
#[test]
fn a_refused_resize_keeps_no_step() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    session.parameter(ParameterName { text: "span".into() }, Expression { text: "40".into() }).expect("a parameter");
    session.bind(made, Dimension::Height, Expression { text: "span/2".into() }).expect("the height expression");
    assert_one_box(&session.document(), made, drawn, 4000.0);

    let empty = session.resize(drawn, Corners { from: qymcad_drive::From { at: (0.0, 0.0) }, to: qymcad_drive::To { at: (10.0, 0.0) } }).expect_err("the corners do not span a rectangle");
    assert_eq!(empty.text, "sketch-rectangle-empty");
    let foreign = session.resize(Profile { sketch: SketchId { id: 0 }, contour: drawn.contour }, rectangle_10_by_10()).expect_err("the profile is not a closed contour of that sketch");
    assert_eq!(foreign.text, "error-profile-not-found");
    let other = session.resize(Profile { sketch: drawn.sketch, contour: ContourId { id: 0 } }, rectangle_10_by_10()).expect_err("the profile is not a closed contour of that sketch");
    assert_eq!(other.text, "error-profile-not-found");
    session.back().expect("a step back");
    assert_one_box(&session.document(), made, drawn, 6000.0);
}

fn assert_one_box(doc: &qymcad_drive::Document, made: BodyId, drawn: Profile, want: f64) {
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert_eq!(doc.bodies.len(), 1, "one body");
    let body = &doc.bodies[0];
    assert_eq!(body.id, made.id, "the body id is the one the extrusion returned");
    assert_eq!(body.sound, Sound::Valid, "the shape is valid");
    match body.measure {
        Measure::Live { volume, solids } => {
            assert_eq!(solids, 1, "one solid");
            assert!((volume.mm3 - want).abs() < 1.0, "volume {} mm^3, wanted {want} within 1", volume.mm3);
        }
        Measure::Absent => panic!("the body has no live shape"),
    }
    assert_eq!(doc.components.iter().filter(|c| c.kind == Kind::Part).count(), 1, "one part");
    assert!(doc.sketches.iter().any(|s| s.id == drawn.sketch.id && s.plane == Plane::Xz), "the sketch stays on Xz");
}

fn rectangle_10_by_20() -> Corners {
    Corners { from: qymcad_drive::From { at: (0.0, 0.0) }, to: qymcad_drive::To { at: (10.0, 20.0) } }
}

fn rectangle_10_by_10() -> Corners {
    Corners { from: qymcad_drive::From { at: (0.0, 0.0) }, to: qymcad_drive::To { at: (10.0, 10.0) } }
}
