use cxx_build::CFG;
use pywinpty_findlib::win_calls::Windows::Win32::System::SystemServices::{
    GetModuleHandleW, GetProcAddress,
};
use std::env;
use std::i64;
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
    println!("cargo:rerun-if-changed=include/");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let include_path = Path::new(&manifest_dir).join("include");
    CFG.exported_header_dirs.push(&include_path);
    CFG.exported_header_dirs.push(&Path::new(&manifest_dir));
    // Check if ConPTY is enabled
    let reg_entry = "HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion";

    let major_version = command_output(
        Command::new("Reg")
            .arg("Query")
            .arg(&reg_entry)
            .arg("/v")
            .arg("CurrentMajorVersionNumber"),
    );
    let version_parts: Vec<&str> = major_version.split("REG_DWORD").collect();
    let major_version =
        i64::from_str_radix(version_parts[1].trim().trim_start_matches("0x"), 16).unwrap();

    let build_version = command_output(
        Command::new("Reg")
            .arg("Query")
            .arg(&reg_entry)
            .arg("/v")
            .arg("CurrentBuildNumber"),
    );
    let build_parts: Vec<&str> = build_version.split("REG_SZ").collect();
    let build_version = i64::from_str_radix(build_parts[1].trim(), 10).unwrap();

    println!("Windows major version: {:?}", major_version);
    println!("Windows build number: {:?}", build_version);

    let conpty_enabled;
    let kernel32 = unsafe { GetModuleHandleW("kernel32.dll") };
    let conpty = unsafe { GetProcAddress(kernel32, "CreatePseudoConsole") };
    match conpty {
        Some(_) => {
            conpty_enabled = "1";
        }
        None => {
            conpty_enabled = "0";
        }
    }

    println!("ConPTY enabled: {}", conpty_enabled);

    // Check if winpty is installed
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

    // Check if building under debug mode
    let debug;
    match env::var("PROFILE") {
        Ok(profile) => match profile.as_str() {
            "debug" => {
                debug = "1";
            }
            _ => {
                debug = "0";
            }
        },
        Err(_) => {
            debug = "0";
        }
    }

    cxx_build::bridge("src/native.rs")
        .file("src/csrc/base.cpp")
        .file("src/csrc/pty.cpp")
        .file("src/csrc/wrapper.cpp")
        .file("src/csrc/winpty_common.cpp")
        .file("src/csrc/conpty_common.cpp")
        .file("src/csrc/StackWalker.cpp")
        // .flag_if_supported("-std=c++17")
        .flag_if_supported("-std=gnu++14")
        .flag_if_supported("/EHsc")
        .define("_GLIBCXX_USE_CXX11_ABI", "0")
        .define("ENABLE_WINPTY", winpty_enabled)
        .define("ENABLE_CONPTY", conpty_enabled)
        .define("DEBUG", debug)
        .warnings(false)
        .extra_warnings(false)
        .compile("winptywrapper");

    println!("cargo:rustc-link-lib=dylib=winpty");
}
