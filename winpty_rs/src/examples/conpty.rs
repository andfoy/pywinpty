extern crate winptyrs;
use std::ffi::OsString;
use winptyrs::{PTY, PTYArgs, PTYBackend, AgentConfig, MouseMode};

fn main() {
    let pty_args = PTYArgs {
        cols: 80,
        rows: 25,
        mouse_mode: MouseMode::WINPTY_MOUSE_MODE_NONE,
        timeout: 10000,
        agent_config: AgentConfig::WINPTY_FLAG_COLOR_ESCAPES
    };

    match PTY::new_with_backend(&pty_args, PTYBackend::ConPTY) {
        Ok(mut pty) => {
            println!("Creating");
            let appname = OsString::from("c:\\windows\\system32\\cmd.exe");
            println!("{:?}", appname);
            match pty.spawn(appname, None, None, None) {
                Ok(_) => {
                    let mut output = pty.read(1000, false);
                    match output {
                        Ok(out) => println!("{}", out.to_str().unwrap()),
                        Err(err) => panic!("{:?}", err)
                    }

                    output = pty.read(1000, false);
                    match output {
                        Ok(out) => println!("{}", out.to_str().unwrap()),
                        Err(err) => panic!("{:?}", err)
                    }

                    match pty.write(OsString::from("echo \"aaaa 😀\"")) {
                        Ok(bytes) => println!("Bytes written: {}", bytes),
                        Err(err) => panic!("{:?}", err)
                    }

                    output = pty.read(1000, false);
                    match output {
                        Ok(out) => println!("{}", out.to_str().unwrap()),
                        Err(err) => panic!("{:?}", err)
                    }

                    output = pty.read(1000, false);
                    match output {
                        Ok(out) => println!("{}", out.to_str().unwrap()),
                        Err(err) => panic!("{:?}", err)
                    }

                    match pty.is_alive() {
                        Ok(alive) => println!("Is alive {}", alive),
                        Err(err) => panic!("{:?}", err)
                    }

                    match pty.write(OsString::from("\r\n")) {
                        Ok(bytes) => println!("Bytes written: {}", bytes),
                        Err(err) => panic!("{:?}", err)
                    }

                    output = pty.read(1000, false);
                    match output {
                        Ok(out) => println!("{}", out.to_str().unwrap()),
                        Err(err) => panic!("{:?}", err)
                    }

                    output = pty.read(1000, false);
                    match output {
                        Ok(out) => println!("{}", out.to_str().unwrap()),
                        Err(err) => panic!("{:?}", err)
                    }
                },
                Err(err) => {
                    println!("{:?}", err);
                    panic!("{:?}", err)
                }
            }
        },
        Err(err) => {panic!("{:?}", err)}
    }
}
