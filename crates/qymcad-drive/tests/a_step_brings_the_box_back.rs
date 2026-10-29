use qymcad_drive::{BodyId, Corners, Document, From, Height, Measure, Radius, Session, SolidRead, Sound, StoreError, To, Volume, WorldPlane};

/// The 10 by 20 by 30 mm box comes back from the STEP file as one valid solid of 6000 mm^3.
#[test]
fn the_box_comes_back_from_step() {
    let mut session = Session::empty();
    grow(&mut session);
    let doc = session.document();
    still_the_box(&doc);
    let path = kept("box");
    session.save_step(&path).expect("the file");
    let solids = Session::open_step(&path).expect("the read");
    assert_eq!(solids.len(), 1, "the file");
    one_solid(brought(&solids), "the solid");
    let again = session.document();
    still_the_box(&again);
    let _ = std::fs::remove_file(&path);
}

/// A 2 mm round of one edge comes back as the rounded solid. The blank of 6000 mm^3 stays out of the file.
#[test]
fn the_rounded_box_comes_back_alone() {
    let mut session = Session::empty();
    grow(&mut session);
    let doc = session.document();
    let edge = doc.bodies[0].edges[0].id;
    let block = BodyId { id: doc.bodies[0].id };
    let rounded = session.fillet(block, edge, Radius { mm: 2.0 }).expect("a fillet");
    let doc = session.document();
    let path = kept("round");
    session.save_step(&path).expect("the file");
    let solids = Session::open_step(&path).expect("the read");
    assert_eq!(solids.len(), 1, "the blank came back");
    the_result(brought(&solids), volume_of(&doc, rounded).mm3, "the rounded box");
    let _ = std::fs::remove_file(&path);
}

/// A write before the box has been read creates no file.
#[test]
fn a_write_before_the_read_creates_no_file() {
    let mut session = Session::empty();
    grow(&mut session);
    let path = kept("early");
    refused(session.save_step(&path).expect_err("the write"), &path);
}

/// An empty document creates no file.
#[test]
fn an_empty_document_creates_no_file() {
    let session = Session::empty();
    let path = kept("empty");
    refused(session.save_step(&path).expect_err("the write"), &path);
}

/// A path with no file is not reported as a solid that could not be read.
#[test]
fn a_missing_file_is_not_a_broken_solid() {
    let path = kept("missing");
    let err = Session::open_step(&path).expect_err("the read");
    let name = path.to_str().expect("the path");
    assert_eq!(err.text, format!("cad-file-not-found#{name}"));
}

fn grow(session: &mut Session) {
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    let _block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
}

fn kept(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("qymcad-drive-step-{name}-{}.step", std::process::id()));
    let _ = std::fs::remove_file(&path);
    path
}

fn brought(solids: &[SolidRead]) -> &SolidRead {
    solids.first().unwrap_or_else(|| panic!("the box is not in the file"))
}

fn the_result(solid: &SolidRead, mm3: f64, what: &str) {
    assert_eq!(solid.sound, Sound::Valid, "{what}");
    let weighed = live(solid.measure, what);
    within_a_millimetre(weighed.volume, mm3, what);
    assert_eq!(weighed.solids, 1, "{what}");
}

fn volume_of(doc: &Document, body: BodyId) -> Volume {
    let found = doc.bodies.iter().find(|item| item.id == body.id).expect("the body");
    live(found.measure, "the body").volume
}

fn one_solid(solid: &SolidRead, what: &str) {
    the_result(solid, 6000.0, what);
}

fn still_the_box(doc: &Document) {
    assert_eq!(doc.bodies.len(), 1, "the document");
    let body = &doc.bodies[0];
    assert_eq!(body.sound, Sound::Valid, "the box");
    let weighed = live(body.measure, "the box");
    the_box(weighed.volume, "the box");
    assert_eq!(weighed.solids, 1, "the box");
}

fn refused(err: StoreError, path: &std::path::Path) {
    assert_eq!(err.text, "cad-step-nothing-to-export");
    assert!(!path.is_file(), "the file was created");
}

struct Weighed {
    volume: Volume,
    solids: u32,
}

fn live(measure: Measure, what: &str) -> Weighed {
    match measure {
        Measure::Live { volume, solids } => Weighed { volume, solids },
        Measure::Absent => panic!("{what} has no volume"),
    }
}

fn within_a_millimetre(got: Volume, mm3: f64, what: &str) {
    let gap = (got.mm3 - mm3).abs();
    assert!(gap < 1.0, "{what}");
}

fn the_box(got: Volume, what: &str) {
    let gap = (got.mm3 - 6000.0).abs();
    assert!(gap < 1e-9, "{what}");
}
