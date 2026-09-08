use super::*;
use amigo_panels::PresetProvider;
use std::sync::Arc;

/// Appearance-only persistence; transforms, camera, light and paper are scene-owned.
pub struct LookPresetProvider(pub Arc<NprPlaygroundState>);
impl PresetProvider for LookPresetProvider {
    fn id(&self) -> &'static str {
        "npr-look"
    }
    fn snapshot(&self) -> Result<serde_yaml::Value, String> {
        let settings = self.0.snapshot();
        let style = if settings.style_scope == "Obiekt"
            && settings.objects[&settings.selected].override_style
        {
            settings.objects[&settings.selected].style
        } else {
            settings.global
        };
        let mut value = serde_yaml::to_value(style).map_err(|e| e.to_string())?;
        let mapping = value.as_mapping_mut().unwrap();
        mapping.remove(serde_yaml::Value::String("paper".into()));
        mapping.remove(serde_yaml::Value::String("light_direction".into()));
        Ok(value)
    }
    fn apply(&self, value: serde_yaml::Value) -> Result<(), String> {
        if *self.0.preview_before.lock().unwrap() {
            return Err("disable Before comparison to load a look".into());
        }
        let allowed = self.snapshot()?;
        let fields = value.as_mapping().ok_or("look preset must be a mapping")?;
        if fields.len() != allowed.as_mapping().unwrap().len()
            || fields
                .keys()
                .any(|k| !allowed.as_mapping().unwrap().contains_key(k))
        {
            return Err("invalid look preset fields".into());
        }
        let mut style: ComicInk = serde_yaml::from_value(value).map_err(|e| e.to_string())?;
        validate_style(style)?;
        let mut settings = self.0.settings.lock().unwrap();
        let before = settings.clone();
        style.paper = settings.global.paper;
        style.light_direction = settings.global.light_direction;
        let selected = settings.selected.clone();
        if settings.style_scope == "Obiekt" {
            let object = settings.objects.get_mut(&selected).unwrap();
            object.override_style = true;
            object.style = style;
        } else {
            settings.global = style;
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
