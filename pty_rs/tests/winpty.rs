#![cfg(feature="winpty")]

use std::ffi::OsString;
use winptyrs::{PTY, PTYArgs, PTYBackend, MouseMode, AgentConfig};

#[test]
fn it_works() {
    let pty_args = PTYArgs {
        cols: 80,
        rows: 25,
        mouse_mode: MouseMode::WINPTY_MOUSE_MODE_NONE,
        timeout: 10000,
        agent_config: AgentConfig::WINPTY_FLAG_COLOR_ESCAPES
    };

    match PTY::new_with_backend(&pty_args, PTYBackend::WinPTY) {
        Ok(mut pty) => {
            let appname = OsString::from("C:\\Windows\\System32\\cmd.exe");
            match pty.spawn(appname, None, None, None) {
                Ok(_) => {
                    ()
                },
                Err(err) => {
                    panic!("{:?}", err)
                }
            }
        },
        Err(err) => {panic!("{:?}", err)}
    }

    let result = 2 + 2;
    assert_eq!(result, 4);
}
