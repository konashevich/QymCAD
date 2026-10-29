use qymcad_drive::{CornerX, CornerY, CornerZ, Corners, From, Height, HighCorner, LowCorner, MeshRead, Part, Place, Session, To, Volume, WorldPlane};

/// A part shifted 100 mm along X comes back as one mesh. Its corner is that shift, within 1e-6 mm.
#[test]
fn the_part_stands_in_the_mesh() {
    let mut session = Session::empty();
    let _placed = session.part(Part { place: shift_x(100.0) }).expect("a part");
    grow(&mut session);
    let _doc = session.document();
    let path = kept("shift");
    session.save_stl(&path).expect("the file");
    let read = Session::open_stl(&path).expect("the read");
    stood_clear(read.low.x);
    one_mesh(&read, "the shifted box");
    the_shifted_mesh(&read);
    let _ = std::fs::remove_file(&path);
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

fn stood_clear(x: CornerX) {
    if x.mm.abs() < 1e-6 {
        panic!("the part is still at the origin");
    }
}

fn the_shifted_mesh(read: &MeshRead) {
    low_corner(read.low, 100.0, "the shifted box");
    high_corner(read.high, 110.0, "the shifted box");
}

fn low_corner(got: LowCorner, along: f64, what: &str) {
    along_x(got.x, along, what);
    along_y(got.y, 0.0, what);
    along_z(got.z, 0.0, what);
}

fn high_corner(got: HighCorner, along: f64, what: &str) {
    along_x(got.x, along, what);
    along_y(got.y, 20.0, what);
    along_z(got.z, 30.0, what);
}

fn along_x(got: CornerX, mm: f64, what: &str) {
    let gap = (got.mm - mm).abs();
    assert!(gap < 1e-6, "{what}");
}

fn along_y(got: CornerY, mm: f64, what: &str) {
    let gap = (got.mm - mm).abs();
    assert!(gap < 1e-6, "{what}");
}

fn along_z(got: CornerZ, mm: f64, what: &str) {
    let gap = (got.mm - mm).abs();
    assert!(gap < 1e-6, "{what}");
}

fn one_mesh(read: &MeshRead, what: &str) {
    within_a_millimetre(read.volume, 6000.0, what);
    assert_eq!(read.triangles, 12, "{what}");
}

fn within_a_millimetre(got: Volume, mm3: f64, what: &str) {
    let gap = (got.mm3 - mm3).abs();
    assert!(gap < 1.0, "{what}");
}

fn shift_x(mm: f64) -> Place {
    Place { mat: [1.0, 0.0, 0.0, mm, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0] }
}
