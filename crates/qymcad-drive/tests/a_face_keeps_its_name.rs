use qymcad_drive::{Corners, EdgeId, FaceId, FaceRole, Height, LineId, Measure, Rebuild, Session, Sound, WorldPlane};

/// A face and an edge of the box are found by name, and the same names still point at them after the
/// sketch above the body is resized.
#[test]
fn a_face_keeps_its_name() {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, rectangle_10_by_20()).expect("a rectangle");
    let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
    let first = session.document();
    let picked = named(&first, 200.0, 600.0, 20.0);
    assert_eq!(first.bodies[0].faces.len(), 6, "six faces");
    assert_eq!(first.bodies[0].edges.len(), 12, "twelve edges");
    assert!(ids_are_distinct(&first));

    let again = session.document();
    let kept = named(&again, 200.0, 600.0, 20.0);
    assert_eq!(kept.cap, picked.cap, "a clean read issued a new cap name");
    assert_eq!(kept.wall, picked.wall, "a clean read issued a new wall name");
    assert_eq!(kept.source, picked.source, "a clean read issued a new wall line");
    assert_eq!(kept.edge, picked.edge, "a clean read issued a new edge name");

    session.resize(drawn, rectangle_10_by_10()).expect("the rectangle moves");
    let shrunk = session.document();
    assert_eq!(shrunk.rebuild, Rebuild::Quiet, "the rebuild is quiet");
    assert_eq!(shrunk.bodies.len(), 1, "one body");
    assert_eq!(shrunk.bodies[0].id, made.id, "the body id is the one the extrusion returned");
    assert_eq!(shrunk.bodies[0].sound, Sound::Valid, "the shape is valid");
    let volume = match shrunk.bodies[0].measure {
        Measure::Live { volume, solids } => {
            assert_eq!(solids, 1, "one solid");
            volume.mm3
        }
        Measure::Absent => panic!("the body has no live shape"),
    };
    assert!((volume - 3000.0).abs() < 1.0, "volume {volume} mm^3, wanted 3000 within 1");
    assert_eq!(shrunk.sketches.len(), 1, "the resize does not add a sketch");
    assert_eq!(shrunk.sketches[0].id, drawn.sketch.id, "the sketch id is unchanged");
    let cap_now = face_area(&shrunk, picked.cap);
    let wall_now = face_area(&shrunk, picked.wall);
    let edge_now = edge_length(&shrunk, picked.edge);
    assert!((cap_now - 100.0).abs() < 1.0, "end cap area {cap_now} mm^2, wanted 100 within 1");
    assert!((wall_now - 300.0).abs() < 1.0, "wall area {wall_now} mm^2, wanted 300 within 1");
    assert!((edge_now - 10.0).abs() < 0.1, "edge length {edge_now} mm, wanted 10 within 0.1");
    let wall_role = shrunk.bodies[0].faces.iter().find(|face| face.id == picked.wall).expect("the wall is gone").role;
    assert_eq!(wall_role, FaceRole::Wall { source: picked.source }, "the wall is no longer that sketch line");
}

struct Picked {
    cap: FaceId,
    wall: FaceId,
    source: LineId,
    edge: EdgeId,
}

fn named(doc: &qymcad_drive::Document, cap_mm2: f64, wall_mm2: f64, edge_mm: f64) -> Picked {
    assert_eq!(doc.bodies.len(), 1, "one body");
    let body = &doc.bodies[0];
    let caps: Vec<_> = body.faces.iter().filter(|face| face.role == FaceRole::CapEnd).collect();
    assert_eq!(caps.len(), 1, "one end cap");
    assert_eq!(body.faces.iter().filter(|face| face.role == FaceRole::CapStart).count(), 1, "one start cap");
    let cap = caps[0];
    assert!((cap.area.mm2 - cap_mm2).abs() < 1.0, "end cap area {} mm^2, wanted {cap_mm2} within 1", cap.area.mm2);
    let wall = body
        .faces
        .iter()
        .find(|face| match face.role {
            FaceRole::Wall { .. } => (face.area.mm2 - wall_mm2).abs() < 1.0,
            _ => false,
        })
        .unwrap_or_else(|| panic!("wall area {} mm^2 was not in the read, wanted {wall_mm2} within 1", wall_mm2));
    let FaceRole::Wall { source } = wall.role else { unreachable!() };
    let edge = body.edges.iter().find(|edge| (edge.length.mm - edge_mm).abs() < 0.1).unwrap_or_else(|| panic!("edge length {edge_mm} mm was not in the read"));
    Picked { cap: cap.id, wall: wall.id, source, edge: edge.id }
}

fn face_area(doc: &qymcad_drive::Document, id: FaceId) -> f64 {
    doc.bodies[0].faces.iter().find(|face| face.id == id).unwrap_or_else(|| panic!("face {} is gone", id.id)).area.mm2
}

fn edge_length(doc: &qymcad_drive::Document, id: EdgeId) -> f64 {
    doc.bodies[0].edges.iter().find(|edge| edge.id == id).unwrap_or_else(|| panic!("edge {} is gone", id.id)).length.mm
}

fn ids_are_distinct(doc: &qymcad_drive::Document) -> bool {
    let body = &doc.bodies[0];
    let mut faces: Vec<_> = body.faces.iter().map(|face| face.id).collect();
    let mut edges: Vec<_> = body.edges.iter().map(|edge| edge.id).collect();
    let face_count = faces.len();
    let edge_count = edges.len();
    faces.sort();
    faces.dedup();
    edges.sort();
    edges.dedup();
    faces.len() == face_count && edges.len() == edge_count
}

fn rectangle_10_by_20() -> Corners {
    Corners { from: qymcad_drive::From { at: (0.0, 0.0) }, to: qymcad_drive::To { at: (10.0, 20.0) } }
}

fn rectangle_10_by_10() -> Corners {
    Corners { from: qymcad_drive::From { at: (0.0, 0.0) }, to: qymcad_drive::To { at: (10.0, 10.0) } }
}
