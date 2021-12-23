
use std::ffi::OsString;

// Default implementation if winpty is not available
use super::PTYArgs;
use super::base::PTYImpl;

pub struct ConPTY {}

impl PTYImpl for ConPTY {
    fn new(_args: &PTYArgs) -> Result<Box<dyn PTYImpl>, OsString> {
        Err(OsString::from("pty_rs was compiled without ConPTY enabled"))
    }

    fn spawn(&mut self, _appname: OsString, _cmdline: Option<OsString>, _cwd: Option<OsString>, _env: Option<OsString>) -> Result<bool, OsString> {
        Err(OsString::from("pty_rs was compiled without ConPTY enabled"))
    }

    fn set_size(&self, _cols: i32, _rows: i32) -> Result<(), OsString> {
        Err(OsString::from("pty_rs was compiled without ConPTY enabled"))
    }

    fn read(&self, _length: u32, _blocking: bool) -> Result<OsString, OsString> {
        Err(OsString::from("pty_rs was compiled without ConPTY enabled"))
    }

    fn write(&self, _buf: OsString) -> Result<u32, OsString> {
        Err(OsString::from("pty_rs was compiled without ConPTY enabled"))
    }

    fn is_eof(&mut self) -> Result<bool, OsString> {
        Err(OsString::from("pty_rs was compiled without ConPTY enabled"))
    }

    fn get_exitstatus(&mut self) -> Result<Option<u32>, OsString> {
        Err(OsString::from("pty_rs was compiled without ConPTY enabled"))
    }

    fn is_alive(&mut self) -> Result<bool, OsString> {
        Err(OsString::from("pty_rs was compiled without ConPTY enabled"))
    }
}
