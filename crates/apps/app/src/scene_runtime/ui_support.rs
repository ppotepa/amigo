use super::super::*;
use amigo_runtime_bundles::{
    UiDocument as RuntimeUiDocument, collect_scene_ui_font_asset_keys,
    scene_ui_document_to_runtime_document,
};
use amigo_scene::SceneUiDocument;

pub(super) fn register_ui_font_asset_references(
    asset_catalog: &AssetCatalog,
    source_mod: &str,
    document: &SceneUiDocument,
) {
    for font in collect_scene_ui_font_asset_keys(document) {
        crate::app_helpers::register_mod_asset_reference(
            asset_catalog,
            source_mod,
            &font,
            "fonts",
            "font-2d",
        );
    }
}
#[allow(dead_code)]
pub(super) fn convert_scene_ui_document(document: &SceneUiDocument) -> RuntimeUiDocument {
    scene_ui_document_to_runtime_document(document)
}
