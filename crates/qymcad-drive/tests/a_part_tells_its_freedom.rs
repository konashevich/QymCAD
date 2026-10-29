use qymcad_drive::{BodyId, Bring, Corners, Freedom, From, Height, Kind, Mate, Part, PartId, Place, Rebuild, Session, Sound, Stay, To, WorldPlane};

/// A free part has six ways to move, and fastening it to another free part leaves both at six.
#[test]
fn the_free_part_tells_its_count() {
    let mut session = Session::empty();
    let block = draw(&mut session);
    let held = session.document();
    let first = named(&held, "name-part-n#1");
    shows(first.freedom, "the part");
    shows(named(&held, "name-assembly").freedom, "the root");
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
    exact(solid(&held, block), "the first box");
    let made = session.part(Part { place: shift_x(100.0) }).expect("a part");
    let second = draw(&mut session);
    let placed = session.document();
    shows(component(&placed, PartId { id: first.id }).freedom, "the first part");
    shows(component(&placed, made).freedom, "the second part");
    assert_eq!(placed.rebuild, Rebuild::Quiet, "{}", refusal(&placed));
    let _joined = session.mate(Mate { stay: Stay { part: PartId { id: first.id } }, bring: Bring { part: made } }).expect("a mate");
    let doc = session.document();
    shows(named(&doc, "name-assembly").freedom, "the fastened root");
    shows(component(&doc, PartId { id: first.id }).freedom, "the staying part");
    shows(component(&doc, made).freedom, "the brought part");
    near_identity(component(&doc, PartId { id: first.id }).place, "the staying part");
    near_identity(component(&doc, made).place, "the brought part");
    assert_eq!(parts(&doc), 2, "a third part appeared");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    exact(solid(&doc, block), "the first box");
    exact(solid(&doc, second), "the second box");
    let mut bare = Session::empty();
    let empty = bare.document();
    shows(named(&empty, "name-assembly").freedom, "the empty root");
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

fn shows(got: Freedom, what: &str) {
    if got.count == 0 {
        panic!("{what} shows 0");
    }
    assert_eq!(got.count, 6, "{what}");
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

fn solid(doc: &qymcad_drive::Document, id: BodyId) -> f64 {
    let found = doc.bodies.iter().find(|body| body.id == id.id).unwrap_or_else(|| panic!("body {} is gone", id.id));
    assert_eq!(found.sound, Sound::Valid, "the shape is valid");
    match found.measure {
        qymcad_drive::Measure::Live { volume, solids } => {
            assert_eq!(solids, 1, "one solid");
            volume.mm3
        }
        qymcad_drive::Measure::Absent => panic!("body {} has no live shape", id.id),
    }
}

fn exact(got: f64, what: &str) {
    assert!((got - 6000.0).abs() < 1e-9, "{what}: volume {got} mm^3");
}

fn near_identity(got: Place, what: &str) {
    let want = shift_x(0.0);
    for index in 0..got.mat.len() {
        assert!((got.mat[index] - want.mat[index]).abs() < 1e-6, "{what} differs at {index}");
    }
}

fn refusal(doc: &qymcad_drive::Document) -> String {
    match &doc.rebuild {
        Rebuild::Refused { refusals } => refusals.iter().map(|refusal| refusal.text.clone()).collect::<Vec<_>>().join(" "),
        Rebuild::Quiet => "quiet".into(),
        Rebuild::Stopped => "stopped".into(),
    }
}
