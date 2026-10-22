use qymcad_drive::{Corners, Form, Kind, Plane, Rebuild, Session, WorldPlane};

/// A rectangle on XZ is a part, a sketch on that plane, and one closed contour. XZ is not the plane a
/// new sketch already has, so a call that ignores the plane does not pass.
#[test]
fn a_rectangle_lies_on_xz() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, corners()).expect("a rectangle");
    let doc = session.document();

    let sketch = doc.sketches.iter().find(|s| s.plane == Plane::Xz).expect("a sketch lies on Xz");
    assert_ne!(drawn.sketch.id, 0, "the sketch has an id");
    assert_ne!(drawn.contour.id, 0, "the contour has an id");
    assert_eq!(sketch.id, drawn.sketch.id);
    assert_eq!(sketch.name, "name-sketch-n#1");
    assert_eq!(sketch.contours.len(), 1, "a rectangle is one contour");
    assert_eq!(sketch.contours[0], qymcad_drive::Contour { id: drawn.contour.id, form: Form::Closed });

    assert_eq!(doc.components.len(), 2, "the root assembly and the part the sketch belongs to");
    let root = doc.components.iter().find(|c| c.parent.is_none()).expect("the root");
    assert_eq!(root.kind, Kind::Assembly);
    assert_eq!(root.name, "name-assembly");
    let part = doc.components.iter().find(|c| c.kind == Kind::Part).expect("the part");
    assert_eq!(part.name, "name-part-n#1");
    assert_eq!(part.parent, Some(root.id));

    assert!(doc.bodies.is_empty(), "a rectangle is not a body yet");
    assert_eq!(doc.nodes.len(), 1, "the sketch is the only timeline node");
    assert_eq!(doc.nodes[0].id, drawn.sketch.id);
    assert_eq!(doc.nodes[0].parent, Some(part.id));
    assert_eq!(doc.rebuild, Rebuild::Quiet);

    let again = session.document();
    assert_eq!(again.sketches[0].id, drawn.sketch.id, "a second read is the same sketch");
    assert_eq!(again.sketches[0].contours[0].id, drawn.contour.id, "a second read is the same contour");
}

/// A second rectangle is another sketch. Both cannot carry the first sketch's name, or the tree cannot
/// tell them apart.
#[test]
fn a_second_rectangle_is_the_next_sketch() {
    let mut session = Session::empty();
    let first = session.rectangle(WorldPlane::Xz, corners()).expect("a rectangle");
    let second = session.rectangle(WorldPlane::Yz, corners()).expect("a rectangle");
    let doc = session.document();

    assert_ne!(first.sketch.id, second.sketch.id, "each rectangle is its own sketch");
    let names: Vec<_> = doc.sketches.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, vec!["name-sketch-n#1", "name-sketch-n#2"], "the second sketch is the next name, saw {names:?}");
    let node_names: Vec<_> = doc.nodes.iter().map(|n| n.name.as_str()).collect();
    assert_eq!(node_names, names, "the timeline uses the same names");
    assert!(doc.sketches.iter().any(|s| s.id == second.sketch.id && s.plane == Plane::Yz), "the second sketch lies on Yz");
    assert_eq!(doc.components.iter().filter(|c| c.kind == Kind::Part).count(), 1, "both sketches stand in the one part");
    assert!(doc.bodies.is_empty(), "a rectangle is not a body yet");
}

fn corners() -> Corners {
    Corners { from: qymcad_drive::From { at: (0.0, 0.0) }, to: qymcad_drive::To { at: (10.0, 20.0) } }
}
