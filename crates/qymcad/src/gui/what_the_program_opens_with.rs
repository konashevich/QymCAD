//! WHAT THE PROGRAM OPENS WITH, and why it is decided in one named place.
//!
//! Two reports, and they turned out to be one question:
//!
//! * "in an empty project remove the empty part - a person opens the CAD and sees an empty part; the first
//!   time round that spoils everything";
//! * "add a setting to reopen the last project ... and a tick for always showing the start screen".
//!
//! Both are answers to "which document does the program start with", and that decision used to live inside
//! the launch closure - a place no check can run. So it was "always reopen the last project", written once
//! and measured never.
//!
//! It is `qymcad_ui_state::opening` now: two flags in, three outcomes out, every one of them somebody's
//! first minute. A path on the command line is a fourth, and it comes ahead of the remembered project.
//!
//! WHY THE EMPTY DOCUMENT IS NOT `App::default()`. That one hands out a document WITH a part, and about
//! five hundred checks build their scene on it - measured: taking the part out of it fails 75 of them. A
//! part is right for a check that needs somewhere to draw, and right for "New part", which is what the
//! person asked for. It is wrong only for the first minute, and that is what changed.
#[cfg(test)]
mod tests {
    use qymcad_ui_state::{asked_file, opening, AskedFile, Opening, Settings};

    /// AN EMPTY DOCUMENT HOLDS NO PART.
    ///
    /// The root stays: a document without one has nowhere to put anything, and the tree would show an
    /// empty pane instead of a document.
    #[test]
    fn an_empty_document_has_a_root_and_no_part() {
        let mut p = qymcad_core::model::Project::default();
        let root = p.new_empty_document();

        assert_eq!(p.root, root, "the root of the document must be the component that was made");
        assert_eq!(p.components.len(), 1, "an empty document holds the root ALONE, and it holds {} components", p.components.len());
        assert!(p.components.iter().all(|c| c.kind != qymcad_core::feature::ComponentKind::Part), "a part nobody asked for is in the empty document - that is the report word for word");
        assert!(p.timeline.is_empty(), "and nothing in the timeline");
    }

    /// AND `new_document` STILL MAKES ONE, because that is what "New part" means.
    ///
    /// Without this the check above would be green for a program in which a part cannot be made at all.
    #[test]
    fn new_document_still_makes_a_part() {
        let mut p = qymcad_core::model::Project::default();
        let part = p.new_document();
        assert_eq!(p.component_kind(part), Some(qymcad_core::feature::ComponentKind::Part), "\"New part\" must give a part");
    }

    /// A path that certainly exists, and one that certainly does not.
    fn a_real_file() -> std::path::PathBuf {
        let p = std::env::temp_dir().join("qymcad-opening-probe.qcad");
        std::fs::write(&p, b"probe").expect("the probe file is written");
        p
    }

    /// ALL FOUR COMBINATIONS OF THE TWO TICKS.
    ///
    /// Written out rather than looped: each line is a different first minute, and a loop would hide which
    /// of them broke.
    #[test]
    fn the_two_ticks_decide_the_opening_between_them() {
        let file = a_real_file();
        let path = file.to_string_lossy().into_owned();

        let mut set = Settings { open_last: true, show_start_screen: true, ..Default::default() };
        assert_eq!(opening(&set, Some(&path), AskedFile::None), Opening::LastProject(path.clone()), "asked to reopen and there is something to reopen");

        set.show_start_screen = false;
        assert_eq!(opening(&set, Some(&path), AskedFile::None), Opening::LastProject(path.clone()), "the start screen has no say in WHICH document");

        set.open_last = false;
        set.show_start_screen = true;
        assert_eq!(opening(&set, Some(&path), AskedFile::None), Opening::Empty { start_screen: true }, "not asked to reopen: an empty document, and the person is greeted");

        set.show_start_screen = false;
        assert_eq!(opening(&set, Some(&path), AskedFile::None), Opening::Empty { start_screen: false }, "and with the greeting off, an empty document in silence");
    }

    /// A REMEMBERED PATH THAT NO LONGER NAMES A FILE IS NOT AN EMPTY WINDOW IN SILENCE.
    ///
    /// The project may have been moved or deleted between sessions. Falling through to an empty document
    /// is right; falling through to an empty document with the greeting suppressed would leave a person
    /// staring at nothing with no clue why.
    #[test]
    fn a_project_that_has_gone_falls_back_to_the_start_screen() {
        let set = Settings { open_last: true, show_start_screen: true, ..Default::default() };
        let gone = std::env::temp_dir().join("qymcad-no-such-project-9c1f.qcad");
        let _ = std::fs::remove_file(&gone);

        assert_eq!(
            opening(&set, Some(&gone.to_string_lossy()), AskedFile::None),
            Opening::Empty { start_screen: true },
            "the remembered project is gone, so the person must be greeted rather than left with an empty window"
        );
    }

    /// THE FACTORY ANSWER: an empty document and the start screen.
    ///
    /// It is the first minute of everybody who has never opened the settings, so it is checked by name
    /// rather than left to follow from the defaults.
    #[test]
    fn out_of_the_box_the_program_opens_empty_and_greets() {
        assert_eq!(opening(&Settings::default(), None, AskedFile::None), Opening::Empty { start_screen: true });
    }

    /// A FILE NAMED ON THE COMMAND LINE OPENS, and the remembered project stays where it was.
    #[test]
    fn a_named_file_opens_ahead_of_the_remembered_project() {
        let named = a_real_file();
        let remembered = std::env::temp_dir().join("qymcad-opening-remembered.qcad");
        std::fs::write(&remembered, b"remembered").expect("the remembered file is written");
        let set = Settings { open_last: true, show_start_screen: true, ..Default::default() };
        let asked = AskedFile::Path(named.to_string_lossy().into_owned());
        assert_eq!(
            opening(&set, Some(&remembered.to_string_lossy()), asked.clone()),
            Opening::Asked(named.to_string_lossy().into_owned()),
            "the remembered project opened instead of the file this start was given"
        );
        let gone = std::env::temp_dir().join("qymcad-opening-named-gone.qcad");
        let _ = std::fs::remove_file(&gone);
        assert_eq!(
            opening(&set, Some(&remembered.to_string_lossy()), AskedFile::Path(gone.to_string_lossy().into_owned())),
            Opening::Empty { start_screen: true },
            "a missing named file fell through to the remembered project"
        );
    }

    /// THE FIRST WORD AFTER THE PROGRAM NAME IS THE FILE. A later word is not a second file.
    #[test]
    fn the_first_argument_is_the_file() {
        assert_eq!(asked_file(["/tmp/cap.qcad", "--later"]), AskedFile::Path("/tmp/cap.qcad".into()));
        assert_eq!(asked_file(std::iter::empty::<&str>()), AskedFile::None);
        assert_eq!(asked_file([""]), AskedFile::None, "an empty word counted as a named file");
    }
}
