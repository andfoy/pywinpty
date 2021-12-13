/// Actual WinPTY backend implementation.

use windows::Win32::Foundation::{PWSTR, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_GENERIC_READ, FILE_SHARE_NONE,
    OPEN_EXISTING, FILE_GENERIC_WRITE,
    FILE_ATTRIBUTE_NORMAL};
use num_traits::ToPrimitive;

use std::ptr;
use std::slice::from_raw_parts;
use std::ffi::OsString;
use std::os::windows::prelude::*;

use super::bindings::*;
use crate::pty::base::PTYProcess;
use crate::pty::PTYArgs;

struct WinPTYPtr {
    ptr: *mut winpty_t,
}

impl WinPTYPtr {
    pub fn get_agent_process(&self) -> HANDLE {
        unsafe {
            let void_mem = winpty_agent_process(self.ptr);
            HANDLE(void_mem as isize)
        }
    }

    pub fn get_conin_name(&self) -> *const u16 {
        unsafe { winpty_conin_name(self.ptr) }
    }

    pub fn get_conout_name(&self) -> *const u16 {
        unsafe { winpty_conout_name(self.ptr) }
    }
}

impl Drop for WinPTYPtr {
    fn drop(&mut self) {
        unsafe { winpty_free(self.ptr) }
    }
}

// Winpty_t object claims to be thread-safe on the header documentation.
unsafe impl Send for WinPTYPtr {}
unsafe impl Sync for WinPTYPtr {}


// struct WinPTYError {
//     ptr: *mut winpty_error_t
// }

// impl WinPTYError {
//     pub fn get_error_message(&'a self) ->  {

//     }
// }

// struct HandleWrapper<'a> {
//     handle: *const HANDLE,
//     phantom: PhantomData<&'a HandleWrapper>
// }

// fn from<'a>(_: &'a WinPTYPtr, handle: *const )

unsafe fn get_error_message(err_ptr: *mut winpty_error_t) -> OsString {
    let err_msg: *const u16 = winpty_error_msg(err_ptr);
    let mut size = 0;
    let mut ptr = err_msg;
    while *ptr != 0 {
        size += 1;
        ptr = ptr.wrapping_offset(1);

    }
    let msg_slice: &[u16] = from_raw_parts(err_msg, size);
    winpty_error_free(err_ptr);
    OsString::from_wide(msg_slice)
}


/// FFi-safe wrapper around `winpty` library calls and objects.
pub struct WinPTY {
    ptr: WinPTYPtr,
    process: PTYProcess
}

impl WinPTY {
    pub fn new(args: &mut PTYArgs) -> Result<WinPTY, OsString> {
        unsafe {
            //let mut err: Box<winpty_error_t> = Box::new_uninit();
            //let mut err_ptr: *mut winpty_error_t = &mut *err;
            let mut err_ptr: *mut winpty_error_t = ptr::null_mut();
            let config = winpty_config_new(args.agent_config.bits(), err_ptr);
            //err.assume_init();

            if config.is_null() {
                return Err(get_error_message(err_ptr));
            }

            if args.cols <= 0 || args.rows <= 0 {
                let err: OsString = OsString::from(format!(
                    "PTY cols and rows must be positive and non-zero. Got: ({}, {})", args.cols, args.rows));
                return Err(err);
            }

            winpty_config_set_initial_size(config, args.cols, args.rows);
            winpty_config_set_mouse_mode(config, args.mouse_mode.to_i32().unwrap());
            winpty_config_set_agent_timeout(config, args.timeout);

            // err = Box::new_uninit();
            // err_ptr = &mut *err;
            err_ptr = ptr::null_mut();

            let pty_ref = winpty_open(config, err_ptr);
            winpty_config_free(config);

            if pty_ref.is_null() {
                return Err(get_error_message(err_ptr));
            }

            let pty_ptr = WinPTYPtr { ptr: pty_ref };
            let handle = pty_ptr.get_agent_process();
            let conin_name = pty_ptr.get_conin_name();
            let conout_name = pty_ptr.get_conout_name();

            let empty_handle = HANDLE(0);
            let conin = CreateFileW(
                PWSTR(conin_name as *mut u16), FILE_GENERIC_WRITE, FILE_SHARE_NONE, ptr::null(),
                OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, empty_handle
            );

            let conout = CreateFileW(
                PWSTR(conout_name as *mut u16), FILE_GENERIC_READ, FILE_SHARE_NONE, ptr::null(),
                OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, empty_handle
            );

            let process = PTYProcess::new(handle, conin, conout, false);
            Ok(WinPTY { ptr: pty_ptr, process: process })
        }
    }
}
