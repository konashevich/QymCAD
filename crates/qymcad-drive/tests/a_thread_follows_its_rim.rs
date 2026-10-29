use qymcad_drive::{
    Angle, BodyId, BoxLength, Corners, Cuboid, EdgeId, From, Height, Kind, Nominal, Pitch, Radius, Rebuild, Session, SketchAxis, Sound, Thread, ThreadLength, To, Volume, Width, WorldPlane,
};

/// An M10x1.5 thread, 20 mm long, on a rim of the diameter 10 mm rod.
#[test]
fn the_thread_cuts_its_rod() {
    let mut session = Session::empty();
    let made = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let cut = session.thread(metric(made.body, edge, ThreadLength { mm: 20.0 })).expect("a thread");
    let doc = session.document();
    near(mesh(&doc, cut), cut_mesh(), "the thread");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(faces_of(&doc, cut), 63, "the threaded rod");
    assert_eq!(doc.components.iter().filter(|item| item.kind == Kind::Part).count(), 1, "one part");
    assert_eq!(threads(&doc), 1, "the thread node");
    assert_ne!(cut, made.body, "the threaded body is the rod");
}

/// A pitch of 0 is the coarse pitch of M10, which is 1.5 mm.
#[test]
fn a_pitch_of_zero_is_the_coarse_pitch() {
    let mut session = Session::empty();
    let made = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let cut = session.thread(sized(made.body, edge, Nominal { mm: 10.0 }, Pitch { mm: 0.0 }, ThreadLength { mm: 20.0 })).expect("a thread");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(mesh(&doc, cut), cut_mesh(), "the coarse pitch");
}

/// A thread longer than the rod writes one node and leaves the rod as it is.
#[test]
fn a_thread_longer_than_the_rod_stays_the_rod() {
    let mut session = Session::empty();
    let made = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let cut = session.thread(metric(made.body, edge, ThreadLength { mm: 30.0 })).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "a thread of 30.0 on a face 25.0 long", "the refusal was {}", refusal(&doc));
    near(live(&doc, cut), cylinder(Radius { mm: 5.0 }, Height { mm: 25.0 }), "the thread");
    near(live(&doc, made.body), cylinder(Radius { mm: 5.0 }, Height { mm: 25.0 }), "the rod");
}

/// M10 does not fit a rod of diameter 20 mm.
#[test]
fn a_thread_of_the_wrong_diameter_stays_the_thicker_rod() {
    let mut session = Session::empty();
    let made = stock(&mut session, Radius { mm: 10.0 }, Height { mm: 25.0 });
    let doc = session.document();
    near(live(&doc, made.body), cylinder(Radius { mm: 10.0 }, Height { mm: 25.0 }), "the rod");
    let edge = rim(&doc, made.body);
    let cut = session.thread(metric(made.body, edge, ThreadLength { mm: 20.0 })).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "a thread of 10.00 does not fit a face of 20.00", "the refusal was {}", refusal(&doc));
    near(live(&doc, cut), cylinder(Radius { mm: 10.0 }, Height { mm: 25.0 }), "the thread");
}

/// A rim id of 0 writes no node. The real thread is still the cut rod.
#[test]
fn a_rim_id_of_zero_writes_no_thread() {
    let mut session = Session::empty();
    let made = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let missing = session.thread(metric(made.body, EdgeId { id: 0 }, ThreadLength { mm: 20.0 }));
    assert_eq!(missing.expect_err("the rim").text, "error-edges-not-found");
    let held = session.document();
    assert_eq!(threads(&held), 0, "a rim id of 0 wrote a thread");
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
    let cut = session.thread(metric(made.body, edge, ThreadLength { mm: 20.0 })).expect("a thread");
    let doc = session.document();
    near(mesh(&doc, cut), cut_mesh(), "the thread");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// A straight edge of the 10 by 20 by 30 mm block is not a rim.
#[test]
fn a_straight_edge_is_not_a_rim() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = doc.bodies.iter().find(|body| body.id == block.id).expect("the block").edges.iter().find(|edge| (edge.length.mm - 30.0).abs() < 0.1).expect("the long edge").id;
    let cut = session.thread(metric(block, edge, ThreadLength { mm: 20.0 })).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-thread-rim-not-found", "the refusal was {}", refusal(&doc));
    near(live(&doc, cut), Volume { mm3: 6000.0 }, "the thread");
    near(live(&doc, block), Volume { mm3: 6000.0 }, "the block");
}

/// The seam of the rod is not a rim. The rim rewrites that same node.
#[test]
fn the_seam_is_replaced_by_the_rim() {
    let mut session = Session::empty();
    let made = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let along = doc.bodies.iter().find(|body| body.id == made.body.id).expect("the rod").edges.iter().find(|item| (item.length.mm - 25.0).abs() < 1.0).expect("the seam").id;
    let failed = session.thread(metric(made.body, along, ThreadLength { mm: 20.0 })).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-thread-rim-not-found", "the refusal was {}", refusal(&doc));
    near(live(&doc, failed), cylinder(Radius { mm: 5.0 }, Height { mm: 25.0 }), "the seam");
    let zero = session.thread(metric(made.body, EdgeId { id: 0 }, ThreadLength { mm: 20.0 }));
    assert_eq!(zero.expect_err("the rim").text, "error-edges-not-found");
    let held = session.document();
    assert_eq!(refusal(&held), "error-thread-rim-not-found", "the refusal was {}", refusal(&held));
    assert_eq!(threads(&held), 1, "the refused thread was rewritten");
    let cut = session.thread(metric(made.body, edge, ThreadLength { mm: 20.0 })).expect("the rim is tried again");
    assert_eq!(cut, failed, "a second thread was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(threads(&doc), 1, "the refused thread was left beside the new one");
    near(mesh(&doc, cut), cut_mesh(), "the thread");
}

/// A length that is not a thread, and a length of 0, do not rewrite the long thread. Length 20 does.
#[test]
fn the_long_thread_is_replaced_when_the_rod_is_named_again() {
    let mut session = Session::empty();
    let made = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let failed = session.thread(metric(made.body, edge, ThreadLength { mm: 30.0 })).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "a thread of 30.0 on a face 25.0 long", "the refusal was {}", refusal(&doc));
    held_back(&mut session, metric(made.body, edge, ThreadLength { mm: f64::NAN }), "error-thread-length");
    held_back(&mut session, metric(made.body, edge, ThreadLength { mm: f64::INFINITY }), "error-thread-length");
    held_back(&mut session, metric(made.body, edge, ThreadLength { mm: 0.0 }), "error-thread-length-unset");
    held_back(&mut session, metric(made.body, edge, ThreadLength { mm: -5.0 }), "error-thread-length-unset");
    held_back(&mut session, sized(made.body, edge, Nominal { mm: f64::NAN }, Pitch { mm: 1.5 }, ThreadLength { mm: 20.0 }), "error-thread-diameter");
    held_back(&mut session, sized(made.body, edge, Nominal { mm: 0.0 }, Pitch { mm: 1.5 }, ThreadLength { mm: 20.0 }), "error-thread-diameter");
    held_back(&mut session, sized(made.body, edge, Nominal { mm: 10.0 }, Pitch { mm: f64::NAN }, ThreadLength { mm: 20.0 }), "error-thread-pitch");
    held_back(&mut session, sized(made.body, edge, Nominal { mm: 10.0 }, Pitch { mm: -1.0 }, ThreadLength { mm: 20.0 }), "error-thread-pitch");
    let held = session.document();
    assert_eq!(refusal(&held), "a thread of 30.0 on a face 25.0 long", "the refusal was {}", refusal(&held));
    assert_eq!(threads(&held), 1, "the long thread was rewritten");
    let cut = session.thread(metric(made.body, edge, ThreadLength { mm: 20.0 })).expect("the length is tried again");
    assert_eq!(cut, failed, "a second thread was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(mesh(&doc, cut), cut_mesh(), "the thread");
    let again = session.thread(metric(made.body, edge, ThreadLength { mm: 20.0 }));
    assert_eq!(again.expect_err("the rod").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(threads(&doc), 1, "a second thread was written");
    near(mesh(&doc, cut), cut_mesh(), "the thread");
}

/// The id a refused thread returned is that same thread. A second rod is a different one.
#[test]
fn a_long_thread_named_by_its_own_id_is_the_same_thread() {
    let mut session = Session::empty();
    let first = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    let edge = rim(&doc, first.body);
    let failed = session.thread(metric(first.body, edge, ThreadLength { mm: 30.0 })).expect("the node is written");
    let _held = session.document();
    let second = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    assert_eq!(unions(&doc), 0, "the second rod joined the refused thread");
    near(live(&doc, second.body), cylinder(Radius { mm: 5.0 }, Height { mm: 25.0 }), "the second rod");
    let other_edge = rim(&doc, second.body);
    let other = session.thread(metric(second.body, other_edge, ThreadLength { mm: 30.0 })).expect("the node is written");
    let _held = session.document();
    let cut = session.thread(metric(other, other_edge, ThreadLength { mm: 20.0 })).expect("the length is tried again");
    assert_eq!(cut, other, "a second thread was added");
    assert_ne!(cut, failed, "the two retries are one body");
    let doc = session.document();
    near(mesh(&doc, cut), cut_mesh(), "the second thread");
    assert_eq!(refusal(&doc), "a thread of 30.0 on a face 25.0 long", "the refusal was {}", refusal(&doc));
}

/// An id of 0 and a sketch id write no node. The real thread is still the cut rod.
#[test]
fn a_missing_body_is_refused_before_a_thread() {
    let mut session = Session::empty();
    let made = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let missing = session.thread(metric(BodyId { id: 0 }, edge, ThreadLength { mm: 20.0 }));
    assert_eq!(missing.expect_err("id 0").text, "error-source-body-not-built");
    let sketch = session.thread(metric(BodyId { id: made.drawn.sketch.id }, edge, ThreadLength { mm: 20.0 }));
    assert_eq!(sketch.expect_err("a sketch").text, "error-source-body-deleted");
    let held = session.document();
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
    assert_eq!(threads(&held), 0, "a refused id wrote a thread");
    let cut = session.thread(metric(made.body, edge, ThreadLength { mm: 20.0 })).expect("a thread");
    let doc = session.document();
    near(mesh(&doc, cut), cut_mesh(), "the thread");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// A turn across its axis is not a thread source.
#[test]
fn a_turn_across_the_axis_is_not_a_thread_source() {
    let mut session = Session::empty();
    let made = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let across = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, -5.0) }, to: To { at: (10.0, 15.0) } }).expect("a rectangle across the axis");
    let turned = session.revolve(across, SketchAxis::X, Angle { deg: 360.0 }).expect("the node is written");
    let _held = session.document();
    let failed = session.thread(metric(turned, edge, ThreadLength { mm: 20.0 }));
    assert_eq!(failed.expect_err("the turn").text, "error-source-body-not-built");
    let doc = session.document();
    assert_eq!(threads(&doc), 0, "a thread was written");
    assert_eq!(refusal(&doc), "error-revolve-profile-crosses-axis", "the refusal was {}", refusal(&doc));
    near(live(&doc, made.body), cylinder(Radius { mm: 5.0 }, Height { mm: 25.0 }), "the rod");
}

/// A fillet the rebuild has refused still holds its extrusion.
#[test]
fn a_refused_fillet_holds_the_extrusion() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = doc.bodies.iter().find(|body| body.id == block.id).expect("the extrusion").edges.iter().find(|item| (item.length.mm - 20.0).abs() < 0.1).expect("the 20 mm edge").id;
    let _rounded = session.fillet(block, edge, Radius { mm: 100.0 }).expect("the node is written");
    let _refused = session.document();
    let failed = session.thread(metric(block, edge, ThreadLength { mm: 20.0 }));
    assert_eq!(failed.expect_err("the extrusion").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(threads(&doc), 0, "a thread was written");
    assert_eq!(refusal(&doc), "fillet R100.00 failed on 1 edges", "the refusal was {}", refusal(&doc));
    near(live(&doc, block), Volume { mm3: 6000.0 }, "the extrusion");
}

/// An unread thread on a straight edge is not joined to the next box.
#[test]
fn an_unread_straight_edge_is_not_joined_to_the_next_box() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = doc.bodies.iter().find(|body| body.id == block.id).expect("the block").edges.iter().find(|item| (item.length.mm - 30.0).abs() < 0.1).expect("the long edge").id;
    let missed = session.thread(metric(block, edge, ThreadLength { mm: 20.0 })).expect("the node is written");
    let joined = session.cuboid(Cuboid { length: BoxLength { mm: 2.0 }, width: Width { mm: 2.0 }, height: Height { mm: 2.0 } }).expect("a box");
    let doc = session.document();
    assert_eq!(unions(&doc), 0, "the box joined the refused thread");
    assert_eq!(doc.nodes.iter().find(|node| node.id == joined.id).expect("the box").name, "feat-name-box");
    assert_eq!(refusal(&doc), "error-thread-rim-not-found", "the refusal was {}", refusal(&doc));
    near(live(&doc, joined), Volume { mm3: 8.0 }, "the box");
    near(live(&doc, missed), Volume { mm3: 6000.0 }, "the thread");
    near(live(&doc, block), Volume { mm3: 6000.0 }, "the block");
}

/// An unread thread on the rim unites with the next box. 4 mm^3 of the box lies inside the rod.
#[test]
fn an_unread_thread_unites_with_the_next_box() {
    let mut session = Session::empty();
    let made = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let _cut = session.thread(metric(made.body, edge, ThreadLength { mm: 20.0 })).expect("a thread");
    let joined = session.cuboid(Cuboid { length: BoxLength { mm: 2.0 }, width: Width { mm: 2.0 }, height: Height { mm: 2.0 } }).expect("a box");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.nodes.iter().find(|node| node.id == joined.id).expect("the union").name, "feat-name-body-union");
    assert_eq!(unions(&doc), 1, "the box stayed beside the thread");
    near(mesh(&doc, joined), Volume { mm3: 1685.3005687272903 }, "the union");
}

/// The threaded rod follows the sketch. Five more millimetres of rod are 125 pi mm^3.
#[test]
fn the_cut_rod_follows_the_sketch() {
    let mut session = Session::empty();
    let made = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let cut = session.thread(metric(made.body, edge, ThreadLength { mm: 20.0 })).expect("a thread");
    let at_twenty_five = session.document();
    near(mesh(&at_twenty_five, cut), cut_mesh(), "the thread at 25 mm");
    session.resize(made.drawn, span(Height { mm: 30.0 }, Radius { mm: 5.0 })).expect("the rod is lengthened");
    let at_thirty = session.document();
    assert_eq!(at_thirty.rebuild, Rebuild::Quiet, "{}", refusal(&at_thirty));
    near(mesh(&at_thirty, cut), Volume { mm3: 2073.86117310783 }, "the thread at 30 mm");
}

/// A thread that is still too long shows the current rod when the sketch gets shorter.
#[test]
fn a_long_thread_shows_the_shorter_rod() {
    let mut session = Session::empty();
    let made = stock(&mut session, Radius { mm: 5.0 }, Height { mm: 25.0 });
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let failed = session.thread(metric(made.body, edge, ThreadLength { mm: 30.0 })).expect("the node is written");
    let _held = session.document();
    session.resize(made.drawn, span(Height { mm: 15.0 }, Radius { mm: 5.0 })).expect("the rod is shortened");
    let shorter = session.document();
    assert_eq!(refusal(&shorter), "a thread of 30.0 on a face 15.0 long", "the refusal was {}", refusal(&shorter));
    near(live(&shorter, failed), cylinder(Radius { mm: 5.0 }, Height { mm: 15.0 }), "the rod");
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

fn metric(body: BodyId, rim: EdgeId, length: ThreadLength) -> Thread {
    sized(body, rim, Nominal { mm: 10.0 }, Pitch { mm: 1.5 }, length)
}

fn sized(body: BodyId, rim: EdgeId, diameter: Nominal, pitch: Pitch, length: ThreadLength) -> Thread {
    Thread { body, rim, diameter, pitch, length }
}

fn held_back(session: &mut Session, asked: Thread, text: &str) {
    let err = session.thread(asked).expect_err(text);
    assert_eq!(err.text, text, "the refusal was {}", err.text);
    let held = session.document();
    assert_eq!(threads(&held), 1, "the long thread was rewritten");
}

fn rim(doc: &qymcad_drive::Document, id: BodyId) -> EdgeId {
    body(doc, id).edges.iter().find(|edge| edge.length.mm < 1.0).expect("a rim").id
}

fn cylinder(radius: Radius, height: Height) -> Volume {
    Volume { mm3: std::f64::consts::PI * radius.mm * radius.mm * height.mm }
}

fn cut_mesh() -> Volume {
    Volume { mm3: 1681.3954982007497 }
}

fn threads(doc: &qymcad_drive::Document) -> usize {
    doc.nodes.iter().filter(|node| node.name == "feat-name-thread").count()
}

fn unions(doc: &qymcad_drive::Document) -> usize {
    doc.nodes.iter().filter(|node| node.name == "feat-name-body-union").count()
}

fn faces_of(doc: &qymcad_drive::Document, id: BodyId) -> usize {
    body(doc, id).faces.len()
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

fn mesh(doc: &qymcad_drive::Document, id: BodyId) -> f64 {
    let found = body(doc, id);
    assert_eq!(found.sound, Sound::Valid, "the shape is valid");
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
