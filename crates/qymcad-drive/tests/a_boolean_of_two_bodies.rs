use qymcad_drive::{Base, BodyId, BoolOp, Boolean, BoxLength, Corners, Cuboid, From, Height, Kind, Measure, Radius, Rebuild, Session, Sound, To, Tool, Width, WorldPlane};

/// The 12000 mm^3 box cut by the 1000 mm^3 extrusion inside it leaves 11000 mm^3, one cut node.
#[test]
fn a_cut_of_the_box_by_the_inner_extrusion_leaves_eleven_thousand() {
    let mut session = Session::empty();
    let (block, tool) = box_and_extrusion(&mut session, 0.0, 0.0, 10.0, 10.0);
    let cut = session.boolean(asked(block, tool, BoolOp::Cut)).expect("a cut");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.components.iter().filter(|item| item.kind == Kind::Part).count(), 1, "the two solids are one part");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-body-cut").count(), 1, "the cut node");
    near(live(&doc, cut), 11000.0, "the cut");
    let again = session.boolean(asked(block, tool, BoolOp::Cut));
    assert_eq!(again.expect_err("the box").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-body-cut").count(), 1, "a second cut was written");
    near(live(&doc, cut), 11000.0, "the cut after the second ask");
}

/// The union of the box and the extrusion inside it is the box, 12000 mm^3.
#[test]
fn a_union_of_the_box_and_the_inner_extrusion_is_the_box() {
    let mut session = Session::empty();
    let (block, tool) = box_and_extrusion(&mut session, 0.0, 0.0, 10.0, 10.0);
    let joined = session.boolean(asked(block, tool, BoolOp::Union)).expect("a union");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-body-union").count(), 1, "the union node");
    near(live(&doc, joined), 12000.0, "the union");
}

/// The common part of the box and the extrusion inside it is the extrusion, 1000 mm^3.
#[test]
fn an_intersection_of_the_box_and_the_inner_extrusion_is_one_thousand() {
    let mut session = Session::empty();
    let (block, tool) = box_and_extrusion(&mut session, 0.0, 0.0, 10.0, 10.0);
    let shared = session.boolean(asked(block, tool, BoolOp::Intersect)).expect("an intersection");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-body-intersect").count(), 1, "the intersection node");
    near(live(&doc, shared), 1000.0, "the common part");
}

/// The same body on both sides is refused before a node.
#[test]
fn the_same_body_on_both_sides_is_refused_before_a_node() {
    let mut session = Session::empty();
    let block = session.cuboid(box_40()).expect("a box");
    let same = session.boolean(asked(block, block, BoolOp::Union));
    assert_eq!(same.expect_err("the same body").text, "sketch-boolean-same");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-body-union"), "a node was written");
}

/// An id of 0 is refused before a node. A sketch id is refused before a node.
#[test]
fn an_id_that_is_not_a_body_is_refused_before_a_node() {
    let mut session = Session::empty();
    let (block, tool) = box_and_extrusion(&mut session, 0.0, 0.0, 10.0, 10.0);
    let zero = session.boolean(asked(BodyId { id: 0 }, tool, BoolOp::Cut));
    assert_eq!(zero.expect_err("the id").text, "error-source-body-not-built");
    let sketch = session.rectangle(WorldPlane::Xy, corners(0.0, 0.0, 10.0, 10.0)).expect("a rectangle");
    let foreign = session.boolean(asked(BodyId { id: sketch.sketch.id }, tool, BoolOp::Cut));
    assert_eq!(foreign.expect_err("the sketch").text, "error-source-body-deleted");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-body-cut"), "a node was written");
    let cut = session.boolean(asked(block, tool, BoolOp::Cut)).expect("a cut");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, cut), 11000.0, "the cut after the missing bodies");
}

/// A tool that misses the box is refused when the document is rebuilt.
#[test]
fn a_cut_that_misses_the_box_is_refused_on_the_read() {
    let mut session = Session::empty();
    let (block, tool) = box_and_extrusion(&mut session, 100.0, 100.0, 110.0, 110.0);
    let cut = session.boolean(asked(block, tool, BoolOp::Cut)).expect("the node is written");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-cut-removed-nothing", "the refusal was {}", refusal(&doc));
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-body-cut").count(), 1, "the cut node");
    assert_ne!(cut, block, "the refused cut is the box");
}

/// A cut that has not been read is not a body the next box can join. The read stays the miss.
#[test]
fn a_missed_cut_is_not_joined_to_the_next_box() {
    let mut session = Session::empty();
    let (block, tool) = box_and_extrusion(&mut session, 100.0, 100.0, 110.0, 110.0);
    let _cut = session.boolean(asked(block, tool, BoolOp::Cut)).expect("the node is written");
    let small = session.cuboid(Cuboid { length: BoxLength { mm: 2.0 }, width: Width { mm: 2.0 }, height: Height { mm: 2.0 } }).expect("a box");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-cut-removed-nothing", "the refusal was {}", refusal(&doc));
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-body-union").count(), 0, "the next box joined the cut");
    near(live(&doc, small), 8.0, "the next box");
    near(live(&doc, block), 12000.0, "the box");
}

/// A turn that crosses its axis is refused as a boolean tool before a node. The box stays.
#[test]
fn a_turn_that_will_not_build_is_not_a_boolean_tool() {
    let mut session = Session::empty();
    let block = session.cuboid(Cuboid { length: BoxLength { mm: 10.0 }, width: Width { mm: 20.0 }, height: Height { mm: 30.0 } }).expect("a box");
    let built = session.document();
    assert_eq!(built.rebuild, Rebuild::Quiet, "{}", refusal(&built));
    let across = session.rectangle(WorldPlane::Xz, corners(0.0, -5.0, 10.0, 15.0)).expect("a rectangle across the axis");
    let turned = session.revolve(across, qymcad_drive::SketchAxis::X, qymcad_drive::Angle { deg: 360.0 }).expect("the node is written");
    let bad = session.boolean(asked(block, turned, BoolOp::Cut));
    assert_eq!(bad.expect_err("the turn").text, "error-source-body-not-built");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-revolve-profile-crosses-axis", "the refusal was {}", refusal(&doc));
    assert!(doc.nodes.iter().all(|node| node.name != "feat-name-body-cut"), "a cut was written");
    near(live(&doc, block), 6000.0, "the box");
}

/// After the miss has been read, the next box stays 8 mm^3. Joining it to the tool that missed would be two solids.
#[test]
fn a_box_after_a_missed_cut_stays_its_own_solid() {
    let mut session = Session::empty();
    let (block, tool) = box_and_extrusion(&mut session, 100.0, 100.0, 110.0, 110.0);
    let _cut = session.boolean(asked(block, tool, BoolOp::Cut)).expect("the node is written");
    let missed = session.document();
    assert_eq!(refusal(&missed), "error-cut-removed-nothing", "the refusal was {}", refusal(&missed));
    let small = session.cuboid(Cuboid { length: BoxLength { mm: 2.0 }, width: Width { mm: 2.0 }, height: Height { mm: 2.0 } }).expect("a box");
    let doc = session.document();
    assert_eq!(refusal(&doc), "error-cut-removed-nothing", "the refusal was {}", refusal(&doc));
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-body-union").count(), 0, "the next box was joined");
    near(live(&doc, block), 12000.0, "the box");
    near(live(&doc, small), 8.0, "the next box");
}

/// A fillet the rebuild has refused still holds the extrusion. A cut of it must not be written beside that refusal.
#[test]
fn a_refused_fillet_holds_the_body_a_cut_would_take() {
    let mut session = Session::empty();
    let block = extrusion_held_by_a_refused_fillet(&mut session);
    let drawn = session.rectangle(WorldPlane::Xy, corners(0.0, 0.0, 10.0, 10.0)).expect("a rectangle");
    let tool = session.extrude(drawn, Height { mm: 10.0 }).expect("an extrusion");
    let failed = session.boolean(asked(block, tool, BoolOp::Cut));
    assert_eq!(failed.expect_err("the extrusion").text, "msg-body-consumed");
    let doc = session.document();
    assert_eq!(refusal(&doc), "fillet R100.00 failed on 1 edges", "the refusal was {}", refusal(&doc));
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-body-cut").count(), 0, "the held extrusion wrote a cut");
    near(live(&doc, block), 6000.0, "the extrusion beside the refusal");
}

fn extrusion_held_by_a_refused_fillet(session: &mut Session) -> BodyId {
    let drawn = session.rectangle(WorldPlane::Xz, corners(0.0, 0.0, 10.0, 20.0)).expect("a rectangle");
    let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let doc = session.document();
    let edge = doc.bodies.iter().find(|body| body.id == block.id).expect("the extrusion").edges.iter().find(|edge| (edge.length.mm - 20.0).abs() < 0.1).expect("the 20 mm edge").id;
    let _rounded = session.fillet(block, edge, Radius { mm: 100.0 }).expect("the node is written");
    let _refused = session.document();
    block
}

fn box_40() -> Cuboid {
    Cuboid { length: BoxLength { mm: 40.0 }, width: Width { mm: 30.0 }, height: Height { mm: 10.0 } }
}

fn box_and_extrusion(session: &mut Session, x0: f64, y0: f64, x1: f64, y1: f64) -> (BodyId, BodyId) {
    let block = session.cuboid(box_40()).expect("a box");
    let drawn = session.rectangle(WorldPlane::Xy, corners(x0, y0, x1, y1)).expect("a rectangle");
    let tool = session.extrude(drawn, Height { mm: 10.0 }).expect("an extrusion");
    (block, tool)
}

fn asked(base: BodyId, tool: BodyId, op: BoolOp) -> Boolean {
    Boolean { base: Base { body: base }, op, tool: Tool { body: tool } }
}

fn corners(x0: f64, y0: f64, x1: f64, y1: f64) -> Corners {
    Corners { from: From { at: (x0, y0) }, to: To { at: (x1, y1) } }
}

fn live(doc: &qymcad_drive::Document, id: BodyId) -> f64 {
    let body = doc.bodies.iter().find(|body| body.id == id.id).unwrap_or_else(|| panic!("body {} is gone", id.id));
    assert_eq!(body.sound, Sound::Valid, "the shape is valid");
    match body.measure {
        Measure::Live { volume, solids } => {
            assert_eq!(solids, 1, "one solid");
            volume.mm3
        }
        Measure::Absent => panic!("body {} has no live shape", id.id),
    }
}

fn near(got: f64, want: f64, what: &str) {
    assert!((got - want).abs() < 1.0, "{what}: volume {got} mm^3, wanted {want} within 1");
}

fn refusal(doc: &qymcad_drive::Document) -> String {
    match &doc.rebuild {
        Rebuild::Refused { refusals } => refusals.iter().map(|refusal| refusal.text.clone()).collect::<Vec<_>>().join(" "),
        Rebuild::Quiet => "quiet".into(),
        Rebuild::Stopped => "stopped".into(),
    }
}
