use qymcad_drive::{BodyId, Corners, From, Height, Kind, Part, PartId, Place, Rebuild, Session, Sound, To, Volume, WorldPlane};

/// A second part, shifted 100 mm along X, keeps its own 6000 mm^3 box.
#[test]
fn the_second_part_stands_clear() {
    let mut session = Session::empty();
    let block = stock(&mut session);
    let made = session.part(Part { place: shift_x(100.0) }).expect("a part");
    let doc = session.document();
    let placed = component(&doc, made);
    place_is(placed.place, 100.0, "the new part");
    assert_eq!(placed.name, "name-part-n#2");
    assert_eq!(placed.parent, Some(1), "the parent is not the root");
    place_is(named(&doc, "name-part-n#1").place, 0.0, "the first part");
    assert_eq!(parts(&doc), 2, "the part count");
    let second = draw(&mut session);
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(solid(&doc, second), Volume { mm3: 6000.0 }, "the new box");
    near(solid(&doc, block), Volume { mm3: 6000.0 }, "the first box");
    assert_eq!(owner_of(&doc, second), made.id, "the new box is not in the new part");
    assert_eq!(owner_of(&doc, block), named(&doc, "name-part-n#1").id, "the first box left its part");
    assert_eq!(parts(&doc), 2, "a third part appeared");
}

/// An empty document gains one part at the identity, and the next box belongs to it.
#[test]
fn an_empty_document_gains_a_part() {
    let mut session = Session::empty();
    let made = session.part(Part { place: shift_x(0.0) }).expect("a part");
    let doc = session.document();
    let placed = component(&doc, made);
    assert_eq!(placed.name, "name-part-n#1");
    assert_eq!(placed.parent, Some(1), "the parent is not the root");
    place_is(placed.place, 0.0, "the part");
    assert_eq!(doc.components.iter().find(|part| part.kind == Kind::Assembly).expect("the root").name, "name-assembly");
    assert!(doc.bodies.is_empty(), "a body appeared");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    let block = draw(&mut session);
    let doc = session.document();
    assert_eq!(owner_of(&doc, block), made.id, "the box is not in the new part");
    near(solid(&doc, block), Volume { mm3: 6000.0 }, "the box");
    assert_eq!(parts(&doc), 1, "a second part appeared");
}

/// A placement that is not finite adds no part.
#[test]
fn a_bad_placement_writes_no_part() {
    let mut session = Session::empty();
    let block = stock(&mut session);
    let bad = session.part(Part { place: unfixed() });
    assert_eq!(bad.expect_err("the placement").text, "error-part-place");
    let held = session.document();
    assert_eq!(parts(&held), 1, "a refused call wrote a part");
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
    let made = session.part(Part { place: shift_x(100.0) }).expect("a part");
    let doc = session.document();
    place_is(component(&doc, made).place, 100.0, "the new part");
    assert_eq!(component(&doc, made).name, "name-part-n#2");
    let second = draw(&mut session);
    let doc = session.document();
    near(solid(&doc, second), Volume { mm3: 6000.0 }, "the new box");
    near(solid(&doc, block), Volume { mm3: 6000.0 }, "the first box");
    assert_eq!(parts(&doc), 2, "the part count");
}

/// The placement and both boxes are still there after the file is opened.
#[test]
fn the_placed_part_opens_where_it_stood() {
    let mut session = Session::empty();
    let block = stock(&mut session);
    let made = session.part(Part { place: shift_x(100.0) }).expect("a part");
    let second = draw(&mut session);
    let path = std::env::temp_dir().join(format!("qymcad-drive-part-{}.qcad", std::process::id()));
    session.save(&path).expect("save");
    let mut opened = Session::open(&path).expect("open");
    let doc = opened.document();
    let _gone = std::fs::remove_file(&path);
    place_is(component(&doc, made).place, 100.0, "the opened part");
    near(solid(&doc, block), Volume { mm3: 6000.0 }, "the first box");
    near(solid(&doc, second), Volume { mm3: 6000.0 }, "the new box");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

fn stock(session: &mut Session) -> BodyId {
    draw(session)
}

fn draw(session: &mut Session) -> BodyId {
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("the box");
    let block = session.extrude(drawn, Height { mm: 30.0 }).expect("the box body");
    let _read = session.document();
    block
}

fn shift_x(mm: f64) -> Place {
    Place { mat: [1.0, 0.0, 0.0, mm, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0] }
}

fn unfixed() -> Place {
    let mut by = shift_x(100.0);
    by.mat[3] = f64::NAN;
    by
}

fn component(doc: &qymcad_drive::Document, id: PartId) -> &qymcad_drive::Component {
    doc.components.iter().find(|part| part.id == id.id).unwrap_or_else(|| panic!("part {} is gone", id.id))
}

fn named(doc: &qymcad_drive::Document, name: &str) -> qymcad_drive::Component {
    doc.components.iter().find(|part| part.name == name).unwrap_or_else(|| panic!("part {name} is gone")).clone()
}

fn parts(doc: &qymcad_drive::Document) -> usize {
    doc.components.iter().filter(|part| part.kind == Kind::Part).count()
}

fn owner_of(doc: &qymcad_drive::Document, body: BodyId) -> u64 {
    doc.nodes.iter().find(|node| node.id == body.id).and_then(|node| node.parent).unwrap_or_else(|| panic!("body {} has no part", body.id))
}

fn body(doc: &qymcad_drive::Document, id: BodyId) -> &qymcad_drive::Body {
    doc.bodies.iter().find(|body| body.id == id.id).unwrap_or_else(|| panic!("body {} is gone", id.id))
}

struct Read {
    volume: f64,
    solids: u32,
}

fn read(doc: &qymcad_drive::Document, id: BodyId) -> Read {
    let found = body(doc, id);
    assert_eq!(found.sound, Sound::Valid, "the shape is valid");
    match found.measure {
        qymcad_drive::Measure::Live { volume, solids } => Read { volume: volume.mm3, solids },
        qymcad_drive::Measure::Absent => panic!("body {} has no live shape", id.id),
    }
}

fn solid(doc: &qymcad_drive::Document, id: BodyId) -> f64 {
    let found = read(doc, id);
    assert_eq!(found.solids, 1, "one solid");
    found.volume
}

fn near(got: f64, want: Volume, what: &str) {
    assert!((got - want.mm3).abs() < 1.0, "{what}: volume {got} mm^3, wanted {} within 1", want.mm3);
}

fn place_is(got: Place, mm: f64, what: &str) {
    let want = shift_x(mm);
    for index in 0..got.mat.len() {
        assert!((got.mat[index] - want.mat[index]).abs() < 1e-9, "{what} differs at {index}");
    }
}

fn refusal(doc: &qymcad_drive::Document) -> String {
    match &doc.rebuild {
        Rebuild::Refused { refusals } => refusals.iter().map(|refusal| refusal.text.clone()).collect::<Vec<_>>().join(" "),
        Rebuild::Quiet => "quiet".into(),
        Rebuild::Stopped => "stopped".into(),
    }
}
