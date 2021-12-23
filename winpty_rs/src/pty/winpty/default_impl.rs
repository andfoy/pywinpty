
use crate::pty::{PTYArgs, PTYImpl};

pub struct WinPTY {}

impl PTYImpl for WinPTY {
    fn new(_args: &PTYArgs) -> Result<Box<dyn PTYImpl>, OsString> {
        Err(OsString::from("winpty_rs was compiled without WinPTY enabled"))
    }

    fn spawn(&mut self, _appname: OsString, _cmdline: Option<OsString>, _cwd: Option<OsString>, _env: Option<OsString>) -> Result<bool, OsString> {
        Err(OsString::from("winpty_rs was compiled without WinPTY enabled"))
    }

    fn set_size(&self, _cols: i32, _rows: i32) -> Result<(), OsString> {
        Err(OsString::from("winpty_rs was compiled without WinPTY enabled"))
    }

    fn read(&self, _length: u32, _blocking: bool) -> Result<OsString, OsString> {
        Err(OsString::from("winpty_rs was compiled without WinPTY enabled"))
    }

    fn write(&self, _buf: OsString) -> Result<u32, OsString> {
        Err(OsString::from("winpty_rs was compiled without WinPTY enabled"))
    }

    fn is_eof(&mut self) -> Result<bool, OsString> {
        Err(OsString::from("winpty_rs was compiled without WinPTY enabled"))
    }

    fn get_exitstatus(&mut self) -> Result<Option<u32>, OsString> {
        Err(OsString::from("winpty_rs was compiled without WinPTY enabled"))
    }

    fn is_alive(&mut self) -> Result<bool, OsString> {
        Err(OsString::from("winpty_rs was compiled without WinPTY enabled"))
    }
}
