use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn client_assets(dist: &Path) -> Vec<(String, PathBuf)> {
    fn collect(dist: &Path, directory: &Path, assets: &mut Vec<(String, PathBuf)>) {
        for entry in fs::read_dir(directory).expect("read frontend dist directory") {
            let entry = entry.expect("read frontend dist entry");
            let path = entry.path();
            if entry.file_type().expect("inspect frontend dist entry").is_dir() {
                collect(dist, &path, assets);
            } else if entry.file_type().expect("inspect frontend dist entry").is_file() {
                let relative = path
                    .strip_prefix(dist)
                    .expect("frontend asset stays inside dist")
                    .to_string_lossy()
                    .replace('\\', "/");
                assets.push((relative, path));
            }
        }
    }

    let mut assets = Vec::new();
    collect(dist, dist, &mut assets);
    assets.sort_by(|left, right| left.0.cmp(&right.0));
    assets
}

fn main() {
    println!("cargo:rerun-if-changed=playground-client/src");
    println!("cargo:rerun-if-changed=playground-client/index.html");
    println!("cargo:rerun-if-changed=playground-client/package-lock.json");
    println!("cargo:rerun-if-changed=playground-client/vite.config.ts");
    println!("cargo:rerun-if-changed=playground-client/svelte.config.js");
    if env::var_os("CARGO_FEATURE_PLAYGROUND_CLIENT").is_none() {
        return;
    }
    let client =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("playground-client");
    let mut command = if cfg!(windows) {
        let mut c = Command::new("cmd");
        c.args(["/c", "npm.cmd"]);
        c
    } else {
        Command::new("npm")
    };
    let status = command
        .args(["run", "build"])
        .current_dir(&client)
        .status()
        .expect("Node.js/npm is required to package playground clients");
    assert!(
        status.success(),
        "NprPlayground frontend build failed; run npm ci in playground-client"
    );
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let mut generated = String::from(
        "pub fn playground_client_assets() -> Vec<(&'static str, &'static [u8])> { vec![\n",
    );
    for (name, source) in client_assets(&client.join("dist")) {
        let target = out.join(name.replace('/', std::path::MAIN_SEPARATOR_STR));
        fs::create_dir_all(target.parent().expect("frontend asset parent")).unwrap();
        fs::copy(source, &target).unwrap();
        generated.push_str(&format!(
            "({:?}, include_bytes!({:?})),\n",
            name,
            target.to_string_lossy()
        ));
    }
    generated.push_str("] }\n");
    fs::write(out.join("playground_client.rs"), generated).unwrap();
}
