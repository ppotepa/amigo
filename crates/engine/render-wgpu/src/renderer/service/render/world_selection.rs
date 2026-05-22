use super::world_filters::{
    LayeredImagePartFilter, WorldLayerFilter, WorldObjectFilter, WorldPassLoad,
};
use super::*;

pub(super) trait WorldPassLoadExt {
    fn to_load_op(self) -> wgpu::LoadOp<wgpu::Color>;
}

impl WorldPassLoadExt for WorldPassLoad {
    fn to_load_op(self) -> wgpu::LoadOp<wgpu::Color> {
        match self {
            WorldPassLoad::Clear => wgpu::LoadOp::Clear(wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }),
            WorldPassLoad::ClearTransparent => wgpu::LoadOp::Clear(wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            }),
            WorldPassLoad::Load => wgpu::LoadOp::Load,
        }
    }
}

pub(super) enum OwnedWorldLayerFilter {
    All,
    Include {
        layers: BTreeSet<String>,
        include_layerless: bool,
    },
    Exclude(BTreeSet<String>),
}

pub(super) enum OwnedWorldObjectFilter {
    All,
    Include(BTreeSet<String>),
    IncludeSubtrees(BTreeSet<String>),
    Exclude(BTreeSet<String>),
    ExcludeSubtrees(BTreeSet<String>),
    ExcludeCombined {
        objects: BTreeSet<String>,
        subtrees: BTreeSet<String>,
    },
}

pub(super) enum OwnedLayeredImagePartFilter {
    All,
    Exclude(BTreeMap<String, BTreeSet<String>>),
}

pub(super) struct WorldRenderSelection<'a> {
    pub layer_filter: WorldLayerFilter<'a>,
    pub object_filter: WorldObjectFilter<'a>,
    pub layered_image_part_filter: LayeredImagePartFilter<'a>,
    pub pass_load: WorldPassLoad,
}

pub(super) struct OwnedWorldRenderSelection {
    pub layer_filter: OwnedWorldLayerFilter,
    pub object_filter: OwnedWorldObjectFilter,
    pub layered_image_part_filter: OwnedLayeredImagePartFilter,
    pub pass_load: WorldPassLoad,
}

impl OwnedWorldRenderSelection {
    pub(super) fn all(pass_load: WorldPassLoad) -> Self {
        Self {
            layer_filter: OwnedWorldLayerFilter::All,
            object_filter: OwnedWorldObjectFilter::All,
            layered_image_part_filter: OwnedLayeredImagePartFilter::All,
            pass_load,
        }
    }

    pub(super) fn include_layers(
        layers: BTreeSet<String>,
        include_layerless: bool,
        pass_load: WorldPassLoad,
    ) -> Self {
        Self {
            layer_filter: OwnedWorldLayerFilter::Include {
                layers,
                include_layerless,
            },
            ..Self::all(pass_load)
        }
    }

    pub(super) fn exclude_layers(layers: BTreeSet<String>, pass_load: WorldPassLoad) -> Self {
        Self {
            layer_filter: OwnedWorldLayerFilter::Exclude(layers),
            ..Self::all(pass_load)
        }
    }

    pub(super) fn with_excluded_layers(mut self, excluded_layers: &BTreeSet<String>) -> Self {
        if excluded_layers.is_empty() {
            return self;
        }
        self.layer_filter = match self.layer_filter {
            OwnedWorldLayerFilter::All => OwnedWorldLayerFilter::Exclude(excluded_layers.clone()),
            OwnedWorldLayerFilter::Exclude(mut excluded) => {
                excluded.extend(excluded_layers.iter().cloned());
                OwnedWorldLayerFilter::Exclude(excluded)
            }
            OwnedWorldLayerFilter::Include {
                mut layers,
                include_layerless,
            } => {
                layers.retain(|layer| !excluded_layers.contains(layer));
                OwnedWorldLayerFilter::Include {
                    layers,
                    include_layerless,
                }
            }
        };
        self
    }

    pub(super) fn draw_layer(draw_layer_id: &str, pass_load: WorldPassLoad) -> Self {
        let mut layers = BTreeSet::new();
        layers.insert(draw_layer_id.to_owned());
        Self::include_layers(layers, false, pass_load)
    }

    pub(super) fn scene_object(scene_object_id: &str, pass_load: WorldPassLoad) -> Self {
        let mut objects = BTreeSet::new();
        objects.insert(scene_object_id.to_owned());
        Self {
            object_filter: OwnedWorldObjectFilter::Include(objects),
            ..Self::all(pass_load)
        }
    }

    pub(super) fn group_subtree(root_scene_object_id: &str, pass_load: WorldPassLoad) -> Self {
        let mut roots = BTreeSet::new();
        roots.insert(root_scene_object_id.to_owned());
        Self {
            object_filter: OwnedWorldObjectFilter::IncludeSubtrees(roots),
            ..Self::all(pass_load)
        }
    }

    pub(super) fn borrowed(&self) -> WorldRenderSelection<'_> {
        let layer_filter = match &self.layer_filter {
            OwnedWorldLayerFilter::All => WorldLayerFilter::All,
            OwnedWorldLayerFilter::Include {
                layers,
                include_layerless,
            } => WorldLayerFilter::Include {
                layers,
                include_layerless: *include_layerless,
            },
            OwnedWorldLayerFilter::Exclude(layers) => WorldLayerFilter::Exclude(layers),
        };
        let object_filter = match &self.object_filter {
            OwnedWorldObjectFilter::All => WorldObjectFilter::All,
            OwnedWorldObjectFilter::Include(objects) => WorldObjectFilter::Include(objects),
            OwnedWorldObjectFilter::IncludeSubtrees(roots) => {
                WorldObjectFilter::IncludeSubtrees(roots)
            }
            OwnedWorldObjectFilter::Exclude(objects) => WorldObjectFilter::Exclude(objects),
            OwnedWorldObjectFilter::ExcludeSubtrees(roots) => {
                WorldObjectFilter::ExcludeSubtrees(roots)
            }
            OwnedWorldObjectFilter::ExcludeCombined { objects, subtrees } => {
                WorldObjectFilter::ExcludeCombined { objects, subtrees }
            }
        };
        let layered_image_part_filter = match &self.layered_image_part_filter {
            OwnedLayeredImagePartFilter::All => LayeredImagePartFilter::All,
            OwnedLayeredImagePartFilter::Exclude(parts) => LayeredImagePartFilter::Exclude(parts),
        };
        WorldRenderSelection {
            layer_filter,
            object_filter,
            layered_image_part_filter,
            pass_load: self.pass_load,
        }
    }
}

pub(super) fn base_world_selection(
    post_fx_stacks: &[amigo_render_api::ScopedPostFx2dStack],
    render_layers: &[RenderLayer2dCommand],
) -> OwnedWorldRenderSelection {
    let plan = super::focus_depth_plan::focus_blur_layer_plan(post_fx_stacks, render_layers, None);
    let draw_layer_post_fx_exclusions = draw_layer_post_fx_layers(post_fx_stacks);
    let scene_object_post_fx_exclusions = scene_object_post_fx_objects(post_fx_stacks);
    let scene_group_post_fx_exclusions = scene_group_post_fx_roots(post_fx_stacks);
    let image_part_post_fx_exclusions = image_part_post_fx_targets(post_fx_stacks);

    let layer_filter = if let Some(plan) = plan.as_ref() {
        if plan.has_explicit_render_depth {
            let mut layers = plan.depth_map_layers.clone();
            layers.retain(|layer| !draw_layer_post_fx_exclusions.contains(layer));
            OwnedWorldLayerFilter::Include {
                layers,
                include_layerless: true,
            }
        } else if let Some(layers) = plan.implicit_affected_layers.as_ref() {
            let mut included = layers.clone();
            included.retain(|layer| !draw_layer_post_fx_exclusions.contains(layer));
            OwnedWorldLayerFilter::Include {
                layers: included,
                include_layerless: false,
            }
        } else if draw_layer_post_fx_exclusions.is_empty() {
            OwnedWorldLayerFilter::All
        } else {
            OwnedWorldLayerFilter::Exclude(draw_layer_post_fx_exclusions.clone())
        }
    } else if draw_layer_post_fx_exclusions.is_empty() {
        OwnedWorldLayerFilter::All
    } else {
        OwnedWorldLayerFilter::Exclude(draw_layer_post_fx_exclusions)
    };

    let object_filter = if scene_object_post_fx_exclusions.is_empty()
        && scene_group_post_fx_exclusions.is_empty()
    {
        OwnedWorldObjectFilter::All
    } else if scene_group_post_fx_exclusions.is_empty() {
        OwnedWorldObjectFilter::Exclude(scene_object_post_fx_exclusions)
    } else if scene_object_post_fx_exclusions.is_empty() {
        OwnedWorldObjectFilter::ExcludeSubtrees(scene_group_post_fx_exclusions)
    } else {
        OwnedWorldObjectFilter::ExcludeCombined {
            objects: scene_object_post_fx_exclusions,
            subtrees: scene_group_post_fx_exclusions,
        }
    };

    let layered_image_part_filter = if image_part_post_fx_exclusions.is_empty() {
        OwnedLayeredImagePartFilter::All
    } else {
        OwnedLayeredImagePartFilter::Exclude(image_part_post_fx_exclusions)
    };

    OwnedWorldRenderSelection {
        layer_filter,
        object_filter,
        layered_image_part_filter,
        pass_load: WorldPassLoad::Clear,
    }
}

pub(super) fn draw_layer_post_fx_layers(
    stacks: &[amigo_render_api::ScopedPostFx2dStack],
) -> BTreeSet<String> {
    stacks
        .iter()
        .filter_map(|stack| {
            if !matches!(
                stack.pipeline,
                amigo_render_api::PostFxPipelineKind::OffscreenDrawLayer
            ) {
                return None;
            }
            let amigo_render_api::PostFxScope2d::DrawLayer { draw_layer_id } = &stack.scope else {
                return None;
            };
            if stack.effects.iter().any(|effect| effect.effect.is_active()) {
                Some(draw_layer_id.clone())
            } else {
                None
            }
        })
        .collect()
}

pub(super) fn scene_object_post_fx_objects(
    stacks: &[amigo_render_api::ScopedPostFx2dStack],
) -> BTreeSet<String> {
    stacks
        .iter()
        .filter_map(|stack| {
            if !matches!(
                stack.pipeline,
                amigo_render_api::PostFxPipelineKind::OffscreenObject
            ) {
                return None;
            }
            let amigo_render_api::PostFxScope2d::SceneObjectPixels { scene_object_id } =
                &stack.scope
            else {
                return None;
            };
            if stack.effects.iter().any(|effect| effect.effect.is_active()) {
                Some(scene_object_id.clone())
            } else {
                None
            }
        })
        .collect()
}

pub(super) fn scene_group_post_fx_roots(
    stacks: &[amigo_render_api::ScopedPostFx2dStack],
) -> BTreeSet<String> {
    stacks
        .iter()
        .filter_map(|stack| {
            if !matches!(
                stack.pipeline,
                amigo_render_api::PostFxPipelineKind::OffscreenGroup
            ) {
                return None;
            }
            let amigo_render_api::PostFxScope2d::GroupSubtree {
                root_scene_object_id,
            } = &stack.scope
            else {
                return None;
            };
            if stack.effects.iter().any(|effect| effect.effect.is_active()) {
                Some(root_scene_object_id.clone())
            } else {
                None
            }
        })
        .collect()
}

pub(super) fn image_part_post_fx_targets(
    stacks: &[amigo_render_api::ScopedPostFx2dStack],
) -> BTreeMap<String, BTreeSet<String>> {
    let mut targets = BTreeMap::new();
    for stack in stacks {
        if !matches!(
            stack.pipeline,
            amigo_render_api::PostFxPipelineKind::CachedImage
        ) {
            continue;
        }
        let amigo_render_api::PostFxScope2d::ImagePart {
            owner_scene_object_id,
            part_id,
            ..
        } = &stack.scope
        else {
            continue;
        };
        if !stack.effects.iter().any(|effect| effect.effect.is_active()) {
            continue;
        }
        targets
            .entry(owner_scene_object_id.clone())
            .or_insert_with(BTreeSet::new)
            .insert(part_id.clone());
    }
    targets
}
