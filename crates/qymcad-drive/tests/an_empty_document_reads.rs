use qymcad_drive::{Kind, Rebuild, Session};

/// An empty document is the root assembly alone, and the rebuild of it says nothing.
#[test]
fn an_empty_document_reads_as_the_root_assembly() {
    let mut session = Session::empty();
    let doc = session.document();

    assert_eq!(doc.components.len(), 1, "an empty document is one root assembly");
    let root = &doc.components[0];
    assert_eq!(root.name, "name-assembly", "the root keeps the name the document gives it");
    assert_eq!(root.kind, Kind::Assembly);
    assert_ne!(root.id, 0, "the root has an id");
    assert_eq!(root.parent, None, "the root has no parent");
    assert_eq!(doc.sketches, Vec::new(), "an empty document has no sketches");
    assert_eq!(doc.nodes, Vec::new(), "an empty document has no timeline nodes");
    assert!(doc.bodies.is_empty(), "an empty document has no bodies");
    assert_eq!(doc.rebuild, Rebuild::Quiet);

    let id = root.id;
    let again = session.document();
    assert_eq!(again.components.len(), 1, "a second read is still the one root");
    assert_eq!(again.components[0].id, id, "a second read is the same document");
    assert_eq!(again.rebuild, Rebuild::Quiet);
}
