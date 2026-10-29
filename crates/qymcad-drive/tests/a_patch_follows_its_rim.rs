use qymcad_drive::{Angle, Area, BodyId, BoxLength, Corners, Cuboid, EdgeId, From, Height, Kind, Meet, Patch, Radius, Rebuild, Session, SketchAxis, Sound, To, Volume, Width, WorldPlane};

/// The rim of the diameter 10 mm rod is a disc of 78.25 mm^2. The rod stays.
#[test]
fn the_rim_is_a_disc() {
    let mut session = Session::empty();
    let made = rod(&mut session);
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let spanned = session.patch(Patch { body: made.body, edge, meet: Meet::ByPosition }).expect("a patch");
    let doc = session.document();
    near_area(sheet(&doc, spanned), disc(), "the sheet");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.components.iter().filter(|item| item.kind == Kind::Part).count(), 1, "one part");
    assert_eq!(patches(&doc), 1, "the patch node");
    assert_ne!(spanned, made.body, "the sheet is the rod");
    near(solid(&doc, made.body), cylinder(Radius { mm: 5.0 }, Height { mm: 25.0 }), "the rod");
}

/// The same rim, asked to meet smoothly, is the slightly larger disc.
#[test]
fn the_rim_meets_smoothly() {
    let mut session = Session::empty();
    let made = rod(&mut session);
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let spanned = session.patch(Patch { body: made.body, edge, meet: Meet::Smooth }).expect("a patch");
    let doc = session.document();
    near_area(sheet(&doc, spanned), smooth_disc(), "the sheet");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(solid(&doc, made.body), cylinder(Radius { mm: 5.0 }, Height { mm: 25.0 }), "the rod");
}

/// The rim at the other end is the same disc.
#[test]
fn the_other_rim_is_the_same_disc() {
    let mut session = Session::empty();
    let made = rod(&mut session);
    let doc = session.document();
    let first = rim(&doc, made.body);
    let second = body(&doc, made.body).edges.iter().find(|edge| edge.length.mm < 1.0 && edge.id != first).expect("the other rim").id;
    let flat = session.patch(Patch { body: made.body, edge: first, meet: Meet::ByPosition }).expect("a patch");
    let again = session.patch(Patch { body: made.body, edge: second, meet: Meet::ByPosition }).expect("the other rim");
    let doc = session.document();
    assert_ne!(again, flat, "the other rim is the first disc");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(patches(&doc), 2, "one patch was rewritten");
    near_area(sheet(&doc, flat), disc(), "the first sheet");
    near_area(sheet(&doc, again), disc(), "the second sheet");
    near(solid(&doc, made.body), cylinder(Radius { mm: 5.0 }, Height { mm: 25.0 }), "the rod");
}

/// The 25 mm generator is replaced by the rim.
#[test]
fn the_generator_is_replaced_by_the_rim() {
    let mut session = Session::empty();
    let made = rod(&mut session);
    let doc = session.document();
    let edge = generator(&doc, made.body);
    let round = rim(&doc, made.body);
    let failed = session.patch(Patch { body: made.body, edge, meet: Meet::ByPosition }).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "operation failed: Patch", "the refusal was {}", refusal(&doc));
    near(solid(&doc, made.body), cylinder(Radius { mm: 5.0 }, Height { mm: 25.0 }), "the rod");
    assert!(doc.bodies.iter().all(|body| body.id != failed.id), "the patch is among the bodies");
    let missing = session.patch(Patch { body: made.body, edge: EdgeId { id: 0 }, meet: Meet::ByPosition });
    assert_eq!(missing.expect_err("the edge").text, "error-edges-not-found");
    let smooth = session.patch(Patch { body: made.body, edge, meet: Meet::Smooth }).expect("the generator is tried smoothly");
    assert_eq!(smooth, failed, "a second patch was added");
    let held = session.document();
    assert_eq!(refusal(&held), "operation failed: Patch", "the refusal was {}", refusal(&held));
    assert_eq!(patches(&held), 1, "the refused patch was rewritten");
    assert!(held.bodies.iter().all(|body| body.id != failed.id), "the patch is among the bodies");
    let spanned = session.patch(Patch { body: made.body, edge: round, meet: Meet::ByPosition }).expect("the rim is tried");
    assert_eq!(spanned, failed, "a second patch was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(patches(&doc), 1, "the refused patch was left beside the new one");
    near_area(sheet(&doc, spanned), disc(), "the sheet");
}

/// The id a refused patch returned is that same patch.
#[test]
fn a_refused_patch_named_by_its_own_id_is_the_same_patch() {
    let mut session = Session::empty();
    let made = rod(&mut session);
    let doc = session.document();
    let edge = generator(&doc, made.body);
    let round = rim(&doc, made.body);
    let failed = session.patch(Patch { body: made.body, edge, meet: Meet::ByPosition }).expect("the node is written");
    let _held = session.document();
    let spanned = session.patch(Patch { body: failed, edge: round, meet: Meet::ByPosition }).expect("the rim is tried");
    assert_eq!(spanned, failed, "a second patch was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(patches(&doc), 1, "the refused patch was left beside the new one");
    near_area(sheet(&doc, spanned), disc(), "the sheet");
    near(solid(&doc, made.body), cylinder(Radius { mm: 5.0 }, Height { mm: 25.0 }), "the rod");
}

/// The same rim patched again is a second sheet. The rod was not taken.
#[test]
fn the_same_rim_is_patched_again() {
    let mut session = Session::empty();
    let made = rod(&mut session);
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let first = session.patch(Patch { body: made.body, edge, meet: Meet::ByPosition }).expect("a patch");
    let second = session.patch(Patch { body: made.body, edge, meet: Meet::ByPosition }).expect("a second patch");
    let doc = session.document();
    assert_ne!(second, first, "the second patch is the first");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(patches(&doc), 2, "one patch was rewritten");
    near_area(sheet(&doc, first), disc(), "the first sheet");
    near_area(sheet(&doc, second), disc(), "the second sheet");
    near(solid(&doc, made.body), cylinder(Radius { mm: 5.0 }, Height { mm: 25.0 }), "the rod");
}

/// An edge id of 0 writes no node. The real patch is still the disc.
#[test]
fn an_edge_id_of_zero_writes_no_patch() {
    let mut session = Session::empty();
    let made = rod(&mut session);
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let missing = session.patch(Patch { body: made.body, edge: EdgeId { id: 0 }, meet: Meet::ByPosition });
    assert_eq!(missing.expect_err("the edge").text, "error-edges-not-found");
    let held = session.document();
    assert_eq!(patches(&held), 0, "an edge id of 0 wrote a patch");
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
    let spanned = session.patch(Patch { body: made.body, edge, meet: Meet::ByPosition }).expect("a patch");
    let doc = session.document();
    near_area(sheet(&doc, spanned), disc(), "the sheet");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// An id of 0 and a sketch id write no node. The real patch is still the disc.
#[test]
fn a_missing_body_is_refused_before_a_patch() {
    let mut session = Session::empty();
    let made = rod(&mut session);
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let missing = session.patch(Patch { body: BodyId { id: 0 }, edge, meet: Meet::ByPosition });
    assert_eq!(missing.expect_err("id 0").text, "error-source-body-not-built");
    let sketch = session.patch(Patch { body: BodyId { id: made.drawn.sketch.id }, edge, meet: Meet::ByPosition });
    assert_eq!(sketch.expect_err("a sketch").text, "error-source-body-deleted");
    let held = session.document();
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
    assert_eq!(patches(&held), 0, "a refused id wrote a patch");
    let spanned = session.patch(Patch { body: made.body, edge, meet: Meet::ByPosition }).expect("a patch");
    let doc = session.document();
    near_area(sheet(&doc, spanned), disc(), "the sheet");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// A turn across its axis is not a patch source.
#[test]
fn a_turn_across_the_axis_is_not_a_patch_source() {
    let mut session = Session::empty();
    let made = rod(&mut session);
    let _rod = session.document();
    let across = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, -5.0) }, to: To { at: (10.0, 15.0) } }).expect("a rectangle across the axis");
    let turned = session.revolve(across, SketchAxis::X, Angle { deg: 360.0 }).expect("the node is written");
    let _held = session.document();
    let failed = session.patch(Patch { body: turned, edge: EdgeId { id: 1 }, meet: Meet::ByPosition });
    assert_eq!(failed.expect_err("the turn").text, "error-source-body-not-built");
    let doc = session.document();
    assert_eq!(patches(&doc), 0, "a patch was written");
    assert_eq!(refusal(&doc), "error-revolve-profile-crosses-axis", "the refusal was {}", refusal(&doc));
    near(solid(&doc, made.body), cylinder(Radius { mm: 5.0 }, Height { mm: 25.0 }), "the rod");
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
    let failed = session.patch(Patch { body: block, edge, meet: Meet::ByPosition });
    assert_eq!(failed.expect_err("the extrusion").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(patches(&doc), 0, "a patch was written");
    assert_eq!(refusal(&doc), "fillet R100.00 failed on 1 edges", "the refusal was {}", refusal(&doc));
    near(solid(&doc, block), Volume { mm3: 6000.0 }, "the extrusion");
}

/// A patch of a straight edge that has not been read joins the box, and the patch stays out of the list.
#[test]
fn an_unread_straight_edge_joins_the_box() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = doc.bodies.iter().find(|body| body.id == block.id).expect("the box").edges.iter().find(|item| (item.length.mm - 30.0).abs() < 0.1).expect("the long edge").id;
    let failed = session.patch(Patch { body: block, edge, meet: Meet::ByPosition }).expect("the node is written");
    let joined = session.cuboid(Cuboid { length: BoxLength { mm: 2.0 }, width: Width { mm: 2.0 }, height: Height { mm: 2.0 } }).expect("a box");
    let doc = session.document();
    assert_eq!(doc.nodes.iter().find(|node| node.id == joined.id).expect("the union").name, "feat-name-body-union");
    near(solid(&doc, joined), Volume { mm3: 6006.0 }, "the union");
    assert!(doc.bodies.iter().all(|body| body.id != failed.id), "the patch is among the bodies");
    assert_eq!(refusal(&doc), "operation failed: Patch", "the refusal was {}", refusal(&doc));
    near(solid(&doc, block), Volume { mm3: 6000.0 }, "the box");
}

/// A disc that has not been read stays a sheet beside the union.
#[test]
fn an_unread_disc_leaves_the_sheet_beside_the_rod() {
    let mut session = Session::empty();
    let made = rod(&mut session);
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let spanned = session.patch(Patch { body: made.body, edge, meet: Meet::ByPosition }).expect("a patch");
    let joined = session.cuboid(Cuboid { length: BoxLength { mm: 2.0 }, width: Width { mm: 2.0 }, height: Height { mm: 2.0 } }).expect("a box");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.nodes.iter().find(|node| node.id == joined.id).expect("the union").name, "feat-name-body-union");
    near(solid(&doc, joined), Volume { mm3: 1967.495408493621 }, "the union");
    near_area(sheet(&doc, spanned), disc(), "the sheet");
    near(solid(&doc, made.body), cylinder(Radius { mm: 5.0 }, Height { mm: 25.0 }), "the rod");
}

/// The disc follows the rod. A radius of 6 mm is 112.68 mm^2.
#[test]
fn the_disc_follows_the_rod() {
    let mut session = Session::empty();
    let made = rod(&mut session);
    let doc = session.document();
    let edge = rim(&doc, made.body);
    let spanned = session.patch(Patch { body: made.body, edge, meet: Meet::ByPosition }).expect("a patch");
    let _read = session.document();
    session.resize(made.drawn, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (25.0, 6.0) } }).expect("a thicker rod");
    let thicker = session.document();
    assert_eq!(thicker.rebuild, Rebuild::Quiet, "{}", refusal(&thicker));
    near(solid(&thicker, made.body), cylinder(Radius { mm: 6.0 }, Height { mm: 25.0 }), "the rod");
    near_area(sheet(&thicker, spanned), Area { mm2: 112.67595346041196 }, "the sheet");
}

struct Stock {
    drawn: qymcad_drive::Profile,
    body: BodyId,
}

fn rod(session: &mut Session) -> Stock {
    let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (25.0, 5.0) } }).expect("a rectangle");
    let body = session.revolve(drawn, SketchAxis::X, Angle { deg: 360.0 }).expect("a rod");
    Stock { drawn, body }
}

fn rim(doc: &qymcad_drive::Document, id: BodyId) -> EdgeId {
    body(doc, id).edges.iter().find(|edge| edge.length.mm < 1.0).expect("a rim").id
}

fn generator(doc: &qymcad_drive::Document, id: BodyId) -> EdgeId {
    body(doc, id).edges.iter().find(|edge| (edge.length.mm - 25.0).abs() < 0.1).expect("the generator").id
}

fn disc() -> Area {
    Area { mm2: 78.24719059514672 }
}

fn smooth_disc() -> Area {
    Area { mm2: 78.48298509631955 }
}

fn cylinder(radius: Radius, height: Height) -> Volume {
    Volume { mm3: std::f64::consts::PI * radius.mm * radius.mm * height.mm }
}

fn patches(doc: &qymcad_drive::Document) -> usize {
    doc.nodes.iter().filter(|node| node.name == "feat-name-patch").count()
}

fn body(doc: &qymcad_drive::Document, id: BodyId) -> &qymcad_drive::Body {
    doc.bodies.iter().find(|body| body.id == id.id).unwrap_or_else(|| panic!("body {} is gone", id.id))
}

fn solid(doc: &qymcad_drive::Document, id: BodyId) -> f64 {
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

fn sheet(doc: &qymcad_drive::Document, id: BodyId) -> f64 {
    let found = body(doc, id);
    assert_eq!(found.sound, Sound::Valid, "the shape is valid");
    match found.measure {
        qymcad_drive::Measure::Live { volume, solids } => {
            assert_eq!(solids, 0, "a sheet has no solid");
            assert!(volume.mm3.abs() < 1.0, "a sheet holds {} mm^3", volume.mm3);
        }
        qymcad_drive::Measure::Absent => panic!("body {} has no live shape", id.id),
    }
    assert_eq!(found.faces.len(), 1, "one face");
    assert_eq!(found.faces[0].role, qymcad_drive::FaceRole::Other, "the sheet face");
    found.faces[0].area.mm2
}

fn near(got: f64, want: Volume, what: &str) {
    assert!((got - want.mm3).abs() < 1.0, "{what}: volume {got} mm^3, wanted {} within 1", want.mm3);
}

fn near_area(got: f64, want: Area, what: &str) {
    assert!((got - want.mm2).abs() < 1.0, "{what}: area {got} mm^2, wanted {} within 1", want.mm2);
}

fn refusal(doc: &qymcad_drive::Document) -> String {
    match &doc.rebuild {
        Rebuild::Refused { refusals } => refusals.iter().map(|refusal| refusal.text.clone()).collect::<Vec<_>>().join(" "),
        Rebuild::Quiet => "quiet".into(),
        Rebuild::Stopped => "stopped".into(),
    }
}
