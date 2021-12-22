/// Actual WinPTY backend implementation.

use windows::Win32::Foundation::{PWSTR, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_GENERIC_READ, FILE_SHARE_NONE,
    OPEN_EXISTING, FILE_GENERIC_WRITE,
    FILE_ATTRIBUTE_NORMAL};
use num_traits::ToPrimitive;

use std::ptr;
use std::mem::MaybeUninit;
use std::slice::from_raw_parts;
use std::ffi::{OsString, c_void};
use std::os::windows::prelude::*;
use std::os::windows::ffi::OsStrExt;

use super::bindings::*;
use crate::pty::{PTYProcess, PTYImpl};
use crate::pty::PTYArgs;

struct WinPTYPtr {
    ptr: *mut winpty_t,
}

impl WinPTYPtr {
    // pub fn get_agent_process(&self) -> HANDLE {
    //     unsafe {
    //         let void_mem = winpty_agent_process(self.ptr);
    //         HANDLE(void_mem as isize)
    //     }
    // }

    pub fn get_conin_name(&self) -> *const u16 {
        unsafe { winpty_conin_name(self.ptr) }
    }

    pub fn get_conout_name(&self) -> *const u16 {
        unsafe { winpty_conout_name(self.ptr) }
    }

    pub fn spawn(&self, appname: *const u16, cmdline: *const u16, cwd: *const u16, env: *const u16) -> Result<HANDLE, OsString> {
        let mut err_ptr: *mut winpty_error_ptr_t = ptr::null_mut();
        unsafe {
            let spawn_config = winpty_spawn_config_new(3u64, appname, cmdline, cwd, env, err_ptr);
            if spawn_config.is_null() {
                return Err(get_error_message(err_ptr));
            }

            err_ptr = ptr::null_mut();
            //let process = HANDLE(0);
            //let process = HANDLE::default();
            //let mut process_ptr = process.0 as *mut c_void;
            let mut handle_value = MaybeUninit::<isize>::uninit();
            let mut handle = ptr::addr_of_mut!((*handle_value.as_mut_ptr())) as *mut c_void;
            //let handle_value = process.0 as *mut c_void;
            //let process: *mut *mut  = ptr::null_mut();
            let succ = winpty_spawn(self.ptr, spawn_config, ptr::addr_of_mut!(handle), ptr::null_mut::<_>(),
                                    ptr::null_mut::<u32>(), err_ptr);
            winpty_spawn_config_free(spawn_config);
            if !succ {
                return Err(get_error_message(err_ptr));
            }

            handle_value.assume_init();
            let process = HANDLE(handle as isize);
            Ok(process)
        }
    }

    pub fn set_size(&self, cols: i32, rows: i32) -> Result<(), OsString> {
        let err_ptr: *mut winpty_error_ptr_t = ptr::null_mut();
        unsafe {
            let succ = winpty_set_size(self.ptr, cols, rows, err_ptr);
            if !succ {
                return Err(get_error_message(err_ptr));
            }
        }
        Ok(())
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

unsafe fn get_error_message(err_ptr: *mut winpty_error_ptr_t) -> OsString {
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

impl PTYImpl for WinPTY {
    fn new(args: &PTYArgs) -> Result<Box<dyn PTYImpl>, OsString> {
        unsafe {
            //let mut err: Box<winpty_error_t> = Box::new_uninit();
            //let mut err_ptr: *mut winpty_error_t = &mut *err;
            let mut err_ptr: *mut winpty_error_ptr_t = ptr::null_mut();
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
            // let handle = pty_ptr.get_agent_process();
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

            let process = PTYProcess::new(conin, conout, false);
            Ok(Box::new(WinPTY { ptr: pty_ptr, process }) as Box<dyn PTYImpl>)
        }
    }

    fn spawn(&mut self, appname: OsString, cmdline: Option<OsString>, cwd: Option<OsString>, env: Option<OsString>) -> Result<bool, OsString> {
        let mut environ: *const u16 = ptr::null();
        let mut working_dir: *const u16 = ptr::null();

        let mut cmdline_oss = OsString::new();
        cmdline_oss.push("\0\0");
        let cmdline_oss_buf: Vec<u16> = cmdline_oss.encode_wide().collect();
        let mut cmd = cmdline_oss_buf.as_ptr();

        if let Some(env_opt) = env {
            let env_buf: Vec<u16> = env_opt.encode_wide().collect();
            environ = env_buf.as_ptr();
        }

        if let Some(cwd_opt) = cwd {
            let cwd_buf: Vec<u16> = cwd_opt.encode_wide().collect();
            working_dir = cwd_buf.as_ptr();
        }

        if let Some(cmdline_opt) = cmdline {
            let cmd_buf: Vec<u16> = cmdline_opt.encode_wide().collect();
            cmd = cmd_buf.as_ptr();
        }

        let mut app_buf: Vec<u16> = appname.encode_wide().collect();
        app_buf.push(0);
        let app = app_buf.as_ptr();

        match self.ptr.spawn(app, cmd, working_dir, environ) {
            Ok(handle) => {
                self.process.set_process(handle, true);
                Ok(true)
            },
            Err(err) => {
                Err(err)
            }
        }
    }

    fn set_size(&self, cols: i32, rows: i32) -> Result<(), OsString> {
        if cols <= 0 || rows <= 0 {
            let err: OsString = OsString::from(format!(
                "PTY cols and rows must be positive and non-zero. Got: ({}, {})", cols, rows));
            return Err(err);
        }
        self.ptr.set_size(cols, rows)
    }

    fn read(&self, length: u32, blocking: bool) -> Result<OsString, OsString> {
        self.process.read(length, blocking)
    }

    fn write(&self, buf: OsString) -> Result<u32, OsString> {
        self.process.write(buf)
    }

    fn is_eof(&mut self) -> Result<bool, OsString> {
        self.process.is_eof()
    }

    fn get_exitstatus(&mut self) -> Result<Option<u32>, OsString> {
        self.process.get_exitstatus()
    }

    fn is_alive(&mut self) -> Result<bool, OsString> {
        self.process.is_alive()
    }
}
