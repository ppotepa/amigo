use amigo_npr_playground_plugin::sources::*;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
};

fn triangle() -> (Vec<u8>, Vec<u8>) {
    let mut bytes = Vec::new();
    for value in [0f32, 0., 0., 1., 0., 0., 0., 1., 0.] {
        bytes.extend(value.to_le_bytes());
    }
    for index in [0u16, 1, 2] {
        bytes.extend(index.to_le_bytes());
    }
    let document = json!({"asset":{"version":"2.0"},"buffers":[{"uri":"mesh.bin","byteLength":42}],"bufferViews":[{"buffer":0,"byteOffset":0,"byteLength":36},{"buffer":0,"byteOffset":36,"byteLength":6}],"accessors":[{"bufferView":0,"componentType":5126,"count":3,"type":"VEC3","min":[0,0,0],"max":[1,1,0]},{"bufferView":1,"componentType":5123,"count":3,"type":"SCALAR"}],"meshes":[{"primitives":[{"attributes":{"POSITION":0},"indices":1}]}],"nodes":[{"mesh":0}],"scenes":[{"nodes":[0]}],"scene":0});
    (serde_json::to_vec(&document).unwrap(), bytes)
}

#[test]
fn complete_gltf_is_copied_and_path_escape_is_rejected_before_publication() {
    let source = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let (document, bytes) = triangle();
    fs::write(source.path().join("model.gltf"), &document).unwrap();
    fs::write(source.path().join("mesh.bin"), &bytes).unwrap();
    let imported = import_local(root.path(), &source.path().join("model.gltf")).unwrap();
    assert_eq!(
        fs::read(imported.path.parent().unwrap().join("mesh.bin")).unwrap(),
        bytes
    );
    let reopened = imported_models(root.path()).unwrap();
    assert_eq!(reopened.len(), 1);
    assert_eq!(reopened[0].id, imported.id);
    assert_eq!(reopened[0].path, imported.path);
    assert!(import_local(root.path(), &source.path().join("model.gltf")).is_err());
    let mut document: serde_json::Value = serde_json::from_slice(&document).unwrap();
    for uri in [
        "../mesh.bin",
        "%2e%2e/mesh.bin",
        "C:/mesh.bin",
        "https://example.org/mesh.bin",
        "sub\\mesh.bin",
    ] {
        document["buffers"][0]["uri"] = json!(uri);
        fs::write(
            source.path().join("bad.gltf"),
            serde_json::to_vec(&document).unwrap(),
        )
        .unwrap();
        assert!(
            import_local(root.path(), &source.path().join("bad.gltf")).is_err(),
            "{uri}"
        );
    }
    assert_eq!(
        fs::read_dir(root.path().join("assets/models"))
            .unwrap()
            .count(),
        1
    );
}

#[test]
fn binary_glb_import_is_validated() {
    let source = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let (document, mut bin) = triangle();
    let mut document: serde_json::Value = serde_json::from_slice(&document).unwrap();
    document["buffers"][0]
        .as_object_mut()
        .unwrap()
        .remove("uri");
    let mut json = serde_json::to_vec(&document).unwrap();
    while json.len() % 4 != 0 {
        json.push(b' ');
    }
    while bin.len() % 4 != 0 {
        bin.push(0);
    }
    let mut glb = b"glTF".to_vec();
    glb.extend(2u32.to_le_bytes());
    glb.extend(((28 + json.len() + bin.len()) as u32).to_le_bytes());
    glb.extend((json.len() as u32).to_le_bytes());
    glb.extend(b"JSON");
    glb.extend(json);
    glb.extend((bin.len() as u32).to_le_bytes());
    glb.extend(b"BIN\0");
    glb.extend(bin);
    let path = source.path().join("triangle.glb");
    fs::write(&path, &glb).unwrap();
    assert!(import_local(root.path(), &path).unwrap().path.exists());
    glb[8] = 0;
    fs::write(&path, glb).unwrap();
    assert!(import_local(root.path(), &path).is_err());
}

#[test]
fn discovered_manifests_cannot_alias_folders_or_builtins() {
    let root = tempfile::tempdir().unwrap();
    let models = root.path().join("assets/models");
    fs::create_dir_all(models.join("custom")).unwrap();
    fs::write(
        models.join("custom/npr-model.json"),
        br#"{"id":"other","entry":"model.gltf","byte_len":0}"#,
    )
    .unwrap();
    assert!(
        imported_models(root.path())
            .unwrap_err()
            .contains("does not match")
    );

    fs::remove_dir_all(models.join("custom")).unwrap();
    fs::create_dir_all(models.join("cube")).unwrap();
    fs::write(
        models.join("cube/npr-model.json"),
        br#"{"id":"cube","entry":"model.gltf","byte_len":0}"#,
    )
    .unwrap();
    assert!(
        imported_models(root.path())
            .unwrap_err()
            .contains("built-in")
    );
}

fn server(responses: Vec<Vec<u8>>) -> (NprCatalogEndpoint, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/catalog.json", listener.local_addr().unwrap());
    let thread = std::thread::spawn(move || {
        for body in responses {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 4096];
            let _ = stream.read(&mut request).unwrap();
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .unwrap();
            stream.write_all(&body).unwrap();
        }
    });
    (
        NprCatalogEndpoint {
            id: "fixture".into(),
            url,
        },
        thread,
    )
}

#[test]
fn remote_hash_failure_leaves_no_partial_asset_and_retry_succeeds() {
    let root = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let (document, bytes) = triangle();
    let model = NprRemoteModel {
        id: "triangle".into(),
        version: "1".into(),
        entry: "model.gltf".into(),
        files: vec![
            NprRemoteFile {
                path: "model.gltf".into(),
                size: document.len() as u64,
                sha256: format!("{:x}", Sha256::digest(&document)),
            },
            NprRemoteFile {
                path: "mesh.bin".into(),
                size: bytes.len() as u64,
                sha256: format!("{:x}", Sha256::digest(&bytes)),
            },
        ],
    };
    let (endpoint, thread) = server(vec![vec![0; document.len()]]);
    assert!(
        import_remote(root.path(), cache.path(), &endpoint, &model)
            .unwrap_err()
            .contains("hash")
    );
    thread.join().unwrap();
    assert!(!root.path().join("assets/models/triangle-1").exists());
    let (endpoint, thread) = server(vec![document, bytes]);
    assert!(
        import_remote(root.path(), cache.path(), &endpoint, &model)
            .unwrap()
            .path
            .exists()
    );
    thread.join().unwrap();
}
