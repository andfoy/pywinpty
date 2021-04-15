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

    let cmd = Command::new("winpty-agent").arg("--version");
    let mut winpty_enabled = false;
    if command_ok(cmd) {
        // let winpty_path = cm
        winpty_enabled = true;
        let winpty_version = command_output(cmd);
        println!("Using Winpty version: {}", &winpty_version);

        let winpty_location = which("winpty-agent").unwrap();
        let winpty_path = winpty_location.parent().unwrap();
        let winpty_include = winpty_path.join("include")

        let winpty_lib = winpty_path.join("lib");

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
        .file("src/csrc/wrapper.cc")
        .file("src/csrc/utils.cc")
        .file("src/csrc/creation.cc")
        .file("src/csrc/info.cc")
        // .flag_if_supported("-std=c++17")
        .flag_if_supported("-std=gnu++14")
        .define("_GLIBCXX_USE_CXX11_ABI", "0")
        .warnings(false)
        .extra_warnings(false)
        .compile("winpty");

    println!("cargo:rustc-link-lib=dylib=winpty");
}
