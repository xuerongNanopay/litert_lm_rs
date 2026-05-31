use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const ENGINE_HEADER: &str = "LiteRT-LM/c/engine.h";

struct Platform {
    bazel_config: Option<&'static str>,
    main_library: &'static str,
    prebuilt_dir: &'static str,
    dependency_extensions: &'static [&'static str],
    install_name: Option<&'static str>,
    use_rpath: bool,
}

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
    println!("cargo:rerun-if-changed=build.rs");

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
    let platform = platform();
    let status = Command::new("bazelisk")
        .current_dir("LiteRT-LM")
        .args(["clean"])
        .status()
        .expect("failed to run bazelisk clean");

    if !status.success() {
        panic!("bazelisk clean failed with status {status}");
    }

    let mut args = vec!["build".to_owned()];
    if let Some(config) = platform.bazel_config {
        args.push(format!("--config={config}"));
    }
    if let Some(install_name) = platform.install_name {
        args.push(format!("--linkopt=-Wl,-install_name,{install_name}"));
    }
    args.push("//python/litert_lm:litert-lm".to_owned());

    let status = Command::new("bazelisk")
        .current_dir("LiteRT-LM")
        .args(args)
        .status()
        .expect("failed to run bazelisk");

    if !status.success() {
        panic!("bazelisk build failed with status {status}");
    }

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let lib_dir = out_dir.join("lib");
    let output_lib = lib_dir.join(platform.main_library);

    fs::create_dir_all(&lib_dir).expect("failed to create native library output directory");
    copy_file(
        &Path::new("LiteRT-LM/bazel-bin/python/litert_lm").join(platform.main_library),
        &output_lib,
    );

    copy_dependencies(&platform, &lib_dir);

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=dylib=litert-lm");
    if platform.use_rpath {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());
    }
}

fn copy_dependencies(platform: &Platform, lib_dir: &Path) {
    let dependencies_dir = Path::new("LiteRT-LM/prebuilt").join(platform.prebuilt_dir);
    let dependencies =
        fs::read_dir(dependencies_dir).expect("failed to read native dependencies directory");

    for dependency in dependencies {
        let dependency = dependency.expect("failed to read native dependency");
        let source = dependency.path();
        let extension = source.extension().and_then(|extension| extension.to_str());

        if !platform
            .dependency_extensions
            .iter()
            .any(|item| Some(*item) == extension)
        {
            continue;
        }

        let output = lib_dir.join(dependency.file_name());
        copy_file(&source, &output);
    }
}

fn copy_file(source: &Path, output: &Path) {
    if output.exists() {
        fs::remove_file(output).expect("failed to remove previous native library");
    }

    fs::copy(source, output).expect("failed to copy native library");
}

fn platform() -> Platform {
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS is set by Cargo");
    let target_arch =
        env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH is set by Cargo");

    match (target_os.as_str(), target_arch.as_str()) {
        ("macos", "aarch64") => Platform {
            bazel_config: Some("macos_arm64"),
            main_library: "liblitert-lm.dylib",
            prebuilt_dir: "macos_arm64",
            dependency_extensions: &["dylib"],
            install_name: Some("@rpath/liblitert-lm.dylib"),
            use_rpath: true,
        },
        ("linux", "x86_64") => Platform {
            bazel_config: None,
            main_library: "liblitert-lm.so",
            prebuilt_dir: "linux_x86_64",
            dependency_extensions: &["so"],
            install_name: None,
            use_rpath: true,
        },
        ("linux", "aarch64") => Platform {
            bazel_config: None,
            main_library: "liblitert-lm.so",
            prebuilt_dir: "linux_arm64",
            dependency_extensions: &["so"],
            install_name: None,
            use_rpath: true,
        },
        ("windows", "x86_64") => Platform {
            bazel_config: Some("windows"),
            main_library: "litert-lm.dll",
            prebuilt_dir: "windows_x86_64",
            dependency_extensions: &["dll", "lib"],
            install_name: None,
            use_rpath: false,
        },
        _ => panic!("unsupported target: {target_arch}-{target_os}"),
    }
}
