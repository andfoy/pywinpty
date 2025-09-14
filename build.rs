use std::env;
use std::env::consts::ARCH;
use std::path::{Path, PathBuf};
use std::process::Command;
use glob::glob;

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

        if command_ok(
            Command::new("nuget.exe")
                .current_dir(manifest_dir.to_str().unwrap())
                .arg("install")
                .arg("Microsoft.Windows.Console.ConPTY"),
        ) {
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
