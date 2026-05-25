use std::fs;

use amigo_core::AmigoResult;
use amigo_modding::ModCatalog;
use amigo_runtime_bundles::ParticlePreset2dService;
use amigo_runtime::Runtime;
use amigo_runtime_bundles::load_particle_preset_file;

use crate::runtime_context::required;

pub(crate) fn load_particle_preset_catalog(runtime: &Runtime) -> AmigoResult<()> {
    let mod_catalog = required::<ModCatalog>(runtime)?;
    let presets = required::<ParticlePreset2dService>(runtime)?;
    presets.clear();

    for discovered_mod in mod_catalog.mods() {
        let preset_dir = discovered_mod.root_path.join("presets");
        if !preset_dir.is_dir() {
            continue;
        }

        for entry in fs::read_dir(&preset_dir)? {
            let path = entry?.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("yml") {
                continue;
            }
            let preset = load_particle_preset_file(&discovered_mod.manifest.id, &path)?;
            presets.register(preset);
        }
    }

    Ok(())
}
