use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy)]
pub(super) enum WorldLayerFilter<'a> {
    All,
    Include {
        layers: &'a BTreeSet<String>,
        include_layerless: bool,
    },
    Exclude(&'a BTreeSet<String>),
}

impl WorldLayerFilter<'_> {
    pub(super) fn allows(self, render_layer: &str) -> bool {
        match self {
            Self::All => true,
            Self::Include { layers, .. } => layers.contains(render_layer),
            Self::Exclude(layers) => !layers.contains(render_layer),
        }
    }

    pub(super) fn allows_layerless(self) -> bool {
        match self {
            Self::All | Self::Exclude(_) => true,
            Self::Include {
                include_layerless, ..
            } => include_layerless,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum WorldObjectFilter<'a> {
    All,
    Include(&'a BTreeSet<String>),
    IncludeSubtrees(&'a BTreeSet<String>),
    Exclude(&'a BTreeSet<String>),
    ExcludeSubtrees(&'a BTreeSet<String>),
    ExcludeCombined {
        objects: &'a BTreeSet<String>,
        subtrees: &'a BTreeSet<String>,
    },
}

impl WorldObjectFilter<'_> {
    pub(super) fn allows(self, entity_name: &str) -> bool {
        match self {
            Self::All => true,
            Self::Include(objects) => objects.contains(entity_name),
            Self::IncludeSubtrees(roots) => roots
                .iter()
                .any(|root| entity_matches_subtree(entity_name, root)),
            Self::Exclude(objects) => !objects.contains(entity_name),
            Self::ExcludeSubtrees(roots) => !roots
                .iter()
                .any(|root| entity_matches_subtree(entity_name, root)),
            Self::ExcludeCombined { objects, subtrees } => {
                !objects.contains(entity_name)
                    && !subtrees
                        .iter()
                        .any(|root| entity_matches_subtree(entity_name, root))
            }
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum LayeredImagePartFilter<'a> {
    All,
    Exclude(&'a BTreeMap<String, BTreeSet<String>>),
}

impl<'a> LayeredImagePartFilter<'a> {
    pub(super) fn included_parts(
        &self,
        owner_scene_object_id: &str,
    ) -> Option<&'a BTreeSet<String>> {
        let _ = owner_scene_object_id;
        None
    }

    pub(super) fn excluded_parts(
        &self,
        owner_scene_object_id: &str,
    ) -> Option<&'a BTreeSet<String>> {
        match self {
            Self::Exclude(parts) => parts.get(owner_scene_object_id),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum WorldPassLoad {
    Clear,
    ClearTransparent,
    Load,
}

pub(super) fn entity_matches_subtree(entity_name: &str, root_scene_object_id: &str) -> bool {
    entity_name == root_scene_object_id || entity_name.starts_with(root_scene_object_id)
}
