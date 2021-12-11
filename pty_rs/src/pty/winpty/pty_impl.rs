/// Actual WinPTY backend implementation.

use windows::Win32::Foundation::{PWSTR, HANDLE};
use windows::Win32::Storage::FileSystem::{CreateFileW, FILE_ACCESS_FLAGS, FILE_CREATION_DISPOSITION};

use std::ptr;
use std::cmp::min;
use std::ffi::OsString;
use std::os::windows::prelude::*;
use std::os::windows::ffi::OsStrExt;

use super::bindings::*;
use crate::pty::base::PTYProcess;
use crate::pty::PTYArgs;

struct WinPTYPtr {
    ptr: *mut winpty_t;
}

impl WinPTYPtr {
    pub fn get_agent_process(&'a self) -> &'a HANDLE {
        unsafe { winpty_agent_process(self.ptr) }
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
    let ptr = err_msg;
    while *ptr != 0 {
        size += 1;
        ptr = ptr.wrapping_offset(1);

    }
    let msg_slice: &[u16] = from_raw_parts(err_msg, size);
    winpty_error_free(err_ptr);
    OsString::from_wide(msg_slice)
}


/// FFi-safe wrapper around `winpty` library calls and objects.
struct WinPTY {
    ptr: WinPTYPtr,
    process: PTYProcess
}

impl WinPTY {
    pub fn new(args: PTYArgs) -> Result<WinPTY, OsString> {
        unsafe {
            let mut err: Box<winpty_error_t> = Box::new_uninit();
            let mut err_ptr: *mut winpty_error_t = &mut *err;
            // let err_ptr: *mut winpty_error_t = ptr::null_mut();
            let config = winpty_config_new(args.agent_config as u64, err_ptr);
            err.assume_init();

            if config == ptr::null() {
                Err(get_error_message(err_ptr))
            }

            if args.cols <= 0 || args.rows <= 0 {
                let err: OsString = format!(
                    "PTY cols and rows must be positive and non-zero. Got: ({}, {})", args.cols, args.rows);
                Err(err)
            }

            winpty_config_set_initial_size(config, args.cols, args.rows);
            winpty_config_set_mouse_mode(config, args.mouse_mode);
            winpty_config_set_agent_timeout(config, args.timeout);

            err = Box::new_uninit();
            err_ptr = &mut *err;

            let pty_ref = winpty_open(config, err_ptr);
            winpty_config_free(config);

            if pty_ref == ptr::null() {
                Err(get_error_message(err_ptr))
            }

            let pty_ptr = WinPTYPtr { pty_ref };
            let handle = pty_ptr.get_agent_process();
            let conin_name = pty_ptr.get_conin_name();
            let conout_name = pty_ptr.get_conout_name();

            let conin = CreateFileW(
                conin_name, FILE_ACCESS_FLAGS::GENERIC_WRITE, 0, ptr::null(),
                FILE_CREATION_DISPOSITION::OPEN_EXISTING, 0, ptr::null()
            );

            let conout = CreateFileW(
                conout_name, FILE_ACCESS_FLAGS::GENERIC_READ, 0, ptr::null(),
                FILE_CREATION_DISPOSITION::OPEN_EXISTING, 0, ptr::null()
            );

            let process = PTYProcess {
                handle, conin, conout, 0, -1, false, false
            }

            Ok(WinPTY { pty_ptr, process })
        }
    }
}
