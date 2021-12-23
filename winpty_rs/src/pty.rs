
//! This module declares the [`PTY`] struct, which enables a Rust
//! program to create a pseudoterminal (PTY) in Windows.
//!
//! Additionally, this module also contains several generic structs used to
//! perform I/O operations with a process, [`PTYProcess`]. Also it defines
//! the main interface ([`PTYImpl`]) that a PTY backend should comply with.
//! These structs and traits should be used in order to extend the library.

// External imports

// Local modules
mod winpty;
mod conpty;
mod base;

use std::ffi::OsString;

// Local imports
use self::winpty::WinPTY;
pub use self::winpty::{MouseMode, AgentConfig};
use self::conpty::ConPTY;
pub use base::{PTYImpl, PTYProcess};

/// Available backends to create pseudoterminals.
#[derive(Primitive)]
#[derive(Copy, Clone, Debug)]
pub enum PTYBackend {
	/// Use the native Windows API, available from Windows 10 (Build version 1809).
	ConPTY = 0,
	/// Use the [winpty](https://github.com/rprichard/winpty) library, useful in older Windows systems.
	WinPTY = 1,
    /// Placeholder value used to select the PTY backend automatically
	Auto = 2,
	/// Placeholder value used to declare that a PTY was created with no backend.
	NoBackend = 3,
}

/// Data struct that represents the possible arguments used to create a pseudoterminal
pub struct PTYArgs {
	// Common arguments
	/// Number of character columns to display.
	pub cols: i32,
	/// Number of line rows to display
	pub rows: i32,
	// WinPTY backend-specific arguments
	/// Mouse capture settings for the winpty backend.
	pub mouse_mode: MouseMode,
	/// Amount of time to wait for the agent (in ms) to startup and to wait for any given
    /// agent RPC request.
	pub timeout: u32,
	/// General configuration settings for the winpty backend.
	pub agent_config: AgentConfig
}


/// Pseudoterminal struct that communicates with a spawned process.
///
/// This struct spawns a terminal given a set of arguments, as well as a backend,
/// which can be determined automatically or be given automatically using one of the values
/// listed on the [`PTYBackend`] struct.
///
/// # Examples
///
/// ## Creating a PTY setting the backend automatically
/// ```
/// use std::ffi::OsString;
/// use winptyrs::{PTY, PTYArgs, MouseMode, AgentConfig};
///
/// let cmd = OsString::from("c:\\windows\\system32\\cmd.exe");
/// let pty_args = PTYArgs {
///     cols: 80,
///     rows: 25,
///     mouse_mode: MouseMode::WINPTY_MOUSE_MODE_NONE,
///     timeout: 10000,
///     agent_config: AgentConfig::WINPTY_FLAG_COLOR_ESCAPES
/// };
///
/// // Initialize a pseudoterminal.
/// let mut pty = PTY::new(&pty_args).unwrap();
///
/// // Spawn a process inside the pseudoterminal.
/// pty.spawn(cmd, None, None, None).unwrap();
///
/// // Read the spawned process standard output (non-blocking).
/// let output = pty.read(1000, false);
///
/// // Write to the spawned process standard input.
/// let to_write = OsString::from("echo \"some str\"\r\n");
/// let num_bytes = pty.write(to_write).unwrap();
///
/// // Change the PTY size.
/// pty.set_size(80, 45).unwrap();
///
/// // Know if the process running inside the PTY is alive.
/// let is_alive = pty.is_alive().unwrap();
///
/// // Get the process exit status (if the process has stopped).
/// let exit_status = pty.get_exitstatus().unwrap();
/// ```
///
/// ## Creating a pseudoterminal using a specific backend.
/// ```
/// use std::ffi::OsString;
/// use winptyrs::{PTY, PTYArgs, MouseMode, AgentConfig, PTYBackend};
///
/// let cmd = OsString::from("c:\\windows\\system32\\cmd.exe");
/// let pty_args = PTYArgs {
///     cols: 80,
///     rows: 25,
///     mouse_mode: MouseMode::WINPTY_MOUSE_MODE_NONE,
///     timeout: 10000,
///     agent_config: AgentConfig::WINPTY_FLAG_COLOR_ESCAPES
/// };
///
/// // Initialize a winpty and a conpty pseudoterminal.
/// let winpty = PTY::new_with_backend(&pty_args, PTYBackend::WinPTY).unwrap();
/// let conpty = PTY::new_with_backend(&pty_args, PTYBackend::ConPTY).unwrap();
/// ```
pub struct PTY {
	 /// Backend used by the current pseudoterminal, must be one of [`self::PTYBackend`].
	 /// If the value is [`self::PTYBackend::NoBackend`], then no operations will be available.
	 backend: PTYBackend,
	 /// Reference to the PTY handler which depends on the value of `backend`.
	 pty: Box<dyn PTYImpl>
}

impl PTY {
	/// Create a new pseudoterminal setting the backend automatically.
	pub fn new(args: &PTYArgs) -> Result<PTY, OsString> {
		let mut errors: OsString = OsString::from("There were some errors trying to instantiate a PTY:");
		// Try to create a PTY using the ConPTY backend
		let conpty_instance: Result<Box<dyn PTYImpl>, OsString> = ConPTY::new(args);
	 	let pty: Option<PTY> =
			match conpty_instance {
				Ok(conpty) => {
					let pty_instance = PTY {
						backend: PTYBackend::ConPTY,
						pty: conpty
					};
					Some(pty_instance)
				},
				Err(err) => {
					errors = OsString::from(format!("{:?} (ConPTY) -> {:?};", errors, err));
					None
				}
			};

		// Try to create a PTY instance using the WinPTY backend
		match pty {
			Some(pty) => Ok(pty),
			None => {
				let winpty_instance: Result<Box<dyn PTYImpl>, OsString> = WinPTY::new(args);
				match winpty_instance {
					Ok(winpty) => {
						let pty_instance = PTY {
							backend: PTYBackend::WinPTY,
							pty: winpty
						};
						Ok(pty_instance)
					},
					Err(err) => {
						errors = OsString::from(format!("{:?} (WinPTY) -> {:?}", errors, err));
						Err(errors)
					}
				}
			}
		}
	}

	/// Create a new pseudoterminal using a given backend
	pub fn new_with_backend(args: &PTYArgs, backend: PTYBackend) -> Result<PTY, OsString> {
		match backend {
			PTYBackend::ConPTY => {
				match ConPTY::new(args) {
					Ok(conpty) => {
						let pty = PTY {
							backend,
							pty: conpty
						};
						Ok(pty)
					},
					Err(err) => Err(err)
				}
			},
			PTYBackend::WinPTY => {
				match WinPTY::new(args) {
					Ok(winpty) => {
						let pty = PTY {
							backend,
							pty: winpty
						};
						Ok(pty)
					},
					Err(err) => Err(err)
				}
			},
			PTYBackend::Auto => PTY::new(args),
			PTYBackend::NoBackend => Err(OsString::from("NoBackend is not a valid option"))
		}
	}

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
	pub fn spawn(&mut self, appname: OsString, cmdline: Option<OsString>, cwd: Option<OsString>, env: Option<OsString>) -> Result<bool, OsString> {
		self.pty.spawn(appname, cmdline, cwd, env)
	}

	/// Change the PTY size.
    ///
    /// # Arguments
    /// * `cols` - Number of character columns to display.
    /// * `rows` - Number of line rows to display.
	pub fn set_size(&self, cols: i32, rows: i32) -> Result<(), OsString> {
		self.pty.set_size(cols, rows)
	}

	/// Get the backend used by the current PTY.
	pub fn get_backend(&self) -> PTYBackend {
		self.backend
	}

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
	pub fn read(&self, length: u32, blocking: bool) -> Result<OsString, OsString> {
        self.pty.read(length, blocking)
    }

	/// Write a (possibly) UTF-16 string into the standard input of a process.
    ///
    /// # Arguments
    /// * `buf` - [`OsString`] containing the string to write.
    ///
    /// # Returns
    /// The total number of characters written if the call was successful, else
    /// an [`OsString`] containing an human-readable error.
    pub fn write(&self, buf: OsString) -> Result<u32, OsString> {
        self.pty.write(buf)
    }

	/// Check if a process reached End-of-File (EOF).
    ///
    /// # Returns
    /// `true` if the process reached EOL, false otherwise. If an error occurs, then a [`OsString`]
    /// containing a human-readable error is raised.
    pub fn is_eof(&mut self) -> Result<bool, OsString> {
		self.pty.is_eof()
    }

	/// Retrieve the exit status of the process.
    ///
    /// # Returns
    /// `None` if the process has not exited, else the exit code of the process.
    pub fn get_exitstatus(&mut self) -> Result<Option<u32>, OsString> {
        self.pty.get_exitstatus()
    }

	/// Determine if the process is still alive.
    pub fn is_alive(&mut self) -> Result<bool, OsString> {
        self.pty.is_alive()
    }
}
