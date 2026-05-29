use std::env;
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
