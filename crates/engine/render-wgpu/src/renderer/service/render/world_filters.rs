use std::collections::{BTreeMap, BTreeSet};

use amigo_render_api::RenderObjectId;

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
    Include(&'a BTreeSet<RenderObjectId>),
    IncludeSubtrees(&'a BTreeSet<RenderObjectId>),
    Exclude(&'a BTreeSet<RenderObjectId>),
    ExcludeSubtrees(&'a BTreeSet<RenderObjectId>),
    ExcludeCombined {
        objects: &'a BTreeSet<RenderObjectId>,
        subtrees: &'a BTreeSet<RenderObjectId>,
    },
}

impl WorldObjectFilter<'_> {
    pub(super) fn allows(self, object_id: &RenderObjectId) -> bool {
        match self {
            Self::All => true,
            Self::Include(objects) => objects.contains(object_id),
            Self::IncludeSubtrees(roots) => {
                roots.iter().any(|root| object_id.matches_subtree(root))
            }
            Self::Exclude(objects) => !objects.contains(object_id),
            Self::ExcludeSubtrees(roots) => {
                !roots.iter().any(|root| object_id.matches_subtree(root))
            }
            Self::ExcludeCombined { objects, subtrees } => {
                !objects.contains(object_id)
                    && !subtrees.iter().any(|root| object_id.matches_subtree(root))
            }
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum LayeredImagePartFilter<'a> {
    All,
    Exclude(&'a BTreeMap<RenderObjectId, BTreeSet<String>>),
}

impl<'a> LayeredImagePartFilter<'a> {
    pub(super) fn included_parts(
        &self,
        object_id: &RenderObjectId,
    ) -> Option<&'a BTreeSet<String>> {
        let _ = object_id;
        None
    }

    pub(super) fn excluded_parts(
        &self,
        object_id: &RenderObjectId,
    ) -> Option<&'a BTreeSet<String>> {
        match self {
            Self::Exclude(parts) => parts.get(object_id),
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
