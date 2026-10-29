use qymcad_drive::{BodyId, Corners, Distance, EdgeId, Height, Measure, Rebuild, Session, Sound, WorldPlane};

/// One named edge of the box is beveled, and the same chamfer follows that edge after the sketch above
/// it is resized.
#[test]
fn a_chamfer_follows_its_edge() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = long_edge(&doc);
    let beveled = session.chamfer(made, edge, Distance { mm: 2.0 }).expect("a chamfer");
    let built = session.document();
    assert_eq!(built.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    let volume = live(&built, beveled);
    let want = 6000.0 - removed(2.0, 20.0);
    assert!((volume - want).abs() < 1.0, "volume {volume} mm^3, wanted {want} within 1");
    assert_ne!(beveled, made, "the chamfer is the extrusion");

    session.resize(drawn, rectangle_10_by_10()).expect("the rectangle moves");
    let shrunk = session.document();
    assert_eq!(shrunk.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert_eq!(shrunk.sketches.len(), 1, "the resize does not add a sketch");
    assert_eq!(shrunk.sketches[0].id, drawn.sketch.id, "the sketch id is unchanged");
    let again = live(&shrunk, beveled);
    let want = 3000.0 - removed(2.0, 10.0);
    assert!((again - want).abs() < 1.0, "volume {again} mm^3, wanted {want} within 1");
}

/// A distance the edge cannot take is the kernel's own sentence, and the solid stays the box.
#[test]
fn a_distance_the_edge_cannot_take_is_refused() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = long_edge(&doc);
    let beveled = session.chamfer(made, edge, Distance { mm: 100.0 }).expect("the node is written");
    let doc = session.document();
    let text = refusal(&doc);
    assert!(text.contains("chamfer 100.00 too big"), "the refusal was {text}");
    let volume = live(&doc, beveled);
    assert!((volume - 6000.0).abs() < 1.0, "volume {volume} mm^3, the refusal still beveled the edge");
}

/// An id that is not an edge of this body is refused. After the first has been read, the second reuses that node.
#[test]
fn an_id_that_is_not_an_edge_is_refused() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let cap = doc.bodies[0].faces.iter().find(|face| face.role == qymcad_drive::FaceRole::CapEnd).expect("the end cap").id;
    let first = session.chamfer(made, EdgeId { id: 1 }, Distance { mm: 2.0 }).expect("the node is written");
    let doc = session.document();
    assert!(refusal(&doc).contains("error-edges-not-found"), "the refusal was {}", refusal(&doc));
    assert!((live(&doc, first) - 6000.0).abs() < 1.0, "a missing edge beveled the box");
    let second = session.chamfer(made, EdgeId { id: cap.id }, Distance { mm: 2.0 }).expect("the node is written");
    assert_eq!(second, first, "a second chamfer was added");
    let doc = session.document();
    assert!(refusal(&doc).contains("error-edges-not-found"), "the refusal was {}", refusal(&doc));
    assert!((live(&doc, second) - 6000.0).abs() < 1.0, "a missing edge beveled the box");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-chamfer").count(), 1, "two chamfer nodes were left");
}

/// An edge id of 0 is a pick of nothing. A pick of nothing bevels every edge and the rebuild stays quiet.
#[test]
fn an_unnamed_edge_does_not_bevel_the_box() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let _doc = session.document();
    let refused = session.chamfer(made, EdgeId { id: 0 }, Distance { mm: 2.0 }).expect_err("an unnamed edge");
    assert!(refused.text.contains("error-edges-not-found"), "the refusal was {refused:?}");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-chamfer"), "an unnamed edge still added a chamfer");
    let volume = live(&doc, made);
    assert!((volume - 6000.0).abs() < 1.0, "volume {volume} mm^3, an unnamed edge beveled the box");
}

/// A second chamfer of the extrusion starts another body, even before the first has been read.
#[test]
fn a_second_chamfer_of_the_extrusion_is_refused() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edges: Vec<EdgeId> = doc.bodies[0].edges.iter().filter(|edge| (edge.length.mm - 20.0).abs() < 0.1).map(|edge| edge.id).take(2).collect();
    assert_eq!(edges.len(), 2, "two long edges");
    let first = session.chamfer(made, edges[0], Distance { mm: 2.0 }).expect("a chamfer");
    let refused = session.chamfer(made, edges[1], Distance { mm: 2.0 }).expect_err("a second branch");
    assert!(refused.text.contains("msg-body-consumed"), "the refusal was {refused:?}");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-chamfer").count(), 1, "a second chamfer was added");
    let volume = live(&doc, first);
    let want = 6000.0 - removed(2.0, 20.0);
    assert!((volume - want).abs() < 1.0, "volume {volume} mm^3, wanted {want} within 1");
}

/// A distance of -2 mm is the kernel's own sentence, and the solid stays the box.
#[test]
fn a_negative_distance_does_not_bevel_the_edge() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = long_edge(&doc);
    let beveled = session.chamfer(made, edge, Distance { mm: -2.0 }).expect("the node is written");
    let doc = session.document();
    let text = refusal(&doc);
    assert!(text.contains("chamfer -2.00 too big"), "the refusal was {text}");
    let volume = live(&doc, beveled);
    assert!((volume - 6000.0).abs() < 1.0, "volume {volume} mm^3, the refusal still beveled the edge");
    let want = 6000.0 - removed(2.0, 20.0);
    assert!((volume - want).abs() >= 1.0, "volume {volume} mm^3 is the beveled solid");
}

/// A distance the edge cannot take is replaced when the extrusion is named again.
#[test]
fn a_refused_distance_can_be_replaced() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = long_edge(&doc);
    let failed = session.chamfer(made, edge, Distance { mm: 100.0 }).expect("the node is written");
    let doc = session.document();
    assert!(refusal(&doc).contains("chamfer 100.00 too big"), "the refusal was {}", refusal(&doc));
    let beveled = session.chamfer(made, edge, Distance { mm: 2.0 }).expect("the chamfer is tried again");
    assert_eq!(beveled, failed, "a second chamfer was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-chamfer").count(), 1, "the refused chamfer was left beside the new one");
    let volume = live(&doc, beveled);
    let want = 6000.0 - removed(2.0, 20.0);
    assert!((volume - want).abs() < 1.0, "volume {volume} mm^3, wanted {want} within 1");
}

/// The id a refused chamfer returned is that same chamfer.
#[test]
fn a_refused_chamfer_named_again_is_the_same_chamfer() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = long_edge(&doc);
    let failed = session.chamfer(made, edge, Distance { mm: 100.0 }).expect("the node is written");
    let _doc = session.document();
    let beveled = session.chamfer(failed, edge, Distance { mm: 2.0 }).expect("the chamfer is tried again");
    assert_eq!(beveled, failed, "a second chamfer was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-chamfer").count(), 1, "the refused chamfer was left beside the new one");
    let volume = live(&doc, beveled);
    let want = 6000.0 - removed(2.0, 20.0);
    assert!((volume - want).abs() < 1.0, "volume {volume} mm^3, wanted {want} within 1");
}

/// The volume a right-angled corner loses when it is beveled: distance^2 / 2 * length.
fn removed(distance: f64, length: f64) -> f64 {
    distance * distance / 2.0 * length
}

fn long_edge(doc: &qymcad_drive::Document) -> EdgeId {
    doc.bodies[0].edges.iter().find(|edge| (edge.length.mm - 20.0).abs() < 0.1).expect("the 20 mm edge").id
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

fn refusal(doc: &qymcad_drive::Document) -> String {
    match &doc.rebuild {
        Rebuild::Refused { refusals } => refusals.iter().map(|refusal| refusal.text.clone()).collect::<Vec<_>>().join(" "),
        other => panic!("the rebuild was {other:?}"),
    }
}

/// An id that was never a solid is refused before a node is added. A later chamfer of the real body can then be read.
#[test]
fn a_body_that_was_never_built_is_refused() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = long_edge(&doc);
    let refused = session.chamfer(BodyId { id: 0 }, edge, Distance { mm: 2.0 }).expect_err("a body that was never built");
    assert!(refused.text.contains("error-source-body-not-built"), "the refusal was {refused:?}");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-chamfer"), "a chamfer was added on body 0");
    let beveled = session.chamfer(made, edge, Distance { mm: 2.0 }).expect("a chamfer");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-chamfer").count(), 1, "the empty body left a chamfer beside the real one");
    let volume = live(&doc, beveled);
    let want = 6000.0 - removed(2.0, 20.0);
    assert!((volume - want).abs() < 1.0, "volume {volume} mm^3, wanted {want} within 1");
}

/// A sketch id is not a solid. A chamfer of it stays in the read after the real body is beveled.
#[test]
fn a_sketch_is_not_a_body_to_chamfer() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = long_edge(&doc);
    let refused = session.chamfer(BodyId { id: drawn.sketch.id }, edge, Distance { mm: 2.0 }).expect_err("a sketch");
    assert!(refused.text.contains("error-source-body-deleted"), "the refusal was {refused:?}");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-chamfer"), "a chamfer was added on the sketch");
    let beveled = session.chamfer(made, edge, Distance { mm: 2.0 }).expect("a chamfer");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-chamfer").count(), 1, "the sketch left a chamfer beside the real one");
    let volume = live(&doc, beveled);
    let want = 6000.0 - removed(2.0, 20.0);
    assert!((volume - want).abs() < 1.0, "volume {volume} mm^3, wanted {want} within 1");
}

/// A fillet of an id that was never a solid is the same hole: the later rounding of the box cannot be read on its own.
#[test]
fn a_fillet_of_a_body_that_was_never_built_is_refused() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = long_edge(&doc);
    let refused = session.fillet(BodyId { id: 0 }, edge, qymcad_drive::Radius { mm: 2.0 }).expect_err("a body that was never built");
    assert!(refused.text.contains("error-source-body-not-built"), "the refusal was {refused:?}");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-fillet"), "a fillet was added on body 0");
    let rounded = session.fillet(made, edge, qymcad_drive::Radius { mm: 2.0 }).expect("a fillet");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-fillet").count(), 1, "the empty body left a fillet beside the real one");
    let volume = live(&doc, rounded);
    let want = 6000.0 - 2.0 * 2.0 * (1.0 - std::f64::consts::PI / 4.0) * 20.0;
    assert!((volume - want).abs() < 1.0, "volume {volume} mm^3, wanted {want} within 1");
}

fn rectangle_10_by_20() -> Corners {
    Corners { from: qymcad_drive::From { at: (0.0, 0.0) }, to: qymcad_drive::To { at: (10.0, 20.0) } }
}

/// A fillet the rebuild refused still holds the extrusion. A chamfer beside it would leave that refusal in the read.
#[test]
fn a_refused_fillet_is_not_a_chamfer() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = long_edge(&doc);
    let rounded = session.fillet(made, edge, qymcad_drive::Radius { mm: 100.0 }).expect("the fillet node");
    let doc = session.document();
    assert!(refusal(&doc).contains("fillet R100.00 failed on 1 edges"), "the refusal was {}", refusal(&doc));
    let refused = session.chamfer(made, edge, Distance { mm: 2.0 }).expect_err("a chamfer beside the fillet");
    assert!(refused.text.contains("msg-body-consumed"), "the refusal was {refused:?}");
    let doc = session.document();
    assert!(refusal(&doc).contains("fillet R100.00 failed on 1 edges"), "the refusal was {}", refusal(&doc));
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-chamfer"), "a chamfer was added beside the fillet");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-fillet").count(), 1, "the fillet was removed");
    assert!((live(&doc, rounded) - 6000.0).abs() < 1.0, "the fillet no longer passes the box through");
}

/// The id a refused fillet returned was never built. A chamfer of that id adds a node the rebuild cannot make.
#[test]
fn a_refused_fillets_id_is_not_a_body() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = long_edge(&doc);
    let rounded = session.fillet(made, edge, qymcad_drive::Radius { mm: 100.0 }).expect("the fillet node");
    let _doc = session.document();
    let refused = session.chamfer(rounded, edge, Distance { mm: 2.0 }).expect_err("a chamfer of the fillet id");
    assert!(refused.text.contains("error-source-body-not-built"), "the refusal was {refused:?}");
    let doc = session.document();
    assert_eq!(refusal(&doc), "fillet R100.00 failed on 1 edges", "the refusal was {}", refusal(&doc));
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-chamfer"), "a chamfer was added on the fillet id");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-fillet").count(), 1, "the fillet was removed");
}

/// A chamfer the rebuild refused still holds the extrusion. A fillet beside it would leave that refusal in the read.
#[test]
fn a_refused_chamfer_is_not_a_fillet() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = long_edge(&doc);
    let beveled = session.chamfer(made, edge, Distance { mm: 100.0 }).expect("the chamfer node");
    let doc = session.document();
    assert!(refusal(&doc).contains("chamfer 100.00 too big"), "the refusal was {}", refusal(&doc));
    let refused = session.fillet(made, edge, qymcad_drive::Radius { mm: 2.0 }).expect_err("a fillet beside the chamfer");
    assert!(refused.text.contains("msg-body-consumed"), "the refusal was {refused:?}");
    let doc = session.document();
    assert!(refusal(&doc).contains("chamfer 100.00 too big"), "the refusal was {}", refusal(&doc));
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-fillet"), "a fillet was added beside the chamfer");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-chamfer").count(), 1, "the chamfer was removed");
    assert!((live(&doc, beveled) - 6000.0).abs() < 1.0, "the chamfer no longer passes the box through");
}

/// The id a refused chamfer returned was never built. A fillet of that id adds a node the rebuild cannot make.
#[test]
fn a_refused_chamfers_id_is_not_a_body() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = long_edge(&doc);
    let beveled = session.chamfer(made, edge, Distance { mm: 100.0 }).expect("the chamfer node");
    let _doc = session.document();
    let refused = session.fillet(beveled, edge, qymcad_drive::Radius { mm: 2.0 }).expect_err("a fillet of the chamfer id");
    assert!(refused.text.contains("error-source-body-not-built"), "the refusal was {refused:?}");
    let doc = session.document();
    assert_eq!(refusal(&doc), "chamfer 100.00 too big", "the refusal was {}", refusal(&doc));
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-fillet"), "a fillet was added on the chamfer id");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-chamfer").count(), 1, "the chamfer was removed");
}

fn rectangle_10_by_10() -> Corners {
    Corners { from: qymcad_drive::From { at: (0.0, 0.0) }, to: qymcad_drive::To { at: (10.0, 10.0) } }
}
