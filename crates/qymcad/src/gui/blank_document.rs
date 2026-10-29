use super::*;

/// An empty document, stood at the origin of a new file.
pub(crate) fn start(app: &mut App) {
    app.project = Project::default();
    app.project.new_document();
    app.live.shapes.clear();
    app.chosen.sel = Sel::None;
    app.disk.project_path = None;
    app.disk.dxf_path = None;
    app.tools.pending_import.draw_pts = None;
    invalidate(&mut app.regen);
    app.viewing.view.initialized = false;
    app.viewing.cam.init = false;
    app.disk.edits.fresh(); // a clean start: no steps of the document before, the baseline taken on the next frame
    app.status = Notice::info(crate::i18n::tr("g-new-project"));
}
