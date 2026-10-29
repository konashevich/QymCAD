use qymcad_drive::{
    Angle, BodyId, BoxLength, Corners, Cuboid, Depth, Diameter, EdgeId, Expression, FaceId, FaceRole, From, Height, Kind, ParameterName, Radius, Rebuild, RemoveFace, Session, SketchAxis, Sound, To,
    Volume, Width, WorldPlane,
};

/// The wall of a 4 mm hole through the 10 by 20 by 30 mm block is taken away, and the volume returns to the block.
#[test]
fn the_hole_closes_when_its_wall_is_taken_away() {
    let mut session = Session::empty();
    let drilled = open_hole(&mut session);
    let doc = session.document();
    let wall = hole_wall(&doc, drilled);
    let made = session.remove_face(taken(drilled, wall)).expect("the wall is taken away");
    let closed = session.document();
    near(live(&closed, made), block(Height { mm: 30.0 }), "the body");
    assert_eq!(closed.rebuild, Rebuild::Quiet, "{}", refusal(&closed));
    assert_eq!(faces_of(&closed, made), 6, "the closed block");
    assert!(bare(&closed, made).is_none(), "the hole wall is still there");
    assert_eq!(closed.components.iter().filter(|item| item.kind == Kind::Part).count(), 1, "one part");
    assert_ne!(made, drilled, "the closed body is the hole");
    assert_eq!(removals(&closed), 1, "the removal node");
}

/// The end cap of the drilled block stays a cap. Taking it away does not close the hole.
#[test]
fn the_end_cap_of_the_drilled_body_does_not_close() {
    let mut session = Session::empty();
    let drilled = open_hole(&mut session);
    let doc = session.document();
    let cap = end_cap(&doc, drilled);
    let made = session.remove_face(taken(drilled, cap)).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "cannot remove faces: cad-neighbours-not-extendable", "the refusal was {}", refusal(&doc));
    near(live(&doc, made), opened(Height { mm: 30.0 }), "the cap");
    near(live(&doc, drilled), opened(Height { mm: 30.0 }), "the hole");
}

/// A face id of 0 writes no node. The real removal is still the restored block.
#[test]
fn a_face_id_of_zero_writes_no_removal() {
    let mut session = Session::empty();
    let drilled = open_hole(&mut session);
    let doc = session.document();
    let wall = hole_wall(&doc, drilled);
    let missing = session.remove_face(taken(drilled, FaceId { id: 0 }));
    assert_eq!(missing.expect_err("the face").text, "error-faces-not-found");
    let held = session.document();
    assert_eq!(removals(&held), 0, "a face id of 0 wrote a removal");
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
    let made = session.remove_face(taken(drilled, wall)).expect("the wall is taken away");
    let doc = session.document();
    near(live(&doc, made), block(Height { mm: 30.0 }), "the body");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// A face id of 1 writes one node. A face id of 0 does not rewrite it.
#[test]
fn a_missing_face_is_replaced_by_the_wall() {
    let mut session = Session::empty();
    let drilled = open_hole(&mut session);
    let doc = session.document();
    let wall = hole_wall(&doc, drilled);
    let failed = session.remove_face(taken(drilled, FaceId { id: 1 })).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-faces-not-found", "the refusal was {}", refusal(&doc));
    near(live(&doc, failed), opened(Height { mm: 30.0 }), "the missing face");
    near(live(&doc, drilled), opened(Height { mm: 30.0 }), "the hole");
    let zero = session.remove_face(taken(drilled, FaceId { id: 0 }));
    assert_eq!(zero.expect_err("the face").text, "error-faces-not-found");
    let held = session.document();
    assert_eq!(refusal(&held), "error-faces-not-found", "the refusal was {}", refusal(&held));
    assert_eq!(removals(&held), 1, "the refused removal was rewritten");
    let made = session.remove_face(taken(drilled, wall)).expect("the wall is tried again");
    assert_eq!(made, failed, "a second removal was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(removals(&doc), 1, "the refused removal was left beside the new one");
    assert_eq!(faces_of(&doc, made), 6, "the closed block");
    near(live(&doc, made), block(Height { mm: 30.0 }), "the body");
}

/// The end cap is replaced when the hole is named again. A fillet beside it writes nothing.
#[test]
fn the_end_cap_is_replaced_when_the_hole_is_named_again() {
    let mut session = Session::empty();
    let drilled = open_hole(&mut session);
    let doc = session.document();
    let cap = end_cap(&doc, drilled);
    let wall = hole_wall(&doc, drilled);
    let failed = session.remove_face(taken(drilled, cap)).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "cannot remove faces: cad-neighbours-not-extendable", "the refusal was {}", refusal(&doc));
    let zero = session.remove_face(taken(drilled, FaceId { id: 0 }));
    assert_eq!(zero.expect_err("the face").text, "error-faces-not-found");
    let held = session.document();
    assert_eq!(refusal(&held), "cannot remove faces: cad-neighbours-not-extendable", "the refusal was {}", refusal(&held));
    assert_eq!(removals(&held), 1, "the removal was rewritten");
    let edge = long_edge(&held, drilled);
    let beside = session.fillet(drilled, edge, Radius { mm: 2.0 });
    assert_eq!(beside.expect_err("the fillet").text, "msg-body-consumed");
    let held = session.document();
    assert_eq!(refusal(&held), "cannot remove faces: cad-neighbours-not-extendable", "the refusal was {}", refusal(&held));
    assert!(held.nodes.iter().all(|node| node.name != "feat-name-fillet"), "a fillet was written");
    let made = session.remove_face(taken(drilled, wall)).expect("the wall is tried again");
    assert_eq!(made, failed, "a second removal was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, made), block(Height { mm: 30.0 }), "the body");
    let again = session.remove_face(taken(drilled, wall));
    assert_eq!(again.expect_err("the hole").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(removals(&doc), 1, "a second removal was written");
    near(live(&doc, made), block(Height { mm: 30.0 }), "the body");
}

/// The id a refused removal returned is that same removal.
#[test]
fn a_cap_named_by_its_own_id_is_the_same_removal() {
    let mut session = Session::empty();
    let drilled = open_hole(&mut session);
    let doc = session.document();
    let cap = end_cap(&doc, drilled);
    let wall = hole_wall(&doc, drilled);
    let failed = session.remove_face(taken(drilled, cap)).expect("the node is written");
    let _doc = session.document();
    let made = session.remove_face(taken(failed, wall)).expect("the wall is tried again");
    assert_eq!(made, failed, "a second removal was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(removals(&doc), 1, "the refused removal was left beside the new one");
    near(live(&doc, made), block(Height { mm: 30.0 }), "the body");
    let again = open_hole(&mut session);
    let doc = session.document();
    let cap = end_cap(&doc, again);
    let wall = hole_wall(&doc, again);
    let other = session.remove_face(taken(again, cap)).expect("the node is written");
    let _held = session.document();
    let closed = session.remove_face(taken(other, wall)).expect("the wall is tried again");
    assert_eq!(closed, other, "a second removal was added");
    assert_ne!(closed, made, "the two retries are one body");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, closed), block(Height { mm: 30.0 }), "the other body");
    near(live(&doc, made), block(Height { mm: 30.0 }), "the first body");
}

/// An id of 0 and a sketch id write no node. The real removal of the hole is still the restored block.
#[test]
fn a_missing_body_is_refused_before_a_removal() {
    let mut session = Session::empty();
    let drilled = open_hole(&mut session);
    let doc = session.document();
    let wall = hole_wall(&doc, drilled);
    let missing = session.remove_face(taken(BodyId { id: 0 }, wall));
    assert_eq!(missing.expect_err("id 0").text, "error-source-body-not-built");
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 10.0) } }).expect("a rectangle");
    let sketch = session.remove_face(taken(BodyId { id: drawn.sketch.id }, wall));
    assert_eq!(sketch.expect_err("a sketch").text, "error-source-body-deleted");
    let held = session.document();
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
    assert_eq!(removals(&held), 0, "a refused id wrote a removal");
    let made = session.remove_face(taken(drilled, wall)).expect("the wall is taken away");
    let doc = session.document();
    near(live(&doc, made), block(Height { mm: 30.0 }), "the body");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// The hole is consumed as soon as the removal is written, so a second removal of it writes nothing.
#[test]
fn a_second_removal_of_the_hole_is_refused() {
    let mut session = Session::empty();
    let drilled = open_hole(&mut session);
    let doc = session.document();
    let wall = hole_wall(&doc, drilled);
    let made = session.remove_face(taken(drilled, wall)).expect("the wall is taken away");
    let again = session.remove_face(taken(drilled, wall));
    assert_eq!(again.expect_err("the hole").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(removals(&doc), 1, "a second removal was written");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, made), block(Height { mm: 30.0 }), "the body");
}

/// A turn across its axis is not a removal source. The drilled block stays open and the crossing refusal stays alone.
#[test]
fn a_turn_across_the_axis_is_not_a_removal_source() {
    let mut session = Session::empty();
    let drilled = open_hole(&mut session);
    let doc = session.document();
    let wall = hole_wall(&doc, drilled);
    let across = turn_across(&mut session);
    let failed = session.remove_face(taken(across, wall));
    assert_eq!(failed.expect_err("the turn").text, "error-source-body-not-built");
    assert_ne!(across, drilled, "the refused turn is the hole");
    let doc = session.document();
    assert_eq!(removals(&doc), 0, "the turn wrote a removal");
    assert_eq!(refusal(&doc), "error-revolve-profile-crosses-axis", "the refusal was {}", refusal(&doc));
    near(live(&doc, drilled), opened(Height { mm: 30.0 }), "the hole");
}

/// A fillet the rebuild has refused still holds its extrusion. Removing a face of that extrusion writes nothing.
#[test]
fn a_refused_fillet_holds_the_extrusion() {
    let mut session = Session::empty();
    let block = extrusion_held_by_a_refused_fillet(&mut session);
    let doc = session.document();
    assert_eq!(refusal(&doc), "fillet R100.00 failed on 1 edges", "the refusal was {}", refusal(&doc));
    let cap = end_cap(&doc, block);
    let failed = session.remove_face(taken(block, cap));
    assert_eq!(failed.expect_err("the removal").text, "msg-body-consumed");
    let held = session.document();
    assert_eq!(refusal(&held), "fillet R100.00 failed on 1 edges", "the refusal was {}", refusal(&held));
    assert_eq!(removals(&held), 0, "a removal was written");
    near(live(&held, block), block_volume(Height { mm: 30.0 }), "the extrusion");
}

/// A missing face that has not been read is not united with the next box.
#[test]
fn an_unread_missing_face_is_not_joined_to_the_next_box() {
    let mut session = Session::empty();
    let drilled = open_hole(&mut session);
    let _read = session.document();
    let _missed = session.remove_face(taken(drilled, FaceId { id: 1 })).expect("the node is written");
    let small = session.cuboid(Cuboid { length: BoxLength { mm: 2.0 }, width: Width { mm: 2.0 }, height: Height { mm: 2.0 } }).expect("a box");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-faces-not-found", "the refusal was {}", refusal(&doc));
    assert_eq!(unions(&doc), 0, "the next box joined the removal");
    near(live(&doc, small), Volume { mm3: 8.0 }, "the next box");
    near(live(&doc, drilled), opened(Height { mm: 30.0 }), "the hole");
}

/// A removal that has not been read still unites with the next box. The part stays one body.
#[test]
fn an_unread_removal_unites_with_the_next_box() {
    let mut session = Session::empty();
    let drilled = open_hole(&mut session);
    let doc = session.document();
    let wall = hole_wall(&doc, drilled);
    let removed = session.remove_face(taken(drilled, wall)).expect("the wall is taken away");
    let joined = session.cuboid(Cuboid { length: BoxLength { mm: 2.0 }, width: Width { mm: 2.0 }, height: Height { mm: 2.0 } }).expect("a box");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_ne!(joined, removed, "the union is the removal");
    assert_eq!(doc.nodes.iter().find(|node| node.id == joined.id).expect("the union").name, "feat-name-body-union");
    near(live(&doc, joined), Volume { mm3: 6006.0 }, "the union");
}

/// The closed body follows the extrusion height. The wall id is the one taken at height 30.
#[test]
fn the_closed_body_follows_the_height() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    let grown = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    session.parameter(ParameterName { text: "span".into() }, Expression { text: "30".into() }).expect("a parameter");
    session.bind(grown, qymcad_drive::Dimension::Height, Expression { text: "span".into() }).expect("the height");
    let before = session.document();
    let cap = end_cap(&before, grown);
    let drilled = session.hole(grown, cap, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect("a hole");
    let opened_doc = session.document();
    let wall = hole_wall(&opened_doc, drilled);
    let made = session.remove_face(taken(drilled, wall)).expect("the wall is taken away");
    let at_thirty = session.document();
    near(live(&at_thirty, made), block(Height { mm: 30.0 }), "the body at 30 mm");
    assert_eq!(faces_of(&at_thirty, made), 6, "the closed block at 30 mm");
    assert_eq!(at_thirty.rebuild, Rebuild::Quiet, "{}", refusal(&at_thirty));
    session.parameter(ParameterName { text: "span".into() }, Expression { text: "20".into() }).expect("the height");
    let at_twenty = session.document();
    near(live(&at_twenty, made), block(Height { mm: 20.0 }), "the body at 20 mm");
    assert_eq!(faces_of(&at_twenty, made), 6, "the closed block at 20 mm");
    assert_eq!(at_twenty.rebuild, Rebuild::Quiet, "{}", refusal(&at_twenty));
}

/// A missing face that is still the refusal shows the current hole when the block gets shorter.
#[test]
fn a_missing_face_shows_the_shorter_hole() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    let grown = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    session.parameter(ParameterName { text: "span".into() }, Expression { text: "30".into() }).expect("a parameter");
    session.bind(grown, qymcad_drive::Dimension::Height, Expression { text: "span".into() }).expect("the height");
    let before = session.document();
    let cap = end_cap(&before, grown);
    let drilled = session.hole(grown, cap, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect("a hole");
    let _read = session.document();
    let failed = session.remove_face(taken(drilled, FaceId { id: 1 })).expect("the node is written");
    let _held = session.document();
    session.parameter(ParameterName { text: "span".into() }, Expression { text: "20".into() }).expect("the height");
    let shorter = session.document();
    assert_eq!(refusal(&shorter), "error-faces-not-found", "the refusal was {}", refusal(&shorter));
    near(live(&shorter, failed), opened(Height { mm: 20.0 }), "the hole");
}

fn open_hole(session: &mut Session) -> BodyId {
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let before = session.document();
    let cap = end_cap(&before, block);
    session.hole(block, cap, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect("a hole")
}

fn end_cap(doc: &qymcad_drive::Document, block: BodyId) -> FaceId {
    body(doc, block).faces.iter().find(|face| face.role == FaceRole::CapEnd).expect("the end cap").id
}

/// The one face of the hole that is neither a cap nor a side of the block. Its area is the 30 mm wall.
fn hole_wall(doc: &qymcad_drive::Document, drilled: BodyId) -> FaceId {
    let wall = bare(doc, drilled).expect("the hole wall");
    let area = body(doc, drilled).faces.iter().find(|face| face.id == wall).expect("the wall").area.mm2;
    assert!((area - std::f64::consts::PI * 4.0 * 30.0).abs() < 1.0, "the wall area is {area} mm^2");
    wall
}

fn bare(doc: &qymcad_drive::Document, id: BodyId) -> Option<FaceId> {
    let found = body(doc, id);
    let walls: Vec<_> = found.faces.iter().filter(|face| !matches!(face.role, FaceRole::CapStart | FaceRole::CapEnd | FaceRole::Wall { .. })).collect();
    match walls.as_slice() {
        [] => None,
        [one] => Some(one.id),
        _ => panic!("body {} has more than one face that is neither a cap nor a side", id.id),
    }
}

fn taken(body: BodyId, face: FaceId) -> RemoveFace {
    RemoveFace { body, face }
}

/// The 10 by 20 mm block grown by `height`.
fn block(height: Height) -> Volume {
    Volume { mm3: 200.0 * height.mm }
}

/// The same block with a 4 mm hole through it. The depth is longer than the block, so the hole removes `height`.
fn opened(height: Height) -> Volume {
    Volume { mm3: block(height).mm3 - std::f64::consts::PI * 4.0 * height.mm }
}

fn block_volume(height: Height) -> Volume {
    block(height)
}

fn removals(doc: &qymcad_drive::Document) -> usize {
    doc.nodes.iter().filter(|node| node.name == "feat-name-remove-face").count()
}

fn unions(doc: &qymcad_drive::Document) -> usize {
    doc.nodes.iter().filter(|node| node.name == "feat-name-body-union").count()
}

fn faces_of(doc: &qymcad_drive::Document, id: BodyId) -> usize {
    body(doc, id).faces.len()
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
