use qymcad_drive::{
    Angle, BodyId, BoxLength, Corners, Cuboid, Dimension, Draft, EdgeId, Expression, FaceId, FaceRole, From, Height, Kind, Neutral, ParameterName, Radius, Rebuild, Session, SketchAxis, Sound, Tilted,
    To, Volume, Width, WorldPlane,
};

/// One short wall of the 10 by 20 by 30 mm box tilts 10 degrees about the end cap, and the volume grows by that wedge.
#[test]
fn a_wall_leans_out_by_ten_degrees() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    let before = session.document();
    let chosen = faces_of(&before, block);
    let made = session.draft(lean(block, chosen, Angle { deg: 10.0 })).expect("a draft");
    let doc = session.document();
    near(live(&doc, made), outward(Height { mm: 30.0 }), "the body");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.components.iter().filter(|item| item.kind == Kind::Part).count(), 1, "one part");
    assert_ne!(made, block, "the drafted body is the extrusion");
    assert_eq!(drafts(&doc), 1, "the draft node");
}

/// A fresh extrusion drafted by -10 degrees leans in by the same wedge.
#[test]
fn a_negative_angle_leans_in() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    let before = session.document();
    let chosen = faces_of(&before, block);
    let made = session.draft(lean(block, chosen, Angle { deg: -10.0 })).expect("a draft");
    let doc = session.document();
    near(live(&doc, made), inward(Height { mm: 30.0 }), "the body");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// An angle of 0, an angle that is not finite, a face id of 0, and the same face as both write no node.
#[test]
fn a_bad_angle_or_the_same_face_writes_no_draft() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    let before = session.document();
    let chosen = faces_of(&before, block);
    let zero = session.draft(lean(block, chosen, Angle { deg: 0.0 }));
    assert_eq!(zero.expect_err("zero").text, "error-draft-angle-zero");
    for deg in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let failed = session.draft(lean(block, chosen, Angle { deg }));
        assert_eq!(failed.expect_err("not finite").text, "error-draft-angle", "an angle of {deg}");
    }
    let wall = session.draft(Draft { body: block, tilted: Tilted { face: FaceId { id: 0 } }, neutral: Neutral { face: chosen.neutral }, angle: Angle { deg: 10.0 } });
    assert_eq!(wall.expect_err("the wall").text, "error-face-not-found");
    let cap = session.draft(Draft { body: block, tilted: Tilted { face: chosen.tilted }, neutral: Neutral { face: FaceId { id: 0 } }, angle: Angle { deg: 10.0 } });
    assert_eq!(cap.expect_err("the neutral face").text, "error-face-not-found");
    let same = session.draft(Draft { body: block, tilted: Tilted { face: chosen.neutral }, neutral: Neutral { face: chosen.neutral }, angle: Angle { deg: 10.0 } });
    assert_eq!(same.expect_err("the same face").text, "error-draft-needs-faces");
    let held = session.document();
    assert_eq!(drafts(&held), 0, "a refused angle or face wrote a draft");
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
    let made = session.draft(lean(block, chosen, Angle { deg: 10.0 })).expect("a draft");
    let doc = session.document();
    near(live(&doc, made), outward(Height { mm: 30.0 }), "the body");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// A face id of 1 writes one node. An angle of 0 does not rewrite it. The real wall does.
#[test]
fn a_missing_face_is_replaced_by_the_wall() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    let before = session.document();
    let chosen = faces_of(&before, block);
    let failed = session.draft(Draft { body: block, tilted: Tilted { face: FaceId { id: 1 } }, neutral: Neutral { face: chosen.neutral }, angle: Angle { deg: 10.0 } }).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-draft-needs-faces", "the refusal was {}", refusal(&doc));
    near(live(&doc, failed), Volume { mm3: 6000.0 }, "the missing face");
    near(live(&doc, block), Volume { mm3: 6000.0 }, "the extrusion");
    let zero = session.draft(lean(block, chosen, Angle { deg: 0.0 }));
    assert_eq!(zero.expect_err("zero").text, "error-draft-angle-zero");
    let held = session.document();
    assert_eq!(refusal(&held), "error-draft-needs-faces", "the refusal was {}", refusal(&held));
    assert_eq!(drafts(&held), 1, "the refused draft was rewritten");
    let made = session.draft(lean(block, chosen, Angle { deg: 10.0 })).expect("the draft is tried again");
    assert_eq!(made, failed, "a second draft was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(drafts(&doc), 1, "the refused draft was left beside the new one");
    near(live(&doc, made), outward(Height { mm: 30.0 }), "the body");
}

/// An angle of 90 degrees is replaced when the extrusion is named again. A fillet beside it writes nothing.
#[test]
fn a_right_angle_is_replaced_when_the_extrusion_is_named_again() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    let before = session.document();
    let chosen = faces_of(&before, block);
    let failed = session.draft(lean(block, chosen, Angle { deg: 90.0 })).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "draft 90.0 failed", "the refusal was {}", refusal(&doc));
    near(live(&doc, block), Volume { mm3: 6000.0 }, "the extrusion");
    let edge = long_edge(&doc, block);
    let beside = session.fillet(block, edge, Radius { mm: 2.0 });
    assert_eq!(beside.expect_err("the fillet").text, "msg-body-consumed");
    let held = session.document();
    assert_eq!(refusal(&held), "draft 90.0 failed", "the refusal was {}", refusal(&held));
    assert_eq!(drafts(&held), 1, "the draft was rewritten");
    assert!(held.nodes.iter().all(|node| node.name != "feat-name-fillet"), "a fillet was written");
    let made = session.draft(lean(block, chosen, Angle { deg: 10.0 })).expect("the draft is tried again");
    assert_eq!(made, failed, "a second draft was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, made), outward(Height { mm: 30.0 }), "the body");
    let again = session.draft(lean(block, chosen, Angle { deg: 10.0 }));
    assert_eq!(again.expect_err("the extrusion").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(drafts(&doc), 1, "a second draft was written");
    near(live(&doc, made), outward(Height { mm: 30.0 }), "the body");
}

/// The id a refused draft returned is that same draft.
#[test]
fn a_right_angle_named_by_its_own_id_is_the_same_draft() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    let before = session.document();
    let chosen = faces_of(&before, block);
    let failed = session.draft(lean(block, chosen, Angle { deg: 90.0 })).expect("the node is written");
    let _doc = session.document();
    let made = session.draft(lean(failed, chosen, Angle { deg: 10.0 })).expect("the draft is tried again");
    assert_eq!(made, failed, "a second draft was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(drafts(&doc), 1, "the refused draft was left beside the new one");
    near(live(&doc, made), outward(Height { mm: 30.0 }), "the body");
}

/// An id of 0 and a sketch id write no node. The real draft of the extrusion is still the 10 degree wedge.
#[test]
fn a_missing_body_is_refused_before_a_draft() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    let before = session.document();
    let chosen = faces_of(&before, block);
    let missing = session.draft(lean(BodyId { id: 0 }, chosen, Angle { deg: 10.0 }));
    assert_eq!(missing.expect_err("id 0").text, "error-source-body-not-built");
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 10.0) } }).expect("a rectangle");
    let sketch = session.draft(lean(BodyId { id: drawn.sketch.id }, chosen, Angle { deg: 10.0 }));
    assert_eq!(sketch.expect_err("a sketch").text, "error-source-body-deleted");
    let held = session.document();
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
    assert_eq!(drafts(&held), 0, "a refused id wrote a draft");
    let made = session.draft(lean(block, chosen, Angle { deg: 10.0 })).expect("a draft");
    let doc = session.document();
    near(live(&doc, made), outward(Height { mm: 30.0 }), "the body");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// A draft the wall cannot take has not been read. The next box is not united with it, and the read stays that draft.
#[test]
fn an_unread_draft_the_wall_cannot_take_is_not_joined_to_the_next_box() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    let before = session.document();
    let chosen = faces_of(&before, block);
    let _drafted = session.draft(lean(block, chosen, Angle { deg: 90.0 })).expect("the node is written");
    let small = session.cuboid(Cuboid { length: BoxLength { mm: 2.0 }, width: Width { mm: 2.0 }, height: Height { mm: 2.0 } }).expect("a box");
    let doc = session.document();
    assert_eq!(refusal(&doc), "draft 90.0 failed", "the refusal was {}", refusal(&doc));
    assert_eq!(unions(&doc), 0, "the next box joined the draft");
    near(live(&doc, small), Volume { mm3: 8.0 }, "the next box");
    near(live(&doc, block), Volume { mm3: 6000.0 }, "the extrusion");
}

/// A draft that has not been read still unites with the next box. The part stays one body.
#[test]
fn an_unread_draft_unites_with_the_next_box() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    let before = session.document();
    let chosen = faces_of(&before, block);
    let drafted = session.draft(lean(block, chosen, Angle { deg: 10.0 })).expect("a draft");
    let joined = session.cuboid(Cuboid { length: BoxLength { mm: 2.0 }, width: Width { mm: 2.0 }, height: Height { mm: 2.0 } }).expect("a box");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_ne!(joined, drafted, "the union is the draft");
    assert_eq!(doc.nodes.iter().find(|node| node.id == joined.id).expect("the union").name, "feat-name-body-union");
    let volume = live(&doc, joined);
    let wedge = outward(Height { mm: 30.0 }).mm3;
    assert!(volume > wedge && volume < wedge + 8.0, "volume {volume} mm^3, the box was not united with the draft");
}

/// The extrusion is consumed as soon as the draft is written, so a second draft of it writes nothing.
#[test]
fn a_second_draft_of_the_extrusion_is_refused() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    let before = session.document();
    let chosen = faces_of(&before, block);
    let made = session.draft(lean(block, chosen, Angle { deg: 10.0 })).expect("a draft");
    let again = session.draft(lean(block, chosen, Angle { deg: 10.0 }));
    assert_eq!(again.expect_err("the extrusion").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(drafts(&doc), 1, "a second draft was written");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, made), outward(Height { mm: 30.0 }), "the body");
}

/// A turn across its axis is not a draft source. The extrusion stays 6000 mm^3 and the crossing refusal stays alone.
#[test]
fn a_turn_across_the_axis_is_not_a_draft_source() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    let before = session.document();
    let chosen = faces_of(&before, block);
    let across = turn_across(&mut session);
    let failed = session.draft(lean(across, chosen, Angle { deg: 10.0 }));
    assert_eq!(failed.expect_err("the turn").text, "error-source-body-not-built");
    assert_ne!(across, block, "the refused turn is the extrusion");
    let doc = session.document();
    assert_eq!(drafts(&doc), 0, "the turn wrote a draft");
    assert_eq!(refusal(&doc), "error-revolve-profile-crosses-axis", "the refusal was {}", refusal(&doc));
    near(live(&doc, block), Volume { mm3: 6000.0 }, "the extrusion");
}

/// A fillet the rebuild has refused still holds its extrusion. Drafting that extrusion writes nothing.
#[test]
fn a_refused_fillet_holds_the_extrusion() {
    let mut session = Session::empty();
    let block = extrusion_held_by_a_refused_fillet(&mut session);
    let doc = session.document();
    assert_eq!(refusal(&doc), "fillet R100.00 failed on 1 edges", "the refusal was {}", refusal(&doc));
    let chosen = faces_of(&doc, block);
    let failed = session.draft(lean(block, chosen, Angle { deg: 10.0 }));
    assert_eq!(failed.expect_err("the draft").text, "msg-body-consumed");
    let held = session.document();
    assert_eq!(refusal(&held), "fillet R100.00 failed on 1 edges", "the refusal was {}", refusal(&held));
    assert_eq!(drafts(&held), 0, "a draft was written");
    near(live(&held, block), Volume { mm3: 6000.0 }, "the extrusion");
}

/// The drafted body follows the extrusion height. A parameter named angle does not change the stored angle.
#[test]
fn the_drafted_body_follows_the_height() {
    let mut session = Session::empty();
    let block = grown(&mut session);
    session.parameter(ParameterName { text: "span".into() }, Expression { text: "30".into() }).expect("a parameter");
    session.bind(block, Dimension::Height, Expression { text: "span".into() }).expect("the height");
    let before = session.document();
    let chosen = faces_of(&before, block);
    let made = session.draft(lean(block, chosen, Angle { deg: 10.0 })).expect("a draft");
    let at_thirty = session.document();
    near(live(&at_thirty, made), outward(Height { mm: 30.0 }), "the body at 30 mm");
    assert_eq!(at_thirty.rebuild, Rebuild::Quiet, "{}", refusal(&at_thirty));
    session.parameter(ParameterName { text: "span".into() }, Expression { text: "20".into() }).expect("the height");
    let at_twenty = session.document();
    near(live(&at_twenty, made), outward(Height { mm: 20.0 }), "the body at 20 mm");
    assert_eq!(at_twenty.rebuild, Rebuild::Quiet, "{}", refusal(&at_twenty));
    session.parameter(ParameterName { text: "angle".into() }, Expression { text: "45".into() }).expect("a parameter named angle");
    let held = session.document();
    near(live(&held, made), outward(Height { mm: 20.0 }), "the angle stays 10 degrees");
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
}

#[derive(Clone, Copy)]
struct Chosen {
    tilted: FaceId,
    neutral: FaceId,
}

fn grown(session: &mut Session) -> BodyId {
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion")
}

fn faces_of(doc: &qymcad_drive::Document, block: BodyId) -> Chosen {
    let found = body(doc, block);
    let neutral = found.faces.iter().find(|face| face.role == FaceRole::CapEnd).expect("the end cap").id;
    let tilted = found.faces.iter().find(|face| (face.area.mm2 - 300.0).abs() < 1.0).expect("the short wall").id;
    Chosen { tilted, neutral }
}

fn lean(body: BodyId, chosen: Chosen, angle: Angle) -> Draft {
    Draft { body, tilted: Tilted { face: chosen.tilted }, neutral: Neutral { face: chosen.neutral }, angle }
}

/// The box of a 10 by 20 mm footprint at `height`, plus the wedge of its 10 mm hinge at 10 degrees.
fn outward(height: Height) -> Volume {
    Volume { mm3: 200.0 * height.mm + 5.0 * height.mm * height.mm * 10.0_f64.to_radians().tan() }
}

/// The same box with that wedge taken away.
fn inward(height: Height) -> Volume {
    Volume { mm3: 200.0 * height.mm - 5.0 * height.mm * height.mm * 10.0_f64.to_radians().tan() }
}

fn drafts(doc: &qymcad_drive::Document) -> usize {
    doc.nodes.iter().filter(|node| node.name == "feat-name-draft").count()
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
