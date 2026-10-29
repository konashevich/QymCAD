use qymcad_drive::{Angle, BodyId, ContourId, Corners, From, Height, Measure, Profile, Rebuild, Session, SketchAxis, Sound, To, WorldPlane};

/// A full turn of the 10 by 20 mm rectangle about sketch X is a cylinder of 4000 pi mm^3.
/// Shortening the sketch Y side to 10 mm leaves the same body at 1000 pi mm^3.
#[test]
fn a_revolve_follows_its_profile() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle(0.0, 0.0, 10.0, 20.0)).expect("a rectangle");
    let turned = session.revolve(drawn, SketchAxis::X, Angle { deg: 360.0 }).expect("a full turn");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, turned), 4000.0 * std::f64::consts::PI, "the full turn");
    session.resize(drawn, rectangle(0.0, 0.0, 10.0, 10.0)).expect("the rectangle is shortened");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-revolve").count(), 1, "the resize added a revolve");
    near(live(&doc, turned), 1000.0 * std::f64::consts::PI, "the shortened turn");
}

/// A half turn is half of that cylinder.
#[test]
fn a_half_turn_is_half_the_cylinder() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle(0.0, 0.0, 10.0, 20.0)).expect("a rectangle");
    let turned = session.revolve(drawn, SketchAxis::X, Angle { deg: 180.0 }).expect("a half turn");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, turned), 2000.0 * std::f64::consts::PI, "the half turn");
}

/// An angle that is not a turn from just above zero to a full turn is refused before a node.
#[test]
fn a_bad_angle_is_refused_before_a_node() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle(0.0, 0.0, 10.0, 20.0)).expect("a rectangle");
    for deg in [0.0, -30.0, 361.0, f64::NAN, f64::INFINITY] {
        let bad = session.revolve(drawn, SketchAxis::X, Angle { deg });
        let err = bad.expect_err("the angle");
        assert_eq!(err.text, "sketch-revolve-angle", "the refusal was {}", err.text);
        let doc = session.document();
        assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
        assert!(doc.nodes.iter().all(|node| node.name != "feat-name-revolve"), "a node was written");
    }
    let turned = session.revolve(drawn, SketchAxis::X, Angle { deg: 360.0 }).expect("a full turn");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, turned), 4000.0 * std::f64::consts::PI, "the turn after the refusals");
}

/// A contour that is not a closed contour of the sketch is refused before a node.
#[test]
fn a_missing_contour_is_refused_before_a_node() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle(0.0, 0.0, 10.0, 20.0)).expect("a rectangle");
    let missing = Profile { sketch: drawn.sketch, contour: ContourId { id: 1 } };
    let bad = session.revolve(missing, SketchAxis::X, Angle { deg: 360.0 });
    let err = bad.expect_err("the contour");
    assert_eq!(err.text, "error-profile-not-found", "the refusal was {}", err.text);
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-revolve"), "a node was written");
    let turned = session.revolve(drawn, SketchAxis::X, Angle { deg: 360.0 }).expect("a full turn");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, turned), 4000.0 * std::f64::consts::PI, "the turn after the missing contour");
}

/// A turn across the axis does not unite with a box that is already there. A later turn that lies on the
/// axis unites with that box, and the crossing refusal stays.
#[test]
fn a_turn_across_the_axis_leaves_the_box() {
    let mut session = Session::empty();
    let block = session.cuboid(qymcad_drive::Cuboid { length: qymcad_drive::BoxLength { mm: 10.0 }, width: qymcad_drive::Width { mm: 20.0 }, height: Height { mm: 30.0 } }).expect("a box");
    let built = session.document();
    assert_eq!(built.rebuild, Rebuild::Quiet, "{}", refusal(&built));
    near(live(&built, block), 6000.0, "the box");
    let across = session.rectangle(WorldPlane::Xz, rectangle(0.0, -5.0, 10.0, 15.0)).expect("a rectangle across the axis");
    let failed = session.revolve(across, SketchAxis::X, Angle { deg: 360.0 }).expect("the node is written");
    assert_ne!(failed, block, "the refused turn is the box");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-revolve-profile-crosses-axis", "the refusal was {}", refusal(&doc));
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-body-union"), "the refused turn was united with the box");
    near(live(&doc, block), 6000.0, "the box beside the refusal");
    let lying = session.rectangle(WorldPlane::Xz, rectangle(0.0, 0.0, 10.0, 20.0)).expect("a rectangle on the axis");
    let turned = session.revolve(lying, SketchAxis::X, Angle { deg: 360.0 }).expect("a full turn");
    assert_ne!(turned, failed, "the refused turn was rewritten");
    assert_ne!(turned, block, "the union is the box");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-revolve-profile-crosses-axis", "the refusal was {}", refusal(&doc));
    assert_eq!(doc.nodes.iter().find(|node| node.id == turned.id).expect("the union").name, "feat-name-body-union");
    let volume = live(&doc, turned);
    assert!(volume > 4000.0 * std::f64::consts::PI + 1.0, "the turn was not united with the box: volume {volume} mm^3");
}

/// A profile that crosses the axis is refused, and a later profile that lies on the axis is its own body.
#[test]
fn a_profile_across_the_axis_is_not_joined_to_the_next_turn() {
    let mut session = Session::empty();
    let across = session.rectangle(WorldPlane::Xz, rectangle(0.0, -5.0, 10.0, 15.0)).expect("a rectangle across the axis");
    let failed = session.revolve(across, SketchAxis::X, Angle { deg: 360.0 }).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-revolve-profile-crosses-axis", "the refusal was {}", refusal(&doc));
    let lying = session.rectangle(WorldPlane::Xz, rectangle(0.0, 0.0, 10.0, 20.0)).expect("a rectangle on the axis");
    let turned = session.revolve(lying, SketchAxis::X, Angle { deg: 360.0 }).expect("a full turn");
    assert_ne!(turned, failed, "the refused turn was rewritten");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-revolve-profile-crosses-axis", "the refusal was {}", refusal(&doc));
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-body-union"), "the new turn was joined to the refused one");
    near(live(&doc, turned), 4000.0 * std::f64::consts::PI, "the turn beside the refusal");
}

fn rectangle(x0: f64, y0: f64, x1: f64, y1: f64) -> Corners {
    Corners { from: From { at: (x0, y0) }, to: To { at: (x1, y1) } }
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

fn near(got: f64, want: f64, what: &str) {
    assert!((got - want).abs() < 1.0, "{what}: volume {got} mm^3, wanted {want} within 1");
}

fn refusal(doc: &qymcad_drive::Document) -> String {
    match &doc.rebuild {
        Rebuild::Refused { refusals } => refusals.iter().map(|refusal| refusal.text.clone()).collect::<Vec<_>>().join(" "),
        Rebuild::Quiet => "quiet".into(),
        Rebuild::Stopped => "stopped".into(),
    }
}
