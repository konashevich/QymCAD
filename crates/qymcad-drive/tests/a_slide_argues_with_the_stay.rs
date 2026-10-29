use qymcad_drive::{BodyId, Bring, Corners, Fastened, Freedom, From, Height, Kind, Mate, MateId, Part, PartId, Place, Rebuild, Session, Slide, Sound, Stay, To, Travel, WorldPlane};

/// A rigid mate and a slider travel of 25 mm cannot both hold, and the brought part stays at 100 mm along X.
#[test]
fn the_slider_argues_with_the_stay() {
    let mut session = Session::empty();
    let made = fasten(&mut session, Travel { mm: 25.0 });
    let doc = session.document();
    argues(&doc, made.slider);
    assert_eq!(listed(&doc), vec![made.rigid.id, made.slider.id], "the mates that argue");
    assert_eq!(fastened(&doc, made.rigid).name, "name-joint-kind-rigid-n#1");
    assert_eq!(fastened(&doc, made.slider).name, "name-joint-kind-slider-n#2");
    along_x(component(&doc, made.stay).place, 0.0, "the staying part");
    along_x(component(&doc, made.bring).place, 100.0, "the brought part");
    free(named(&doc, "name-assembly").freedom, "the root");
    free(component(&doc, made.stay).freedom, "the staying part");
    free(component(&doc, made.bring).freedom, "the brought part");
    assert_eq!(parts(&doc), 2, "a third part appeared");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    exact(solid(&doc, made.first), "the first box");
    exact(solid(&doc, made.second), "the second box");
    let again = session.document();
    assert_eq!(listed(&again), vec![made.rigid.id, made.slider.id], "the second read cleared the argument");
    along_x(component(&again, made.bring).place, 100.0, "the brought part");
}

/// A travel the other way argues too, and the brought part still stays where it was placed.
#[test]
fn a_negative_travel_also_argues() {
    let mut session = Session::empty();
    let made = fasten(&mut session, Travel { mm: -25.0 });
    let doc = session.document();
    argues(&doc, made.slider);
    assert_eq!(listed(&doc), vec![made.rigid.id, made.slider.id], "the mates that argue");
    along_x(component(&doc, made.bring).place, 100.0, "the brought part");
    exact(solid(&doc, made.first), "the first box");
    exact(solid(&doc, made.second), "the second box");
}

/// A travel of 0 agrees with the rigid mate, and the brought part comes to the one that stays.
#[test]
fn a_zero_travel_agrees() {
    let mut session = Session::empty();
    let made = fasten(&mut session, Travel { mm: 0.0 });
    let doc = session.document();
    assert!(doc.violated.is_empty(), "a travel of 0 argues");
    assert_eq!(fastened(&doc, made.slider).name, "name-joint-kind-slider-n#2");
    along_x(component(&doc, made.bring).place, 0.0, "the brought part");
    free(component(&doc, made.bring).freedom, "the brought part");
    exact(solid(&doc, made.first), "the first box");
    exact(solid(&doc, made.second), "the second box");
}

/// A slider on its own holds, and the travel is along Z.
#[test]
fn the_slider_alone_shifts_along_z() {
    let mut session = Session::empty();
    let first = draw(&mut session);
    let held = session.document();
    let stay = PartId { id: named(&held, "name-part-n#1").id };
    let made = session.part(Part { place: shift_x(100.0) }).expect("a part");
    let second = grow(&mut session);
    let slider = session.slide(Slide { stay: Stay { part: stay }, bring: Bring { part: made }, by: Travel { mm: 25.0 } }).expect("a slide");
    let doc = session.document();
    assert!(doc.violated.is_empty(), "a slider alone argues");
    assert_eq!(fastened(&doc, slider).name, "name-joint-kind-slider-n#1");
    along_x(component(&doc, stay).place, 0.0, "the staying part");
    along_z(component(&doc, made).place, 25.0, "the brought part");
    free(named(&doc, "name-assembly").freedom, "the root");
    exact(solid(&doc, first), "the first box");
    exact(solid(&doc, second), "the second box");
}

/// A slider added after the rigid mate has been read leaves the brought part where that read put it.
#[test]
fn the_argument_leaves_the_solved_mate() {
    let mut session = Session::empty();
    let first = draw(&mut session);
    let held = session.document();
    let stay = PartId { id: named(&held, "name-part-n#1").id };
    let made = session.part(Part { place: shift_x(100.0) }).expect("a part");
    let second = grow(&mut session);
    let rigid = session.mate(Mate { stay: Stay { part: stay }, bring: Bring { part: made } }).expect("a mate");
    let mid = session.document();
    assert!(mid.violated.is_empty(), "the rigid mate argues");
    along_x(component(&mid, made).place, 0.0, "the brought part");
    let slider = session.slide(Slide { stay: Stay { part: stay }, bring: Bring { part: made }, by: Travel { mm: 25.0 } }).expect("a slide");
    let doc = session.document();
    argues(&doc, slider);
    assert_eq!(listed(&doc), vec![rigid.id, slider.id], "the mates that argue");
    along_x(component(&doc, made).place, 0.0, "the brought part");
    free(component(&doc, made).freedom, "the brought part");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    exact(solid(&doc, first), "the first box");
    exact(solid(&doc, second), "the second box");
}

/// A side that is not a part, the same part twice, and a travel that is not finite add no slider.
#[test]
fn a_slide_that_is_not_two_parts_writes_nothing() {
    let mut session = Session::empty();
    let block = draw(&mut session);
    let held = session.document();
    let stay = PartId { id: named(&held, "name-part-n#1").id };
    let root = PartId { id: held.components.iter().find(|part| part.kind == Kind::Assembly).expect("the root").id };
    let missing = session.slide(Slide { stay: Stay { part: PartId { id: 0 } }, bring: Bring { part: stay }, by: Travel { mm: 25.0 } });
    assert_eq!(missing.expect_err("the part").text, "error-mate-part");
    let assembly = session.slide(Slide { stay: Stay { part: root }, bring: Bring { part: stay }, by: Travel { mm: 25.0 } });
    assert_eq!(assembly.expect_err("the assembly").text, "error-mate-part");
    let body = session.slide(Slide { stay: Stay { part: PartId { id: block.id } }, bring: Bring { part: stay }, by: Travel { mm: 25.0 } });
    assert_eq!(body.expect_err("the body").text, "error-mate-part");
    let same = session.slide(Slide { stay: Stay { part: stay }, bring: Bring { part: stay }, by: Travel { mm: 25.0 } });
    assert_eq!(same.expect_err("the same part").text, "error-mate-same");
    let refused = session.document();
    assert!(refused.mates.is_empty(), "a refused call wrote a mate");
    assert_eq!(parts(&refused), 1, "a refused call wrote a part");
    let made = session.part(Part { place: shift_x(100.0) }).expect("a part");
    let infinite = session.slide(Slide { stay: Stay { part: stay }, bring: Bring { part: made }, by: Travel { mm: f64::NAN } });
    assert_eq!(infinite.expect_err("the travel").text, "error-slide-travel");
    let still = session.document();
    assert!(still.mates.is_empty(), "a refused travel wrote a mate");
    assert_eq!(parts(&still), 2, "a refused travel wrote a part");
    let _second = grow(&mut session);
    let rigid = session.mate(Mate { stay: Stay { part: stay }, bring: Bring { part: made } }).expect("a mate");
    let slider = session.slide(Slide { stay: Stay { part: stay }, bring: Bring { part: made }, by: Travel { mm: 25.0 } }).expect("a slide");
    let doc = session.document();
    assert_eq!(fastened(&doc, rigid).name, "name-joint-kind-rigid-n#1");
    assert_eq!(fastened(&doc, slider).name, "name-joint-kind-slider-n#2");
    argues(&doc, slider);
    along_x(component(&doc, made).place, 100.0, "the brought part");
}

/// The argument is still there after the file is opened, and a second read leaves the part at 100 mm.
#[test]
fn the_argument_opens_where_the_part_was_put() {
    let mut session = Session::empty();
    let made = fasten(&mut session, Travel { mm: 25.0 });
    let path = std::env::temp_dir().join(format!("qymcad-drive-slide-{}.qcad", std::process::id()));
    session.save(&path).expect("save");
    let mut opened = Session::open(&path).expect("open");
    let _gone = std::fs::remove_file(&path);
    let doc = opened.document();
    argues(&doc, made.slider);
    assert_eq!(listed(&doc), vec![made.rigid.id, made.slider.id], "the mates that argue");
    assert_eq!(fastened(&doc, made.slider).name, "name-joint-kind-slider-n#2");
    along_x(component(&doc, made.bring).place, 100.0, "the opened part");
    free(component(&doc, made.bring).freedom, "the opened part");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    exact(solid(&doc, made.first), "the first box");
    exact(solid(&doc, made.second), "the second box");
    let again = opened.document();
    assert_eq!(listed(&again), vec![made.rigid.id, made.slider.id], "the second read cleared the argument");
    along_x(component(&again, made.bring).place, 100.0, "the opened part");
}

struct Fastening {
    rigid: MateId,
    stay: PartId,
    first: BodyId,
    slider: MateId,
    bring: PartId,
    second: BodyId,
}

fn fasten(session: &mut Session, by: Travel) -> Fastening {
    let first = draw(session);
    let held = session.document();
    let stay = PartId { id: named(&held, "name-part-n#1").id };
    let bring = session.part(Part { place: shift_x(100.0) }).expect("a part");
    let second = grow(session);
    let rigid = session.mate(Mate { stay: Stay { part: stay }, bring: Bring { part: bring } }).expect("a mate");
    let slider = session.slide(Slide { stay: Stay { part: stay }, bring: Bring { part: bring }, by }).expect("a slide");
    Fastening { rigid, stay, first, slider, bring, second }
}

fn draw(session: &mut Session) -> BodyId {
    let block = grow(session);
    let _read = session.document();
    block
}

fn grow(session: &mut Session) -> BodyId {
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("the box");
    session.extrude(drawn, Height { mm: 30.0 }).expect("the box body")
}

fn shift_x(mm: f64) -> Place {
    Place { mat: [1.0, 0.0, 0.0, mm, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0] }
}

fn shift_z(mm: f64) -> Place {
    Place { mat: [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, mm] }
}

fn argues(doc: &qymcad_drive::Document, id: MateId) {
    if !doc.violated.iter().any(|mate| mate.id == id.id) {
        panic!("the mates do not argue");
    }
}

fn listed(doc: &qymcad_drive::Document) -> Vec<u64> {
    doc.violated.iter().map(|mate| mate.id).collect()
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

fn free(got: Freedom, what: &str) {
    assert_eq!(got.count, 6, "{what}");
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

fn along_x(got: Place, mm: f64, what: &str) {
    let want = shift_x(mm);
    for index in 0..got.mat.len() {
        assert!((got.mat[index] - want.mat[index]).abs() < 1e-6, "{what} differs at {index}");
    }
}

fn along_z(got: Place, mm: f64, what: &str) {
    let want = shift_z(mm);
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
