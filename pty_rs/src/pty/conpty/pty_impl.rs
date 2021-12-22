/// Actual ConPTY implementation.

use windows::Win32::Foundation::{
    GetLastError, CloseHandle, PWSTR, HANDLE,
    S_OK, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_GENERIC_READ, FILE_SHARE_READ,
    FILE_SHARE_WRITE, OPEN_EXISTING, FILE_GENERIC_WRITE,
    FILE_ATTRIBUTE_NORMAL, FILE_FLAGS_AND_ATTRIBUTES};
use windows::Win32::System::Console::{
    HPCON, AllocConsole, GetConsoleWindow,
    GetConsoleMode, CONSOLE_MODE, ENABLE_VIRTUAL_TERMINAL_PROCESSING,
    SetConsoleMode, SetStdHandle, GetStdHandle, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE,
    STD_INPUT_HANDLE, COORD, CreatePseudoConsole, ResizePseudoConsole,
    ClosePseudoConsole, FreeConsole};
use windows::Win32::System::Pipes::CreatePipe;
use windows::Win32::System::Threading::{
    PROCESS_INFORMATION, STARTUPINFOEXW, STARTUPINFOW,
    LPPROC_THREAD_ATTRIBUTE_LIST, InitializeProcThreadAttributeList,
    UpdateProcThreadAttribute, CreateProcessW,
    EXTENDED_STARTUPINFO_PRESENT, CREATE_UNICODE_ENVIRONMENT,
    DeleteProcThreadAttributeList, STARTF_USESTDHANDLES, CREATE_NEW_CONSOLE};
use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};
use windows::core::HRESULT;

use std::ptr;
use std::mem;
use std::mem::MaybeUninit;
use std::ffi::OsString;
use std::os::windows::prelude::*;
use std::os::windows::ffi::OsStrExt;

use crate::pty::{PTYProcess, PTYImpl};
use crate::pty::PTYArgs;

/// Struct that contains the required information to spawn a console
/// using the Windows API `CreatePseudoConsole` call.
pub struct ConPTY {
    handle: HPCON,
    process_info: PROCESS_INFORMATION,
    startup_info: STARTUPINFOEXW,
    process: PTYProcess
}

impl PTYImpl for ConPTY {
    fn new(args: &PTYArgs) -> Result<Box<dyn PTYImpl>, OsString> {
        let mut result: HRESULT = S_OK;
        if args.cols <= 0 || args.rows <= 0 {
            let err: OsString = OsString::from(format!(
                "PTY cols and rows must be positive and non-zero. Got: ({}, {})", args.cols, args.rows));
            return Err(err);
        }

        unsafe {
            // Create a console window in case ConPTY is running in a GUI application.
            let valid = AllocConsole().as_bool();
            if valid {
                ShowWindow(GetConsoleWindow(), SW_HIDE);
            }

            // Recreate the standard stream inputs in case the parent process
            // has redirected them.
            let conout_name = OsString::from("CONOUT$\0");
            let mut conout_vec: Vec<u16> = conout_name.encode_wide().collect();
            let conout_pwstr = PWSTR(conout_vec.as_mut_ptr());

            let h_console = CreateFileW(
                conout_pwstr, FILE_GENERIC_READ | FILE_GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                ptr::null(), OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, HANDLE(0));

            if h_console == INVALID_HANDLE_VALUE {
                result = GetLastError().into();
            }

            if result.is_err() {
                let result_msg = result.message();
                let err_msg: &[u16] = result_msg.as_wide();
                let string = OsString::from_wide(err_msg);
                return Err(string);
            }

            let conin_name = OsString::from("CONIN$\0");
            let mut conin_vec: Vec<u16> = conin_name.encode_wide().collect();
            let conin_pwstr = PWSTR(conin_vec.as_mut_ptr());

            let h_in = CreateFileW(
                conin_pwstr,
                FILE_GENERIC_READ | FILE_GENERIC_WRITE,
                FILE_SHARE_READ, ptr::null(),
                OPEN_EXISTING, FILE_FLAGS_AND_ATTRIBUTES(0), HANDLE(0));

            if h_in == INVALID_HANDLE_VALUE {
                result = GetLastError().into();
            }

            if result.is_err() {
                let result_msg = result.message();
                let err_msg: &[u16] = result_msg.as_wide();
                let string = OsString::from_wide(err_msg);
                return Err(string);
            }

            let mut console_mode_un = MaybeUninit::<CONSOLE_MODE>::uninit();
            let console_mode_ptr = console_mode_un.as_mut_ptr();

            result =
                if GetConsoleMode(h_console, console_mode_ptr).as_bool() {
                    S_OK
                } else {
                    GetLastError().into()
                };

            if result.is_err() {
                let result_msg = result.message();
                let err_msg: &[u16] = result_msg.as_wide();
                let string = OsString::from_wide(err_msg);
                return Err(string);
            }

            let console_mode = console_mode_un.assume_init();

            // Enable stream to accept VT100 input sequences
            result =
                if SetConsoleMode(h_console, console_mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING).as_bool() {
                    S_OK
                } else {
                    GetLastError().into()
                };

            if result.is_err() {
                let result_msg = result.message();
                let err_msg: &[u16] = result_msg.as_wide();
                let string = OsString::from_wide(err_msg);
                return Err(string);
            }

            // Set new streams
            result = if SetStdHandle(STD_OUTPUT_HANDLE, h_console).as_bool() {S_OK} else {GetLastError().into()};

            if result.is_err() {
                let result_msg = result.message();
                let err_msg: &[u16] = result_msg.as_wide();
                let string = OsString::from_wide(err_msg);
                return Err(string);
            }

            result = if SetStdHandle(STD_ERROR_HANDLE, h_console).as_bool() {S_OK} else {GetLastError().into()};

            if result.is_err() {
                let result_msg = result.message();
                let err_msg: &[u16] = result_msg.as_wide();
                let string = OsString::from_wide(err_msg);
                return Err(string);
            }

            result = if SetStdHandle(STD_INPUT_HANDLE, h_in).as_bool() {S_OK} else {GetLastError().into()};
            println!("{:?}", h_in);
            if result.is_err() {
                let result_msg = result.message();
                let err_msg: &[u16] = result_msg.as_wide();
                let string = OsString::from_wide(err_msg);
                return Err(string);
            }

            // Create communication channels
            // - Close these after CreateProcess of child application with pseudoconsole object.
            let mut input_read_side = INVALID_HANDLE_VALUE;
            let mut output_write_side = INVALID_HANDLE_VALUE;
            //let mut input_read_side_u = MaybeUninit::<HANDLE>::uninit();
            //let input_read_side_p = input_read_side_u.as_mut_ptr();
            // let mut output_write_side_u = MaybeUninit::<HANDLE>::uninit();
            // let output_write_side_p = output_write_side_u.as_mut_ptr();

            // - Hold onto these and use them for communication with the child through the pseudoconsole.
            let mut output_read_side = INVALID_HANDLE_VALUE;
            let mut input_write_side = INVALID_HANDLE_VALUE;
            // let mut output_read_side_u = MaybeUninit::<HANDLE>::uninit();
            // let output_read_side_p = output_read_side_u.as_mut_ptr();
            // let mut input_write_side_u = MaybeUninit::<HANDLE>::uninit();
            // let input_write_side_p = input_write_side_u.as_mut_ptr();

            // Setup PTY size
            let size = COORD {X: args.cols as i16, Y: args.rows as i16};

            if !CreatePipe(&mut input_read_side, &mut input_write_side, ptr::null(), 0).as_bool() {
                result = GetLastError().into();
                let result_msg = result.message();
                let err_msg: &[u16] = result_msg.as_wide();
                let string = OsString::from_wide(err_msg);
                return Err(string);
            }

            // let input_read_side = input_read_side_u.assume_init();
            // let input_write_side = input_write_side_u.assume_init();

            if !CreatePipe(&mut output_read_side, &mut output_write_side, ptr::null(), 0).as_bool() {
                result = GetLastError().into();
                let result_msg = result.message();
                let err_msg: &[u16] = result_msg.as_wide();
                let string = OsString::from_wide(err_msg);
                return Err(string);
            }

            // let output_read_side = output_read_side_u.assume_init();
            // let output_write_side = output_write_side_u.assume_init();

            let pty_handle =
                match CreatePseudoConsole(size, input_read_side, output_write_side, 0) {
                    Ok(pty) => pty,
                    Err(err) => {
                        let result_msg = err.message();
                        let err_msg: &[u16] = result_msg.as_wide();
                        let string = OsString::from_wide(err_msg);
                        return Err(string);
                    }
                };

            CloseHandle(input_read_side);
            CloseHandle(output_write_side);

            let pty_process = PTYProcess::new(input_write_side, output_read_side, true);

            Ok(Box::new(ConPTY {
                handle: pty_handle,
                process_info: PROCESS_INFORMATION::default(),
                startup_info: STARTUPINFOEXW::default(),
                process: pty_process
            }) as Box<dyn PTYImpl>)
        }
    }

    fn spawn(&mut self, appname: OsString, cmdline: Option<OsString>, cwd: Option<OsString>, env: Option<OsString>) -> Result<bool, OsString> {
        let result: HRESULT;
        let mut environ: *const u16 = ptr::null();
        let mut working_dir: *mut u16 = ptr::null_mut();

        let mut cmdline_oss = OsString::new();
        cmdline_oss.push("");
        let mut cmdline_oss_buf: Vec<u16> = cmdline_oss.encode_wide().collect();
        let mut cmd = cmdline_oss_buf.as_mut_ptr();

        if let Some(env_opt) = env {
            let env_buf: Vec<u16> = env_opt.encode_wide().collect();
            environ = env_buf.as_ptr();
        }

        if let Some(cwd_opt) = cwd {
            let mut cwd_buf: Vec<u16> = cwd_opt.encode_wide().collect();
            working_dir = cwd_buf.as_mut_ptr();
        }

        if let Some(cmdline_opt) = cmdline {
            let mut cmd_buf: Vec<u16> = cmdline_opt.encode_wide().collect();
            cmd = cmd_buf.as_mut_ptr();
        }

        let mut app_buf: Vec<u16> = appname.encode_wide().collect();
        app_buf.push(0);
        let app = app_buf.as_mut_ptr();

        unsafe {
            println!("{:?}", GetStdHandle(STD_INPUT_HANDLE));
            // Discover the size required for the list
            let mut required_bytes_u = MaybeUninit::<usize>::uninit();
            let required_bytes_ptr = required_bytes_u.as_mut_ptr();
            InitializeProcThreadAttributeList(LPPROC_THREAD_ATTRIBUTE_LIST::default(), 1, 0, required_bytes_ptr);

            // Allocate memory to represent the list
            let mut required_bytes = required_bytes_u.assume_init();
            let mut lp_attribute_list: Box<[u8]> = vec![0; required_bytes].into_boxed_slice();
            let proc_thread_list = LPPROC_THREAD_ATTRIBUTE_LIST(lp_attribute_list.as_mut_ptr().cast::<_>());

            // Prepare Startup Information structure
            // let mut siEx = STARTUPINFOEXW::default();
            // siEx.StartupInfo.cb = mem::size_of::<STARTUPINFOEXW>() as u32;

            // let mut size: usize = 0;
            // InitializeProcThreadAttributeList(LPPROC_THREAD_ATTRIBUTE_LIST::default(), 1, 0, &mut size);

            // let lpAttributeList = vec![0u8; size].into_boxed_slice();
            // let lpAttributeList = Box::leak(lpAttributeList);

            // siEx.lpAttributeList = LPPROC_THREAD_ATTRIBUTE_LIST(lpAttributeList.as_mut_ptr().cast::<_>());

            let mut start_info = STARTUPINFOEXW {
                StartupInfo: STARTUPINFOW {
                    cb: mem::size_of::<STARTUPINFOEXW>() as u32,
                    ..Default::default()
                },
                lpAttributeList: proc_thread_list,
            };

            // start_info.StartupInfo.dwFlags |= STARTF_USESTDHANDLES;

            // Initialize the list memory location
            if !InitializeProcThreadAttributeList(start_info.lpAttributeList, 1, 0, &mut required_bytes).as_bool() {
                result = GetLastError().into();
                let result_msg = result.message();
                let err_msg: &[u16] = result_msg.as_wide();
                let string = OsString::from_wide(err_msg);
                return Err(string);
            }

            // let handle_ptr = &self.handle as *const HPCON;

            // Set the pseudoconsole information into the list
            if !UpdateProcThreadAttribute(
                    start_info.lpAttributeList, 0, 0x00020016,
                    self.handle.0 as _, mem::size_of::<HPCON>(),
                    ptr::null_mut(), ptr::null_mut()).as_bool() {
                result = GetLastError().into();
                let result_msg = result.message();
                let err_msg: &[u16] = result_msg.as_wide();
                let string = OsString::from_wide(err_msg);
                return Err(string);
            }

            self.startup_info = start_info;
            //let mut pi = Box::new(self.process_info);
            let pi_ptr = &mut self.process_info as *mut _;

            let si = self.startup_info.StartupInfo;
            let si_ptr = &si as *const STARTUPINFOW;
            // let si_ptr = &start_info as *const STARTUPINFOEXW;

            let succ = CreateProcessW(
                PWSTR(ptr::null_mut()),
                PWSTR(app),
                ptr::null_mut(),
                ptr::null_mut(),
                false,
                EXTENDED_STARTUPINFO_PRESENT | CREATE_UNICODE_ENVIRONMENT,
                environ as _,
                PWSTR(working_dir),
                si_ptr as *const _,
                pi_ptr
            ).as_bool();

            if !succ {
                result = GetLastError().into();
                let result_msg = result.message();
                let err_msg: &[u16] = result_msg.as_wide();
                let string = OsString::from_wide(err_msg);
                return Err(string);
            }

            self.process.set_process(self.process_info.hProcess);
            println!("{:?}", self.process.get_exitstatus().unwrap());
            Ok(true)
        }
    }

    fn set_size(&self, cols: i32, rows: i32) -> Result<(), OsString> {
        if cols <= 0 || rows <= 0 {
            let err: OsString = OsString::from(format!(
                "PTY cols and rows must be positive and non-zero. Got: ({}, {})", cols, rows));
            return Err(err);
        }

        let size = COORD {X: cols as i16, Y: rows as i16};
        unsafe {
            match ResizePseudoConsole(self.handle, size) {
                Ok(_) => Ok(()),
                Err(err) => {
                    let result_msg = err.message();
                    let err_msg: &[u16] = result_msg.as_wide();
                    let string = OsString::from_wide(err_msg);
                    Err(string)
                }
            }
        }
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

impl Drop for ConPTY {
    fn drop(&mut self) {
       unsafe {
           CloseHandle(self.process_info.hThread);
           CloseHandle(self.process_info.hProcess);

           DeleteProcThreadAttributeList(self.startup_info.lpAttributeList);

           ClosePseudoConsole(self.handle);
           FreeConsole();
        }
    }
}
