use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelKind {
    Obj,
    #[allow(dead_code)]
    Fbx,
    AnimClip,
}

#[derive(Clone, Debug)]
pub struct BuiltInModel {
    pub id: &'static str,
    pub label: &'static str,
    pub kind: ModelKind,
    pub path: &'static str,
}

pub const BUILT_INS: &[BuiltInModel] = &[
    BuiltInModel {
        id: "suzanne",
        label: "embedded Suzanne OBJ",
        kind: ModelKind::Obj,
        path: "assets/models/suzanne.obj",
    },
    BuiltInModel {
        id: "goku",
        label: "Goku.obj",
        kind: ModelKind::Obj,
        path: "assets/models/Goku.obj",
    },
    BuiltInModel {
        id: "new_york",
        label: "new_york.obj",
        kind: ModelKind::Obj,
        path: "assets/models/new_york.obj",
    },
    BuiltInModel {
        id: "walking",
        label: "walking.fbx baked clip",
        kind: ModelKind::AnimClip,
        path: "assets/models/walking.amc",
    },
];

pub fn built_in(id: &str) -> &'static BuiltInModel {
    BUILT_INS
        .iter()
        .find(|model| model.id == id)
        .unwrap_or(&BUILT_INS[0])
}

pub fn project_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}
