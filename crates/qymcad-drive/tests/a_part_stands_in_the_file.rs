use qymcad_drive::{CornerX, CornerY, CornerZ, Corners, From, Height, HighCorner, LowCorner, Measure, Part, Place, Session, SolidRead, Sound, To, Volume, WorldPlane};

/// A part shifted 100 mm along X comes back as the second solid. Its corner is that shift, within 1e-6 mm.
#[test]
fn the_part_stands_in_the_file() {
    let mut session = Session::empty();
    grow(&mut session);
    let _placed = session.part(Part { place: shift_x(100.0) }).expect("a part");
    grow(&mut session);
    let _doc = session.document();
    let path = kept("shift");
    session.save_step(&path).expect("the file");
    let solids = Session::open_step(&path).expect("the read");
    assert_eq!(solids.len(), 2, "the file");
    let shifted = solids.get(1).unwrap_or_else(|| panic!("the shifted part is not in the file"));
    stood_clear(shifted.low.x);
    let home = solids.first().unwrap_or_else(|| panic!("the first box is not in the file"));
    one_box(home, "the first box");
    the_home_box(home);
    one_box(shifted, "the shifted box");
    the_shifted_box(shifted);
    let _ = std::fs::remove_file(&path);
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

fn stood_clear(x: CornerX) {
    if x.mm.abs() < 1e-6 {
        panic!("the part is still at the origin");
    }
}

fn the_home_box(solid: &SolidRead) {
    low_corner(solid.low, 0.0, "the first box");
    high_corner(solid.high, 10.0, "the first box");
}

fn the_shifted_box(solid: &SolidRead) {
    low_corner(solid.low, 100.0, "the shifted box");
    high_corner(solid.high, 110.0, "the shifted box");
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

fn one_box(solid: &SolidRead, what: &str) {
    assert_eq!(solid.sound, Sound::Valid, "{what}");
    let weighed = live(solid.measure, what);
    within_a_millimetre(weighed.volume, 6000.0, what);
    assert_eq!(weighed.solids, 1, "{what}");
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

fn shift_x(mm: f64) -> Place {
    Place { mat: [1.0, 0.0, 0.0, mm, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0] }
}
