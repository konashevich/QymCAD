use qymcad_drive::{
    Area, BodyId, BoxLength, CopyFace, Corners, Cuboid, FaceId, FaceRole, From, Height, Kind, OffsetFace, Outset, Radius, Rebuild, ReplaceFace, Session, Sound, To, Volume, Width, WorldPlane,
};

/// The end cap, replaced by that cap lifted 10 mm, makes the box 8000 mm^3.
#[test]
fn the_lifted_cap_makes_the_box_taller() {
    let mut session = Session::empty();
    let made = stock(&mut session);
    let sheet = moved(&mut session, &made, Outset { mm: 10.0 });
    let replaced = session.replace_face(sew(made.block, made.cap, sheet)).expect("a replacement");
    let doc = session.document();
    near(solid(&doc, replaced), Volume { mm3: 8000.0 }, "the lifted box");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.components.iter().filter(|item| item.kind == Kind::Part).count(), 1, "one part");
    assert_eq!(replacements(&doc), 1, "the replacement node");
    assert_eq!(doc.nodes.iter().find(|node| node.id == replaced.id).expect("the node").name, "feat-name-surface-replace");
    assert_ne!(replaced, made.block, "the replacement is the box");
    near(solid(&doc, made.block), Volume { mm3: 6000.0 }, "the box");
    let _held = taken(&mut session, made.block, made.cap);
    let face = body(&doc, sheet).faces[0].id;
    let _sheet = taken(&mut session, sheet, face);
    let kept = body(&doc, replaced).faces[0].id;
    let _copy = session.copy_face(CopyFace { body: replaced, face: kept }).expect("the replacement was taken");
}

/// The end cap's own copy, sewn back, leaves the box at 6000 mm^3.
#[test]
fn the_cap_put_back_stays_the_box() {
    let mut session = Session::empty();
    let made = stock(&mut session);
    let sheet = copied_cap(&mut session, &made);
    let replaced = session.replace_face(sew(made.block, made.cap, sheet)).expect("a replacement");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(solid(&doc, replaced), Volume { mm3: 6000.0 }, "the box");
    one_face(sheet_areas(&doc, sheet), Area { mm2: 200.0 }, "the copy");
    near(solid(&doc, made.block), Volume { mm3: 6000.0 }, "the box record");
}

/// The end cap, replaced by that cap sunk 10 mm, makes the box 4000 mm^3.
#[test]
fn the_sunk_cap_makes_the_box_shorter() {
    let mut session = Session::empty();
    let made = stock(&mut session);
    let sheet = moved(&mut session, &made, Outset { mm: -10.0 });
    let replaced = session.replace_face(sew(made.block, made.cap, sheet)).expect("a replacement");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(solid(&doc, replaced), Volume { mm3: 4000.0 }, "the sunk box");
    near(solid(&doc, made.block), Volume { mm3: 6000.0 }, "the box");
}

/// A wall that misses the opening is replaced by the lifted cap.
#[test]
fn a_wall_that_misses_is_replaced_by_the_lifted_cap() {
    let mut session = Session::empty();
    let made = stock(&mut session);
    let wall = copied_wall(&mut session, &made);
    let sheet = moved(&mut session, &made, Outset { mm: 10.0 });
    let failed = session.replace_face(sew(made.block, made.cap, wall)).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "surface leaves 6 free edges", "the refusal was {}", refusal(&doc));
    absent(&doc, failed);
    near(solid(&doc, made.block), Volume { mm3: 6000.0 }, "the box");
    one_face(sheet_areas(&doc, wall), Area { mm2: 300.0 }, "the wall");
    let gap = session.replace_face(sew(made.block, FaceId { id: 0 }, sheet));
    assert_eq!(gap.expect_err("the face").text, "error-faces-not-found");
    let solid_sheet = session.replace_face(sew(made.block, made.cap, made.block));
    assert_eq!(solid_sheet.expect_err("the box").text, "error-replace-sheet");
    let held = session.document();
    assert_eq!(refusal(&held), "surface leaves 6 free edges", "the refusal was {}", refusal(&held));
    assert_eq!(replacements(&held), 1, "the refused replacement was rewritten");
    absent(&held, failed);
    let replaced = session.replace_face(sew(made.block, made.cap, sheet)).expect("the lift is tried");
    assert_eq!(replaced, failed, "a second replacement was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(replacements(&doc), 1, "the refused replacement was left beside the new one");
    near(solid(&doc, replaced), Volume { mm3: 8000.0 }, "the lifted box");
    let _held = taken(&mut session, made.block, made.cap);
    let face = body(&doc, sheet).faces[0].id;
    let _sheet = taken(&mut session, sheet, face);
    let kept = body(&doc, replaced).faces[0].id;
    let _copy = session.copy_face(CopyFace { body: replaced, face: kept }).expect("the replacement was taken");
}

/// The id a refused replacement returned is that same replacement.
#[test]
fn a_refused_replacement_named_by_its_own_id_is_the_same_replacement() {
    let mut session = Session::empty();
    let made = stock(&mut session);
    let wall = copied_wall(&mut session, &made);
    let sheet = moved(&mut session, &made, Outset { mm: 10.0 });
    let failed = session.replace_face(sew(made.block, made.cap, wall)).expect("the node is written");
    let _held = session.document();
    let replaced = session.replace_face(sew(failed, made.cap, sheet)).expect("the lift is tried");
    assert_eq!(replaced, failed, "a second replacement was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(replacements(&doc), 1, "the refused replacement was left beside the new one");
    near(solid(&doc, replaced), Volume { mm3: 8000.0 }, "the lifted box");
    near(solid(&doc, made.block), Volume { mm3: 6000.0 }, "the box");
}

/// A sheet built after the refused replacement is not sewn in. Opened again, the miss is unchanged.
#[test]
fn a_later_sheet_is_not_the_surface() {
    let mut session = Session::empty();
    let made = stock(&mut session);
    let wall = copied_wall(&mut session, &made);
    let early = moved(&mut session, &made, Outset { mm: 10.0 });
    let failed = session.replace_face(sew(made.block, made.cap, wall)).expect("the node is written");
    let _held = session.document();
    let other = stock(&mut session);
    let later = copied_cap(&mut session, &other);
    let forward = session.replace_face(sew(made.block, made.cap, later));
    assert_eq!(forward.expect_err("the later sheet").text, "error-replace-built");
    let by_id = session.replace_face(sew(failed, made.cap, later));
    assert_eq!(by_id.expect_err("the later sheet").text, "error-replace-built");
    let doc = session.document();
    assert_eq!(refusal(&doc), "surface leaves 6 free edges", "the refusal was {}", refusal(&doc));
    assert_eq!(replacements(&doc), 1, "the refused replacement was rewritten");
    absent(&doc, failed);
    let opened = opened(&session, "later");
    assert_eq!(refusal(&opened), "surface leaves 6 free edges", "the refusal was {}", refusal(&opened));
    absent(&opened, failed);
    let replaced = session.replace_face(sew(failed, made.cap, early)).expect("the lift is tried");
    assert_eq!(replaced, failed, "a second replacement was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(solid(&doc, replaced), Volume { mm3: 8000.0 }, "the lifted box");
}

/// A face that is not on the box is replaced by the lifted cap.
#[test]
fn a_missing_face_is_replaced_by_the_lifted_cap() {
    let mut session = Session::empty();
    let made = stock(&mut session);
    let sheet = moved(&mut session, &made, Outset { mm: 10.0 });
    let failed = session.replace_face(sew(made.block, FaceId { id: 1 }, sheet)).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-faces-not-found", "the refusal was {}", refusal(&doc));
    absent(&doc, failed);
    near(solid(&doc, made.block), Volume { mm3: 6000.0 }, "the box");
    let replaced = session.replace_face(sew(made.block, made.cap, sheet)).expect("the lift is tried");
    assert_eq!(replaced, failed, "a second replacement was added");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(replacements(&doc), 1, "the refused replacement was left beside the new one");
    near(solid(&doc, replaced), Volume { mm3: 8000.0 }, "the lifted box");
}

/// The box already replaced is not replaced again.
#[test]
fn the_replaced_box_is_not_replaced_again() {
    let mut session = Session::empty();
    let made = stock(&mut session);
    let sheet = moved(&mut session, &made, Outset { mm: 10.0 });
    let replaced = session.replace_face(sew(made.block, made.cap, sheet)).expect("a replacement");
    let _read = session.document();
    let again = session.replace_face(sew(made.block, made.cap, sheet));
    assert_eq!(again.expect_err("the box").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(replacements(&doc), 1, "a second replacement was added");
    near(solid(&doc, replaced), Volume { mm3: 8000.0 }, "the lifted box");
}

/// A missing body, a missing sheet, an unread body, and a sheet asked as the solid write no node.
#[test]
fn a_missing_body_or_sheet_writes_no_replacement() {
    let mut session = Session::empty();
    let made = stock(&mut session);
    let sheet = moved(&mut session, &made, Outset { mm: 10.0 });
    let missing_body = session.replace_face(sew(BodyId { id: 0 }, made.cap, sheet));
    assert_eq!(missing_body.expect_err("id 0").text, "error-source-body-not-built");
    let missing_sheet = session.replace_face(sew(made.block, made.cap, BodyId { id: 0 }));
    assert_eq!(missing_sheet.expect_err("id 0").text, "error-source-body-not-built");
    let sketch = session.replace_face(sew(BodyId { id: made.drawn.sketch.id }, made.cap, sheet));
    assert_eq!(sketch.expect_err("a sketch").text, "error-source-body-deleted");
    let sketch_sheet = session.replace_face(sew(made.block, made.cap, BodyId { id: made.drawn.sketch.id }));
    assert_eq!(sketch_sheet.expect_err("a sketch").text, "error-source-body-deleted");
    let raw = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (40.0, 40.0) }, to: To { at: (50.0, 50.0) } }).expect("a square");
    let unread_body = session.extrude(raw, Height { mm: 10.0 }).expect("a body");
    let no_body = session.replace_face(sew(unread_body, made.cap, sheet));
    assert_eq!(no_body.expect_err("the body").text, "error-source-body-not-built");
    let unread_sheet = session.offset_face(OffsetFace { body: made.block, face: made.cap, by: Outset { mm: 10.0 } }).expect("an offset");
    let no_sheet = session.replace_face(sew(made.block, made.cap, unread_sheet));
    assert_eq!(no_sheet.expect_err("the sheet").text, "error-source-body-not-built");
    let cap = copied_cap(&mut session, &made);
    let sheet_body = session.replace_face(sew(cap, made.cap, sheet));
    assert_eq!(sheet_body.expect_err("the sheet").text, "error-replace-body");
    let solid_sheet = session.replace_face(sew(made.block, made.cap, made.block));
    assert_eq!(solid_sheet.expect_err("the box").text, "error-replace-sheet");
    let held = session.document();
    assert_eq!(replacements(&held), 0, "a refused call wrote a replacement");
    assert_eq!(held.rebuild, Rebuild::Quiet, "{}", refusal(&held));
    let replaced = session.replace_face(sew(made.block, made.cap, sheet)).expect("a replacement");
    let doc = session.document();
    near(solid(&doc, replaced), Volume { mm3: 8000.0 }, "the lifted box");
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
}

/// A fillet the rebuild has refused still holds its extrusion.
#[test]
fn a_refused_fillet_holds_the_extrusion() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = body(&doc, block).edges.iter().find(|item| (item.length.mm - 20.0).abs() < 0.1).expect("the 20 mm edge").id;
    let cap = body(&doc, block).faces.iter().find(|face| face.role == FaceRole::CapEnd).expect("the cap").id;
    let sheet = session.copy_face(CopyFace { body: block, face: cap }).expect("a copy");
    let _sheet = session.document();
    let _rounded = session.fillet(block, edge, Radius { mm: 100.0 }).expect("the node is written");
    let _refused = session.document();
    let failed = session.replace_face(sew(block, cap, sheet));
    assert_eq!(failed.expect_err("the extrusion").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(replacements(&doc), 0, "a replacement was written");
    assert_eq!(refusal(&doc), "fillet R100.00 failed on 1 edges", "the refusal was {}", refusal(&doc));
    near(solid(&doc, block), Volume { mm3: 6000.0 }, "the extrusion");
}

/// An unread replacement of the cap is the solid the cube joins.
#[test]
fn an_unread_replacement_joins_the_cube() {
    let mut session = Session::empty();
    let made = stock(&mut session);
    let sheet = moved(&mut session, &made, Outset { mm: 10.0 });
    let replaced = session.replace_face(sew(made.block, made.cap, sheet)).expect("a replacement");
    let joined = session.cuboid(Cuboid { length: BoxLength { mm: 2.0 }, width: Width { mm: 2.0 }, height: Height { mm: 2.0 } }).expect("a box");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.nodes.iter().find(|node| node.id == joined.id).expect("the union").name, "feat-name-body-union");
    near(solid(&doc, joined), Volume { mm3: 8006.0 }, "the union");
    near(solid(&doc, replaced), Volume { mm3: 8000.0 }, "the replacement");
    near(solid(&doc, made.block), Volume { mm3: 6000.0 }, "the box");
    let _held = taken(&mut session, made.block, made.cap);
    let kept = body(&doc, replaced).faces[0].id;
    let _copy = taken(&mut session, replaced, kept);
}

/// The replaced box follows the rectangle. A 10 by 10 mm cap lifted 10 mm is 4000 mm^3.
#[test]
fn the_replaced_box_follows_the_rectangle() {
    let mut session = Session::empty();
    let made = stock(&mut session);
    let sheet = moved(&mut session, &made, Outset { mm: 10.0 });
    let replaced = session.replace_face(sew(made.block, made.cap, sheet)).expect("a replacement");
    let _read = session.document();
    session.resize(made.drawn, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 10.0) } }).expect("a smaller rectangle");
    let smaller = session.document();
    assert_eq!(smaller.rebuild, Rebuild::Quiet, "{}", refusal(&smaller));
    near(solid(&smaller, replaced), Volume { mm3: 4000.0 }, "the lifted box");
    near(solid(&smaller, made.block), Volume { mm3: 3000.0 }, "the box");
}

struct Stock {
    drawn: qymcad_drive::Profile,
    block: BodyId,
    cap: FaceId,
}

fn stock(session: &mut Session) -> Stock {
    let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("the box");
    let block = session.extrude(drawn, Height { mm: 30.0 }).expect("the box body");
    let doc = session.document();
    let cap = body(&doc, block).faces.iter().find(|face| face.role == FaceRole::CapEnd).expect("the cap").id;
    Stock { drawn, block, cap }
}

fn moved(session: &mut Session, made: &Stock, by: Outset) -> BodyId {
    let sheet = session.offset_face(OffsetFace { body: made.block, face: made.cap, by }).expect("an offset");
    let _read = session.document();
    sheet
}

fn copied_cap(session: &mut Session, made: &Stock) -> BodyId {
    let sheet = session.copy_face(CopyFace { body: made.block, face: made.cap }).expect("a copy");
    let _read = session.document();
    sheet
}

fn copied_wall(session: &mut Session, made: &Stock) -> BodyId {
    let doc = session.document();
    let wall = body(&doc, made.block).faces.iter().find(|face| (face.area.mm2 - 300.0).abs() < 1.0).expect("the short wall").id;
    let sheet = session.copy_face(CopyFace { body: made.block, face: wall }).expect("the wall");
    let _read = session.document();
    sheet
}

fn sew(body: BodyId, face: FaceId, surface: BodyId) -> ReplaceFace {
    ReplaceFace { body, face, surface }
}

fn opened(session: &Session, name: &str) -> qymcad_drive::Document {
    let path = std::env::temp_dir().join(format!("qymcad-drive-replace-{name}-{}.qcad", std::process::id()));
    session.save(&path).expect("save");
    let mut opened = Session::open(&path).expect("open");
    let doc = opened.document();
    let _gone = std::fs::remove_file(&path);
    doc
}

fn replacements(doc: &qymcad_drive::Document) -> usize {
    doc.nodes.iter().filter(|node| node.name == "feat-name-surface-replace").count()
}

fn body(doc: &qymcad_drive::Document, id: BodyId) -> &qymcad_drive::Body {
    doc.bodies.iter().find(|body| body.id == id.id).unwrap_or_else(|| panic!("body {} is gone", id.id))
}

fn absent(doc: &qymcad_drive::Document, id: BodyId) {
    assert!(doc.bodies.iter().all(|body| body.id != id.id), "body {} is still listed", id.id);
}

fn solid(doc: &qymcad_drive::Document, id: BodyId) -> f64 {
    let found = body(doc, id);
    assert_eq!(found.sound, Sound::Valid, "the shape is valid");
    match found.measure {
        qymcad_drive::Measure::Live { volume, solids } => {
            assert_eq!(solids, 1, "one solid");
            volume.mm3
        }
        qymcad_drive::Measure::Absent => panic!("body {} has no live shape", id.id),
    }
}

fn sheet_areas(doc: &qymcad_drive::Document, id: BodyId) -> Vec<f64> {
    let found = body(doc, id);
    assert_eq!(found.sound, Sound::Valid, "the shape is valid");
    match found.measure {
        qymcad_drive::Measure::Live { volume, solids } => {
            assert_eq!(solids, 0, "a sheet has no solid");
            assert!(volume.mm3.abs() < 1.0, "a sheet holds {} mm^3", volume.mm3);
        }
        qymcad_drive::Measure::Absent => panic!("body {} has no live shape", id.id),
    }
    assert!(found.faces.iter().all(|face| face.role == FaceRole::Other), "a face kept a solid role");
    found.faces.iter().map(|face| face.area.mm2).collect()
}

fn one_face(got: Vec<f64>, want: Area, what: &str) {
    assert_eq!(got.len(), 1, "{what}: {} faces", got.len());
    assert!((got[0] - want.mm2).abs() < 1.0, "{what}: area {} mm^2, wanted {} within 1", got[0], want.mm2);
}

fn near(got: f64, want: Volume, what: &str) {
    assert!((got - want.mm3).abs() < 1.0, "{what}: volume {got} mm^3, wanted {} within 1", want.mm3);
}

fn taken(session: &mut Session, body: BodyId, face: FaceId) -> Result<BodyId, qymcad_drive::DrawError> {
    let copied = session.copy_face(CopyFace { body, face });
    assert_eq!(copied.expect_err("the body").text, "msg-body-consumed");
    Err(qymcad_drive::DrawError { text: "msg-body-consumed".into() })
}

fn refusal(doc: &qymcad_drive::Document) -> String {
    match &doc.rebuild {
        Rebuild::Refused { refusals } => refusals.iter().map(|refusal| refusal.text.clone()).collect::<Vec<_>>().join(" "),
        Rebuild::Quiet => "quiet".into(),
        Rebuild::Stopped => "stopped".into(),
    }
}
