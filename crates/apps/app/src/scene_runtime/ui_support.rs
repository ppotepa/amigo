use super::super::*;
use amigo_scene::SceneUiDocument;

pub(super) fn register_ui_font_asset_references(
    asset_catalog: &AssetCatalog,
    source_mod: &str,
    document: &SceneUiDocument,
) {
    for font in amigo_ui::collect_scene_ui_font_asset_keys(document) {
        crate::app_helpers::register_mod_asset_reference(
            asset_catalog,
            source_mod,
            &font,
            "ui",
            "font",
        );
    }
}

pub(super) fn convert_scene_ui_document(document: &SceneUiDocument) -> RuntimeUiDocument {
    amigo_ui::scene_ui_document_to_runtime_document(document)
}
