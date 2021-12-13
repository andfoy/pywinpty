/// Base struct used to generalize some of the PTY I/O operations.

use windows::Win32::Foundation::{PWSTR, HANDLE, S_OK, GetLastError, STATUS_PENDING, BOOL};
use windows::Win32::Storage::FileSystem::{GetFileSizeEx, ReadFile, WriteFile};
use windows::Win32::System::Pipes::PeekNamedPipe;
use windows::Win32::System::IO::OVERLAPPED;
use windows::Win32::System::Threading::{GetExitCodeProcess, GetProcessId};
use windows::core::HRESULT;

use std::ptr;
use std::cmp::min;
use std::ffi::{OsString, c_void};
use std::os::windows::prelude::*;
use std::os::windows::ffi::OsStrExt;


fn read(mut length: u32, blocking: bool, stream: HANDLE, using_pipes: bool) -> Result<OsString, OsString> {
    let mut result: HRESULT;
    if !blocking {
        if using_pipes {
            let mut bytes_ptr: *mut u32 = ptr::null_mut();
            //let mut available_bytes = Box::<>::new_uninit();
            //let bytes_ptr: *mut u32 = &mut *available_bytes;
            unsafe {
                result =
                    if PeekNamedPipe(stream, ptr::null_mut::<c_void>(), 0,
                                     ptr::null_mut::<u32>(), bytes_ptr, ptr::null_mut::<u32>()).as_bool() {
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
                length = min(length, *bytes_ptr);
            }
        } else {
            //let mut size: Box<i64> = Box::new_uninit();
            //let size_ptr: *mut i64 = &mut *size;
            let size_ptr: *mut i64 = ptr::null_mut();
            unsafe {
                result = if GetFileSizeEx(stream, size_ptr).as_bool() { S_OK } else { GetLastError().into() };

                if result.is_err() {
                    let result_msg = result.message();
                    let err_msg: &[u16] = result_msg.as_wide();
                    let string = OsString::from_wide(err_msg);
                    return Err(string);
                }
                length = min(length, *size_ptr as u32);
            }
        }
    }

    let mut buf: Vec<u16> = Vec::with_capacity((length + 1) as usize);
    buf.fill(0);
    let chars_read: *mut u32 = ptr::null_mut();
    let null_overlapped: *mut OVERLAPPED = ptr::null_mut();
    unsafe {
        result =
            if ReadFile(stream, buf[..].as_mut_ptr() as *mut c_void, length, chars_read, null_overlapped).as_bool() {
                S_OK
            } else {
                GetLastError().into()
            };
    }
    if result.is_err() {
        let result_msg = result.message();
        let err_msg: &[u16] = result_msg.as_wide();
        let string = OsString::from_wide(err_msg);
        return Err(string);
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
    exitstatus: Option<u32>,
    /// Attribute that declares if the process is alive.
    alive: bool,
    /// Process is using Windows pipes and not files.
    using_pipes: bool
}

impl PTYProcess {
    pub fn new(process: HANDLE, conin: HANDLE, conout: HANDLE, using_pipes: bool) -> PTYProcess {
        PTYProcess {
            process: process,
            conin: conin,
            conout: conout,
            pid: 0,
            exitstatus: None,
            alive: false,
            using_pipes: using_pipes

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
        let bytes_ptr: *mut u32 = ptr::null_mut();
        let vec_buf: Vec<u16> = buf.encode_wide().collect();
        let null_overlapped: *mut OVERLAPPED = ptr::null_mut();
        let result: HRESULT;
        unsafe {
            result =
                if WriteFile(self.conin, vec_buf[..].as_ptr() as *const c_void, vec_buf.len() as u32, bytes_ptr, null_overlapped).as_bool() {
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
            Ok(*bytes_ptr)
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
        let bytes_ptr: *mut u32 = ptr::null_mut();

        unsafe {
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
                    (false, GetLastError().into())
                };

                if result.is_err() {
                    let result_msg = result.message();
                    let err_msg: &[u16] = result_msg.as_wide();
                    let string = OsString::from_wide(err_msg);
                    return Err(string);
                }
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
        let exit_ptr: *mut u32 = ptr::null_mut();
        unsafe {
            let succ = GetExitCodeProcess(self.process, exit_ptr).as_bool();

            if succ {
                let actual_exit = *exit_ptr;
                self.alive = actual_exit == STATUS_PENDING.0;
                if !self.alive {
                    self.exitstatus = Some(actual_exit);
                }
                Ok(self.alive)
            } else {
                let err: HRESULT = GetLastError().into();
                let result_msg = err.message();
                let err_msg: &[u16] = result_msg.as_wide();
                let string = OsString::from_wide(err_msg);
                Err(string)
            }
        }
    }

    pub fn retrieve_pid(mut self) {
        unsafe {
            self.pid = GetProcessId(self.process);
            self.alive = true;
        }
    }

}
