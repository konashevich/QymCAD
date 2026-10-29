use qymcad_drive::{BodyId, Corners, FaceId, Height, Measure, Rebuild, Session, Sound, Thickness, WorldPlane};

/// One named cap of the box is opened, and the same shell follows that face after the sketch above it
/// is resized.
#[test]
fn a_shell_follows_its_face() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let face = cap(&doc);
    let opened = session.shell(made, face, Thickness { mm: 2.0 }).expect("a shell");
    let built = session.document();
    assert_eq!(built.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    let volume = live(&built, opened);
    let want = hollow(Thickness { mm: 2.0 }, box_10_by_20_by_30_mm());
    assert!((volume - want).abs() < 1.0, "volume {volume} mm^3, wanted {want} within 1");
    assert_ne!(opened, made, "the shell is the extrusion");

    session.resize(drawn, rectangle_10_by_10()).expect("the rectangle moves");
    let shrunk = session.document();
    assert_eq!(shrunk.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert_eq!(shrunk.sketches.len(), 1, "the resize does not add a sketch");
    assert_eq!(shrunk.sketches[0].id, drawn.sketch.id, "the sketch id is unchanged");
    let again = live(&shrunk, opened);
    let want = hollow(Thickness { mm: 2.0 }, BoxMm { width: 10.0, depth: 10.0, height: 30.0 });
    assert!((again - want).abs() < 1.0, "volume {again} mm^3, wanted {want} within 1");
}

/// The outer size of the box the shell opens, in mm.
struct BoxMm {
    width: f64,
    depth: f64,
    height: f64,
}

fn box_10_by_20_by_30_mm() -> BoxMm {
    BoxMm { width: 10.0, depth: 20.0, height: 30.0 }
}

/// The volume left when an end cap is opened and the wall is `thickness` thick.
///
/// The four side walls each take `thickness`, and the closed cap takes it once. The open cap takes none.
fn hollow(thickness: Thickness, outer: BoxMm) -> f64 {
    let solid = outer.width * outer.depth * outer.height;
    let cavity = (outer.width - 2.0 * thickness.mm) * (outer.depth - 2.0 * thickness.mm) * (outer.height - thickness.mm);
    solid - cavity
}

fn cap(doc: &qymcad_drive::Document) -> FaceId {
    doc.bodies[0].faces.iter().find(|face| (face.area.mm2 - 200.0).abs() < 1.0).expect("the end cap").id
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

fn rectangle_10_by_20() -> Corners {
    Corners { from: qymcad_drive::From { at: (0.0, 0.0) }, to: qymcad_drive::To { at: (10.0, 20.0) } }
}

fn rectangle_10_by_10() -> Corners {
    Corners { from: qymcad_drive::From { at: (0.0, 0.0) }, to: qymcad_drive::To { at: (10.0, 10.0) } }
}

/// A wall thicker than the box can hold is the kernel's own sentence, and the solid stays the box.
#[test]
fn a_thickness_the_box_cannot_hold_is_refused() {
    let mut session = Session::empty();
    let made = box_10_by_20_by_30(&mut session);
    let doc = session.document();
    let face = cap(&doc);
    let opened = session.shell(made, face, Thickness { mm: 6.0 }).expect("the node is written");
    let doc = session.document();
    let text = refusal(&doc);
    assert_eq!(text, "kernel failed to offset faces on this body", "the refusal was {text}");
    let volume = live(&doc, opened);
    assert!((volume - 6000.0).abs() < 1.0, "volume {volume} mm^3, the refusal still opened the cap");
    let want = hollow(Thickness { mm: 6.0 }, box_10_by_20_by_30_mm());
    assert!((volume - want).abs() >= 1.0, "volume {volume} mm^3 is the hollow solid");
}

/// Naming the extrusion again replaces the shell the rebuild refused.
#[test]
fn a_refused_thickness_can_be_replaced() {
    let mut session = Session::empty();
    let made = box_10_by_20_by_30(&mut session);
    let doc = session.document();
    let face = cap(&doc);
    let failed = session.shell(made, face, Thickness { mm: 6.0 }).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "kernel failed to offset faces on this body");
    let opened = session.shell(made, face, Thickness { mm: 2.0 }).expect("the shell is tried again");
    assert_eq!(opened, failed, "a second shell was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-shell").count(), 1, "the refused shell was left beside the new one");
    let volume = live(&doc, opened);
    let want = hollow(Thickness { mm: 2.0 }, box_10_by_20_by_30_mm());
    assert!((volume - want).abs() < 1.0, "volume {volume} mm^3, wanted {want} within 1");
}

/// Naming the id the refused call returned replaces that same node.
#[test]
fn a_refused_shell_is_replaced_when_its_own_id_is_named() {
    let mut session = Session::empty();
    let made = box_10_by_20_by_30(&mut session);
    let doc = session.document();
    let face = cap(&doc);
    let failed = session.shell(made, face, Thickness { mm: 6.0 }).expect("the node is written");
    let _doc = session.document();
    let opened = session.shell(failed, face, Thickness { mm: 2.0 }).expect("the shell is tried again");
    assert_eq!(opened, failed, "a second shell was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-shell").count(), 1, "the refused shell was left beside the new one");
    let volume = live(&doc, opened);
    let want = hollow(Thickness { mm: 2.0 }, box_10_by_20_by_30_mm());
    assert!((volume - want).abs() < 1.0, "volume {volume} mm^3, wanted {want} within 1");
}

/// A face that is not a face of this body is refused, and the second attempt reuses that node.
#[test]
fn a_missing_face_is_refused_and_the_node_is_reused() {
    let mut session = Session::empty();
    let made = box_10_by_20_by_30(&mut session);
    let doc = session.document();
    let edge = doc.bodies[0].edges.iter().find(|edge| (edge.length.mm - 20.0).abs() < 0.1).expect("a 20 mm edge").id;
    let missing = session.shell(made, FaceId { id: 1 }, Thickness { mm: 2.0 }).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-faces-not-found", "the refusal was {}", refusal(&doc));
    assert!((live(&doc, made) - 6000.0).abs() < 1.0, "the box was opened");
    let again = session.shell(made, FaceId { id: edge.id }, Thickness { mm: 2.0 }).expect("the same node");
    assert_eq!(again, missing, "a second shell was added");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-faces-not-found", "the refusal was {}", refusal(&doc));
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-shell").count(), 1, "the refused shell was left beside the new one");
    assert!((live(&doc, made) - 6000.0).abs() < 1.0, "the box was opened");
}

/// A face id of 0 is an empty pick. The kernel is not asked to open every face.
#[test]
fn a_face_id_of_zero_is_not_found() {
    let mut session = Session::empty();
    let made = box_10_by_20_by_30(&mut session);
    let _doc = session.document();
    let opened = session.shell(made, FaceId { id: 0 }, Thickness { mm: 2.0 }).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-faces-not-found", "the refusal was {}", refusal(&doc));
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-shell").count(), 1, "the empty pick added no shell");
    let volume = live(&doc, opened);
    assert!((volume - 6000.0).abs() < 1.0, "volume {volume} mm^3, the empty pick opened a face");
}

/// A thickness that is not a positive finite length is refused before a node is added.
#[test]
fn a_thickness_that_is_not_positive_is_refused() {
    for mm in [-2.0, 0.0, f64::NAN, f64::INFINITY] {
        let mut session = Session::empty();
        let made = box_10_by_20_by_30(&mut session);
        let doc = session.document();
        let face = cap(&doc);
        let refused = session.shell(made, face, Thickness { mm }).expect_err("a thickness that is not positive");
        assert_eq!(refused.text, "sketch-shell-thickness", "the refusal was {refused:?}");
        let doc = session.document();
        assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
        assert!(doc.nodes.iter().all(|node| node.name != "feat-name-shell"), "a shell was added for {mm}");
        assert!((live(&doc, made) - 6000.0).abs() < 1.0, "volume changed for {mm}");
    }
}

/// A thickness that is not positive does not rewrite a shell the rebuild has already refused.
#[test]
fn a_thickness_that_is_not_positive_leaves_the_refused_shell() {
    let mut session = Session::empty();
    let made = box_10_by_20_by_30(&mut session);
    let doc = session.document();
    let face = cap(&doc);
    let failed = session.shell(made, face, Thickness { mm: 6.0 }).expect("the node is written");
    let _doc = session.document();
    let refused = session.shell(made, face, Thickness { mm: -2.0 }).expect_err("a thickness that is not positive");
    assert_eq!(refused.text, "sketch-shell-thickness", "the refusal was {refused:?}");
    let doc = session.document();
    assert_eq!(refusal(&doc), "kernel failed to offset faces on this body", "the refusal was {}", refusal(&doc));
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-shell").count(), 1, "the refused shell was replaced");
    assert!((live(&doc, failed) - 6000.0).abs() < 1.0, "the refused shell was hollowed");
}

/// A second shell of the extrusion, before the first has been read, is refused.
#[test]
fn a_second_shell_of_the_extrusion_is_refused() {
    let mut session = Session::empty();
    let made = box_10_by_20_by_30(&mut session);
    let doc = session.document();
    let faces: Vec<FaceId> = doc.bodies[0].faces.iter().filter(|face| (face.area.mm2 - 200.0).abs() < 1.0).map(|face| face.id).take(2).collect();
    assert_eq!(faces.len(), 2, "two end caps");
    let first = session.shell(made, faces[0], Thickness { mm: 2.0 }).expect("a shell");
    let refused = session.shell(made, faces[1], Thickness { mm: 2.0 }).expect_err("a second branch");
    assert!(refused.text.contains("msg-body-consumed"), "the refusal was {refused:?}");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-shell").count(), 1, "a second shell was added");
    let volume = live(&doc, first);
    let want = hollow(Thickness { mm: 2.0 }, box_10_by_20_by_30_mm());
    assert!((volume - want).abs() < 1.0, "volume {volume} mm^3, wanted {want} within 1");
}

/// An id that was never a solid is refused before a node is added.
#[test]
fn a_body_that_was_never_built_is_refused() {
    let mut session = Session::empty();
    let made = box_10_by_20_by_30(&mut session);
    let doc = session.document();
    let face = cap(&doc);
    let refused = session.shell(BodyId { id: 0 }, face, Thickness { mm: 2.0 }).expect_err("a body that was never built");
    assert!(refused.text.contains("error-source-body-not-built"), "the refusal was {refused:?}");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-shell"), "a shell was added on body 0");
    let opened = session.shell(made, face, Thickness { mm: 2.0 }).expect("a shell");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-shell").count(), 1, "the empty body left a shell beside the real one");
    let volume = live(&doc, opened);
    let want = hollow(Thickness { mm: 2.0 }, box_10_by_20_by_30_mm());
    assert!((volume - want).abs() < 1.0, "volume {volume} mm^3, wanted {want} within 1");
}

/// A shell beside a fillet the rebuild has refused is refused, and the fillet stays.
#[test]
fn a_refused_fillet_is_not_a_shell() {
    let mut session = Session::empty();
    let made = box_10_by_20_by_30(&mut session);
    let doc = session.document();
    let face = cap(&doc);
    let edge = doc.bodies[0].edges.iter().find(|edge| (edge.length.mm - 20.0).abs() < 0.1).expect("a 20 mm edge").id;
    let rounded = session.fillet(made, edge, qymcad_drive::Radius { mm: 100.0 }).expect("the fillet node");
    let doc = session.document();
    assert!(refusal(&doc).contains("fillet R100.00 failed on 1 edges"), "the refusal was {}", refusal(&doc));
    let refused = session.shell(made, face, Thickness { mm: 2.0 }).expect_err("a shell beside the fillet");
    assert!(refused.text.contains("msg-body-consumed"), "the refusal was {refused:?}");
    let doc = session.document();
    assert!(refusal(&doc).contains("fillet R100.00 failed on 1 edges"), "the refusal was {}", refusal(&doc));
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-shell"), "a shell was added beside the fillet");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-fillet").count(), 1, "the fillet was removed");
    assert!((live(&doc, rounded) - 6000.0).abs() < 1.0, "the fillet no longer passes the box through");
}

fn box_10_by_20_by_30(session: &mut Session) -> BodyId {
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion")
}

fn refusal(doc: &qymcad_drive::Document) -> String {
    match &doc.rebuild {
        Rebuild::Refused { refusals } => refusals.iter().map(|refusal| refusal.text.clone()).collect::<Vec<_>>().join(" "),
        other => panic!("the rebuild was not refused: {other:?}"),
    }
}
