use qymcad_drive::{BodyId, Corners, Depth, Diameter, Dimension, Distance, Expression, FaceId, Height, Measure, ParameterName, Radius, Rebuild, Session, Sound, Thickness, WorldPlane};

/// A hole through the centre of an end cap, and the same hole after the extrusion above it is shortened.
#[test]
fn a_hole_follows_its_face() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let face = cap(&doc);
    let drilled = session.hole(made, face, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect("a hole");
    let built = session.document();
    assert_eq!(built.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    let volume = live(&built, drilled);
    let want = 6000.0 - cylinder(Diameter { mm: 4.0 }, 30.0);
    assert!((volume - want).abs() < 1.0, "volume {volume} mm^3, wanted {want} within 1");
    assert_ne!(drilled, made, "the hole is the extrusion");

    session.parameter(ParameterName { text: "H".into() }, Expression { text: "15".into() }).expect("a parameter");
    session.bind(made, Dimension::Height, Expression { text: "H".into() }).expect("the height");
    let shortened = session.document();
    assert_eq!(shortened.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert_eq!(shortened.nodes.iter().filter(|node| node.name == "feat-name-hole").count(), 1, "the height added a hole");
    let again = live(&shortened, drilled);
    let want = 3000.0 - cylinder(Diameter { mm: 4.0 }, 15.0);
    assert!((again - want).abs() < 1.0, "volume {again} mm^3, wanted {want} within 1");
}

/// The cylinder a hole of `diameter` takes out of a wall `through` mm thick.
fn cylinder(diameter: Diameter, through: f64) -> f64 {
    std::f64::consts::PI * diameter.mm * diameter.mm / 4.0 * through
}

fn cap(doc: &qymcad_drive::Document) -> FaceId {
    doc.bodies[0].faces.iter().find(|face| (face.area.mm2 - 200.0).abs() < 1.0).expect("the end cap").id
}

fn wall(doc: &qymcad_drive::Document) -> FaceId {
    doc.bodies[0].faces.iter().find(|face| (face.area.mm2 - 600.0).abs() < 1.0).expect("the long wall").id
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

fn refusal(doc: &qymcad_drive::Document) -> String {
    match &doc.rebuild {
        Rebuild::Refused { refusals } => refusals.iter().map(|refusal| refusal.text.clone()).collect::<Vec<_>>().join(" "),
        other => panic!("the rebuild was {other:?}"),
    }
}

fn box_10_by_20_by_30(session: &mut Session) -> BodyId {
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion")
}

/// A face that is not on the box is refused, and the second missing face reuses that node.
#[test]
fn a_missing_face_is_refused_and_the_node_is_reused() {
    let mut session = Session::empty();
    let made = box_10_by_20_by_30(&mut session);
    let doc = session.document();
    let edge = doc.bodies[0].edges[0].id;
    let failed = session.hole(made, FaceId { id: 1 }, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-face-not-found", "the refusal was {}", refusal(&doc));
    let volume = live(&doc, failed);
    assert!((volume - 6000.0).abs() < 1.0, "volume {volume} mm^3, the missing face was drilled");
    let again = session.hole(made, FaceId { id: edge.id }, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect("the hole is tried again");
    assert_eq!(again, failed, "a second hole was added");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-face-not-found", "the refusal was {}", refusal(&doc));
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-hole").count(), 1, "the refused hole was left beside the new one");
    let volume = live(&doc, again);
    assert!((volume - 6000.0).abs() < 1.0, "volume {volume} mm^3, the edge was drilled");
}

/// Naming the extrusion again replaces the hole the rebuild refused, and the new face's centre is drilled.
#[test]
fn a_refused_hole_can_be_replaced() {
    let mut session = Session::empty();
    let made = box_10_by_20_by_30(&mut session);
    let doc = session.document();
    let face = wall(&doc);
    let failed = session.hole(made, FaceId { id: 1 }, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-face-not-found");
    let drilled = session.hole(made, face, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect("the hole is tried again");
    assert_eq!(drilled, failed, "a second hole was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-hole").count(), 1, "the refused hole was left beside the new one");
    let volume = live(&doc, drilled);
    let want = 6000.0 - cylinder(Diameter { mm: 4.0 }, 10.0);
    assert!((volume - want).abs() < 1.0, "volume {volume} mm^3, wanted {want} within 1");
}

/// Naming the id the refused call returned replaces that same node.
#[test]
fn a_refused_hole_is_replaced_when_its_own_id_is_named() {
    let mut session = Session::empty();
    let made = box_10_by_20_by_30(&mut session);
    let doc = session.document();
    let face = wall(&doc);
    let failed = session.hole(made, FaceId { id: 1 }, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect("the node is written");
    let _doc = session.document();
    let drilled = session.hole(failed, face, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect("the hole is tried again");
    assert_eq!(drilled, failed, "a second hole was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-hole").count(), 1, "the refused hole was left beside the new one");
    let volume = live(&doc, drilled);
    let want = 6000.0 - cylinder(Diameter { mm: 4.0 }, 10.0);
    assert!((volume - want).abs() < 1.0, "volume {volume} mm^3, wanted {want} within 1");
}

/// A diameter or a depth that is not a positive finite length is refused before a node is added.
#[test]
fn a_size_that_is_not_positive_is_refused() {
    let diameters = [
        BadSize { mm: -4.0, key: "sketch-hole-diameter" },
        BadSize { mm: 0.0, key: "sketch-hole-diameter" },
        BadSize { mm: f64::NAN, key: "sketch-hole-diameter" },
        BadSize { mm: f64::INFINITY, key: "sketch-hole-diameter" },
        BadSize { mm: f64::NEG_INFINITY, key: "sketch-hole-diameter" },
    ];
    for size in diameters {
        let mut session = Session::empty();
        let made = box_10_by_20_by_30(&mut session);
        let doc = session.document();
        let face = cap(&doc);
        let refused = session.hole(made, face, Diameter { mm: size.mm }, Depth { mm: 40.0 }).expect_err("the diameter is not positive");
        assert_eq!(refused.text, size.key, "diameter {} was {refused:?}", size.mm);
        let doc = session.document();
        assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
        assert!(doc.nodes.iter().all(|node| node.name != "feat-name-hole"), "a hole was added for diameter {}", size.mm);
        let volume = live(&doc, made);
        assert!((volume - 6000.0).abs() < 1.0, "volume {volume} mm^3, diameter {} was drilled", size.mm);
    }
    let depths = [
        BadSize { mm: -40.0, key: "sketch-hole-depth" },
        BadSize { mm: 0.0, key: "sketch-hole-depth" },
        BadSize { mm: f64::NAN, key: "sketch-hole-depth" },
        BadSize { mm: f64::INFINITY, key: "sketch-hole-depth" },
        BadSize { mm: f64::NEG_INFINITY, key: "sketch-hole-depth" },
    ];
    for size in depths {
        let mut session = Session::empty();
        let made = box_10_by_20_by_30(&mut session);
        let doc = session.document();
        let face = cap(&doc);
        let refused = session.hole(made, face, Diameter { mm: 4.0 }, Depth { mm: size.mm }).expect_err("the depth is not positive");
        assert_eq!(refused.text, size.key, "depth {} was {refused:?}", size.mm);
        let doc = session.document();
        assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
        assert!(doc.nodes.iter().all(|node| node.name != "feat-name-hole"), "a hole was added for depth {}", size.mm);
        let volume = live(&doc, made);
        assert!((volume - 6000.0).abs() < 1.0, "volume {volume} mm^3, depth {} was drilled", size.mm);
    }
}

/// One bad length and the catalogue key it must return. The other length stays a positive finite number.
struct BadSize {
    mm: f64,
    key: &'static str,
}

/// A depth the kernel would accept as its absolute value does not rewrite a hole the rebuild already refused.
#[test]
fn a_depth_that_is_not_positive_leaves_the_refused_hole() {
    let mut session = Session::empty();
    let made = box_10_by_20_by_30(&mut session);
    let doc = session.document();
    let face = cap(&doc);
    let failed = session.hole(made, FaceId { id: 1 }, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-face-not-found");
    let depth = session.hole(made, face, Diameter { mm: 4.0 }, Depth { mm: -40.0 }).expect_err("the depth is not positive");
    assert_eq!(depth.text, "sketch-hole-depth", "the refusal was {depth:?}");
    let diameter = session.hole(made, face, Diameter { mm: -4.0 }, Depth { mm: 40.0 }).expect_err("the diameter is not positive");
    assert_eq!(diameter.text, "sketch-hole-diameter", "the refusal was {diameter:?}");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-face-not-found", "the refused hole was rewritten");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-hole").count(), 1, "a hole was added");
    let volume = live(&doc, failed);
    assert!((volume - 6000.0).abs() < 1.0, "volume {volume} mm^3, the negative depth was drilled");
}

/// A face id of 0 is not a pick. It is refused before a node, and a later hole of the real face can be read.
#[test]
fn a_face_id_of_zero_is_not_found() {
    let mut session = Session::empty();
    let made = box_10_by_20_by_30(&mut session);
    let doc = session.document();
    let face = cap(&doc);
    let refused = session.hole(made, FaceId { id: 0 }, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect_err("a face id of zero");
    assert_eq!(refused.text, "error-face-not-found", "the refusal was {refused:?}");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-hole"), "a hole was added for face 0");
    let drilled = session.hole(made, face, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect("a hole");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    let volume = live(&doc, drilled);
    let want = 6000.0 - cylinder(Diameter { mm: 4.0 }, 30.0);
    assert!((volume - want).abs() < 1.0, "volume {volume} mm^3, wanted {want} within 1");
}

/// A second hole of an extrusion that has not been read yet is refused, and the first hole stays the only one.
#[test]
fn a_second_hole_of_the_extrusion_is_refused() {
    let mut session = Session::empty();
    let made = box_10_by_20_by_30(&mut session);
    let doc = session.document();
    let face = cap(&doc);
    let drilled = session.hole(made, face, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect("a hole");
    let refused = session.hole(made, face, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect_err("the body is taken");
    assert!(refused.text.contains("msg-body-consumed"), "the refusal was {refused:?}");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-hole").count(), 1, "a second hole was added");
    let volume = live(&doc, drilled);
    let want = 6000.0 - cylinder(Diameter { mm: 4.0 }, 30.0);
    assert!((volume - want).abs() < 1.0, "volume {volume} mm^3, wanted {want} within 1");
}

/// An id that was never a solid is refused before a node is added. A later hole of the real body can then be read.
#[test]
fn a_body_that_was_never_built_is_refused() {
    let mut session = Session::empty();
    let made = box_10_by_20_by_30(&mut session);
    let doc = session.document();
    let face = cap(&doc);
    let refused = session.hole(BodyId { id: 0 }, face, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect_err("a body that was never built");
    assert!(refused.text.contains("error-source-body-not-built"), "the refusal was {refused:?}");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-hole"), "a hole was added on body 0");
    let drilled = session.hole(made, face, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect("a hole");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-hole").count(), 1, "the empty body left a hole beside the real one");
    let volume = live(&doc, drilled);
    let want = 6000.0 - cylinder(Diameter { mm: 4.0 }, 30.0);
    assert!((volume - want).abs() < 1.0, "volume {volume} mm^3, wanted {want} within 1");
}

/// A sketch id is not a solid. A hole of it is refused, and a later hole of the box can be read.
#[test]
fn a_sketch_is_not_a_body_to_drill() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let face = cap(&doc);
    let refused = session.hole(BodyId { id: drawn.sketch.id }, face, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect_err("a sketch");
    assert!(refused.text.contains("error-source-body-deleted"), "the refusal was {refused:?}");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-hole"), "a hole was added on the sketch");
    let drilled = session.hole(made, face, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect("a hole");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-hole").count(), 1, "the sketch left a hole beside the real one");
    let volume = live(&doc, drilled);
    let want = 6000.0 - cylinder(Diameter { mm: 4.0 }, 30.0);
    assert!((volume - want).abs() < 1.0, "volume {volume} mm^3, wanted {want} within 1");
}

/// A face of another body does not open this one.
#[test]
fn a_face_of_another_body_does_not_open_this_one() {
    let mut session = Session::empty();
    let tall = box_10_by_20_by_30(&mut session);
    let short_drawn = session.rectangle(WorldPlane::Xz, Corners { from: qymcad_drive::From { at: (0.0, 0.0) }, to: qymcad_drive::To { at: (10.0, 10.0) } }).expect("a rectangle");
    let short = session.extrude(short_drawn, Height { mm: 10.0 }).expect("an extrusion");
    let doc = session.document();
    let foreign = doc.bodies.iter().find(|body| body.id == short.id).expect("the short box").faces.iter().find(|face| (face.area.mm2 - 100.0).abs() < 1.0).expect("the short cap").id;
    let drilled = session.hole(tall, foreign, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-face-not-found", "the refusal was {}", refusal(&doc));
    let volume = live(&doc, drilled);
    assert!((volume - 6000.0).abs() < 1.0, "volume {volume} mm^3, the other body's face opened this box");
    let other = live(&doc, short);
    assert!((other - 1000.0).abs() < 1.0, "volume {other} mm^3, the short box was opened");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-hole").count(), 1, "no hole node");
}

/// A fillet the rebuild refused still holds the extrusion. A hole beside it would leave that refusal in the read.
#[test]
fn a_refused_fillet_is_not_a_hole() {
    let mut session = Session::empty();
    let made = box_10_by_20_by_30(&mut session);
    let doc = session.document();
    let edge = doc.bodies[0].edges.iter().find(|edge| (edge.length.mm - 20.0).abs() < 0.1).expect("a 20 mm edge").id;
    let face = cap(&doc);
    let rounded = session.fillet(made, edge, Radius { mm: 100.0 }).expect("the fillet node");
    let doc = session.document();
    assert_eq!(refusal(&doc), "fillet R100.00 failed on 1 edges");
    let refused = session.hole(made, face, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect_err("the body is taken");
    assert!(refused.text.contains("msg-body-consumed"), "the refusal was {refused:?}");
    let doc = session.document();
    assert_eq!(refusal(&doc), "fillet R100.00 failed on 1 edges", "the fillet refusal was replaced");
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-hole"), "a hole was added beside the fillet");
    let volume = live(&doc, rounded);
    assert!((volume - 6000.0).abs() < 1.0, "volume {volume} mm^3, the hole was drilled");
}

/// A chamfer the rebuild refused still holds the extrusion.
#[test]
fn a_refused_chamfer_is_not_a_hole() {
    let mut session = Session::empty();
    let made = box_10_by_20_by_30(&mut session);
    let doc = session.document();
    let edge = doc.bodies[0].edges.iter().find(|edge| (edge.length.mm - 20.0).abs() < 0.1).expect("a 20 mm edge").id;
    let face = cap(&doc);
    let beveled = session.chamfer(made, edge, Distance { mm: 100.0 }).expect("the chamfer node");
    let doc = session.document();
    assert_eq!(refusal(&doc), "chamfer 100.00 too big");
    let refused = session.hole(made, face, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect_err("the body is taken");
    assert!(refused.text.contains("msg-body-consumed"), "the refusal was {refused:?}");
    let doc = session.document();
    assert_eq!(refusal(&doc), "chamfer 100.00 too big", "the chamfer refusal was replaced");
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-hole"), "a hole was added beside the chamfer");
    let volume = live(&doc, beveled);
    assert!((volume - 6000.0).abs() < 1.0, "volume {volume} mm^3, the hole was drilled");
}

/// A shell the rebuild refused still holds the extrusion.
#[test]
fn a_refused_shell_is_not_a_hole() {
    let mut session = Session::empty();
    let made = box_10_by_20_by_30(&mut session);
    let doc = session.document();
    let face = cap(&doc);
    let opened = session.shell(made, face, Thickness { mm: 6.0 }).expect("the shell node");
    let doc = session.document();
    assert_eq!(refusal(&doc), "kernel failed to offset faces on this body");
    let refused = session.hole(made, face, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect_err("the body is taken");
    assert!(refused.text.contains("msg-body-consumed"), "the refusal was {refused:?}");
    let doc = session.document();
    assert_eq!(refusal(&doc), "kernel failed to offset faces on this body", "the shell refusal was replaced");
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-hole"), "a hole was added beside the shell");
    let volume = live(&doc, opened);
    assert!((volume - 6000.0).abs() < 1.0, "volume {volume} mm^3, the hole was drilled");
}

/// A hole the rebuild refused still holds the extrusion. A chamfer beside it, or of the hole's own id, adds nothing.
#[test]
fn a_refused_hole_is_not_a_chamfer() {
    let mut session = Session::empty();
    let made = box_10_by_20_by_30(&mut session);
    let doc = session.document();
    let edge = doc.bodies[0].edges.iter().find(|edge| (edge.length.mm - 20.0).abs() < 0.1).expect("a 20 mm edge").id;
    let failed = session.hole(made, FaceId { id: 1 }, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect("the hole node");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-face-not-found");
    let beside = session.chamfer(made, edge, Distance { mm: 2.0 }).expect_err("the body is taken");
    assert!(beside.text.contains("msg-body-consumed"), "the refusal was {beside:?}");
    let own = session.chamfer(failed, edge, Distance { mm: 2.0 }).expect_err("the hole id is not a solid");
    assert!(own.text.contains("error-source-body-not-built"), "the refusal was {own:?}");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-face-not-found", "the hole refusal was replaced");
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-chamfer"), "a chamfer was added beside the hole");
    let volume = live(&doc, failed);
    assert!((volume - 6000.0).abs() < 1.0, "volume {volume} mm^3, the chamfer was cut");
}

/// A step back drops the face list and keeps the face name. The hole is still drilled at the centre.
///
/// Placing it at the origin lands on the corner of this cap and leaves 5905.75 mm^3, a quarter of the cylinder.
#[test]
fn a_hole_after_a_step_back_stays_at_the_centre() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let face = cap(&doc);
    session.parameter(ParameterName { text: "H".into() }, Expression { text: "20".into() }).expect("a parameter");
    session.back().expect("the step back");
    let drilled = session.hole(made, face, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect("a hole");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    let volume = live(&doc, drilled);
    let want = 6000.0 - cylinder(Diameter { mm: 4.0 }, 30.0);
    assert!((volume - want).abs() < 1.0, "volume {volume} mm^3, wanted {want} within 1");
}

/// A diameter wider than the face is the kernel's own refusal. The cut leaves nothing, so the read says so and the box stays.
#[test]
fn a_diameter_wider_than_the_face_is_the_kernels() {
    let mut session = Session::empty();
    let made = box_10_by_20_by_30(&mut session);
    let doc = session.document();
    let face = cap(&doc);
    let drilled = session.hole(made, face, Diameter { mm: 100.0 }, Depth { mm: 40.0 }).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-empty-result", "the refusal was {}", refusal(&doc));
    let volume = live(&doc, drilled);
    assert!((volume - 6000.0).abs() < 1.0, "volume {volume} mm^3, the refusal did not keep the box");
}
