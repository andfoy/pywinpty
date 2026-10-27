use std::env;
use std::env::consts::ARCH;
use std::path::{Path, PathBuf};
use std::process::Command;
use glob::glob;

/// Version of the `Microsoft.Windows.Console.ConPTY` NuGet package to bundle.
///
/// This is pinned deliberately, and must stay in step with the version
/// `winpty-rs` pins in its own build script. This crate copies `conpty.dll`
/// and `OpenConsole.exe` into the `winpty` package directory for the wheel,
/// while `winpty-rs` links against `conpty.lib` from the same package - so if
/// the two resolved different versions, a wheel could ship a runtime DLL that
/// does not match the import library it was linked against.
///
/// Leaving it unpinned is not safe: every build takes whatever Microsoft
/// published most recently, so an upstream release can break every build with
/// no signal, and a published wheel carries no record of which ConPTY it was
/// validated against. That is not hypothetical - ConPTY 1.24.260512001 ->
/// 1.25.260930003 changed when a client is released, which deadlocked PTY
/// teardown (winpty-rs #136).
///
/// Override with `WINPTY_RS_CONPTY_VERSION`; `latest` resolves the newest
/// release. Dependabot cannot manage this - it only understands NuGet through
/// project manifests, not a version string in a Rust build script.
const CONPTY_NUGET_VERSION: &str = "1.25.260930003";

/// Environment variable that overrides [`CONPTY_NUGET_VERSION`].
const CONPTY_VERSION_ENV: &str = "WINPTY_RS_CONPTY_VERSION";

/// Resolve the ConPTY NuGet version to bundle.
///
/// Returns `None` when the caller asked for `latest`, meaning no `-Version`
/// argument is passed and NuGet picks the newest release.
fn conpty_nuget_version() -> Option<String> {
    println!("cargo:rerun-if-env-changed={CONPTY_VERSION_ENV}");
    match env::var(CONPTY_VERSION_ENV) {
        Ok(v) if v.eq_ignore_ascii_case("latest") => {
            println!("ConPTY version: latest (unpinned, via {CONPTY_VERSION_ENV})");
            None
        }
        Ok(v) if !v.trim().is_empty() => {
            println!("ConPTY version: {v} (from {CONPTY_VERSION_ENV})");
            Some(v)
        }
        _ => {
            println!("ConPTY version: {CONPTY_NUGET_VERSION} (pinned)");
            Some(CONPTY_NUGET_VERSION.to_owned())
        }
    }
}

#[cfg(windows)]
fn command_ok(cmd: &mut Command) -> bool {
    cmd.status().ok().map_or(false, |s| s.success())
}

fn main() {
    println!("cargo:rerun-if-changed=src/");
    println!("Manifest dir: {}", env::var("CARGO_MANIFEST_DIR").unwrap());
    let manifest_dir_str = env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_dir = Path::new(&manifest_dir_str);
    let package_dir = manifest_dir.join("winpty");

    let mut binaries_found = true;
    for bin_name in ["conpty.dll", "OpenConsole.exe"] {
        let bin_path = package_dir.join(bin_name);
        binaries_found = binaries_found && bin_path.exists();
    }

    if !binaries_found {
        let mut nuget = Command::new("nuget.exe");
        let nuget_found = command_ok(nuget.arg("help"));

        if !nuget_found {
            panic!("NuGet is required to build pywinpty");
        }

        let mut nuget_install = Command::new("nuget.exe");
        nuget_install
            .current_dir(manifest_dir.to_str().unwrap())
            .arg("install")
            .arg("Microsoft.Windows.Console.ConPTY");
        // Omitted entirely for `latest`, so NuGet picks the newest release.
        if let Some(version) = conpty_nuget_version() {
            nuget_install.arg("-Version").arg(version);
        }

        if command_ok(&mut nuget_install) {
            let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
            let manifest_path = PathBuf::from(Path::new(&manifest_dir));
            for path in glob(
                manifest_path
                    .join("Microsoft.Windows.Console.ConPTY*")
                    .to_str()
                    .unwrap(),
            )
            .unwrap()
            {
                match path {
                    Ok(folder) => {
                        use std::fs;

                        let simplified_arch = match ARCH {
                            "x86_64" => "x64",
                            "arm" => "arm64",
                            "aarch64" => "arm64",
                            _ => ARCH,
                        };

                        println!("{:?}", folder);
                        println!("{:?}", package_dir);
                        let openconsole = folder
                            .join("build")
                            .join("native")
                            .join("runtimes")
                            .join(simplified_arch)
                            .join("OpenConsole.exe");

                        let binaries_path = folder
                            .join("runtimes")
                            .join(format!("win-{}", simplified_arch));
                        let dll_path = binaries_path.join("native").join("conpty.dll");
                        // let lib_orig = binaries_path.join("lib").join("uap10.0").join("conpty.lib");

                        let openconsole_dst = package_dir.join("OpenConsole.exe");
                        let dll_dst = package_dir.join("conpty.dll");
                        // let lib_dst = package_dir.join("conpty.lib");

                        fs::copy(openconsole, openconsole_dst).unwrap();
                        fs::copy(dll_path, dll_dst).unwrap();
                        // fs::copy(lib_orig, lib_dst).unwrap();
                    }
                    Err(err) => panic!("{:?}", err),
                }
            }
        }
    }
}
