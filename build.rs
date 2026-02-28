use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let feature_enabled = env::var("CARGO_FEATURE_STORAGE_K8SCONFIGMAP").is_ok();
    if !feature_enabled {
        return;
    }

    let go_src_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("k8s-configmap-lib");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    if cfg!(target_os = "windows") {
        // Windows: build as DLL, Rust side uses raw-dylib so no .lib needed
        let output_path = out_dir.join("libfs3k8sconfigmap.dll");
        let status = Command::new("go")
            .arg("build")
            .arg("-buildmode=c-shared")
            .arg("-buildvcs=false")
            .arg("-o")
            .arg(&output_path)
            .arg(".")
            .current_dir(&go_src_dir)
            .status()
            .expect("Failed to run `go build`. Is Go installed?");
        if !status.success() {
            panic!("Go build failed with status: {}", status);
        }
        // Copy DLL next to the final binary so it can be found at runtime
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let _ = std::fs::copy(&output_path, manifest_dir.join("libfs3k8sconfigmap.dll"));
    } else {
        // Unix: build as static archive
        let output_path = out_dir.join("libfs3k8sconfigmap.a");
        let status = Command::new("go")
            .arg("build")
            .arg("-buildmode=c-archive")
            .arg("-buildvcs=false")
            .arg("-o")
            .arg(&output_path)
            .arg(".")
            .current_dir(&go_src_dir)
            .status()
            .expect("Failed to run `go build`. Is Go installed?");
        if !status.success() {
            panic!("Go build failed with status: {}", status);
        }
        println!("cargo:rustc-link-search=native={}", out_dir.display());
        println!("cargo:rustc-link-lib=static=fs3k8sconfigmap");
    }

    println!("cargo:rerun-if-changed=k8s-configmap-lib/main.go");
}
