pub(crate) fn npr_cpu_line_selection_profile(
    settings: &amigo_render_api::NprLineSettings3d,
) -> amigo_render_api::NprLineSelectionProfile3d {
    settings.cpu_strategy_profile.line_selection
}

pub(crate) fn npr_cpu_path_joining_profile(
    settings: &amigo_render_api::NprLineSettings3d,
) -> amigo_render_api::NprPathJoiningProfile3d {
    settings.cpu_strategy_profile.path_joining
}

pub(crate) fn npr_cpu_break_policy_profile(
    settings: &amigo_render_api::NprLineSettings3d,
) -> amigo_render_api::NprBreakPolicyProfile3d {
    settings.cpu_strategy_profile.break_policy
}

pub(crate) fn npr_cpu_stroke_synthesis_profile(
    settings: &amigo_render_api::NprLineSettings3d,
) -> amigo_render_api::NprStrokeSynthesisProfile3d {
    settings.cpu_strategy_profile.stroke_synthesis
}

pub(crate) fn npr_cpu_tessellation_profile(
    settings: &amigo_render_api::NprLineSettings3d,
) -> amigo_render_api::NprTessellationProfile3d {
    settings.cpu_strategy_profile.tessellation
}
