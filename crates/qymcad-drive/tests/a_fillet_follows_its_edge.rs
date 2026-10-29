use qymcad_drive::{BodyId, Corners, EdgeId, FaceRole, Height, Measure, Radius, Rebuild, Session, Sound, WorldPlane};

/// One named edge of the box is rounded, and the same fillet follows that edge after the sketch above
/// it is resized.
#[test]
fn a_fillet_follows_its_edge() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = long_edge(&doc);
    let rounded = session.fillet(made, edge, Radius { mm: 2.0 }).expect("a fillet");
    let built = session.document();
    assert_eq!(built.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    let volume = live(&built, rounded);
    let want = 6000.0 - removed(2.0, 20.0);
    assert!((volume - want).abs() < 1.0, "volume {volume} mm^3, wanted {want} within 1");
    assert_ne!(rounded, made, "the fillet is the extrusion");

    session.resize(drawn, rectangle_10_by_10()).expect("the rectangle moves");
    let shrunk = session.document();
    assert_eq!(shrunk.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert_eq!(shrunk.sketches.len(), 1, "the resize does not add a sketch");
    assert_eq!(shrunk.sketches[0].id, drawn.sketch.id, "the sketch id is unchanged");
    let again = live(&shrunk, rounded);
    let want = 3000.0 - removed(2.0, 10.0);
    assert!((again - want).abs() < 1.0, "volume {again} mm^3, wanted {want} within 1");
}

/// A radius the edge cannot take is the kernel's own sentence, and the solid stays the box.
#[test]
fn a_radius_the_edge_cannot_take_is_refused() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = long_edge(&doc);
    let rounded = session.fillet(made, edge, Radius { mm: 100.0 }).expect("the node is written");
    let doc = session.document();
    let text = refusal(&doc);
    assert!(text.contains("fillet R100.00 failed"), "the refusal was {text}");
    let volume = live(&doc, rounded);
    assert!((volume - 6000.0).abs() < 1.0, "volume {volume} mm^3, the refusal still rounded the edge");
}

/// An id that is not an edge of this body is refused. An empty list would round every edge.
#[test]
fn an_id_that_is_not_an_edge_is_refused() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let cap = doc.bodies[0].faces.iter().find(|face| face.role == FaceRole::CapEnd).expect("the end cap").id;
    for edge in [EdgeId { id: 1 }, EdgeId { id: cap.id }] {
        let rounded = session.fillet(made, edge, Radius { mm: 2.0 }).expect("the node is written");
        let doc = session.document();
        let text = refusal(&doc);
        assert!(text.contains("error-edges-not-found"), "the refusal was {text}");
        let volume = live(&doc, rounded);
        assert!((volume - 6000.0).abs() < 1.0, "volume {volume} mm^3, a missing edge rounded the box");
    }
}

/// A radius of -2 mm is the kernel's own sentence, and the solid stays the box.
///
/// The kernel refuses that radius before it rounds, so the node is written and the read keeps the
/// sentence. The rounded solid of this edge would be about 5982.83 mm^3, and this one is not.
#[test]
fn a_negative_radius_does_not_round_the_edge() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = long_edge(&doc);
    let rounded = session.fillet(made, edge, Radius { mm: -2.0 }).expect("the node is written");
    let doc = session.document();
    let text = refusal(&doc);
    assert!(text.contains("fillet R-2.00 failed"), "the refusal was {text}");
    let volume = live(&doc, rounded);
    assert!((volume - 6000.0).abs() < 1.0, "volume {volume} mm^3, the refusal still rounded the edge");
}

/// A radius the edge cannot take is replaced by a later fillet of the same body. The refused node stays,
/// so the read can come back quiet instead of keeping the refusal beside a second body.
#[test]
fn a_refused_radius_can_be_replaced() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = long_edge(&doc);
    let failed = session.fillet(made, edge, Radius { mm: 100.0 }).expect("the node is written");
    let doc = session.document();
    assert!(refusal(&doc).contains("fillet R100.00 failed"), "the refusal was {}", refusal(&doc));
    let rounded = session.fillet(made, edge, Radius { mm: 2.0 }).expect("the fillet is tried again");
    assert_eq!(rounded, failed, "a second fillet was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-fillet").count(), 1, "the refused fillet was left beside the new one");
    let volume = live(&doc, rounded);
    let want = 6000.0 - removed(2.0, 20.0);
    assert!((volume - want).abs() < 1.0, "volume {volume} mm^3, wanted {want} within 1");

    session.resize(drawn, rectangle_10_by_10()).expect("the rectangle moves");
    let shrunk = session.document();
    assert_eq!(shrunk.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    let again = live(&shrunk, rounded);
    let want = 3000.0 - removed(2.0, 10.0);
    assert!((again - want).abs() < 1.0, "volume {again} mm^3, wanted {want} within 1");
}

/// The id a refused fillet returned is that same fillet. A new node on it has no built body to round.
#[test]
fn a_refused_fillet_named_again_is_the_same_fillet() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = long_edge(&doc);
    let failed = session.fillet(made, edge, Radius { mm: 100.0 }).expect("the node is written");
    let _doc = session.document();
    let rounded = session.fillet(failed, edge, Radius { mm: 2.0 }).expect("the fillet is tried again");
    assert_eq!(rounded, failed, "a second fillet was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-fillet").count(), 1, "the refused fillet was left beside the new one");
    let volume = live(&doc, rounded);
    let want = 6000.0 - removed(2.0, 20.0);
    assert!((volume - want).abs() < 1.0, "volume {volume} mm^3, wanted {want} within 1");
}

/// An edge id of 0 is a pick of nothing. A pick of nothing rounds every edge and the rebuild stays quiet.
#[test]
fn an_unnamed_edge_does_not_round_the_box() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let _doc = session.document();
    let refused = session.fillet(made, EdgeId { id: 0 }, Radius { mm: 2.0 }).expect_err("an unnamed edge");
    assert!(refused.text.contains("error-edges-not-found"), "the refusal was {refused:?}");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-fillet"), "an unnamed edge still added a fillet");
    let volume = live(&doc, made);
    assert!((volume - 6000.0).abs() < 1.0, "volume {volume} mm^3, an unnamed edge rounded the box");
}

/// A second fillet of the extrusion starts another body. The first result is the body to continue from.
#[test]
fn a_second_fillet_of_the_extrusion_is_refused() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edges: Vec<EdgeId> = doc.bodies[0].edges.iter().filter(|edge| (edge.length.mm - 20.0).abs() < 0.1).map(|edge| edge.id).take(2).collect();
    assert_eq!(edges.len(), 2, "two long edges");
    let first = session.fillet(made, edges[0], Radius { mm: 2.0 }).expect("a fillet");
    let refused = session.fillet(made, edges[1], Radius { mm: 2.0 }).expect_err("a second branch");
    assert!(refused.text.contains("msg-body-consumed"), "the refusal was {refused:?}");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-fillet").count(), 1, "a second fillet was added");
    let volume = live(&doc, first);
    let want = 6000.0 - removed(2.0, 20.0);
    assert!((volume - want).abs() < 1.0, "volume {volume} mm^3, wanted {want} within 1");
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

/// The volume a square corner loses when it is rounded: r^2 * (1 - pi/4) * length.
fn removed(radius: f64, length: f64) -> f64 {
    radius * radius * (1.0 - std::f64::consts::PI / 4.0) * length
}

fn rectangle_10_by_20() -> Corners {
    Corners { from: qymcad_drive::From { at: (0.0, 0.0) }, to: qymcad_drive::To { at: (10.0, 20.0) } }
}

fn rectangle_10_by_10() -> Corners {
    Corners { from: qymcad_drive::From { at: (0.0, 0.0) }, to: qymcad_drive::To { at: (10.0, 10.0) } }
}
