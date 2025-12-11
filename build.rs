use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Only copy the example config in release builds
    let profile = env::var("PROFILE").unwrap_or_default();

    if profile == "release" {
        let out_dir = env::var("OUT_DIR").unwrap();
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

        // Navigate from OUT_DIR to the release directory
        // OUT_DIR is typically: target/release/build/wxlistener-<hash>/out
        // We want: target/release/
        let out_path = Path::new(&out_dir);
        if let Some(release_dir) = out_path.ancestors().nth(3) {
            let src = Path::new(&manifest_dir).join("wxlistener.example.toml");
            let dest = release_dir.join("wxlistener.example.toml");

            if src.exists() {
                if let Err(e) = fs::copy(&src, &dest) {
                    println!("cargo:warning=Failed to copy example config: {}", e);
                } else {
                    println!("cargo:warning=Copied wxlistener.example.toml to release directory");
                }
            }
        }
    }

    // Re-run the build script if the example file changes
    println!("cargo:rerun-if-changed=wxlistener.example.toml");
}
