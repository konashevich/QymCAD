use qymcad_drive::{Corners, Document, From, Height, Measure, MeshRead, Session, StoreError, To, Volume, WorldPlane};

/// The 10 by 20 by 30 mm box comes back from the STL file as 12 triangles of 6000 mm^3.
#[test]
fn the_box_comes_back_from_stl() {
    let mut session = Session::empty();
    grow(&mut session);
    let doc = session.document();
    still_the_box(&doc);
    let path = kept("box");
    session.save_stl(&path).expect("the file");
    let read = Session::open_stl(&path).expect("the read");
    brought(&read);
    let again = session.document();
    still_the_box(&again);
    let _ = std::fs::remove_file(&path);
}

/// A write before the box has been read creates no file.
#[test]
fn a_write_before_the_read_creates_no_file() {
    let mut session = Session::empty();
    grow(&mut session);
    let path = kept("early");
    refused(session.save_stl(&path).expect_err("the write"), &path);
}

/// An empty document creates no file.
#[test]
fn an_empty_document_creates_no_file() {
    let session = Session::empty();
    let path = kept("empty");
    refused(session.save_stl(&path).expect_err("the write"), &path);
}

/// A path with no file is not reported as a file the reader could not open.
#[test]
fn a_missing_file_is_not_a_broken_mesh() {
    let path = kept("missing");
    let err = Session::open_stl(&path).expect_err("the read");
    let name = path.to_str().expect("the path");
    assert_eq!(err.text, format!("cad-file-not-found#{name}"));
}

fn grow(session: &mut Session) {
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    let _block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
}

fn kept(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("qymcad-drive-stl-{name}-{}.stl", std::process::id()));
    let _ = std::fs::remove_file(&path);
    path
}

fn brought(read: &MeshRead) {
    if read.volume.mm3.abs() < 1e-12 {
        panic!("the box is not in the file");
    }
    within_a_millimetre(read.volume, 6000.0, "the mesh");
    assert_eq!(read.triangles, 12, "the mesh");
}

fn still_the_box(doc: &Document) {
    assert_eq!(doc.bodies.len(), 1, "the document");
    let body = &doc.bodies[0];
    the_box(kernel(body.measure, "the box"), "the box");
    the_box(body.mesh, "the mesh");
}

fn refused(err: StoreError, path: &std::path::Path) {
    assert_eq!(err.text, "io-stl-no-triangles");
    assert!(!path.is_file(), "the file was created");
}

fn kernel(measure: Measure, what: &str) -> Volume {
    match measure {
        Measure::Live { volume, .. } => volume,
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
