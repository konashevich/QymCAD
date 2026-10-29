use qymcad_drive::{BodyId, Bring, Corners, Fastened, From, Height, Kind, Mate, MateId, Part, PartId, Place, Rebuild, Session, Sound, Stay, To, Volume, WorldPlane};

/// A rigid mate brings the part that stood 100 mm along X onto the part that stays.
#[test]
fn the_brought_part_meets_the_stay() {
    let mut session = Session::empty();
    let block = stock(&mut session);
    let held = session.document();
    let stay = PartId { id: named(&held, "name-part-n#1").id };
    let made = session.part(Part { place: shift_x(100.0) }).expect("a part");
    let second = draw(&mut session);
    let joined = session.mate(Mate { stay: Stay { part: stay }, bring: Bring { part: made } }).expect("a mate");
    let doc = session.document();
    let fastening = fastened(&doc, joined);
    assert_eq!(fastening.name, "name-joint-kind-rigid-n#1");
    assert_eq!(fastening.stay.part.id, stay.id, "the mate left the other part");
    assert_eq!(fastening.bring.part.id, made.id, "the mate brought the other part");
    near_identity(component(&doc, stay).place, "the staying part");
    near_identity(component(&doc, made).place, "the brought part");
    assert_eq!(component(&doc, made).name, "name-part-n#2");
    assert_eq!(component(&doc, made).parent, Some(1), "the parent is not the root");
    assert_eq!(parts(&doc), 2, "a third part appeared");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(solid(&doc, block), Volume { mm3: 6000.0 }, "the first box");
    near(solid(&doc, second), Volume { mm3: 6000.0 }, "the second box");
    let later = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (1.0, 1.0) } }).expect("a rectangle");
    let doc = session.document();
    assert_eq!(owner_of(&doc, later.sketch.id), made.id, "the session left the brought part");
    assert_eq!(parts(&doc), 2, "a third part appeared");
}

/// A side that is not a part, and one part named twice, add no mate.
#[test]
fn a_mate_that_is_not_two_parts_writes_nothing() {
    let mut session = Session::empty();
    let block = stock(&mut session);
    let held = session.document();
    let stay = PartId { id: named(&held, "name-part-n#1").id };
    let root = PartId { id: held.components.iter().find(|part| part.kind == Kind::Assembly).expect("the root").id };
    let missing = session.mate(Mate { stay: Stay { part: PartId { id: 0 } }, bring: Bring { part: stay } });
    assert_eq!(missing.expect_err("the part").text, "error-mate-part");
    let assembly = session.mate(Mate { stay: Stay { part: root }, bring: Bring { part: stay } });
    assert_eq!(assembly.expect_err("the assembly").text, "error-mate-part");
    let body = session.mate(Mate { stay: Stay { part: PartId { id: block.id } }, bring: Bring { part: stay } });
    assert_eq!(body.expect_err("the body").text, "error-mate-part");
    let same = session.mate(Mate { stay: Stay { part: stay }, bring: Bring { part: stay } });
    assert_eq!(same.expect_err("the same part").text, "error-mate-same");
    let refused = session.document();
    assert!(refused.mates.is_empty(), "a refused call wrote a mate");
    assert_eq!(parts(&refused), 1, "a refused call wrote a part");
    let made = session.part(Part { place: shift_x(100.0) }).expect("a part");
    let second = draw(&mut session);
    let joined = session.mate(Mate { stay: Stay { part: stay }, bring: Bring { part: made } }).expect("a mate");
    let doc = session.document();
    near_identity(component(&doc, made).place, "the brought part");
    assert_eq!(fastened(&doc, joined).name, "name-joint-kind-rigid-n#1");
    near(solid(&doc, block), Volume { mm3: 6000.0 }, "the first box");
    near(solid(&doc, second), Volume { mm3: 6000.0 }, "the second box");
}

/// The brought part is still on the staying part after the file is opened.
#[test]
fn the_brought_part_opens_where_the_mate_left_it() {
    let mut session = Session::empty();
    let block = stock(&mut session);
    let held = session.document();
    let stay = PartId { id: named(&held, "name-part-n#1").id };
    let made = session.part(Part { place: shift_x(100.0) }).expect("a part");
    let second = draw(&mut session);
    let _joined = session.mate(Mate { stay: Stay { part: stay }, bring: Bring { part: made } }).expect("a mate");
    let path = std::env::temp_dir().join(format!("qymcad-drive-mate-{}.qcad", std::process::id()));
    session.save(&path).expect("save");
    let mut opened = Session::open(&path).expect("open");
    let doc = opened.document();
    let _gone = std::fs::remove_file(&path);
    near_identity(component(&doc, made).place, "the opened part");
    near_identity(component(&doc, stay).place, "the staying part");
    near(solid(&doc, block), Volume { mm3: 6000.0 }, "the first box");
    near(solid(&doc, second), Volume { mm3: 6000.0 }, "the second box");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.mates.len(), 1, "the mate was not saved");
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

fn fastened(doc: &qymcad_drive::Document, id: MateId) -> &Fastened {
    doc.mates.iter().find(|mate| mate.id == id.id).unwrap_or_else(|| panic!("mate {} is gone", id.id))
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

fn owner_of(doc: &qymcad_drive::Document, id: u64) -> u64 {
    doc.nodes.iter().find(|node| node.id == id).and_then(|node| node.parent).unwrap_or_else(|| panic!("node {id} has no part"))
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
