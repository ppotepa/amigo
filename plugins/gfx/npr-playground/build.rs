use std::{env, fs, path::PathBuf, process::Command};
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
    for entry in fs::read_dir(client.join("dist")).unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let target = out.join(&name);
        fs::copy(entry.path(), &target).unwrap();
        generated.push_str(&format!(
            "({:?}, include_bytes!({:?})),\n",
            name,
            target.to_string_lossy()
        ));
    }
    generated.push_str("] }\n");
    fs::write(out.join("playground_client.rs"), generated).unwrap();
}
