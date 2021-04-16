use cxx_build::CFG;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::str;
use which::which;

fn command_ok(cmd: &mut Command) -> bool {
    cmd.status().ok().map_or(false, |s| s.success())
}

fn command_output(cmd: &mut Command) -> String {
    str::from_utf8(&cmd.output().unwrap().stdout)
        .unwrap()
        .trim()
        .to_string()
}

fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/native.rs");
    println!("cargo:rerun-if-changed=src/csrc");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let include_path = Path::new(&manifest_dir)
        .join("include");
    CFG.exported_header_dirs.push(&include_path);

    let mut cmd = Command::new("winpty-agent");
    let mut winpty_enabled = "0";
    if command_ok(cmd.arg("--version")) {
        // let winpty_path = cm
        winpty_enabled = "1";
        let winpty_version = command_output(cmd.arg("--version"));
        println!("Using Winpty version: {}", &winpty_version);

        let winpty_location = which("winpty-agent").unwrap();
        let winpty_path = winpty_location.parent().unwrap();
        let winpty_root = winpty_path.parent().unwrap();
        let winpty_include = winpty_root.join("include");

        let winpty_lib = winpty_root.join("lib");

        println!(
            "cargo:rustc-link-search=native={}",
            winpty_lib.to_str().unwrap()
        );
        println!(
            "cargo:rustc-link-search=native={}",
            winpty_path.to_str().unwrap()
        );

        CFG.exported_header_dirs.push(&winpty_include);
    }

    cxx_build::bridge("src/native.rs")
        .file("src/csrc/winpty_common.cpp")
        // .flag_if_supported("-std=c++17")
        .flag_if_supported("-std=gnu++14")
        .define("_GLIBCXX_USE_CXX11_ABI", "0")
        .define("ENABLE_WINPTY", winpty_enabled)
        .warnings(false)
        .extra_warnings(false)
        .compile("winptywrapper");

    println!("cargo:rustc-link-lib=dylib=winpty");
}
