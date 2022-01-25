
// use cxx::Exception;
use std::ffi::OsString;

use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
// use pyo3::types::PyBytes;
use winptyrs::{PTY, PTYArgs, PTYBackend, MouseMode, AgentConfig};
// use winptyrs::pty::PTYImpl;

// Package version
const VERSION: &'static str = env!("CARGO_PKG_VERSION");


fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

create_exception!(pywinpty, WinptyError, PyException);

/// Create a pseudo terminal (PTY) of a given size.
///
/// The pseudo-terminal must define a non-zero, positive size for both columns and rows.
///
/// Arguments
/// ---------
/// cols: int
///     Number of columns (width) that the pseudo-terminal should have in characters.
/// rows: int
///     Number of rows (height) that the pseudo-terminal should have in characters.
/// backend: Optional[int]
///     Pseudo-terminal backend to use, see `winpty.Backend`. If None, then the backend
///     will be set automatically based on the available APIs.
/// mouse_mode: Optional[int]
///     Set the mouse mode to one of the WINPTY_MOUSE_MODE_xxx constants.
///     See `winpty.MouseMode`. Default: 0.
/// timeout: Optional[int]
///     Amount of time to wait for the agent (in ms) to startup and to wait for any given
///     agent RPC request.  Must be greater than 0. Default: 30000.
/// agent_config: Optional[int]
///     A set of zero or more WINPTY_FLAG_xxx values. See `winpty.AgentConfig`.
///     Default: WINPTY_FLAG_COLOR_ESCAPES
///
/// Raises
/// ------
/// WinptyError:
///     If an error occurred whilist creating the pseudo terminal instance.
///
/// Notes
/// -----
/// 1. Optional argument values will take effect if and only if the backend is set to
/// `winpty.Backend.Winpty`, either automatically or manually.
///
/// 2. ConPTY backend will take precedence over WinPTY, as it is native to Windows
/// and therefore is faster.
///
/// 3. Automatic backend selection will be determined based on both the compilation
/// flags used to compile pywinpty and the availability of the APIs on the runtime
/// system.
///
#[pyclass(name="PTY")]
struct PyPTY {
    pty: PTY,
}

#[pymethods]
impl PyPTY {
    #[new]
    #[args(
        backend = "None",
        mouse_mode = "0",
        timeout = "30000",
        agent_config = "4"
    )]
    fn new(
        cols: i32,
        rows: i32,
        backend: Option<i32>,
        mouse_mode: i32,
        timeout: u32,
        agent_config: u64,
    ) -> PyResult<Self> {
        let config = PTYArgs {
            cols: cols,
            rows: rows,
            mouse_mode: MouseMode::try_from(mouse_mode).unwrap(),
            timeout: timeout,
            agent_config: AgentConfig::from_bits(agent_config).unwrap()
        };

        let pty: Result<PTY, OsString>;
        match backend {
            Some(backend_value) => {
                pty = PTY::new_with_backend(
                    &config, PTYBackend::try_from(backend_value).unwrap()
                );
            }
            None => {
                pty = PTY::new(&config);
            }
        }

        match pty {
            Ok(pty) => Ok(PyPTY { pty }),
            Err(error) => {
                let error_str: String = error.to_str().unwrap().to_owned();
                Err(WinptyError::new_err(string_to_static_str(error_str)))
            }
        }
    }

    /// Start an application that will communicate through the pseudo-terminal.
    ///
    /// Arguments
    /// ---------
    /// appname: bytes
    ///     Byte string that contains the path to the application that will
    ///     be started.
    /// cmdline: Optional[bytes]
    ///     Byte string that contains the parameters to start the application,
    ///     separated by whitespace.
    /// cwd: Optional[bytes]
    ///     Byte string that contains the working directory that the application
    ///     should have. If None, the application will inherit the current working
    ///     directory of the Python interpreter.
    /// env: Optional[bytes]
    ///     Byte string that contains the name and values of the environment
    ///     variables that the application should have. Each (name, value) pair
    ///     should be declared as `name=value` and each pair must be separated
    ///     by an empty byte `\0`. If None, then the application will inherit
    ///     the environment variables of the Python interpreter.
    ///
    /// Returns
    /// -------
    /// spawned: bool
    ///     True if the application was started successfully and False otherwise.
    ///
    /// Raises
    /// ------
    /// WinptyError
    ///     If an error occurred when trying to start the application process.
    ///
    /// Notes
    /// -----
    /// For a more detailed information about the values of the arguments, see:
    /// https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw
    ///
    #[args(cmdline = "None", cwd = "None", env = "None")]
    fn spawn(
        &mut self,
        appname: OsString,
        cmdline: Option<OsString>,
        cwd: Option<OsString>,
        env: Option<OsString>,
    ) -> PyResult<bool> {
        let result: Result<bool, OsString> = self.pty.spawn(
            appname,
            cmdline,
            cwd,
            env,
        );

        match result {
            Ok(bool_result) => Ok(bool_result),
            Err(error) => {
                let error_str: String = error.to_str().unwrap().to_owned();
                Err(WinptyError::new_err(string_to_static_str(error_str)))
            }
        }
    }

    /// Modify the size of the pseudo terminal.
    ///
    /// The value for the columns and rows should be non-zero and positive.
    ///
    /// Arguments
    /// ---------
    /// cols: int
    ///     Size in characters that the pseudo terminal should have.
    /// rows: int
    ///     Size in characters that the pseudo terminal should have.
    ///
    /// Raises
    /// ------
    /// WinptyError
    ///     If an error occurred whilist resizing the pseudo terminal.
    ///
    fn set_size(&self, cols: i32, rows: i32, py: Python) -> PyResult<()> {
        let result: Result<(), OsString> =
            py.allow_threads(|| self.pty.set_size(cols, rows));
        match result {
            Ok(()) => Ok(()),
            Err(error) => {
                let error_str: String = error.to_str().unwrap().to_owned();
                Err(WinptyError::new_err(string_to_static_str(error_str)))
            }
        }
    }

    /// Read a number of bytes from the pseudoterminal output stream.
    ///
    /// Arguments
    /// ---------
    /// length: int
    ///     Maximum number of bytes to read from the pseudoterminal.
    /// blocking: bool
    ///     If True, the call will be blocked until the requested number of bytes
    ///     are available to read. Otherwise, it will return an empty byte string
    ///     if there are no available bytes to read.
    ///
    /// Returns
    /// -------
    /// output: bytes
    ///     A byte string that contains the output of the pseudoterminal.
    ///
    /// Raises
    /// ------
    /// WinptyError
    ///     If there was an error whilst trying to read the requested number of bytes
    ///     from the pseudoterminal.
    ///
    /// Notes
    /// -----
    /// Use the `blocking=True` mode only if the process is awaiting on a thread, otherwise
    /// this call may block your application, which only can be interrupted by killing the
    /// process.
    ///
    #[args(length = "1000", blocking = "false")]
    fn read<'p>(&self, length: u32, blocking: bool, py: Python<'p>) -> PyResult<OsString> {
        // let result = self.pty.read(length, blocking);
        let result: Result<OsString, OsString> =
            py.allow_threads(move || self.pty.read(length, blocking));

        match result {
            Ok(bytes) => Ok(bytes),
            Err(error) => {
                let error_str: String = error.to_str().unwrap().to_owned();
                Err(WinptyError::new_err(string_to_static_str(error_str)))
            }
        }
    }

    /// Write a byte string into the pseudoterminal input stream.
    ///
    /// Arguments
    /// ---------
    /// to_write: bytes
    ///     The byte sequence that is going to be sent to the pseudoterminal.
    ///
    /// Returns
    /// -------
    /// num_bytes: int
    ///     The number of bytes that were written successfully.
    ///
    /// Raises
    /// ------
    /// WinptyError
    ///     If there was an error whilst trying to write the requested number of bytes
    ///     into the pseudoterminal.
    ///
    fn write(&self, to_write: OsString, py: Python) -> PyResult<u32> {
        //let utf16_str: Vec<u16> = to_write.encode_utf16().collect();
        let result: Result<u32, OsString> =
            py.allow_threads(move || self.pty.write(to_write));
        match result {
            Ok(bytes) => Ok(bytes),
            Err(error) => {
                let error_str: String = error.to_str().unwrap().to_owned();
                Err(WinptyError::new_err(string_to_static_str(error_str)))
            }
        }
    }

    /// Determine if the application process that is running behind the pseudoterminal is alive.
    ///
    /// Returns
    /// -------
    /// alive: bool
    ///     True, the process is alive. False, otherwise.
    ///
    /// Raises
    /// ------
    /// WinptyError
    ///     If there was an error whilst trying to determine the status of the process.
    ///
    fn isalive(&self) -> PyResult<bool> {
        // let result: Result<bool, OsString> = py.allow_threads(move || self.pty.is_alive());
        match self.pty.is_alive() {
            Ok(alive) => Ok(alive),
            Err(error) => {
                let error_str: String = error.to_str().unwrap().to_owned();
                Err(WinptyError::new_err(string_to_static_str(error_str)))
            }
        }
    }

    /// Determine the exit status code of the process that is running behind the pseudoterminal.
    ///
    /// Returns
    /// -------
    /// status: Optional[int]
    ///     None if the process has not started nor finished, otherwise it corresponds to the
    ///     status code at the time of exit.
    ///
    /// Raises
    /// ------
    /// WinptyError
    ///     If there was an error whilst trying to determine the exit status of the process.
    ///
    fn get_exitstatus(&self, py: Python) -> PyResult<Option<u32>> {
        let result: Result<Option<u32>, OsString> =
            py.allow_threads(|| self.pty.get_exitstatus());
        match result {
            Ok(status) => Ok(status),
            Err(error) => {
                let error_str: String = error.to_str().unwrap().to_owned();
                Err(WinptyError::new_err(string_to_static_str(error_str)))
            }
        }
    }

    /// Determine if the application process that is running behind the pseudoterminal reached EOF.
    ///
    /// Returns
    /// -------
    /// eof: False
    ///     True, if the process emitted the end-of-file escape sequence. False, otherwise.
    ///
    /// Raises
    /// ------
    /// WinptyError
    ///     If there was an error whilst trying to determine the EOF status of the process.
    ///
    fn iseof(&self, py: Python) -> PyResult<bool> {
        let result: Result<bool, OsString> = py.allow_threads(|| self.pty.is_eof());
        match result {
            Ok(eof) => Ok(eof),
            Err(error) => {
                let error_str: String = error.to_str().unwrap().to_owned();
                Err(WinptyError::new_err(string_to_static_str(error_str)))
            }
        }
    }

    /// Retrieve the process identifier (PID) of the running process.
    #[getter]
    fn pid(&self) -> PyResult<Option<u32>> {
        let result = self.pty.get_pid();
        match result {
            0 => Ok(None),
            _ => Ok(Some(result)),
        }
    }

    /// Retrieve the process handle number.
    #[getter]
    fn fd(&self) -> PyResult<Option<isize>> {
        match self.pty.get_fd() {
            -1 => Ok(None),
            result => Ok(Some(result))
        }
    }
}

#[pymodule]
fn winpty(py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", VERSION)?;
    m.add("WinptyError", py.get_type::<WinptyError>())?;
    m.add_class::<PyPTY>()?;
    Ok(())
}
