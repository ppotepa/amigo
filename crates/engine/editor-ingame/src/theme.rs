use amigo_assets::AssetKey;
use amigo_editor_authoring::{AuthoringTreeIcon, AuthoringTreeTag};

#[derive(Debug, Clone, Copy, Default)]
pub struct EditorTheme;

pub fn editor_icon_font() -> AssetKey {
    AssetKey::new("core/fonts/fontawesome-free-solid")
}
pub fn icon_glyph(icon: AuthoringTreeIcon) -> &'static str {
    match icon {
        AuthoringTreeIcon::Entity | AuthoringTreeIcon::Scene => "\u{f007}",
        AuthoringTreeIcon::Image | AuthoringTreeIcon::Visual2d => "\u{f03e}",
        AuthoringTreeIcon::Component => "\u{f1c5}",
        AuthoringTreeIcon::Particle => "\u{f0e7}",
        AuthoringTreeIcon::Camera => "\u{f030}",
        AuthoringTreeIcon::DrawLayer => "\u{f5fd}",
        AuthoringTreeIcon::Light => "\u{f0eb}",
        AuthoringTreeIcon::PostFx => "\u{f0d0}",
        AuthoringTreeIcon::Mapping | AuthoringTreeIcon::Sequence | AuthoringTreeIcon::Scalar => {
            "\u{f1c9}"
        }
        AuthoringTreeIcon::Use | AuthoringTreeIcon::Prefab | AuthoringTreeIcon::Override => {
            "\u{f0c1}"
        }
        AuthoringTreeIcon::Text | AuthoringTreeIcon::Ui => "\u{f1c5}",
        AuthoringTreeIcon::Route => "\u{f0e7}",
        AuthoringTreeIcon::Mod => "\u{f085}",
    }
}
pub fn icon_label(icon: AuthoringTreeIcon) -> &'static str {
    match icon {
        AuthoringTreeIcon::Mod => "Mod",
        AuthoringTreeIcon::Use => "Use",
        AuthoringTreeIcon::Scene => "Scene",
        AuthoringTreeIcon::Visual2d => "Visual",
        AuthoringTreeIcon::Entity => "Scene Object",
        AuthoringTreeIcon::Component => "Component",
        AuthoringTreeIcon::Image => "Image",
        AuthoringTreeIcon::Particle => "Particles",
        AuthoringTreeIcon::Text => "Text",
        AuthoringTreeIcon::Camera => "Camera",
        AuthoringTreeIcon::Ui => "UI",
        AuthoringTreeIcon::DrawLayer => "Draw Layer",
        AuthoringTreeIcon::PostFx => "Frame Post FX",
        AuthoringTreeIcon::Light => "Light",
        AuthoringTreeIcon::Route => "Route",
        AuthoringTreeIcon::Prefab => "Prefab",
        AuthoringTreeIcon::Override => "Override",
        AuthoringTreeIcon::Mapping => "Raw Map",
        AuthoringTreeIcon::Sequence => "Raw List",
        AuthoringTreeIcon::Scalar => "Raw Value",
    }
}

pub fn icon_ascii(icon: AuthoringTreeIcon) -> &'static str {
    match icon {
        AuthoringTreeIcon::Mod => "[MOD]",
        AuthoringTreeIcon::Use => "[USE]",
        AuthoringTreeIcon::Scene => "[SCN]",
        AuthoringTreeIcon::Visual2d => "[VIS]",
        AuthoringTreeIcon::Entity => "[OBJ]",
        AuthoringTreeIcon::Component => "[CMP]",
        AuthoringTreeIcon::Image => "[IMG]",
        AuthoringTreeIcon::Particle => "[PAR]",
        AuthoringTreeIcon::Text => "[TXT]",
        AuthoringTreeIcon::Camera => "[CAM]",
        AuthoringTreeIcon::Ui => "[UI]",
        AuthoringTreeIcon::DrawLayer => "[LYR]",
        AuthoringTreeIcon::PostFx => "[FX]",
        AuthoringTreeIcon::Light => "[LGT]",
        AuthoringTreeIcon::Route => "[RTE]",
        AuthoringTreeIcon::Prefab => "[PFB]",
        AuthoringTreeIcon::Override => "[OVR]",
        AuthoringTreeIcon::Mapping => "[MAP]",
        AuthoringTreeIcon::Sequence => "[SEQ]",
        AuthoringTreeIcon::Scalar => "[VAL]",
    }
}
pub fn format_tags(tags: &[AuthoringTreeTag]) -> String {
    tags.iter()
        .map(|tag| format!("[{}]", tag.label))
        .collect::<Vec<_>>()
        .join(" ")
}
pub fn format_compact_tags(tags: &[AuthoringTreeTag]) -> String {
    tags.iter()
        .map(|tag| format!("[{}]", tag.label))
        .collect::<Vec<_>>()
        .join("")
}

pub fn format_primary_tags(tags: &[AuthoringTreeTag]) -> String {
    for wanted in ["Live", "Mock", "Readonly", "Unsupported", "Hidden"] {
        if tags.iter().any(|tag| tag.label == wanted) {
            return format!("[{wanted}]");
        }
    }
    String::new()
}

pub fn format_property_tags(tags: &[String]) -> String {
    for wanted in ["Live", "Mock", "Readonly", "Unsupported"] {
        if tags.iter().any(|tag| tag == wanted) {
            return format!("[{wanted}]");
        }
    }
    String::new()
}
