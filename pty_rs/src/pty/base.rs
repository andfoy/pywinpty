/// Base struct used to generalize some of the PTY I/O operations.

use windows::Win32::Foundation::{HANDLE, S_OK, STATUS_PENDING, CloseHandle, PSTR, PWSTR};
use windows::Win32::Storage::FileSystem::{GetFileSizeEx, ReadFile, WriteFile};
use windows::Win32::System::Pipes::PeekNamedPipe;
use windows::Win32::System::IO::OVERLAPPED;
use windows::Win32::System::Threading::{GetExitCodeProcess, GetProcessId};
use windows::Win32::Globalization::{MultiByteToWideChar, WideCharToMultiByte, CP_UTF8};
use windows::core::{HRESULT, Error};

use std::ptr;
use std::mem::MaybeUninit;
use std::cmp::min;
use std::ffi::{OsString, c_void};
use std::os::windows::prelude::*;
use std::os::windows::ffi::OsStrExt;

use super::PTYArgs;

/// This trait should be implemented by any backend that wants to provide a PTY implementation.
pub trait PTYImpl {
    /// Create a new instance of the PTY backend.
    ///
    /// # Arguments
    /// * `args` - Arguments used to initialize the backend struct.
    ///
    /// # Returns
    /// * `pty`: The instantiated PTY struct.
    #[allow(clippy::new_ret_no_self)]
    fn new(args: &PTYArgs) -> Result<Box<dyn PTYImpl>, OsString>
        where Self: Sized;

    /// Spawn a process inside the PTY.
	///
	/// # Arguments
	/// * `appname` - Full path to the executable binary to spawn.
	/// * `cmdline` - Optional space-delimited arguments to provide to the executable.
	/// * `cwd` - Optional path from where the executable should be spawned.
	/// * `env` - Optional environment variables to provide to the process. Each
	/// variable should be declared as `VAR=VALUE` and be separated by a NUL (0) character.
	///
	/// # Returns
	/// `true` if the call was successful, else an error will be returned.
    fn spawn(&mut self, appname: OsString, cmdline: Option<OsString>, cwd: Option<OsString>, env: Option<OsString>) -> Result<bool, OsString>;

    /// Change the PTY size.
    ///
    /// # Arguments
    /// * `cols` - Number of character columns to display.
    /// * `rows` - Number of line rows to display.
    fn set_size(&self, cols: i32, rows: i32) -> Result<(), OsString>;

    /// Read at most `length` characters from a process standard output.
    ///
    /// # Arguments
    /// * `length` - Upper limit on the number of characters to read.
    /// * `blocking` - Block the reading thread if no bytes are available.
    ///
    /// # Notes
    /// * If `blocking = false`, then the function will check how much characters are available on
    /// the stream and will read the minimum between the input argument and the total number of
    /// characters available.
    ///
    /// * The bytes returned are represented using a [`OsString`] since Windows operates over
    /// `u16` strings.
    fn read(&self, length: u32, blocking: bool) -> Result<OsString, OsString>;

    /// Write a (possibly) UTF-16 string into the standard input of a process.
    ///
    /// # Arguments
    /// * `buf` - [`OsString`] containing the string to write.
    ///
    /// # Returns
    /// The total number of characters written if the call was successful, else
    /// an [`OsString`] containing an human-readable error.
    fn write(&self, buf: OsString) -> Result<u32, OsString>;

    /// Check if a process reached End-of-File (EOF).
    ///
    /// # Returns
    /// `true` if the process reached EOL, false otherwise. If an error occurs, then a [`OsString`]
    /// containing a human-readable error is raised.
    fn is_eof(&mut self) -> Result<bool, OsString>;

    /// Retrieve the exit status of the process
    ///
    /// # Returns
    /// `None` if the process has not exited, else the exit code of the process.
    fn get_exitstatus(&mut self) -> Result<Option<u32>, OsString>;

    /// Determine if the process is still alive.
    fn is_alive(&mut self) -> Result<bool, OsString>;
}


fn read(mut length: u32, blocking: bool, stream: HANDLE, using_pipes: bool) -> Result<OsString, OsString> {
    let mut result: HRESULT;
    if !blocking {
        if using_pipes {
            let mut bytes_u = MaybeUninit::<u32>::uninit();
            let bytes_ptr = bytes_u.as_mut_ptr();
            //let mut available_bytes = Box::<>::new_uninit();
            //let bytes_ptr: *mut u32 = &mut *available_bytes;
            unsafe {
                result =
                    if PeekNamedPipe(stream, ptr::null_mut::<c_void>(), 0,
                                     ptr::null_mut::<u32>(), bytes_ptr, ptr::null_mut::<u32>()).as_bool() {
                        S_OK
                    } else {
                        Error::from_win32().into()
                    };


                if result.is_err() {
                    let result_msg = result.message();
                    let err_msg: &[u16] = result_msg.as_wide();
                    let string = OsString::from_wide(err_msg);
                    return Err(string);
                }
                let num_bytes = bytes_u.assume_init();
                length = min(length, num_bytes);
            }
        } else {
            //let mut size: Box<i64> = Box::new_uninit();
            //let size_ptr: *mut i64 = &mut *size;
            let mut size = MaybeUninit::<i64>::uninit();
            // let size_ptr: *mut i64 = ptr::null_mut();
            unsafe {
                let size_ptr = ptr::addr_of_mut!(*size.as_mut_ptr());
                result = if GetFileSizeEx(stream, size_ptr).as_bool() { S_OK } else { Error::from_win32().into() };

                if result.is_err() {
                    let result_msg = result.message();
                    let err_msg: &[u16] = result_msg.as_wide();
                    let string = OsString::from_wide(err_msg);
                    return Err(string);
                }
                length = min(length, *size_ptr as u32);
                size.assume_init();
            }
        }
    }

    //let mut buf: Vec<u16> = Vec::with_capacity((length + 1) as usize);
    //buf.fill(1);
    let os_str = "\0".repeat((length + 1) as usize);
    let mut buf_vec: Vec<u8> = os_str.as_str().as_bytes().to_vec();
    let mut chars_read = MaybeUninit::<u32>::uninit();
    let total_bytes: u32;
    //let chars_read: *mut u32 = ptr::null_mut();
    let null_overlapped: *mut OVERLAPPED = ptr::null_mut();
    // if length > 0 {
        unsafe {
            let buf_ptr = buf_vec.as_mut_ptr();
            let buf_void = buf_ptr as *mut c_void;
            let chars_read_ptr = ptr::addr_of_mut!(*chars_read.as_mut_ptr());
            result =
                if ReadFile(stream, buf_void, length, chars_read_ptr, null_overlapped).as_bool() {
                    S_OK
                } else {
                    Error::from_win32().into()
                };
            total_bytes = *chars_read_ptr;
            chars_read.assume_init();

            if result.is_err() {
                let result_msg = result.message();
                let err_msg: &[u16] = result_msg.as_wide();
                let string = OsString::from_wide(err_msg);
                return Err(string);
            }
        }
    // }

    // let os_str = OsString::with_capacity(buf_vec.len());
    let mut vec_buf: Vec<u16> = std::iter::repeat(0).take(buf_vec.len()).collect();
    let vec_ptr = vec_buf.as_mut_ptr();
    let pstr = PSTR(buf_vec.as_mut_ptr());
    let pwstr = PWSTR(vec_ptr);
    unsafe {
        MultiByteToWideChar(CP_UTF8, 0, pstr, -1, pwstr, (total_bytes + 1) as i32);
    }

    let os_str = OsString::from_wide(&vec_buf[..]);
    Ok(os_str)
}

/// This struct handles the I/O operations to the standard streams, as well
/// the lifetime of a process running inside a PTY.
pub struct PTYProcess {
    /// Handle to the process to read from.
    process: HANDLE,
    /// Handle to the standard input stream.
    conin: HANDLE,
    /// Handle to the standard output stream.
    conout: HANDLE,
    /// Identifier of the process running inside the PTY.
    pid: u32,
    /// Exit status code of the process running inside the PTY.
    exitstatus: Option<u32>,
    /// Attribute that declares if the process is alive.
    alive: bool,
    /// Process is using Windows pipes and not files.
    using_pipes: bool,
    // Close process when the struct is dropped.
    close_process: bool
}

impl PTYProcess {
    pub fn new(conin: HANDLE, conout: HANDLE, using_pipes: bool) -> PTYProcess {
        PTYProcess {
            process: HANDLE(0),
            conin,
            conout,
            pid: 0,
            exitstatus: None,
            alive: false,
            using_pipes,
            close_process: true
        }
    }

    /// Read at least `length` characters from a process standard output.
    ///
    /// # Arguments
    /// * `length` - Upper limit on the number of characters to read.
    /// * `blocking` - Block the reading thread if no bytes are available.
    ///
    /// # Notes
    /// * If `blocking = false`, then the function will check how much characters are available on
    /// the stream and will read the minimum between the input argument and the total number of
    /// characters available.
    ///
    /// * The bytes returned are represented using a [`OsString`] since Windows operates over
    /// `u16` strings.
    pub fn read(&self, length: u32, blocking: bool) -> Result<OsString, OsString> {
        read(length, blocking, self.conout, self.using_pipes)
    }

    /// Write an (possibly) UTF-16 string into the standard input of a process.
    ///
    /// # Arguments
    /// * `buf` - [`OsString`] containing the string to write.
    ///
    /// # Returns
    /// The total number of characters written if the call was successful, else
    /// an [`OsString`] containing an human-readable error.
    pub fn write(&self, buf: OsString) -> Result<u32, OsString> {
        let mut vec_buf: Vec<u16> = buf.encode_wide().collect();
        vec_buf.push(0);

        let null_overlapped: *mut OVERLAPPED = ptr::null_mut();
        let result: HRESULT;

        unsafe {
            let pwstr = PWSTR(vec_buf.as_mut_ptr());
            let required_size = WideCharToMultiByte(
                CP_UTF8, 0, pwstr, -1, PSTR(ptr::null_mut::<u8>()),
                0, PSTR(ptr::null_mut::<u8>()), ptr::null_mut::<i32>());

            let mut bytes_buf: Vec<u8> = std::iter::repeat(0).take((required_size) as usize).collect();
            let bytes_buf_ptr = bytes_buf.as_mut_ptr();
            let pstr = PSTR(bytes_buf_ptr);

            WideCharToMultiByte(CP_UTF8, 0, pwstr, -1, pstr, required_size, PSTR(ptr::null_mut::<u8>()), ptr::null_mut::<i32>());

            let mut written_bytes = MaybeUninit::<u32>::uninit();
            let bytes_ptr: *mut u32 = ptr::addr_of_mut!(*written_bytes.as_mut_ptr());

            result =
                if WriteFile(self.conin, bytes_buf[..].as_ptr() as *const c_void, bytes_buf.len() as u32, bytes_ptr, null_overlapped).as_bool() {
                    S_OK
                } else {
                    Error::from_win32().into()
                };

            if result.is_err() {
                let result_msg = result.message();
                let err_msg: &[u16] = result_msg.as_wide();
                let string = OsString::from_wide(err_msg);
                return Err(string);
            }
            let total_bytes = written_bytes.assume_init();
            Ok(total_bytes)
        }
    }

    /// Check if a process reached End-of-File (EOF).
    ///
    /// # Returns
    /// `true` if the process reached EOL, false otherwise. If an error occurs, then a [`OsString`]
    /// containing a human-readable error is raised.
    pub fn is_eof(&mut self) -> Result<bool, OsString> {
        // let mut available_bytes: Box<u32> = Box::new_uninit();
        // let bytes_ptr: *mut u32 = &mut *available_bytes;
        // let bytes_ptr: *mut u32 = ptr::null_mut();
        let mut bytes = MaybeUninit::<u32>::uninit();
        unsafe {
            let bytes_ptr: *mut u32 = ptr::addr_of_mut!(*bytes.as_mut_ptr());
            let (succ, result) =
                if PeekNamedPipe(self.conout, ptr::null_mut::<c_void>(), 0,
                                 ptr::null_mut::<u32>(), bytes_ptr, ptr::null_mut::<u32>()).as_bool() {
                    let is_alive =
                        match self.is_alive() {
                            Ok(alive) => alive,
                            Err(err) => {
                                return Err(err);
                            }
                        };

                    if *bytes_ptr == 0 && !is_alive {
                        (false, S_OK)
                    } else {
                        (true, S_OK)
                    }
                } else {
                    (false, Error::from_win32().into())
                };

                if result.is_err() {
                    let result_msg = result.message();
                    let err_msg: &[u16] = result_msg.as_wide();
                    let string = OsString::from_wide(err_msg);
                    return Err(string);
                }
                bytes.assume_init();
                Ok(succ)
        }

    }

    /// Retrieve the exit status of the process
    ///
    /// # Returns
    /// `None` if the process has not exited, else the exit code of the process.
    pub fn get_exitstatus(&mut self) -> Result<Option<u32>, OsString> {
        if self.pid == 0 {
            return Ok(None);
        }
        if self.alive {
            match self.is_alive() {
                Ok(_) => {},
                Err(err) => {
                    return Err(err)
                }
            }
        }
        if self.alive {
            return Ok(None);
        }

        match self.exitstatus {
            Some(exit) => Ok(Some(exit)),
            None => Ok(None)
        }
    }

    /// Determine if the process is still alive.
    pub fn is_alive(&mut self) -> Result<bool, OsString> {
        // let mut exit_code: Box<u32> = Box::new_uninit();
        // let exit_ptr: *mut u32 = &mut *exit_code;
        let mut exit = MaybeUninit::<u32>::uninit();
        unsafe {
            let exit_ptr: *mut u32 = ptr::addr_of_mut!(*exit.as_mut_ptr());
            let succ = GetExitCodeProcess(self.process, exit_ptr).as_bool();

            if succ {
                let actual_exit = *exit_ptr;
                exit.assume_init();
                self.alive = actual_exit == STATUS_PENDING.0 as u32;
                if !self.alive {
                    self.exitstatus = Some(actual_exit);
                }
                Ok(self.alive)
            } else {
                let err: HRESULT = Error::from_win32().into();
                let result_msg = err.message();
                let err_msg: &[u16] = result_msg.as_wide();
                let string = OsString::from_wide(err_msg);
                Err(string)
            }
        }
    }

    /// Set the running process behind the PTY.
    pub fn set_process(&mut self, process: HANDLE, close_process: bool) {
        self.process = process;
        self.close_process = close_process;
        unsafe {
            self.pid = GetProcessId(self.process);
            self.alive = true;
        }
    }

}

impl Drop for PTYProcess {
    fn drop(&mut self) {
        unsafe {
            if !self.conin.is_invalid() {
                CloseHandle(self.conin);
            }

            if !self.conout.is_invalid() {
                CloseHandle(self.conout);
            }

            if self.close_process && !self.process.is_invalid() {
                CloseHandle(self.process);
            }
        }
    }
}
