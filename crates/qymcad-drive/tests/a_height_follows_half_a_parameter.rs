use qymcad_drive::{BodyId, Corners, Dimension, Expression, Height, Kind, Measure, ParameterName, Plane, Rebuild, Session, Sound, WorldPlane};

/// Half of a parameter is the extrusion height. 40 mm gives 4000 mm³, 80 mm gives 8000 mm³, and the
/// same parameter under a different letter case is still that parameter.
#[test]
fn a_height_follows_half_a_parameter() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    assert_one_box(&session.document(), made, 6000.0);

    session.parameter(ParameterName { text: "span".into() }, Expression { text: "40".into() }).expect("a parameter");
    session.bind(made, Dimension::Height, Expression { text: "span/2".into() }).expect("the height expression");
    let half = session.document();
    assert_one_box(&half, made, 4000.0);

    session.parameter(ParameterName { text: "span".into() }, Expression { text: "80".into() }).expect("the same parameter");
    let doubled = session.document();
    assert_one_box(&doubled, made, 8000.0);

    session.parameter(ParameterName { text: "Span".into() }, Expression { text: "40".into() }).expect("the same parameter in another case");
    let again = session.document();
    assert_one_box(&again, made, 4000.0);
}

/// A name the document does not have is refused. The volume staying at 6000 mm³ is not the proof: a
/// stored expression that fails to evaluate falls back to the stored height. Creating the missing
/// parameter afterwards would change the volume if the expression had been stored.
#[test]
fn a_missing_name_is_not_stored_on_the_height() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    assert_one_box(&session.document(), made, 6000.0);

    let err = session.bind(made, Dimension::Height, Expression { text: "missing/2".into() }).expect_err("the name is not in the document");
    assert_eq!(err.text, "error-expr-unknown-name");
    session.parameter(ParameterName { text: "missing".into() }, Expression { text: "10".into() }).expect("a parameter");
    assert_one_box(&session.document(), made, 6000.0);
}

/// A height expression that is not a positive length is refused before it is stored. The model takes
/// the absolute value, so a negative height would rebuild as the positive solid and stay quiet. The
/// same refusal applies when a later parameter value would make that height negative, including when
/// the height reads the parameter through another parameter. A formula that mentions the parameter
/// itself is refused too: evaluation repeats it, and the stored number is not the one that was checked.
#[test]
fn a_height_expression_that_is_not_positive_is_refused() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    assert_one_box(&session.document(), made, 6000.0);

    for formula in ["-15", "0"] {
        let err = session.bind(made, Dimension::Height, Expression { text: formula.into() }).expect_err("the height is not positive");
        assert_eq!(err.text, "sketch-extrude-height", "{formula}");
        assert_one_box(&session.document(), made, 6000.0);
    }

    session.parameter(ParameterName { text: "span".into() }, Expression { text: "40".into() }).expect("a parameter");
    session.bind(made, Dimension::Height, Expression { text: "span/2".into() }).expect("the height expression");
    assert_one_box(&session.document(), made, 4000.0);

    let negative = session.parameter(ParameterName { text: "span".into() }, Expression { text: "-80".into() }).expect_err("the height would not be positive");
    assert_eq!(negative.text, "sketch-extrude-height");
    assert_one_box(&session.document(), made, 4000.0);

    let itself = session.parameter(ParameterName { text: "span".into() }, Expression { text: "span*2".into() }).expect_err("the formula mentions itself");
    assert_eq!(itself.text, "error-expr-unknown-name");
    assert_one_box(&session.document(), made, 4000.0);

    session.parameter(ParameterName { text: "half".into() }, Expression { text: "span/2".into() }).expect("a parameter");
    session.bind(made, Dimension::Height, Expression { text: "half".into() }).expect("the height expression");
    assert_one_box(&session.document(), made, 4000.0);
    let through = session.parameter(ParameterName { text: "span".into() }, Expression { text: "-80".into() }).expect_err("the height would not be positive");
    assert_eq!(through.text, "sketch-extrude-height");
    assert_one_box(&session.document(), made, 4000.0);
}

/// A parameter change that makes the height formula not a number is refused. The rebuild would
/// otherwise drop that formula and use the height stored on the node, so the solid would become the
/// original 30 mm box and the rebuild would stay quiet.
#[test]
fn a_height_formula_that_stops_being_a_number_is_refused() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    session.parameter(ParameterName { text: "other".into() }, Expression { text: "2".into() }).expect("a parameter");
    session.bind(made, Dimension::Height, Expression { text: "100/other".into() }).expect("the height expression");
    assert_one_box(&session.document(), made, 10000.0);

    let err = session.parameter(ParameterName { text: "other".into() }, Expression { text: "0".into() }).expect_err("the height would not be a positive length");
    assert_eq!(err.text, "sketch-extrude-height");
    assert_one_box(&session.document(), made, 10000.0);
}

/// The sketch is not an extrusion. Binding its id is refused.
#[test]
fn a_sketch_is_not_an_extrusion() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let err = session.bind(BodyId { id: drawn.sketch.id }, Dimension::Height, Expression { text: "span/2".into() }).expect_err("a sketch is not an extrusion");
    assert_eq!(err.text, "sketch-extrude-target");
    assert_one_box(&session.document(), made, 6000.0);
}

fn assert_one_box(doc: &qymcad_drive::Document, made: BodyId, want: f64) {
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
    assert!(doc.sketches.iter().any(|s| s.plane == Plane::Xz), "the sketch stays on Xz");
}

fn rectangle_10_by_20() -> Corners {
    Corners { from: qymcad_drive::From { at: (0.0, 0.0) }, to: qymcad_drive::To { at: (10.0, 20.0) } }
}
