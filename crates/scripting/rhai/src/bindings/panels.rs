use super::WorldApi;
use amigo_panels::{PanelService, PresetService};
use std::sync::Arc;
type Result<T> = std::result::Result<T, Box<rhai::EvalAltResult>>;
#[derive(Clone)]
struct Panels(Option<Arc<PanelService>>);
#[derive(Clone)]
struct Presets(Option<Arc<PresetService>>);
impl Panels {
    fn open(&mut self, id: &str) -> Result<()> {
        self.0
            .as_ref()
            .ok_or("panel service unavailable")?
            .open(id)
            .map_err(Into::into)
    }
    fn close(&mut self, id: &str) -> Result<()> {
        self.0
            .as_ref()
            .ok_or("panel service unavailable")?
            .close(id)
            .map_err(Into::into)
    }
}
impl Presets {
    fn action(
        &self,
        action: impl FnOnce(&PresetService) -> std::result::Result<(), String>,
    ) -> Result<()> {
        let service = self.0.as_ref().ok_or("preset service unavailable")?;
        action(service).map_err(|error| {
            service.report_error(error.clone());
            error.into()
        })
    }
    fn save(&mut self, domain: &str, name: &str, overwrite: bool) -> Result<()> {
        self.action(|p| p.save(domain, name, overwrite))
    }
    fn load(&mut self, domain: &str, name: &str) -> Result<()> {
        self.action(|p| p.load(domain, name))
    }
    fn reset(&mut self, domain: &str) -> Result<()> {
        self.action(|p| p.reset(domain))
    }
    fn list(&mut self) -> rhai::Array {
        self.0
            .as_ref()
            .map(|p| p.list().into_iter().map(Into::into).collect())
            .unwrap_or_default()
    }
}
pub(crate) fn register(
    engine: &mut rhai::Engine,
    panels: Option<Arc<PanelService>>,
    presets: Option<Arc<PresetService>>,
) {
    engine
        .register_type_with_name::<Panels>("ScenePanels")
        .register_get("panels", move |_: &mut WorldApi| Panels(panels.clone()))
        .register_fn("open", Panels::open)
        .register_fn("close", Panels::close);
    engine
        .register_type_with_name::<Presets>("ScenePresets")
        .register_get("presets", move |_: &mut WorldApi| Presets(presets.clone()))
        .register_fn("save", Presets::save)
        .register_fn("load", Presets::load)
        .register_fn("reset", Presets::reset)
        .register_fn("list", Presets::list);
}
