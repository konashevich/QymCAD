use qymcad_drive::{BodyId, ContourId, Corners, From, Loft, Lower, Measure, Offset, PlaneId, Profile, Rebuild, Session, Sound, To, Upper, WorldPlane};

/// A 10 mm square on the table and a 20 mm square 30 mm above it are a frustum of 7000 mm^3.
#[test]
fn a_loft_of_two_squares_is_seven_thousand() {
    let mut session = Session::empty();
    let made = loft_of(&mut session).expect("a loft");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, made), 7000.0, "the frustum");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-loft").count(), 1, "the loft node");
}

/// A distance that is not finite is refused before a plane.
#[test]
fn an_offset_that_is_not_finite_is_refused_before_a_plane() {
    let mut session = Session::empty();
    for mm in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let bad = session.offset_plane(WorldPlane::Xy, Offset { mm });
        let err = bad.expect_err("the offset");
        assert_eq!(err.text, "sketch-plane-offset", "the refusal was {}", err.text);
    }
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert!(doc.nodes.iter().all(|node| node.name != "name-plane"), "a plane was written");
}

/// A section that is not a closed contour is refused before a node. A later loft of the two squares is the frustum.
#[test]
fn a_missing_section_is_refused_before_a_node() {
    let mut session = Session::empty();
    let lower = session.rectangle(WorldPlane::Xy, corners(0.0, 0.0, 10.0, 10.0)).expect("the lower square");
    let raised = session.offset_plane(WorldPlane::Xy, Offset { mm: 30.0 }).expect("the plane");
    let upper = session.rectangle_on(raised, corners(-5.0, -5.0, 15.0, 15.0)).expect("the upper square");
    let missing = Upper { profile: Profile { sketch: upper.sketch, contour: ContourId { id: 0 } } };
    let bad = session.loft(Loft { lower: Lower { profile: lower }, upper: missing });
    let err = bad.expect_err("the section");
    assert_eq!(err.text, "error-profile-not-found", "the refusal was {}", err.text);
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-loft"), "a node was written");
    let made = session.loft(Loft { lower: Lower { profile: lower }, upper: Upper { profile: upper } }).expect("a loft");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, made), 7000.0, "the frustum after the missing section");
}

/// Two squares on the table are refused before a node, and a box beside them stays the body.
#[test]
fn a_loft_on_one_plane_leaves_the_box() {
    let mut session = Session::empty();
    let block =
        session.cuboid(qymcad_drive::Cuboid { length: qymcad_drive::BoxLength { mm: 10.0 }, width: qymcad_drive::Width { mm: 20.0 }, height: qymcad_drive::Height { mm: 30.0 } }).expect("a box");
    let built = session.document();
    assert_eq!(built.rebuild, Rebuild::Quiet, "{}", refusal(&built));
    let lower = session.rectangle(WorldPlane::Xy, corners(0.0, 0.0, 10.0, 10.0)).expect("the lower square");
    let upper = session.rectangle(WorldPlane::Xy, corners(-5.0, -5.0, 15.0, 15.0)).expect("the upper square");
    let bad = session.loft(Loft { lower: Lower { profile: lower }, upper: Upper { profile: upper } });
    assert_eq!(bad.expect_err("the plane").text, "sketch-loft-same-plane");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-loft" && node.name != "feat-name-body-union"), "a node was written");
    near(live(&doc, block), 6000.0, "the box");
}

/// A plane id that is not a datum is refused before a sketch, and a box beside it stays the body.
#[test]
fn a_plane_that_is_not_a_datum_is_refused_before_a_sketch() {
    let mut session = Session::empty();
    let block =
        session.cuboid(qymcad_drive::Cuboid { length: qymcad_drive::BoxLength { mm: 10.0 }, width: qymcad_drive::Width { mm: 20.0 }, height: qymcad_drive::Height { mm: 30.0 } }).expect("a box");
    let built = session.document();
    assert_eq!(built.rebuild, Rebuild::Quiet, "{}", refusal(&built));
    let bad = session.rectangle_on(PlaneId { id: 0 }, corners(0.0, 0.0, 10.0, 10.0));
    assert_eq!(bad.expect_err("the plane").text, "sketch-plane-missing");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-loft" && node.name != "feat-name-body-union"), "a node was written");
    near(live(&doc, block), 6000.0, "the box");
}

/// Two squares that meet where the table and the front plane cross are refused before a node.
#[test]
fn a_loft_of_meeting_sections_leaves_the_box() {
    let mut session = Session::empty();
    let block =
        session.cuboid(qymcad_drive::Cuboid { length: qymcad_drive::BoxLength { mm: 10.0 }, width: qymcad_drive::Width { mm: 20.0 }, height: qymcad_drive::Height { mm: 30.0 } }).expect("a box");
    let built = session.document();
    assert_eq!(built.rebuild, Rebuild::Quiet, "{}", refusal(&built));
    let lower = session.rectangle(WorldPlane::Xy, corners(0.0, 0.0, 10.0, 10.0)).expect("the lower square");
    let front = session.rectangle(WorldPlane::Xz, corners(0.0, 0.0, 10.0, 10.0)).expect("the front square");
    let bad = session.loft(Loft { lower: Lower { profile: lower }, upper: Upper { profile: front } });
    assert_eq!(bad.expect_err("the sections").text, "sketch-loft-sections-meet");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-loft" && node.name != "feat-name-body-union"), "a node was written");
    near(live(&doc, block), 6000.0, "the box");
}

/// A front square that starts 0.05 mm above the table still lofts. The table square meets the front plane; the front square does not meet the table.
#[test]
fn a_loft_with_a_small_gap_is_five_hundred() {
    let mut session = Session::empty();
    let lower = session.rectangle(WorldPlane::Xy, corners(0.0, 0.0, 10.0, 10.0)).expect("the lower square");
    let front = session.rectangle(WorldPlane::Xz, corners(0.0, 0.05, 10.0, 10.05)).expect("the front square");
    let made = session.loft(Loft { lower: Lower { profile: lower }, upper: Upper { profile: front } }).expect("a loft");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, made), 502.5, "the loft");
}

fn loft_of(session: &mut Session) -> Result<BodyId, qymcad_drive::DrawError> {
    let lower = session.rectangle(WorldPlane::Xy, corners(0.0, 0.0, 10.0, 10.0))?;
    let raised = session.offset_plane(WorldPlane::Xy, Offset { mm: 30.0 })?;
    let upper = session.rectangle_on(raised, corners(-5.0, -5.0, 15.0, 15.0))?;
    session.loft(Loft { lower: Lower { profile: lower }, upper: Upper { profile: upper } })
}

fn corners(x0: f64, y0: f64, x1: f64, y1: f64) -> Corners {
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
