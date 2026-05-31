use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const ENGINE_HEADER: &str = "LiteRT-LM/c/engine.h";

fn update_submodules() {
    let program = "git";
    let dir = "../";
    let args = ["submodule", "update", "--init", "--recursive"];
    println!(
        "Running command: \"{} {}\" in dir: {}",
        program,
        args.join(" "),
        dir
    );
    let ret = Command::new(program).current_dir(dir).args(args).status();

    match ret.map(|status| (status.success(), status.code())) {
        Ok((true, _)) => (),
        Ok((false, Some(c))) => panic!("Command failed with error code {c}"),
        Ok((false, None)) => panic!("Command got killed"),
        Err(e) => panic!("Command failed with error: {e}"),
    }
}

fn main() {
    println!("cargo:rerun-if-changed={ENGINE_HEADER}");

    if !Path::new("LiteRT-LM/LICENSE").exists() {
        update_submodules()
    }

    generate_bindings();
    compile_litert_lm();
}

fn generate_bindings() {
    if !Path::new(ENGINE_HEADER).exists() {
        panic!("LiteRT-LM header not found at {ENGINE_HEADER}");
    }

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let bindings_path = out_dir.join("bindings.rs");

    let bindings = bindgen::Builder::default()
        .header("LiteRT-LM/c/engine.h")
        .clang_arg("-ILiteRT-LM")
        .allowlist_type("LiteRtLm.*")
        .allowlist_function("litert_lm_.*")
        .allowlist_var("kLiteRtLm.*")
        .generate_comments(true)
        .generate()
        .expect("failed to generate LiteRT-LM bindings");

    bindings
        .write_to_file(bindings_path)
        .expect("failed to write LiteRT-LM bindings");
}

fn compile_litert_lm() {
    let status = Command::new("bazelisk")
        .current_dir("LiteRT-LM")
        .args([
            "build",
            "--linkopt=-Wl,-install_name,@rpath/liblitert-lm.dylib",
            "//python/litert_lm:litert-lm",
        ])
        .status()
        .expect("failed to run bazelisk");

    if !status.success() {
        panic!("bazelisk build failed with status {status}");
    }

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let lib_dir = out_dir.join("lib");

    fs::create_dir_all(&lib_dir).expect("failed to create native library output directory");
    fs::copy(
        "LiteRT-LM/bazel-bin/python/litert_lm/liblitert-lm.dylib",
        lib_dir.join("liblitert-lm.dylib"),
    )
    .expect("failed to copy liblitert-lm.dylib");

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=dylib=litert-lm");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());
}
