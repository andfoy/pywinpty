#![allow(non_camel_case_types)]
/// WinPTY C bindings.

/*
 * Copyright (c) 2011-2016 Ryan Prichard
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to
 * deal in the Software without restriction, including without limitation the
 * rights to use, copy, modify, merge, publish, distribute, sublicense, and/or
 * sell copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
 * FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS
 * IN THE SOFTWARE.
 */

//use std::ptr;
// use std::ffi::c_void;
use std::os::windows::raw::HANDLE;
//use windows::Win32::Foundation::HANDLE;

// Error handling
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct winpty_error_s {
    _unused: [u8; 0],
}

/// An error object.
pub type winpty_error_t = winpty_error_s;
pub type winpty_error_ptr_t = *mut winpty_error_t;

extern "C" {
    /// Gets the error code from the error object.
    // pub fn winpty_error_code(err: *mut winpty_error_ptr_t) -> u32;

    /// Returns a textual representation of the error.  The string is freed when
    /// the error is freed.
    pub fn winpty_error_msg(err: *mut winpty_error_ptr_t) -> *const u16;

    /// Free the error object.  Every error returned from the winpty API must be
    /// freed.
    pub fn winpty_error_free(err: *mut winpty_error_ptr_t);
}

// Configuration of a new agent.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct winpty_config_s {
    _unused: [u8; 0],
}
/// Agent configuration object (not thread-safe).
pub type winpty_config_t = winpty_config_s;

extern "C" {
    /// Allocate a winpty_config_t value.  Returns NULL on error.  There are no
    /// required settings -- the object may immediately be used.  agentFlags is a
    /// set of zero or more WINPTY_FLAG_xxx values.  An unrecognized flag results
    /// in an assertion failure.
    pub fn winpty_config_new(flags: u64, err: *mut winpty_error_ptr_t) -> *mut winpty_config_t;

    /// Free the cfg object after passing it to winpty_open.
    pub fn winpty_config_free(cfg: *mut winpty_config_t);

    /// Set the agent config size.
    pub fn winpty_config_set_initial_size(cfg: *mut winpty_config_t, cols: i32, rows: i32);

    /// Set the mouse mode to one of the [`super::MouseMode`] constants.
    pub fn winpty_config_set_mouse_mode(cfg: *mut winpty_config_t, mouse_mode: i32);

    /// Amount of time to wait for the agent to startup and to wait for any given
    /// agent RPC request.  Must be greater than 0.  Can be INFINITE.
    pub fn winpty_config_set_agent_timeout(cfg: *mut winpty_config_t, timeout: u32);
}

// Start the agent

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct winpty_s {
    _unused: [u8; 0],
}

/// Agent object (thread-safe)
pub type winpty_t = winpty_s;

extern "C" {
    ///  Starts the agent.  Returns NULL on error.  This process will connect to the
    /// agent over a control pipe, and the agent will open data pipes (e.g. CONIN
    /// and CONOUT).
    pub fn winpty_open(cfg: *const winpty_config_t, err: *mut winpty_error_ptr_t) -> *mut winpty_t;

    // A handle to the agent process.  This value is valid for the lifetime of the
    // winpty_t object.  Do not close it.
    // pub fn winpty_agent_process(wp: *mut winpty_t) -> *const c_void;

}

// I/O Pipes
extern "C" {
    /// Returns the names of named pipes used for terminal I/O.  Each input or
    /// output direction uses a different half-duplex pipe.  The agent creates
    /// these pipes, and the client can connect to them using ordinary I/O methods.
    /// The strings are freed when the winpty_t object is freed.
    /// `winpty_conerr_name` returns NULL unless `WINPTY_FLAG_CONERR` is specified.
    pub fn winpty_conin_name(wp: *mut winpty_t) -> *const u16;
    pub fn winpty_conout_name(wp: *mut winpty_t) -> *const u16;
    // pub fn winpty_conerr_name(wp: *mut winpty_t) -> *const u16;
}

// winpty agent RPC call: process creation.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct winpty_spawn_config_s {
    _unused: [u8; 0],
}

/// Configuration object (not thread-safe)
pub type winpty_spawn_config_t = winpty_spawn_config_s;

extern "C" {
    /// winpty_spawn_config strings do not need to live as long as the config
    /// object.  They are copied.  Returns NULL on error.  spawnFlags is a set of
    /// zero or more WINPTY_SPAWN_FLAG_xxx values.  An unrecognized flag results in
    /// an assertion failure.
    ///
    /// env is a a pointer to an environment block like that passed to
    /// CreateProcess--a contiguous array of NUL-terminated "VAR=VAL" strings
    /// followed by a final NUL terminator.
    ///
    /// N.B.: If you want to gather all of the child's output, you may want the
    /// WINPTY_SPAWN_FLAG_AUTO_SHUTDOWN flag.
    pub fn winpty_spawn_config_new(
        spawn_flags: u64,
        appname: *const u16,
        cmdline: *const u16,
        cwd: *const u16,
        env: *const u16,
        err: *mut winpty_error_ptr_t) -> *mut winpty_spawn_config_t;

    /// Free the cfg object after passing it to winpty_spawn.
    pub fn winpty_spawn_config_free(cfg: *mut winpty_spawn_config_t);

    /// Spawns the new process.
    ///
    /// The function initializes all output parameters to zero or NULL.
    ///
    /// On success, the function returns TRUE.  For each of process_handle and
    /// thread_handle that is non-NULL, the HANDLE returned from CreateProcess is
    /// duplicated from the agent and returned to the winpty client.  The client is
    /// responsible for closing these HANDLES.
    ///
    /// On failure, the function returns FALSE, and if err is non-NULL, then *err
    /// is set to an error object.
    ///
    /// If the agent's CreateProcess call failed, then *create_process_error is set
    /// to GetLastError(), and the WINPTY_ERROR_SPAWN_CREATE_PROCESS_FAILED error
    /// is returned.
    ///
    /// winpty_spawn can only be called once per winpty_t object.  If it is called
    /// before the output data pipe(s) is/are connected, then collected output is
    /// buffered until the pipes are connected, rather than being discarded.
    ///
    /// N.B.: GetProcessId works even if the process has exited.  The PID is not
    /// recycled until the NT process object is freed.
    /// (https://blogs.msdn.microsoft.com/oldnewthing/20110107-00/?p=11803)
    pub fn winpty_spawn(
        wp: *mut winpty_t,
        cfg: *const winpty_spawn_config_t,
        process_handle: *mut HANDLE,
        thread_handle: *mut HANDLE,
        create_process_error: *mut u32,
        err: *mut winpty_error_ptr_t) -> bool;
}

// winpty agent RPC calls: everything else
extern "C" {
    /// Change the size of the Windows console window.
    pub fn winpty_set_size(wp: *mut winpty_t, cols: i32, rows: i32, err: *mut winpty_error_ptr_t) -> bool;

    /// Frees the winpty_t object and the OS resources contained in it.  This
    /// call breaks the connection with the agent, which should then close its
    /// console, terminating the processes attached to it.
    ///
    /// This function must not be called if any other threads are using the
    /// winpty_t object.  Undefined behavior results.
    pub fn winpty_free(wp: *mut winpty_t);
}
