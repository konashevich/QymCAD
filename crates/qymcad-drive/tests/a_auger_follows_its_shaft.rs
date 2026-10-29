use qymcad_drive::{
    Angle, Auger, AugerLength, AugerPitch, BodyId, BoxLength, Corners, Cuboid, EdgeId, Flight, From, Height, Kind, Nominal, Outer, Pitch, Radius, Rebuild, Session, SketchAxis, Sound, Thread,
    ThreadLength, To, Volume, Width, WorldPlane,
};

/// An ordinary flight, outer 30 mm, pitch 20 mm, thickness 3 mm, length 20 mm, on a rim of the diameter 10 mm rod.
#[test]
fn the_flight_welds_its_rod() {
    let mut session = Session::empty();
    let made = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let welded = session.auger(ordinary(made.body, edge)).expect("a flight");
    let doc = session.document();
    near(flight_mesh(&doc, welded), welded_mesh(), "the flight");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(faces_of(&doc, welded), 9, "the welded rod");
    assert_eq!(edges_of(&doc, welded), 20, "the welded rod");
    assert_eq!(doc.components.iter().filter(|item| item.kind == Kind::Part).count(), 1, "one part");
    assert_eq!(augers(&doc), 1, "the flight node");
    assert_ne!(welded, made.body, "the welded body is the rod");
}

/// The other rim of the same rod welds the same flight.
#[test]
fn the_other_rim_welds_the_same_flight() {
    let mut session = Session::empty();
    let made = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    let edge = rim_at(&doc, made.body, 1);
    let welded = session.auger(ordinary(made.body, edge)).expect("a flight");
    let doc = session.document();
    near(flight_mesh(&doc, welded), welded_mesh(), "the other rim");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(faces_of(&doc, welded), 9, "the welded rod");
}

/// An outer diameter of 8 mm does not stand proud of the 10 mm shaft.
#[test]
fn a_flight_smaller_than_the_shaft_stays_the_rod() {
    let mut session = Session::empty();
    let made = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let welded = session.auger(flight(made.body, edge, Outer { mm: 8.0 }, AugerPitch { mm: 20.0 }, Flight { mm: 3.0 }, AugerLength { mm: 20.0 })).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "auger outer 8.0 not bigger than shaft 10.0", "the refusal was {}", refusal(&doc));
    near(live(&doc, welded), cylinder(Radius { mm: 5.0 }, Height { mm: 25.0 }), "the flight");
    near(live(&doc, made.body), cylinder(Radius { mm: 5.0 }, Height { mm: 25.0 }), "the rod");
}

/// A flight of 40 mm runs past the end of the 25 mm rod and still welds.
#[test]
fn a_flight_past_the_end_still_welds() {
    let mut session = Session::empty();
    let made = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let welded = session.auger(flight(made.body, edge, Outer { mm: 30.0 }, AugerPitch { mm: 20.0 }, Flight { mm: 3.0 }, AugerLength { mm: 40.0 })).expect("a flight");
    let doc = session.document();
    near(flight_mesh(&doc, welded), Volume { mm3: 5462.857624365697 }, "the long flight");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(faces_of(&doc, welded), 11, "the long flight");
    assert_eq!(edges_of(&doc, welded), 26, "the long flight");
}

/// A rim id of 0 writes no node. The real flight is still the welded rod.
#[test]
fn a_rim_id_of_zero_writes_no_flight() {
    let mut session = Session::empty();
    let made = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let missing = session.auger(ordinary(made.body, EdgeId { id: 0 }));
    assert_eq!(missing.expect_err("the rim").text, "error-edges-not-found");
    let held = session.document();
    assert_eq!(augers(&held), 0, "a rim id of 0 wrote a flight");
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
    let welded = session.auger(ordinary(made.body, edge)).expect("a flight");
    let doc = session.document();
    near(flight_mesh(&doc, welded), welded_mesh(), "the flight");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// A straight edge of the 10 by 20 by 30 mm block is not a rim. A flight on the rod is another node.
#[test]
fn a_straight_edge_is_not_a_rim() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = doc.bodies.iter().find(|body| body.id == block.id).expect("the block").edges.iter().find(|edge| (edge.length.mm - 30.0).abs() < 0.1).expect("the long edge").id;
    let missed = session.auger(ordinary(block, edge)).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-auger-rim-not-found", "the refusal was {}", refusal(&doc));
    near(live(&doc, missed), Volume { mm3: 6000.0 }, "the flight");
    near(live(&doc, block), Volume { mm3: 6000.0 }, "the block");
    let made = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    assert_eq!(unions(&doc), 0, "the rod joined the refused flight");
    let rim = rim(&doc, made.body);
    let welded = session.auger(ordinary(made.body, rim)).expect("a flight");
    assert_ne!(welded, missed, "the block's flight was rewritten");
    let doc = session.document();
    assert_eq!(augers(&doc), 2, "the block's flight was rewritten");
    assert_eq!(refusal(&doc), "error-auger-rim-not-found", "the refusal was {}", refusal(&doc));
    near(live(&doc, missed), Volume { mm3: 6000.0 }, "the block's flight");
    near(flight_mesh(&doc, welded), welded_mesh(), "the rod's flight");
}

/// The seam of the rod is not a rim. The rim rewrites that same node.
#[test]
fn the_seam_is_replaced_by_the_rim() {
    let mut session = Session::empty();
    let made = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let along = doc.bodies.iter().find(|body| body.id == made.body.id).expect("the rod").edges.iter().find(|item| (item.length.mm - 25.0).abs() < 1.0).expect("the seam").id;
    let failed = session.auger(ordinary(made.body, along)).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-auger-rim-not-found", "the refusal was {}", refusal(&doc));
    near(live(&doc, failed), cylinder(Radius { mm: 5.0 }, Height { mm: 25.0 }), "the seam");
    let zero = session.auger(ordinary(made.body, EdgeId { id: 0 }));
    assert_eq!(zero.expect_err("the rim").text, "error-edges-not-found");
    let held = session.document();
    assert_eq!(refusal(&held), "error-auger-rim-not-found", "the refusal was {}", refusal(&held));
    assert_eq!(augers(&held), 1, "the refused flight was rewritten");
    let welded = session.auger(ordinary(made.body, edge)).expect("the rim is tried again");
    assert_eq!(welded, failed, "a second flight was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(augers(&doc), 1, "the refused flight was left beside the new one");
    near(flight_mesh(&doc, welded), welded_mesh(), "the flight");
}

/// A length, an outer diameter, a pitch, or a thickness that is not a flight does not rewrite the small flight.
#[test]
fn the_small_flight_is_replaced_when_the_rod_is_named_again() {
    let mut session = Session::empty();
    let made = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let failed = session.auger(flight(made.body, edge, Outer { mm: 8.0 }, AugerPitch { mm: 20.0 }, Flight { mm: 3.0 }, AugerLength { mm: 20.0 })).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "auger outer 8.0 not bigger than shaft 10.0", "the refusal was {}", refusal(&doc));
    held_back(&mut session, flight(made.body, edge, Outer { mm: 30.0 }, AugerPitch { mm: 20.0 }, Flight { mm: 3.0 }, AugerLength { mm: f64::NAN }), "error-auger-length");
    held_back(&mut session, flight(made.body, edge, Outer { mm: 30.0 }, AugerPitch { mm: 20.0 }, Flight { mm: 3.0 }, AugerLength { mm: f64::INFINITY }), "error-auger-length");
    held_back(&mut session, flight(made.body, edge, Outer { mm: 30.0 }, AugerPitch { mm: 20.0 }, Flight { mm: 3.0 }, AugerLength { mm: 0.0 }), "error-auger-bad-pitch-or-length");
    held_back(&mut session, flight(made.body, edge, Outer { mm: 30.0 }, AugerPitch { mm: 20.0 }, Flight { mm: 3.0 }, AugerLength { mm: -5.0 }), "error-auger-bad-pitch-or-length");
    held_back(&mut session, flight(made.body, edge, Outer { mm: f64::NAN }, AugerPitch { mm: 20.0 }, Flight { mm: 3.0 }, AugerLength { mm: 20.0 }), "error-auger-outer");
    held_back(&mut session, flight(made.body, edge, Outer { mm: 0.0 }, AugerPitch { mm: 20.0 }, Flight { mm: 3.0 }, AugerLength { mm: 20.0 }), "error-auger-outer");
    held_back(&mut session, flight(made.body, edge, Outer { mm: 30.0 }, AugerPitch { mm: f64::NAN }, Flight { mm: 3.0 }, AugerLength { mm: 20.0 }), "error-auger-pitch");
    held_back(&mut session, flight(made.body, edge, Outer { mm: 30.0 }, AugerPitch { mm: 0.0 }, Flight { mm: 3.0 }, AugerLength { mm: 20.0 }), "error-auger-bad-pitch-or-length");
    held_back(&mut session, flight(made.body, edge, Outer { mm: 30.0 }, AugerPitch { mm: -1.0 }, Flight { mm: 3.0 }, AugerLength { mm: 20.0 }), "error-auger-bad-pitch-or-length");
    held_back(&mut session, flight(made.body, edge, Outer { mm: 30.0 }, AugerPitch { mm: 20.0 }, Flight { mm: f64::NAN }, AugerLength { mm: 20.0 }), "error-auger-thickness");
    held_back(&mut session, flight(made.body, edge, Outer { mm: 30.0 }, AugerPitch { mm: 20.0 }, Flight { mm: 0.0 }, AugerLength { mm: 20.0 }), "error-auger-thickness");
    let held = session.document();
    assert_eq!(refusal(&held), "auger outer 8.0 not bigger than shaft 10.0", "the refusal was {}", refusal(&held));
    assert_eq!(augers(&held), 1, "the small flight was rewritten");
    let welded = session.auger(ordinary(made.body, edge)).expect("the flight is tried again");
    assert_eq!(welded, failed, "a second flight was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(flight_mesh(&doc, welded), welded_mesh(), "the flight");
    let again = session.auger(ordinary(made.body, edge));
    assert_eq!(again.expect_err("the rod").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(augers(&doc), 1, "a second flight was written");
    near(flight_mesh(&doc, welded), welded_mesh(), "the flight");
}

/// The id a refused flight returned is that same flight. A second rod is a different one.
#[test]
fn a_small_flight_named_by_its_own_id_is_the_same_flight() {
    let mut session = Session::empty();
    let first = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    let edge = rim(&doc, first.body);
    let failed = session.auger(flight(first.body, edge, Outer { mm: 8.0 }, AugerPitch { mm: 20.0 }, Flight { mm: 3.0 }, AugerLength { mm: 20.0 })).expect("the node is written");
    let _held = session.document();
    let second = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    assert_eq!(unions(&doc), 0, "the second rod joined the refused flight");
    near(live(&doc, second.body), cylinder(Radius { mm: 5.0 }, Height { mm: 25.0 }), "the second rod");
    let other_edge = rim(&doc, second.body);
    let other = session.auger(flight(second.body, other_edge, Outer { mm: 8.0 }, AugerPitch { mm: 20.0 }, Flight { mm: 3.0 }, AugerLength { mm: 20.0 })).expect("the node is written");
    let _held = session.document();
    let welded = session.auger(ordinary(other, other_edge)).expect("the flight is tried again");
    assert_eq!(welded, other, "a second flight was added");
    assert_ne!(welded, failed, "the two retries are one body");
    let doc = session.document();
    near(flight_mesh(&doc, welded), welded_mesh(), "the second flight");
    assert_eq!(refusal(&doc), "auger outer 8.0 not bigger than shaft 10.0", "the refusal was {}", refusal(&doc));
}

/// An id of 0 and a sketch id write no node. The real flight is still the welded rod.
#[test]
fn a_missing_body_is_refused_before_a_flight() {
    let mut session = Session::empty();
    let made = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let missing = session.auger(ordinary(BodyId { id: 0 }, edge));
    assert_eq!(missing.expect_err("id 0").text, "error-source-body-not-built");
    let sketch = session.auger(ordinary(BodyId { id: made.drawn.sketch.id }, edge));
    assert_eq!(sketch.expect_err("a sketch").text, "error-source-body-deleted");
    let held = session.document();
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
    assert_eq!(augers(&held), 0, "a refused id wrote a flight");
    let welded = session.auger(ordinary(made.body, edge)).expect("a flight");
    let doc = session.document();
    near(flight_mesh(&doc, welded), welded_mesh(), "the flight");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// A turn across its axis is not a flight source.
#[test]
fn a_turn_across_the_axis_is_not_a_flight_source() {
    let mut session = Session::empty();
    let made = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let across = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, -5.0) }, to: To { at: (10.0, 15.0) } }).expect("a rectangle across the axis");
    let turned = session.revolve(across, SketchAxis::X, Angle { deg: 360.0 }).expect("the node is written");
    let _held = session.document();
    let failed = session.auger(ordinary(turned, edge));
    assert_eq!(failed.expect_err("the turn").text, "error-source-body-not-built");
    let doc = session.document();
    assert_eq!(augers(&doc), 0, "a flight was written");
    assert_eq!(refusal(&doc), "error-revolve-profile-crosses-axis", "the refusal was {}", refusal(&doc));
    near(live(&doc, made.body), cylinder(Radius { mm: 5.0 }, Height { mm: 25.0 }), "the rod");
}

/// A thread the rebuild has refused still holds its rod.
#[test]
fn a_refused_thread_holds_the_rod() {
    let mut session = Session::empty();
    let made = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let _cut = session.thread(Thread { body: made.body, rim: edge, diameter: Nominal { mm: 10.0 }, pitch: Pitch { mm: 1.5 }, length: ThreadLength { mm: 30.0 } }).expect("the node is written");
    let _refused = session.document();
    let failed = session.auger(ordinary(made.body, edge));
    assert_eq!(failed.expect_err("the rod").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(augers(&doc), 0, "a flight was written");
    assert_eq!(refusal(&doc), "a thread of 30.0 on a face 25.0 long", "the refusal was {}", refusal(&doc));
    near(live(&doc, made.body), cylinder(Radius { mm: 5.0 }, Height { mm: 25.0 }), "the rod");
}

/// An unread flight on a straight edge is not joined to the next box.
#[test]
fn an_unread_straight_edge_is_not_joined_to_the_next_box() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = doc.bodies.iter().find(|body| body.id == block.id).expect("the block").edges.iter().find(|item| (item.length.mm - 30.0).abs() < 0.1).expect("the long edge").id;
    let missed = session.auger(ordinary(block, edge)).expect("the node is written");
    let joined = session.cuboid(Cuboid { length: BoxLength { mm: 2.0 }, width: Width { mm: 2.0 }, height: Height { mm: 2.0 } }).expect("a box");
    let doc = session.document();
    assert_eq!(unions(&doc), 0, "the box joined the refused flight");
    assert_eq!(doc.nodes.iter().find(|node| node.id == joined.id).expect("the box").name, "feat-name-box");
    assert_eq!(refusal(&doc), "error-auger-rim-not-found", "the refusal was {}", refusal(&doc));
    near(live(&doc, joined), Volume { mm3: 8.0 }, "the box");
    near(live(&doc, missed), Volume { mm3: 6000.0 }, "the flight");
    near(live(&doc, block), Volume { mm3: 6000.0 }, "the block");
}

/// An unread flight unites with the next box, and that boolean breaks the solid.
#[test]
fn an_unread_flight_breaks_the_union_with_the_next_box() {
    let mut session = Session::empty();
    let made = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let welded = session.auger(ordinary(made.body, edge)).expect("a flight");
    let joined = session.cuboid(Cuboid { length: BoxLength { mm: 2.0 }, width: Width { mm: 2.0 }, height: Height { mm: 2.0 } }).expect("a box");
    let doc = session.document();
    assert_eq!(doc.nodes.iter().find(|node| node.id == joined.id).expect("the union").name, "feat-name-body-union");
    assert_eq!(unions(&doc), 1, "the box stayed beside the flight");
    assert_eq!(refusal(&doc), "error-broken-solid", "the refusal was {}", refusal(&doc));
    assert!(doc.bodies.iter().all(|body| body.id != joined.id), "the union is among the bodies");
    near(flight_mesh(&doc, welded), welded_mesh(), "the flight");
    assert_eq!(faces_of(&doc, welded), 9, "the flight");
}

/// An unread flight unites with a rod that misses it, and that boolean breaks the solid.
#[test]
fn an_unread_flight_breaks_the_union_with_a_rod_that_misses_it() {
    let mut session = Session::empty();
    let made = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let welded = session.auger(ordinary(made.body, edge)).expect("a flight");
    let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (40.0, 0.0) }, to: To { at: (45.0, 2.0) } }).expect("a rectangle");
    let turned = session.revolve(drawn, SketchAxis::X, Angle { deg: 360.0 }).expect("a rod");
    let doc = session.document();
    assert_eq!(doc.nodes.iter().find(|node| node.id == turned.id).expect("the union").name, "feat-name-body-union");
    assert_eq!(refusal(&doc), "error-broken-solid", "the refusal was {}", refusal(&doc));
    assert!(doc.bodies.iter().all(|body| body.id != turned.id), "the union is among the bodies");
    near(flight_mesh(&doc, welded), welded_mesh(), "the flight");
    assert_eq!(faces_of(&doc, welded), 9, "the flight");
}

/// The welded rod follows the sketch. Five more millimetres of shaft are 125 pi mm^3.
#[test]
fn the_welded_rod_follows_the_sketch() {
    let mut session = Session::empty();
    let made = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let welded = session.auger(ordinary(made.body, edge)).expect("a flight");
    let at_twenty_five = session.document();
    near(flight_mesh(&at_twenty_five, welded), welded_mesh(), "the flight at 25 mm");
    session.resize(made.drawn, span(Height { mm: 30.0 }, Radius { mm: 5.0 })).expect("the rod is lengthened");
    let at_thirty = session.document();
    assert_eq!(at_thirty.rebuild, Rebuild::Quiet, "{}", refusal(&at_thirty));
    near(flight_mesh(&at_thirty, welded), Volume { mm3: 3946.5534367319606 }, "the flight at 30 mm");
    assert_eq!(faces_of(&at_thirty, welded), 9, "the flight at 30 mm");
}

/// A rod shortened under the flight keeps the flight. The flight hangs past the end.
#[test]
fn a_shorter_rod_keeps_the_flight() {
    let mut session = Session::empty();
    let made = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let welded = session.auger(ordinary(made.body, edge)).expect("a flight");
    let _held = session.document();
    session.resize(made.drawn, span(Height { mm: 15.0 }, Radius { mm: 5.0 })).expect("the rod is shortened");
    let shorter = session.document();
    assert_eq!(shorter.rebuild, Rebuild::Quiet, "{}", refusal(&shorter));
    near(flight_mesh(&shorter, welded), Volume { mm3: 2778.277267058199 }, "the flight at 15 mm");
    assert_eq!(faces_of(&shorter, welded), 10, "the flight at 15 mm");
    assert_eq!(edges_of(&shorter, welded), 22, "the flight at 15 mm");
}

struct Stock {
    drawn: qymcad_drive::Profile,
    body: BodyId,
}

fn stock(session: &mut Session, radius: Radius, height: Height) -> Stock {
    let drawn = session.rectangle(WorldPlane::Xz, span(height, radius)).expect("a rectangle");
    let body = session.revolve(drawn, SketchAxis::X, Angle { deg: 360.0 }).expect("a rod");
    Stock { drawn, body }
}

fn span(height: Height, radius: Radius) -> Corners {
    Corners { from: From { at: (0.0, 0.0) }, to: To { at: (height.mm, radius.mm) } }
}

fn ordinary(body: BodyId, rim: EdgeId) -> Auger {
    flight(body, rim, Outer { mm: 30.0 }, AugerPitch { mm: 20.0 }, Flight { mm: 3.0 }, AugerLength { mm: 20.0 })
}

fn flight(body: BodyId, rim: EdgeId, outer: Outer, pitch: AugerPitch, thickness: Flight, length: AugerLength) -> Auger {
    Auger { body, rim, outer, pitch, thickness, length }
}

fn held_back(session: &mut Session, asked: Auger, text: &str) {
    let err = session.auger(asked).expect_err(text);
    assert_eq!(err.text, text, "the refusal was {}", err.text);
    let held = session.document();
    assert_eq!(augers(&held), 1, "the small flight was rewritten");
}

fn rim(doc: &qymcad_drive::Document, id: BodyId) -> EdgeId {
    rim_at(doc, id, 0)
}

fn rim_at(doc: &qymcad_drive::Document, id: BodyId, index: usize) -> EdgeId {
    let rims: Vec<EdgeId> = body(doc, id).edges.iter().filter(|edge| edge.length.mm < 1.0).map(|edge| edge.id).collect();
    rims.into_iter().nth(index).expect("a rim")
}

fn cylinder(radius: Radius, height: Height) -> Volume {
    Volume { mm3: std::f64::consts::PI * radius.mm * radius.mm * height.mm }
}

fn welded_mesh() -> Volume {
    Volume { mm3: 3552.881560104826 }
}

fn augers(doc: &qymcad_drive::Document) -> usize {
    doc.nodes.iter().filter(|node| node.name == "feat-name-auger").count()
}

fn unions(doc: &qymcad_drive::Document) -> usize {
    doc.nodes.iter().filter(|node| node.name == "feat-name-body-union").count()
}

fn faces_of(doc: &qymcad_drive::Document, id: BodyId) -> usize {
    body(doc, id).faces.len()
}

fn edges_of(doc: &qymcad_drive::Document, id: BodyId) -> usize {
    body(doc, id).edges.len()
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

fn flight_mesh(doc: &qymcad_drive::Document, id: BodyId) -> f64 {
    let found = body(doc, id);
    assert_eq!(found.sound, Sound::Valid, "the shape is valid");
    match found.measure {
        qymcad_drive::Measure::Live { solids, .. } => assert_eq!(solids, 1, "one solid"),
        qymcad_drive::Measure::Absent => panic!("body {} has no live shape", id.id),
    }
    found.mesh.mm3
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
