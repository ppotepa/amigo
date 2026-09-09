use super::*;
use amigo_panels::PresetProvider;
use std::sync::Arc;

/// Appearance-only persistence; transforms, camera, light and paper are scene-owned.
pub struct LookPresetProvider(pub Arc<NprPlaygroundState>);

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct LookPresetDocument {
    style: ComicInk,
    layers: NprStyleLayers,
}

impl PresetProvider for LookPresetProvider {
    fn id(&self) -> &'static str {
        "npr-look"
    }
    fn snapshot(&self) -> Result<serde_yaml::Value, String> {
        let settings = self.0.snapshot();
        let style = if settings.style_scope == "object" {
            settings.objects[&settings.selected].effective_style(settings.global)
        } else {
            settings.global
        };
        let mut style = serde_yaml::to_value(style).map_err(|e| e.to_string())?;
        let mapping = style.as_mapping_mut().unwrap();
        mapping.remove(serde_yaml::Value::String("paper".into()));
        mapping.remove(serde_yaml::Value::String("light_direction".into()));
        Ok(serde_yaml::to_value(LookPresetDocument {
            style: serde_yaml::from_value(style).map_err(|e| e.to_string())?,
            layers: settings.objects[&settings.selected].effective_layers(&settings.style_layers),
        })
        .map_err(|e| e.to_string())?)
    }
    fn apply(&self, value: serde_yaml::Value) -> Result<(), String> {
        if *self.0.preview_before.lock().unwrap() {
            return Err("disable Before comparison to load a look".into());
        }
        let document: LookPresetDocument =
            serde_yaml::from_value(value).map_err(|e| e.to_string())?;
        let mut style = document.style;
        validate_style(style)?;
        document.layers.validate()?;
        let mut settings = self.0.settings.lock().unwrap();
        let before = settings.clone();
        style.paper = settings.global.paper;
        style.light_direction = settings.global.light_direction;
        let selected = settings.selected.clone();
        if settings.style_scope == "object" {
            let layer_overrides =
                NprStyleLayerOverrides::from_resolved(&settings.style_layers, &document.layers)?;
            let object = settings.objects.get_mut(&selected).unwrap();
            object.style_overrides = ComicInkOverrides::detached(style);
            object.style_layer_overrides = layer_overrides;
        } else {
            settings.global = style;
            settings.style_layers = document.layers;
        }
        self.0
            .history
            .lock()
            .unwrap()
            .record("load_look", &before, &settings);
        Ok(())
    }
    fn reset(&self) -> Result<(), String> {
        self.0.action("reset_style")
    }
}
