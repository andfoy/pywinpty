/// Base struct used to generalize some of the PTY I/O operations.

use windows::Win32::Foundation::{PWSTR, HANDLE, S_OK, GetLastError, STATUS_PENDING};
use windows::Win32::Storage::FileSystem::{GetFileSizeEx, ReadFile};
use windows::Win32::System::Pipes::PeekNamedPipe;
use windows::Win32::System::Threading::{GetExitCodeProcess, GetProcessId};
use windows::runtime::HRESULT;

use std::ptr;
use std::cmp::min;
use std::ffi::OsString;
use std::os::windows::prelude::*;
use std::os::windows::ffi::OsStrExt;


fn read(mut length: u32, blocking: bool, stream: HANDLE, using_pipes: bool) -> Result<OsString, OsString> {
    if !blocking {
        if using_pipes {
            let mut available_bytes: Box<u32> = Box::new_uninit();
            let bytes_ptr: *mut u32 = &mut *available_bytes;
            unsafe {
                let result: HRESULT =
                    if PeekNamedPipe(stream, ptr::null(), 0, ptr::null(), bytes_ptr, ptr::null()) {
                        S_OK
                    } else {
                        GetLastError()
                    };
                available_bytes.assume_init();
            }

            if (result.is_err()) {
                let err_msg: &[u16] = result.message().as_wide();
                let string = OsString::from_wide(err_msg);
                Err(string)
            }
            length = min(length, *available_bytes);
        } else {
            let mut size: Box<i64> = Box::new_uninit();
            let size_ptr: *mut i64 = &mut *size;
            unsafe {
                let result: HRESULT = if GetFileSizeEx(stream, size_ptr) { S_OK } else { GetLastError() };
                size.assume_init();
            }
            if (result.is_err()) {
                let err_msg: &[u16] = result.message().as_wide();
                let string = OsString::from_wide(err_msg);
                Err(string)
            }
            length = min(length, *size);
        }
    }

    let mut buf: Vec<u16> = Vec::with_capacity(length + 1);
    buf.fill(0);
    unsafe {
        let result: HRESULT =
            if ReadFile(stream, &buf[..].as_ptr(), length, ptr::null()) {
                S_OK
            } else {
                GetLastError()
            };
    }
    if (result.is_err()) {
        let err_msg: &[u16] = result.message().as_wide();
        let string = OsString::from_wide(err_msg);
        Err(string)
    }
    let os_str = OsString::from_wide(&buf[..]);
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
    exitstatus: i32,
    /// Attribute that declares if the process is alive.
    alive: bool,
    /// Process is using Windows pipes and not files.
    using_pipes: bool
}

impl PTYProcess {
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
        let mut num_bytes: Box<u32> = Box::new_uninit();
        let vec_buf: Vec<u16> = buf.encode_wide().collect();
        let bytes_ptr: *mut u32 = &mut *num_bytes;
        unsafe {
            let result: HRESULT =
                if WriteFile(self.conin, &vec_buf[..].as_ptr(), vec_buf.size(), bytes_ptr, ptr::null()) {
                    S_OK
                } else {
                    GetLastError()
                };
            num_bytes.assume_init();
        }
        if (result.is_err()) {
            let err_msg: &[u16] = result.message().as_wide();
            let string = OsString::from_wide(err_msg);
            Err(string)
        }
        Ok(*num_bytes)
    }

    /// Check if a process reached End-of-File (EOF).
    ///
    /// # Returns
    /// `true` if the process reached EOL, false otherwise. If an error occurs, then a [`OsString`]
    /// containing a human-readable error is raised.
    pub fn is_eof(&self) -> Result<bool, OsString> {
        let mut available_bytes: Box<u32> = Box::new_uninit();
        let bytes_ptr: *mut u32 = &mut *available_bytes;

        unsafe {
            let (succ, result) =
                if PeekNamedPipe(self.conout, ptr::null(), 0, ptr::null(), bytes_ptr, ptr::null()) {
                    if *available_bytes == 0 && !self.is_alive() {
                        (false, S_OK)
                    }
                    (true, S_OK)
                } else {
                    (false, GetLastError() as HRESULT)
                }
                available_bytes.assume_init();
        }

        if (result.is_err()) {
            let err_msg: &[u16] = result.message().as_wide();
            let string = OsString::from_wide(err_msg);
            Err(string)
        }
        Ok(succ)
    }

    /// Retrieve the exit status of the process
    ///
    /// # Returns
    /// `-1` if the process has not exited, else the exit code of the process.
    pub fn get_exitstatus(&self) -> Result<i32, OsString> {
        if self.pid == 0 {
            return -1;
        }
        if self.alive == 1 {
            self.is_alive();
        }
        if self.alive == 1 {
            return -1;
        }
        return self.exitstatus;
    }

    /// Determine if the process is still alive.
    pub fn is_alive(&self) -> Result<bool, OsString> {
        let mut exit_code: Box<u32> = Box::new_uninit();
        let exit_ptr: *mut u32 = &mut *exit_code;
        unsafe {
            let succ = GetExitCodeProcess(self.process, exit_ptr);
            exit_code.assume_init();
        }

        if succ {
            let actual_exit = *exit_code;
            self.alive = actual_exit == STATUS_PENDING;
            if !self.alive {
                self.exitstatus() = actual_exit;
            }
            Ok(self.alive)
        } else {
            let err: HRESULT = GetLastError();
            let err_msg: &[u16] = err.message().as_wide();
            let string = OsString::from_wide(err_msg);
            Err(string)
        }
    }

    pub fn retrieve_pid(&self) {
        unsafe {
            self.pid = GetProcessId(self.process);
            self.alive = true;
        }
    }

}
