use amigo_3d_mesh::{MeshGeometryAsset, load_gltf_geometry, load_gltf_geometry_source_space};
use base64::Engine;
use serde_json::{Value, json};

#[derive(Clone)]
struct Fixture {
    document: Value,
    bytes: Vec<u8>,
}
impl Fixture {
    fn new() -> Self {
        let mut f = Self {
            document: json!({"asset":{"version":"2.0"},"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"name":"Model","mesh":0}],"meshes":[{"primitives":[{"attributes":{"POSITION":0}}]}],"buffers":[],"bufferViews":[],"accessors":[],"animations":[]}),
            bytes: vec![],
        };
        f.floats("VEC3", &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
        f
    }
    fn floats(&mut self, kind: &str, values: &[f32]) -> usize {
        let dimension = match kind {
            "VEC3" => 3,
            "VEC4" => 4,
            "MAT4" => 16,
            _ => 1,
        };
        let accessor = self.document["accessors"].as_array().unwrap().len();
        let view = self.document["bufferViews"].as_array().unwrap().len();
        let offset = self.bytes.len();
        self.bytes
            .extend(values.iter().flat_map(|v| v.to_le_bytes()));
        self.document["bufferViews"]
            .as_array_mut()
            .unwrap()
            .push(json!({"buffer":0,"byteOffset":offset,"byteLength":values.len()*4}));
        let mut entry = json!({"bufferView":view,"componentType":5126,"count":values.len()/dimension,"type":kind});
        if dimension == 3 || dimension == 1 {
            entry["min"] = json!(
                (0..dimension)
                    .map(|axis| values
                        .chunks_exact(dimension)
                        .map(|v| v[axis])
                        .fold(f32::INFINITY, f32::min))
                    .collect::<Vec<_>>()
            );
            entry["max"] = json!(
                (0..dimension)
                    .map(|axis| values
                        .chunks_exact(dimension)
                        .map(|v| v[axis])
                        .fold(f32::NEG_INFINITY, f32::max))
                    .collect::<Vec<_>>()
            );
        }
        self.document["accessors"]
            .as_array_mut()
            .unwrap()
            .push(entry);
        accessor
    }
    fn joints(&mut self, values: &[u16]) -> usize {
        self.unsigned_shorts("VEC4", values, false)
    }
    fn unsigned_shorts(&mut self, kind: &str, values: &[u16], normalized: bool) -> usize {
        let dimension = if kind == "VEC4" { 4 } else { 1 };
        let accessor = self.document["accessors"].as_array().unwrap().len();
        let view = self.document["bufferViews"].as_array().unwrap().len();
        let offset = self.bytes.len();
        self.bytes
            .extend(values.iter().flat_map(|v| v.to_le_bytes()));
        while self.bytes.len() % 4 != 0 {
            self.bytes.push(0);
        }
        self.document["bufferViews"]
            .as_array_mut()
            .unwrap()
            .push(json!({"buffer":0,"byteOffset":offset,"byteLength":values.len()*2}));
        self.document["accessors"].as_array_mut().unwrap().push(
            json!({"bufferView":view,"componentType":5123,"count":values.len()/dimension,"type":kind,"normalized":normalized}),
        );
        accessor
    }
    fn channel(
        &mut self,
        node: usize,
        path: &str,
        interpolation: &str,
        times: &[f32],
        values: &[f32],
    ) {
        let input = self.floats("SCALAR", times);
        let output = self.floats(
            match path {
                "rotation" => "VEC4",
                "weights" => "SCALAR",
                _ => "VEC3",
            },
            values,
        );
        if self.document["animations"].as_array().unwrap().is_empty() {
            self.document["animations"] = json!([{"name":"Motion","samplers":[],"channels":[]}]);
        }
        let clip = &mut self.document["animations"][0];
        let sampler = clip["samplers"].as_array().unwrap().len();
        clip["samplers"]
            .as_array_mut()
            .unwrap()
            .push(json!({"input":input,"output":output,"interpolation":interpolation}));
        clip["channels"]
            .as_array_mut()
            .unwrap()
            .push(json!({"sampler":sampler,"target":{"node":node,"path":path}}));
    }
    fn load(self, glb: bool) -> Result<MeshGeometryAsset, String> {
        self.load_with_space(glb, false)
    }

    fn load_source_space(self, glb: bool) -> Result<MeshGeometryAsset, String> {
        self.load_with_space(glb, true)
    }

    fn load_with_space(
        mut self,
        glb: bool,
        source_space: bool,
    ) -> Result<MeshGeometryAsset, String> {
        self.document["buffers"] = json!([{"byteLength":self.bytes.len()}]);
        let dir = tempfile::tempdir().unwrap();
        let path = dir
            .path()
            .join(if glb { "model.glb" } else { "model.gltf" });
        if glb {
            let mut json = serde_json::to_vec(&self.document).unwrap();
            while json.len() % 4 != 0 {
                json.push(b' ');
            }
            while self.bytes.len() % 4 != 0 {
                self.bytes.push(0);
            }
            let mut data = Vec::new();
            for word in [
                0x46546c67u32,
                2,
                (28 + json.len() + self.bytes.len()) as u32,
                json.len() as u32,
                0x4e4f534a,
            ] {
                data.extend(word.to_le_bytes());
            }
            data.extend(json);
            data.extend((self.bytes.len() as u32).to_le_bytes());
            data.extend(0x004e4942u32.to_le_bytes());
            data.extend(self.bytes);
            std::fs::write(&path, data).unwrap();
        } else {
            self.document["buffers"][0]["uri"] = json!(format!(
                "data:application/gltf-buffer;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(self.bytes)
            ));
            std::fs::write(&path, serde_json::to_vec(&self.document).unwrap()).unwrap();
        }
        if source_space {
            load_gltf_geometry_source_space(&path)
        } else {
            load_gltf_geometry(&path)
        }
    }
}
fn close(a: f32, b: f32) {
    assert!((a - b).abs() < 1e-5, "{a} != {b}");
}

#[test]
fn gltf_and_glb_preserve_hierarchy_clip_metadata_and_fixed_normalization() {
    for glb in [false, true] {
        let mut f = Fixture::new();
        f.document["nodes"] = json!([{"name":"Rig","translation":[10,0,0],"children":[1]},{"name":"Mesh","mesh":0,"translation":[1,0,0]}]);
        f.channel(
            0,
            "translation",
            "LINEAR",
            &[0.0, 2.0],
            &[10.0, 0.0, 0.0, 12.0, 0.0, 0.0],
        );
        let asset = f.load(glb).unwrap();
        let clip = &asset.animations()[0];
        assert_eq!(clip.name, "Motion");
        assert_eq!(clip.tracks[0].node_path, "Rig");
        close(clip.duration_seconds, 2.0);
        let pose = asset.sample_animation(0, 1.0).unwrap();
        assert_eq!(pose.indices, asset.indices);
        close(pose.positions[0][0] - asset.positions[0][0], 2.0);
        close(
            asset.sample_animation(0, 9.0).unwrap().positions[0][0] - asset.positions[0][0],
            4.0,
        );
        let metadata = serde_json::to_value(clip).unwrap();
        assert!(metadata["tracks"][0].get("times").is_none());
        assert!(asset.sample_animation(9, 0.0).is_err());
        assert!(asset.sample_animation(0, f32::NAN).is_err());
    }
}

#[test]
fn source_space_import_preserves_authored_geometry_scale() {
    let mut f = Fixture::new();
    f.document["nodes"][0]["translation"] = json!([10.0, 0.0, 0.0]);
    let source = f.load_source_space(true).unwrap();
    let min_x = source
        .positions
        .iter()
        .map(|position| position[0])
        .fold(f32::INFINITY, f32::min);
    let max_x = source
        .positions
        .iter()
        .map(|position| position[0])
        .fold(f32::NEG_INFINITY, f32::max);
    assert!((min_x - 10.0).abs() < 1e-5);
    assert!((max_x - 11.0).abs() < 1e-5);
}

#[test]
fn morph_weights_override_mesh_defaults_and_preserve_collapsed_topology() {
    let mut f = Fixture::new();
    let positions = f.floats("VEC3", &[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    let deltas = f.floats("VEC3", &[0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0]);
    f.document["meshes"][0] = json!({"weights":[0.8],"primitives":[{"attributes":{"POSITION":positions},"targets":[{"POSITION":deltas}]}]});
    f.document["nodes"][0]["weights"] = json!([0.0]);
    f.channel(0, "weights", "LINEAR", &[0.0, 1.0], &[0.0, 1.0]);
    let asset = f.load(false).unwrap();
    assert_eq!(asset.positions.len(), 3);
    assert_eq!(asset.indices.len(), 3);
    assert_eq!(asset.dropped_degenerate_triangles, 0);
    close(asset.positions[1][1], 0.0);
    let pose = asset.sample_animation(0, 0.5).unwrap();
    close(pose.positions[1][1], 1.0);
    assert_eq!(pose.indices, asset.indices);
}

#[test]
fn skin_uses_all_influence_sets_inverse_bind_and_morph_before_joints() {
    let mut f = Fixture::new();
    f.document["nodes"] = json!([{"name":"Rig","children":[1,2,3]},{"name":"Moving joint","translation":[3,0,0]},{"name":"Fixed joint","translation":[6,0,0]},{"name":"Mesh","mesh":0,"skin":0,"translation":[100,0,0]}]);
    let bind = f.floats(
        "MAT4",
        &[
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -3.0, 0.0, 0.0, 1.0, 1.0,
            0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -6.0, 0.0, 0.0, 1.0,
        ],
    );
    f.document["skins"] = json!([{"joints":[1,2],"inverseBindMatrices":bind}]);
    let j0 = f.joints(&[0; 12]);
    let j1 = f.joints(&[1; 12]);
    let w0 = f.floats("VEC4", &[0.25, 0.0, 0.0, 0.0].repeat(3));
    let w1 = f.floats("VEC4", &[0.75, 0.0, 0.0, 0.0].repeat(3));
    let deltas = f.floats("VEC3", &[0.0, 1.0, 0.0].repeat(3));
    f.document["meshes"][0]["primitives"][0] = json!({"attributes":{"POSITION":0,"JOINTS_0":j0,"WEIGHTS_0":w0,"JOINTS_1":j1,"WEIGHTS_1":w1},"targets":[{"POSITION":deltas}]});
    f.channel(
        1,
        "translation",
        "LINEAR",
        &[0.0, 1.0],
        &[3.0, 0.0, 0.0, 5.0, 0.0, 0.0],
    );
    f.channel(3, "weights", "LINEAR", &[0.0, 1.0], &[0.0, 1.0]);
    let asset = f.load(true).unwrap();
    assert_eq!(
        asset.animations()[0].tracks[0].node_path,
        "Rig/Moving joint"
    );
    let pose = asset.sample_animation(0, 0.5).unwrap();
    close(pose.positions[0][0] - asset.positions[0][0], 0.5);
    close(pose.positions[0][1] - asset.positions[0][1], 1.0);
    close(asset.positions[0][0], -1.0);
}

#[test]
fn file_step_rotation_and_cubic_translation_are_sampled() {
    let mut f = Fixture::new();
    f.channel(
        0,
        "rotation",
        "STEP",
        &[0.0, 1.0],
        &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0],
    );
    f.channel(
        0,
        "translation",
        "CUBICSPLINE",
        &[0.0, 2.0],
        &[
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0,
        ],
    );
    let asset = f.load(false).unwrap();
    let before = asset.sample_animation(0, 0.5).unwrap();
    close(before.positions[1][0] - before.positions[0][0], 2.0);
    let middle = asset.sample_animation(0, 1.0).unwrap();
    close(middle.positions[0][0] - asset.positions[0][0], 1.0);
    close(middle.positions[1][0] - middle.positions[0][0], -2.0);
}

#[test]
fn invalid_accessors_channels_and_hierarchies_return_errors() {
    let mut f = Fixture::new();
    f.document["accessors"][0]["count"] = json!(usize::MAX / 2);
    assert!(f.load(false).is_err());
    let mut f = Fixture::new();
    f.document["accessors"][0]["type"] = json!("VEC4");
    assert!(f.load(false).is_err());
    let mut f = Fixture::new();
    f.document["nodes"][0]["children"] = json!([0]);
    assert!(f.load(false).unwrap_err().contains("cyclic"));
    let mut f = Fixture::new();
    f.channel(0, "translation", "LINEAR", &[1.0, 1.0], &[0.0; 6]);
    assert!(f.load(false).unwrap_err().contains("timestamps"));
    let mut f = Fixture::new();
    f.channel(0, "translation", "LINEAR", &[0.0, 1.0], &[0.0; 3]);
    assert!(f.load(false).unwrap_err().contains("counts"));
    let mut f = Fixture::new();
    f.channel(0, "translation", "LINEAR", &[0.0, 1.0], &[0.0; 6]);
    f.channel(0, "translation", "STEP", &[0.0, 1.0], &[0.0; 6]);
    assert!(f.load(false).unwrap_err().contains("duplicate"));
}

#[test]
fn normalized_integer_rotation_and_morph_channels_are_supported() {
    for property in ["rotation", "weights"] {
        let mut f = Fixture::new();
        if property == "weights" {
            let deltas = f.floats("VEC3", &[0.0, 1.0, 0.0].repeat(3));
            f.document["meshes"][0]["primitives"][0]["targets"] = json!([{"POSITION":deltas}]);
        }
        f.channel(
            0,
            property,
            "LINEAR",
            &[0.0, 1.0],
            if property == "rotation" {
                &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0]
            } else {
                &[0.0, 1.0]
            },
        );
        let output = if property == "rotation" {
            f.unsigned_shorts("VEC4", &[0, 0, 0, 65535, 0, 0, 65535, 0], true)
        } else {
            f.unsigned_shorts("SCALAR", &[0, 65535], true)
        };
        f.document["animations"][0]["samplers"][0]["output"] = json!(output);
        let asset = f.clone().load(false).unwrap();
        let pose = asset.sample_animation(0, 1.0).unwrap();
        if property == "rotation" {
            close(pose.positions[1][0], -3.0);
        } else {
            close(pose.positions[0][1] - asset.positions[0][1], 2.0);
        }
        f.document["accessors"][output]["normalized"] = json!(false);
        assert!(f.load(false).unwrap_err().contains("normalized"));
    }
}

#[test]
fn skin_accepts_extra_inverse_bind_elements_and_rejects_invalid_influences() {
    let mut f = Fixture::new();
    f.document["nodes"] = json!([{"mesh":0,"skin":0,"children":[1]},{}]);
    let bind = f.floats("MAT4", &glam::Mat4::IDENTITY.to_cols_array().repeat(2));
    f.document["skins"] = json!([{"joints":[1],"inverseBindMatrices":bind}]);
    let joints = f.joints(&[0; 12]);
    let weights = f.floats("VEC4", &[1.0, 0.0, 0.0, 0.0].repeat(3));
    f.document["meshes"][0]["primitives"][0]["attributes"] =
        json!({"POSITION":0,"JOINTS_0":joints,"WEIGHTS_0":weights});
    assert!(f.clone().load(true).is_ok());
    let mut bad = f.clone();
    let zero = bad.floats("VEC4", &[0.0; 12]);
    bad.document["meshes"][0]["primitives"][0]["attributes"]["WEIGHTS_0"] = json!(zero);
    assert!(bad.load(false).unwrap_err().contains("zero"));
    let invalid = f.joints(&[9; 12]);
    f.document["meshes"][0]["primitives"][0]["attributes"]["JOINTS_0"] = json!(invalid);
    assert!(f.load(false).unwrap_err().contains("influence"));
}
