use std::path::Path;
use std::process::Command;

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
    if !Path::new("LiteRT-LM/LICENSE").exists() {
        update_submodules()
    }
}