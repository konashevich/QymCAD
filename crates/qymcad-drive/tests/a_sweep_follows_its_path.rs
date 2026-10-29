use qymcad_drive::{BodyId, BoxLength, ContourId, Corners, Cuboid, From, Height, Measure, Profile, Rebuild, Segment, Session, Sound, Sweep, To, Width, WorldPlane};

/// A 10 mm square on the table, swept 40 mm up the front plane, is a bar of 4000 mm^3.
#[test]
fn a_bar_swept_along_a_line_is_four_thousand() {
    let mut session = Session::empty();
    let section = session.rectangle(WorldPlane::Xy, corners(0.0, 0.0, 10.0, 10.0)).expect("a square");
    let path = session.line(WorldPlane::Xz, segment(0.0, 0.0, 0.0, 40.0)).expect("a line");
    let made = session.sweep(Sweep { section, path }).expect("a sweep");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, made), 4000.0, "the bar");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-sweep").count(), 1, "the sweep node");
}

/// A 2 mm square swept 10 mm up from the origin sits inside the box, so the union stays 6000 mm^3.
#[test]
fn a_sweep_inside_a_box_stays_the_box() {
    let mut session = Session::empty();
    let block = session.cuboid(Cuboid { length: BoxLength { mm: 10.0 }, width: Width { mm: 20.0 }, height: Height { mm: 30.0 } }).expect("a box");
    let built = session.document();
    assert_eq!(built.rebuild, Rebuild::Quiet, "{}", refusal(&built));
    near(live(&built, block), 6000.0, "the box");
    let section = session.rectangle(WorldPlane::Xy, corners(-1.0, -1.0, 1.0, 1.0)).expect("a square");
    let path = session.line(WorldPlane::Xz, segment(0.0, 0.0, 0.0, 10.0)).expect("a line");
    let joined = session.sweep(Sweep { section, path }).expect("a sweep");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_ne!(joined, block, "the union is the box");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-sweep").count(), 1, "the sweep node");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-body-union").count(), 1, "the union node");
    near(live(&doc, joined), 6000.0, "the union");
}

/// A section that is not a closed contour is refused before a node. A later sweep of the real square is the bar.
#[test]
fn a_missing_section_is_refused_before_a_node() {
    let mut session = Session::empty();
    let section = session.rectangle(WorldPlane::Xy, corners(0.0, 0.0, 10.0, 10.0)).expect("a square");
    let path = session.line(WorldPlane::Xz, segment(0.0, 0.0, 0.0, 40.0)).expect("a line");
    let missing = Profile { sketch: section.sketch, contour: ContourId { id: 0 } };
    let bad = session.sweep(Sweep { section: missing, path });
    let err = bad.expect_err("the section");
    assert_eq!(err.text, "error-profile-not-found", "the refusal was {}", err.text);
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-sweep"), "a node was written");
    let made = session.sweep(Sweep { section, path }).expect("a sweep");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, made), 4000.0, "the bar after the missing section");
}

/// A path contour id of 0 is refused before a node. A later sweep along the real line is the bar.
#[test]
fn a_missing_path_is_refused_before_a_node() {
    let mut session = Session::empty();
    let section = session.rectangle(WorldPlane::Xy, corners(0.0, 0.0, 10.0, 10.0)).expect("a square");
    let path = session.line(WorldPlane::Xz, segment(0.0, 0.0, 0.0, 40.0)).expect("a line");
    let missing = qymcad_drive::Path { sketch: path.sketch, contour: ContourId { id: 0 } };
    let bad = session.sweep(Sweep { section, path: missing });
    let err = bad.expect_err("the path");
    assert_eq!(err.text, "error-sweep-path-missing", "the refusal was {}", err.text);
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-sweep"), "a node was written");
    let made = session.sweep(Sweep { section, path }).expect("a sweep");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, made), 4000.0, "the bar after the missing path");
}

/// Ends that are not two different finite points are refused before a sketch. A later line still sweeps the bar.
#[test]
fn a_line_that_does_not_span_is_refused_before_a_sketch() {
    let mut session = Session::empty();
    let same = session.line(WorldPlane::Xz, segment(0.0, 0.0, 0.0, 0.0));
    assert_eq!(same.expect_err("the ends").text, "sketch-line-empty");
    let broken = session.line(WorldPlane::Xz, segment(0.0, f64::NAN, 0.0, 40.0));
    assert_eq!(broken.expect_err("the end").text, "sketch-line-empty");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert!(doc.sketches.is_empty(), "a sketch was written");
    let section = session.rectangle(WorldPlane::Xy, corners(0.0, 0.0, 10.0, 10.0)).expect("a square");
    let path = session.line(WorldPlane::Xz, segment(0.0, 0.0, 0.0, 40.0)).expect("a line");
    let made = session.sweep(Sweep { section, path }).expect("a sweep");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, made), 4000.0, "the bar after the empty line");
}

/// A square swept along its own contour is refused before a node, and the box stays the body.
#[test]
fn a_sweep_of_its_own_sketch_leaves_the_box() {
    let mut session = Session::empty();
    let block = session.cuboid(Cuboid { length: BoxLength { mm: 10.0 }, width: Width { mm: 20.0 }, height: Height { mm: 30.0 } }).expect("a box");
    let built = session.document();
    assert_eq!(built.rebuild, Rebuild::Quiet, "{}", refusal(&built));
    let section = session.rectangle(WorldPlane::Xy, corners(0.0, 0.0, 10.0, 10.0)).expect("a square");
    let path = qymcad_drive::Path { sketch: section.sketch, contour: section.contour };
    let bad = session.sweep(Sweep { section, path });
    assert_eq!(bad.expect_err("the path").text, "sketch-sweep-same-sketch");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-sweep" && node.name != "feat-name-body-union"), "a node was written");
    near(live(&doc, block), 6000.0, "the box");
}

/// A closed rectangle used as the path is refused before a node, and the box stays the body.
#[test]
fn a_closed_path_leaves_the_box() {
    let mut session = Session::empty();
    let block = session.cuboid(Cuboid { length: BoxLength { mm: 10.0 }, width: Width { mm: 20.0 }, height: Height { mm: 30.0 } }).expect("a box");
    let built = session.document();
    assert_eq!(built.rebuild, Rebuild::Quiet, "{}", refusal(&built));
    let section = session.rectangle(WorldPlane::Xy, corners(-1.0, -1.0, 1.0, 1.0)).expect("a square");
    let looped = session.rectangle(WorldPlane::Xz, corners(0.0, 0.0, 20.0, 20.0)).expect("a loop");
    let path = qymcad_drive::Path { sketch: looped.sketch, contour: looped.contour };
    let bad = session.sweep(Sweep { section, path });
    assert_eq!(bad.expect_err("the path").text, "sketch-sweep-closed-path");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-sweep" && node.name != "feat-name-body-union"), "a node was written");
    near(live(&doc, block), 6000.0, "the box");
}

fn corners(x0: f64, y0: f64, x1: f64, y1: f64) -> Corners {
    Corners { from: From { at: (x0, y0) }, to: To { at: (x1, y1) } }
}

fn segment(x0: f64, y0: f64, x1: f64, y1: f64) -> Segment {
    Segment { from: From { at: (x0, y0) }, to: To { at: (x1, y1) } }
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
