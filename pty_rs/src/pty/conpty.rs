
use std::ffi::OsString;

// Default implementation if winpty is not available
use super::PTYArgs;

pub struct ConPTY {}

impl ConPTY {
    pub fn new(args: &mut PTYArgs) -> Result<ConPTY, OsString> {
        Ok(ConPTY{})
    }
}
