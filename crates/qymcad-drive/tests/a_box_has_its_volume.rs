use std::path::{Path, PathBuf};

use qymcad_drive::{BodyId, ContourId, Corners, Document, Height, Kind, Measure, Plane, Profile, Rebuild, Session, SketchId, Sound, WorldPlane};

/// A 10 by 20 mm rectangle on XZ, extruded 30 mm, is one valid solid of 6000 mm³. The sketch stays on
/// XZ, in the same part, and a second read is the same body.
#[test]
fn a_box_is_ten_by_twenty_by_thirty() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    assert_one_box(&doc, made, drawn, 6000.0);

    let again = session.document();
    assert_eq!(again.bodies.len(), 1, "a second read is still one body");
    assert_eq!(again.bodies[0].id, made.id, "a second read is the same body");
    assert_volume(&again.bodies[0], 6000.0);
}

/// Halving the height halves the volume. A solid baked at 6000 mm³, or at a height of 30 mm, fails.
#[test]
fn a_shorter_extrusion_has_half_the_volume() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 15.0 }).expect("an extrusion");
    assert_one_box(&session.document(), made, drawn, 3000.0);
}

/// A 10 by 10 mm rectangle extruded 30 mm is 3000 mm³. A profile area baked at 200 mm² fails.
#[test]
fn a_smaller_rectangle_has_half_the_volume() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_10()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    assert_one_box(&session.document(), made, drawn, 3000.0);
}

/// The 15 mm box is saved and opened by a second session that does not draw. The volume is the proof
/// the file was read: a fresh 30 mm box would measure 6000 mm³, and a fresh extrusion allocates the
/// same ids.
#[test]
fn a_saved_box_rebuilds_to_the_same_volume() {
    let path = Sweep::new(temp("box15"));
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 15.0 }).expect("an extrusion");
    let before = session.document();
    assert_one_box(&before, made, drawn, 3000.0);
    session.save(&path.0).expect("the box is written");

    let mut opened = Session::open(&path.0).expect("the box opens");
    let doc = opened.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the opened rebuild is quiet");
    assert_eq!(doc.bodies.len(), 1, "the opened document has one body");
    assert_eq!(doc.bodies[0].sound, Sound::Valid, "the opened body is valid");
    assert_volume(&doc.bodies[0], 3000.0);
    assert!(doc.nodes.iter().any(|n| n.name == "feat-name-extrude"), "the opened timeline has the extrusion");
    assert!(doc.sketches.iter().any(|s| s.plane == Plane::Xz), "the opened sketch stays on Xz");
}

/// A contour that is not a closed contour of that sketch is refused before a node is added. The model
/// would otherwise discard the named contour and extrude the sketch's first one, and the rebuild would
/// stay quiet.
#[test]
fn a_contour_from_another_sketch_is_not_extruded() {
    let mut session = Session::empty();
    let first = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let second = session.rectangle(WorldPlane::Yz, rectangle_10_by_10()).expect("a rectangle");
    for profile in [
        Profile { sketch: second.sketch, contour: first.contour },
        Profile { sketch: first.sketch, contour: ContourId { id: 0 } },
        Profile { sketch: SketchId { id: 0 }, contour: first.contour },
    ] {
        let err = session.extrude(profile, Height { mm: 30.0 }).expect_err("the contour is not a closed contour of that sketch");
        assert_eq!(err.text, "error-profile-not-found");
    }
    let doc = session.document();
    assert!(doc.bodies.is_empty(), "a foreign contour does not add a body");
    assert!(doc.nodes.iter().all(|n| n.name != "feat-name-extrude"), "no extrusion was added");
    assert_eq!(doc.rebuild, Rebuild::Quiet);
    let made = session.extrude(second, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    assert_eq!(doc.bodies.len(), 1, "the real contour is the first body");
    assert_eq!(doc.bodies[0].id, made.id);
    assert_volume(&doc.bodies[0], 3000.0);
}

fn assert_one_box(doc: &Document, made: BodyId, drawn: Profile, want: f64) {
    assert_eq!(doc.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert_eq!(doc.bodies.len(), 1, "one body");
    let body = &doc.bodies[0];
    assert_eq!(body.id, made.id, "the returned id is the body's id");
    assert_eq!(body.name, "name-body#1");
    assert_eq!(body.sound, Sound::Valid, "the shape is valid");
    assert_volume(body, want);
    assert_eq!(doc.nodes.len(), 2, "the sketch and the extrusion");
    let sketch_node = doc.nodes.iter().find(|n| n.id == drawn.sketch.id).expect("the sketch node");
    let extrude = doc.nodes.iter().find(|n| n.name == "feat-name-extrude").expect("the extrusion");
    assert_eq!(extrude.id, made.id, "the returned id is the extrude node's id");
    assert_eq!(doc.components.iter().filter(|c| c.kind == Kind::Part).count(), 1, "one part");
    let part = doc.components.iter().find(|c| c.kind == Kind::Part).expect("the part");
    assert_eq!(sketch_node.parent, Some(part.id), "the sketch stays in the part");
    assert_eq!(extrude.parent, Some(part.id), "the extrusion stays in the part");
    assert!(doc.sketches.iter().any(|s| s.id == drawn.sketch.id && s.plane == Plane::Xz), "the sketch stays on Xz");
}

/// One solid, and a volume within 1 mm³ of `want`. A missing shape is not a volume of 0.
fn assert_volume(body: &qymcad_drive::Body, want: f64) {
    match body.measure {
        Measure::Live { volume, solids } => {
            assert_eq!(solids, 1, "one solid");
            assert!((volume.mm3 - want).abs() < 1.0, "volume {} mm^3, wanted {want} within 1", volume.mm3);
        }
        Measure::Absent => panic!("the body has no live shape"),
    }
}

/// A height that is not a positive length is refused before a node is added. Zero, a negative number,
/// and a value that is not a finite number are the same refusal: the model would otherwise take the
/// absolute value, and a negative height would be the positive box with a quiet rebuild.
#[test]
fn a_height_that_is_not_positive_is_refused() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    for mm in [0.0, -30.0, f64::NAN, f64::INFINITY] {
        let err = session.extrude(drawn, Height { mm }).expect_err("a height that is not positive");
        assert_eq!(err.text, "sketch-extrude-height");
    }
    let doc = session.document();
    assert!(doc.bodies.is_empty(), "a refused height does not add a body");
    assert_eq!(doc.nodes.len(), 1, "the sketch stays and no extrusion is added");
    assert_eq!(doc.rebuild, Rebuild::Quiet);
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    assert_one_box(&session.document(), made, drawn, 6000.0);
}

fn rectangle_10_by_20() -> Corners {
    Corners { from: qymcad_drive::From { at: (0.0, 0.0) }, to: qymcad_drive::To { at: (10.0, 20.0) } }
}

fn rectangle_10_by_10() -> Corners {
    Corners { from: qymcad_drive::From { at: (0.0, 0.0) }, to: qymcad_drive::To { at: (10.0, 10.0) } }
}

/// Removes the temporary file when the test ends, including when an assertion fails.
struct Sweep(PathBuf);

impl Sweep {
    fn new(path: PathBuf) -> Self {
        remove(&path);
        Self(path)
    }
}

impl Drop for Sweep {
    fn drop(&mut self) {
        remove(&self.0);
    }
}

fn temp(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("qymcad-drive-{name}-{}.qcad", std::process::id()))
}

fn remove(path: &Path) {
    let text = path.display().to_string();
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(format!("{text}.bak"));
    let _ = std::fs::remove_file(format!("{text}.tmp~"));
}
