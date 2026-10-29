use qymcad_drive::{
    Angle, BodyId, BoxLength, Corners, Cuboid, Dimension, EdgeId, Expression, FaceId, FaceRole, From, Height, Kind, ParameterName, Plate, Radius, Rebuild, Session, SketchAxis, Sound, Thicken, To,
    Volume, Width, WorldPlane,
};

/// The end cap of the 10 by 20 by 30 mm box grows a plate 2 mm thick, and the volume grows by that plate.
#[test]
fn the_end_cap_grows_a_plate_two_millimetres_thick() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    let before = session.document();
    let cap = end_cap(&before, block);
    let made = session.thicken(plated(block, cap, Plate { mm: 2.0 })).expect("a plate");
    let doc = session.document();
    near(live(&doc, made), grown_cap(Height { mm: 30.0 }, Plate { mm: 2.0 }), "the body");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.components.iter().filter(|item| item.kind == Kind::Part).count(), 1, "one part");
    assert_ne!(made, block, "the thickened body is the extrusion");
    assert_eq!(plates(&doc), 1, "the thicken node");
}

/// A thickness of -2 mm puts the plate inside the body. The volume does not change.
#[test]
fn a_plate_inside_the_body_adds_nothing() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    let before = session.document();
    let cap = end_cap(&before, block);
    let made = session.thicken(plated(block, cap, Plate { mm: -2.0 })).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "the plate went into the body and added nothing", "the refusal was {}", refusal(&doc));
    near(live(&doc, made), Volume { mm3: 6000.0 }, "the plate");
    near(live(&doc, block), Volume { mm3: 6000.0 }, "the extrusion");
}

/// A thickness of 0, a thickness that is not finite, and a face id of 0 write no node.
#[test]
fn a_bad_thickness_or_a_missing_face_id_writes_no_plate() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    let before = session.document();
    let cap = end_cap(&before, block);
    let zero = session.thicken(plated(block, cap, Plate { mm: 0.0 }));
    assert_eq!(zero.expect_err("zero").text, "error-zero-thickness");
    for mm in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let failed = session.thicken(plated(block, cap, Plate { mm }));
        assert_eq!(failed.expect_err("not finite").text, "error-thicken-thickness", "a thickness of {mm}");
    }
    let missing = session.thicken(plated(block, FaceId { id: 0 }, Plate { mm: 2.0 }));
    assert_eq!(missing.expect_err("the face").text, "error-face-not-found");
    let held = session.document();
    assert_eq!(plates(&held), 0, "a refused thickness or face wrote a thicken");
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
    let made = session.thicken(plated(block, cap, Plate { mm: 2.0 })).expect("a plate");
    let doc = session.document();
    near(live(&doc, made), grown_cap(Height { mm: 30.0 }, Plate { mm: 2.0 }), "the body");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// A face id of 1 writes one node. A thickness of 0, and a thickness that is not finite, do not rewrite it.
#[test]
fn a_missing_face_is_replaced_by_the_cap() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    let before = session.document();
    let cap = end_cap(&before, block);
    let failed = session.thicken(plated(block, FaceId { id: 1 }, Plate { mm: 2.0 })).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-face-not-found", "the refusal was {}", refusal(&doc));
    near(live(&doc, failed), Volume { mm3: 6000.0 }, "the missing face");
    near(live(&doc, block), Volume { mm3: 6000.0 }, "the extrusion");
    let zero = session.thicken(plated(block, cap, Plate { mm: 0.0 }));
    assert_eq!(zero.expect_err("zero").text, "error-zero-thickness");
    for mm in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let held = session.thicken(plated(block, cap, Plate { mm }));
        assert_eq!(held.expect_err("not finite").text, "error-thicken-thickness", "a thickness of {mm}");
    }
    let held = session.document();
    assert_eq!(refusal(&held), "error-face-not-found", "the refusal was {}", refusal(&held));
    assert_eq!(plates(&held), 1, "the refused thicken was rewritten");
    let made = session.thicken(plated(block, cap, Plate { mm: 2.0 })).expect("the plate is tried again");
    assert_eq!(made, failed, "a second thicken was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(plates(&doc), 1, "the refused thicken was left beside the new one");
    near(live(&doc, made), grown_cap(Height { mm: 30.0 }, Plate { mm: 2.0 }), "the body");
}

/// A plate inside the body is replaced when the extrusion is named again. A fillet beside it writes nothing.
#[test]
fn a_plate_inside_the_body_is_replaced_when_the_extrusion_is_named_again() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    let before = session.document();
    let cap = end_cap(&before, block);
    let failed = session.thicken(plated(block, cap, Plate { mm: -2.0 })).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "the plate went into the body and added nothing", "the refusal was {}", refusal(&doc));
    near(live(&doc, failed), Volume { mm3: 6000.0 }, "the plate");
    near(live(&doc, block), Volume { mm3: 6000.0 }, "the extrusion");
    let zero = session.thicken(plated(block, cap, Plate { mm: 0.0 }));
    assert_eq!(zero.expect_err("zero").text, "error-zero-thickness");
    let infinite = session.thicken(plated(block, cap, Plate { mm: f64::NAN }));
    assert_eq!(infinite.expect_err("not finite").text, "error-thicken-thickness");
    let held = session.document();
    assert_eq!(refusal(&held), "the plate went into the body and added nothing", "the refusal was {}", refusal(&held));
    assert_eq!(plates(&held), 1, "the thicken was rewritten");
    let edge = long_edge(&held, block);
    let beside = session.fillet(block, edge, Radius { mm: 2.0 });
    assert_eq!(beside.expect_err("the fillet").text, "msg-body-consumed");
    let held = session.document();
    assert_eq!(refusal(&held), "the plate went into the body and added nothing", "the refusal was {}", refusal(&held));
    assert_eq!(plates(&held), 1, "the thicken was rewritten");
    assert!(held.nodes.iter().all(|node| node.name != "feat-name-fillet"), "a fillet was written");
    let made = session.thicken(plated(block, cap, Plate { mm: 2.0 })).expect("the plate is tried again");
    assert_eq!(made, failed, "a second thicken was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, made), grown_cap(Height { mm: 30.0 }, Plate { mm: 2.0 }), "the body");
    let again = session.thicken(plated(block, cap, Plate { mm: 2.0 }));
    assert_eq!(again.expect_err("the extrusion").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(plates(&doc), 1, "a second thicken was written");
    near(live(&doc, made), grown_cap(Height { mm: 30.0 }, Plate { mm: 2.0 }), "the body");
}

/// The id a refused thicken returned is that same thicken.
#[test]
fn a_plate_inside_the_body_named_by_its_own_id_is_the_same_thicken() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    let before = session.document();
    let cap = end_cap(&before, block);
    let failed = session.thicken(plated(block, cap, Plate { mm: -2.0 })).expect("the node is written");
    let _doc = session.document();
    let made = session.thicken(plated(failed, cap, Plate { mm: 2.0 })).expect("the plate is tried again");
    assert_eq!(made, failed, "a second thicken was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(plates(&doc), 1, "the refused thicken was left beside the new one");
    near(live(&doc, made), grown_cap(Height { mm: 30.0 }, Plate { mm: 2.0 }), "the body");
}

/// An id of 0 and a sketch id write no node. The real plate of the extrusion is still 2 mm.
#[test]
fn a_missing_body_is_refused_before_a_plate() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    let before = session.document();
    let cap = end_cap(&before, block);
    let missing = session.thicken(plated(BodyId { id: 0 }, cap, Plate { mm: 2.0 }));
    assert_eq!(missing.expect_err("id 0").text, "error-source-body-not-built");
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 10.0) } }).expect("a rectangle");
    let sketch = session.thicken(plated(BodyId { id: drawn.sketch.id }, cap, Plate { mm: 2.0 }));
    assert_eq!(sketch.expect_err("a sketch").text, "error-source-body-deleted");
    let held = session.document();
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
    assert_eq!(plates(&held), 0, "a refused id wrote a thicken");
    let made = session.thicken(plated(block, cap, Plate { mm: 2.0 })).expect("a plate");
    let doc = session.document();
    near(live(&doc, made), grown_cap(Height { mm: 30.0 }, Plate { mm: 2.0 }), "the body");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// The extrusion is consumed as soon as the thicken is written, so a second thicken of it writes nothing.
#[test]
fn a_second_plate_of_the_extrusion_is_refused() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    let before = session.document();
    let cap = end_cap(&before, block);
    let made = session.thicken(plated(block, cap, Plate { mm: 2.0 })).expect("a plate");
    let again = session.thicken(plated(block, cap, Plate { mm: 2.0 }));
    assert_eq!(again.expect_err("the extrusion").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(plates(&doc), 1, "a second thicken was written");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, made), grown_cap(Height { mm: 30.0 }, Plate { mm: 2.0 }), "the body");
}

/// A turn across its axis is not a thicken source. The extrusion stays 6000 mm^3 and the crossing refusal stays alone.
#[test]
fn a_turn_across_the_axis_is_not_a_plate_source() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    let before = session.document();
    let cap = end_cap(&before, block);
    let across = turn_across(&mut session);
    let failed = session.thicken(plated(across, cap, Plate { mm: 2.0 }));
    assert_eq!(failed.expect_err("the turn").text, "error-source-body-not-built");
    assert_ne!(across, block, "the refused turn is the extrusion");
    let doc = session.document();
    assert_eq!(plates(&doc), 0, "the turn wrote a thicken");
    assert_eq!(refusal(&doc), "error-revolve-profile-crosses-axis", "the refusal was {}", refusal(&doc));
    near(live(&doc, block), Volume { mm3: 6000.0 }, "the extrusion");
}

/// A fillet the rebuild has refused still holds its extrusion. Thickening that extrusion writes nothing.
#[test]
fn a_refused_fillet_holds_the_extrusion() {
    let mut session = Session::empty();
    let block = extrusion_held_by_a_refused_fillet(&mut session);
    let doc = session.document();
    assert_eq!(refusal(&doc), "fillet R100.00 failed on 1 edges", "the refusal was {}", refusal(&doc));
    let cap = end_cap(&doc, block);
    let failed = session.thicken(plated(block, cap, Plate { mm: 2.0 }));
    assert_eq!(failed.expect_err("the plate").text, "msg-body-consumed");
    let held = session.document();
    assert_eq!(refusal(&held), "fillet R100.00 failed on 1 edges", "the refusal was {}", refusal(&held));
    assert_eq!(plates(&held), 0, "a thicken was written");
    near(live(&held, block), Volume { mm3: 6000.0 }, "the extrusion");
}

/// A plate that adds nothing has not been read. The next box is not united with it, and the read stays that plate.
#[test]
fn an_unread_plate_that_adds_nothing_is_not_joined_to_the_next_box() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    let before = session.document();
    let cap = end_cap(&before, block);
    let _plated = session.thicken(plated(block, cap, Plate { mm: -2.0 })).expect("the node is written");
    let small = session.cuboid(Cuboid { length: BoxLength { mm: 2.0 }, width: Width { mm: 2.0 }, height: Height { mm: 2.0 } }).expect("a box");
    let doc = session.document();
    assert_eq!(refusal(&doc), "the plate went into the body and added nothing", "the refusal was {}", refusal(&doc));
    assert_eq!(unions(&doc), 0, "the next box joined the plate");
    near(live(&doc, small), Volume { mm3: 8.0 }, "the next box");
    near(live(&doc, block), Volume { mm3: 6000.0 }, "the extrusion");
}

/// A plate that has not been read still unites with the next box. The part stays one body.
#[test]
fn an_unread_plate_unites_with_the_next_box() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    let before = session.document();
    let cap = end_cap(&before, block);
    let plated = session.thicken(plated(block, cap, Plate { mm: 2.0 })).expect("a plate");
    let joined = session.cuboid(Cuboid { length: BoxLength { mm: 2.0 }, width: Width { mm: 2.0 }, height: Height { mm: 2.0 } }).expect("a box");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_ne!(joined, plated, "the union is the plate");
    assert_eq!(doc.nodes.iter().find(|node| node.id == joined.id).expect("the union").name, "feat-name-body-union");
    let volume = live(&doc, joined);
    let grown_out = grown_cap(Height { mm: 30.0 }, Plate { mm: 2.0 }).mm3;
    assert!(volume > grown_out && volume < grown_out + 8.0, "volume {volume} mm^3, the box was not united with the plate");
}

/// The thickened body follows the extrusion height. A parameter named thickness does not change the stored thickness.
#[test]
fn the_thickened_body_follows_the_height() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    session.parameter(ParameterName { text: "span".into() }, Expression { text: "30".into() }).expect("a parameter");
    session.bind(block, Dimension::Height, Expression { text: "span".into() }).expect("the height");
    let before = session.document();
    let cap = end_cap(&before, block);
    let made = session.thicken(plated(block, cap, Plate { mm: 2.0 })).expect("a plate");
    let at_thirty = session.document();
    near(live(&at_thirty, made), grown_cap(Height { mm: 30.0 }, Plate { mm: 2.0 }), "the body at 30 mm");
    assert_eq!(at_thirty.rebuild, Rebuild::Quiet, "{}", refusal(&at_thirty));
    session.parameter(ParameterName { text: "span".into() }, Expression { text: "20".into() }).expect("the height");
    let at_twenty = session.document();
    near(live(&at_twenty, made), grown_cap(Height { mm: 20.0 }, Plate { mm: 2.0 }), "the body at 20 mm");
    assert_eq!(at_twenty.rebuild, Rebuild::Quiet, "{}", refusal(&at_twenty));
    session.parameter(ParameterName { text: "thickness".into() }, Expression { text: "9".into() }).expect("a parameter named thickness");
    let held = session.document();
    near(live(&held, made), grown_cap(Height { mm: 20.0 }, Plate { mm: 2.0 }), "the thickness stays 2 mm");
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
}

fn grown(session: &mut Session) -> BodyId {
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion")
}

fn end_cap(doc: &qymcad_drive::Document, block: BodyId) -> FaceId {
    body(doc, block).faces.iter().find(|face| face.role == FaceRole::CapEnd).expect("the end cap").id
}

fn plated(body: BodyId, face: FaceId, plate: Plate) -> Thicken {
    Thicken { body, face, plate }
}

/// The 10 by 20 mm footprint grown by `height`, then by the plate on its end cap.
fn grown_cap(height: Height, plate: Plate) -> Volume {
    Volume { mm3: 200.0 * (height.mm + plate.mm) }
}

fn plates(doc: &qymcad_drive::Document) -> usize {
    doc.nodes.iter().filter(|node| node.name == "feat-name-thicken").count()
}

fn unions(doc: &qymcad_drive::Document) -> usize {
    doc.nodes.iter().filter(|node| node.name == "feat-name-body-union").count()
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
    body(doc, block).edges.iter().find(|edge| (edge.length.mm - 20.0).abs() < 0.1).expect("the 20 mm edge").id
}

fn turn_across(session: &mut Session) -> BodyId {
    let across = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, -5.0) }, to: To { at: (10.0, 15.0) } }).expect("a rectangle across the axis");
    session.revolve(across, SketchAxis::X, Angle { deg: 360.0 }).expect("the node is written")
}

fn body(doc: &qymcad_drive::Document, id: BodyId) -> &qymcad_drive::Body {
    doc.bodies.iter().find(|body| body.id == id.id).unwrap_or_else(|| panic!("body {} is gone", id.id))
}

fn live(doc: &qymcad_drive::Document, id: BodyId) -> f64 {
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
