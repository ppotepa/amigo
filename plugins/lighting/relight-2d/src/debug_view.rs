use amigo_camera::{CameraDebugViewDescriptor, CameraDebugViewId};

pub fn relight_debug_view_descriptors() -> Vec<CameraDebugViewDescriptor> {
    [
        ("aux_depth", "Plate Relight Aux Depth", &["plate_relight_aux_depth", "plate_aux_depth", "relight_aux_depth"][..]),
        ("aux_height", "Plate Relight Aux Height", &["plate_relight_aux_height", "plate_aux_height", "relight_aux_height"][..]),
        ("aux_occluder", "Plate Relight Aux Occluder", &["plate_relight_aux_occluder", "plate_aux_occluder", "relight_aux_occluder"][..]),
        ("aux_valid", "Plate Relight Aux Valid", &["plate_relight_aux_valid", "plate_aux_valid", "relight_aux_valid"][..]),
        ("surface_reflect", "Plate Relight Surface Reflect", &["plate_relight_surface_reflect", "plate_surface_reflect", "relight_surface_reflect"][..]),
        ("surface_rough", "Plate Relight Surface Rough", &["plate_relight_surface_rough", "plate_surface_rough", "relight_surface_rough"][..]),
        ("surface_glass", "Plate Relight Surface Glass", &["plate_relight_surface_glass", "plate_surface_glass", "relight_surface_glass"][..]),
        ("surface_mask", "Plate Relight Surface Mask", &["plate_relight_surface_mask", "plate_surface_mask", "relight_surface_mask"][..]),
        ("effective_depth", "Plate Relight Effective Depth", &["plate_relight_effective_depth", "plate_effective_depth", "relight_effective_depth"][..]),
        ("normal", "Plate Relight Normal", &["plate_relight_normal", "plate_normal", "relight_normal"][..]),
        ("occlusion", "Plate Relight Occlusion", &["plate_relight_occlusion", "plate_occlusion", "relight_occlusion"][..]),
        ("contribution", "Plate Relight Contribution", &["plate_relight_contribution", "plate_contribution", "relight_contribution"][..]),
        ("shadow", "Plate Relight Shadow", &["plate_relight_shadow", "plate_shadow", "relight_shadow"][..]),
        ("light_mask", "Plate Relight Light Mask", &["plate_relight_light_mask", "plate_light_mask", "relight_light_mask"][..]),
        ("ndl", "Plate Relight NDL", &["plate_relight_ndl", "plate_ndl", "relight_ndl"][..]),
        ("specular", "Plate Relight Specular", &["plate_relight_specular", "plate_specular", "relight_specular"][..]),
        ("material_gate", "Plate Relight Material Gate", &["plate_relight_material_gate", "plate_material_gate", "relight_material_gate"][..]),
        ("lit_raw", "Plate Relight Lit Raw", &["plate_relight_lit_raw", "plate_lit_raw", "relight_lit_raw"][..]),
    ]
    .into_iter()
    .map(|(suffix, label, aliases)| CameraDebugViewDescriptor {
        id: CameraDebugViewId::new(format!("relight.plate.{suffix}")),
        label: label.to_owned(),
        aliases: aliases.iter().map(|alias| (*alias).to_owned()).collect(),
        tags: vec!["relight".to_owned(), "debug".to_owned()],
        stop_after_feature: Some("relight".to_owned()),
    })
    .collect()
}

pub fn is_plate_relight_debug_view(view: &CameraDebugViewId) -> bool {
    view.as_str().starts_with("relight.plate.")
}

pub fn is_plate_relight_render_debug_view(view: &amigo_render_api::CameraDebugView2d) -> bool {
    view.as_str().starts_with("relight.plate.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relight_debug_view_identifies_plate_debug_views() {
        assert!(is_plate_relight_debug_view(&CameraDebugViewId::new(
            "relight.plate.normal"
        )));
        assert!(!is_plate_relight_debug_view(
            &CameraDebugViewId::final_output()
        ));
        assert!(is_plate_relight_render_debug_view(
            &amigo_render_api::CameraDebugView2d::new("relight.plate.normal")
        ));
        assert!(!is_plate_relight_render_debug_view(
            &amigo_render_api::CameraDebugView2d::final_output()
        ));
    }
}
