use std::path::{Path, PathBuf};

use qymcad_drive::{Kind, Rebuild, Session};

/// An empty document written out and opened again is the same tree, and a file that is not a bundle is refused.
#[test]
fn an_empty_document_roundtrips() {
    let path = Sweep::new(temp("empty"));
    let plain = Sweep::new(temp("plain"));
    let fake = Sweep::new(temp("fake-pk"));
    let mut session = Session::empty();
    let before = session.document();
    assert_eq!(before.components.len(), 1, "the saved document is one root assembly");
    assert_eq!(before.components[0].kind, Kind::Assembly);
    assert_eq!(before.sketches, Vec::new(), "the saved document has no sketches");
    assert_eq!(before.nodes, Vec::new(), "the saved document has no timeline nodes");
    assert_eq!(before.rebuild, Rebuild::Quiet);
    session.save(&path.0).expect("the empty document is written");
    let bytes = std::fs::read(&path.0).expect("the empty document is a file");
    assert!(is_bundle(&bytes), "the written file is a zip bundle, not a pair of PK bytes");

    let mut opened = Session::open(&path.0).expect("the bundle opens");
    assert_eq!(opened.document(), before, "the opened document is the one that was saved");

    std::fs::write(&plain.0, b"not a qcad").expect("the plain file is planted");
    let plain_err = match Session::open(&plain.0) {
        Err(err) => err,
        Ok(_) => panic!("plain text is not a bundle"),
    };
    assert!(plain_err.text.contains("io-not-a-qcad"), "plain text was refused as {plain_err:?}");

    std::fs::write(&fake.0, b"PK not a zip").expect("the fake bundle is planted");
    assert!(Session::open(&fake.0).is_err(), "bytes that only start with PK are not a bundle");
}

/// A path the file layer cannot name is refused. Rewriting it with a replacement character would write a
/// different file, and an empty document already at that other path would make the rewrite look successful.
#[test]
fn a_path_that_is_not_unicode_is_refused() {
    let raw = RawPath(non_unicode("bad"));
    let substituted = Sweep::new(PathBuf::from(raw.0.to_string_lossy().as_ref()));
    assert_ne!(&substituted.0, &raw.0, "the rewritten path is a different file");
    Session::empty().save(&substituted.0).expect("the other path holds an empty document");

    match Session::open(&raw.0) {
        Err(err) => assert!(err.text.contains("io-path-not-utf8"), "opening was refused as {err:?}"),
        Ok(_) => panic!("a path that is not Unicode was opened"),
    }
    match Session::empty().save(&raw.0) {
        Err(err) => assert!(err.text.contains("io-path-not-utf8"), "saving was refused as {err:?}"),
        Ok(()) => panic!("a path that is not Unicode was written"),
    }
    Session::open(&substituted.0).expect("the other path is still the document that was written there");
}

/// The real path, removed when the check ends. Its name is not Unicode, so the sidecar names are not derived
/// from a display string.
struct RawPath(PathBuf);

impl Drop for RawPath {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Removes the bundle and the sidecars a write leaves beside it, including when a check fails.
struct Sweep(PathBuf);

impl Sweep {
    fn new(path: PathBuf) -> Self {
        remove(&path);
        Self(path)
    }
}

impl Drop for Sweep {
    fn drop(&mut self) {
        remove(&self.0);
    }
}

fn temp(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("qymcad-drive-{name}-{}.qcad", std::process::id()))
}

/// A file name that is not Unicode. The byte is one the file layer cannot take as a path string.
fn non_unicode(name: &str) -> PathBuf {
    use std::os::unix::ffi::OsStringExt;
    let mut bytes = std::env::temp_dir().into_os_string().into_vec();
    bytes.push(b'/');
    bytes.extend(format!("qymcad-drive-{name}-{}-", std::process::id()).into_bytes());
    bytes.push(0xff);
    bytes.extend(b".qcad");
    PathBuf::from(std::ffi::OsString::from_vec(bytes))
}

fn remove(path: &Path) {
    let text = path.display().to_string();
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(format!("{text}.bak"));
    let _ = std::fs::remove_file(format!("{text}.tmp~"));
}

/// A bundle carries the document entry. Two bytes `PK` do not.
fn is_bundle(bytes: &[u8]) -> bool {
    bytes.starts_with(b"PK") && bytes.windows(b"document.ron".len()).any(|window| window == b"document.ron")
}
