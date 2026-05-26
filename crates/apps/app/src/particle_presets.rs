use amigo_core::AmigoResult;
use amigo_runtime::Runtime;

pub(crate) fn load_particle_preset_catalog(runtime: &Runtime) -> AmigoResult<()> {
    amigo_runtime_bundles::load_particle_preset_catalog(runtime)
}
