
/// This module declares the [`self::PTY`] struct, which enables a Rust
/// program to create a pseudoterminal (PTY) in Windows.

// External imports
#[macro_use]
extern crate enum_primitive_derive;
extern crate num_traits;

// Local modules
mod winpty;
mod conpty;
pub mod base;

// Local imports
use self::winpty::{WinPTY, MouseMode, AgentConfig};
use self::conpty::ConPTY;

// Windows imports
use windows::Win32::Foundation::PWSTR;

/// Available backends to create pseudoterminals.
#[derive(Primitive)]
pub enum PTYBackend {
	/// Use the native Windows API, available from Windows 10 (Build version 1809).
	ConPTY,
	/// Use the [winpty](https://github.com/rprichard/winpty) library, useful in older Windows systems.
	WinPTY,
    /// Placeholder value used to select the PTY backend automatically
	Auto,
	/// Placeholder value used to declare that a PTY was created with no backend.
	NoBackend
}

/// Data struct that represents the possible arguments used to create a pseudoterminal
pub struct PTYArgs {
	// Common arguments
	/// Number of character columns to display.
	cols: i32,
	/// Number of line rows to display
	rows: i32,
	// WinPTY backend-specific arguments
	/// Mouse capture settings for the winpty backend.
	mouse_mode: MouseMode,
	/// Amount of time to wait for the agent (in ms) to startup and to wait for any given
    /// agent RPC request.
	timeout: u32,
	/// General configuration settings for the winpty backend.
	agent_config: AgentConfig
}


/// Pseudoterminal struct that communicates with a spawned process.
pub struct PTY {
	 /// Backend used by the current pseudoterminal, must be one of [`self::PTYBackend`].
	 /// If the value is [`self::PTYBackend::None`], then no operations will be available.
	 backend: PTYBackend,
	 /// Reference to the winpty PTY handler when [`backend`] is [`self::PTYBackend::WinPTY`].
	 winpty: Option(WinPTY),
	 /// Reference to the conpty PTY handler when [`backend`] is [`self::PTYBackend::ConPTY`].
	 conpty: Option(ConPTY)
}

impl PTY {
	/// Create a new pseudoterminal setting the backend automatically.
	pub fn new(args: PTYArgs) -> Result<PTY, &str> {
		let mut errors = "There were some errors trying to instantiate a PTY:";
		// Try to create a PTY using the ConPTY backend
		let conpty_instance: Result<ConPTY, &str> = ConPTY::new(args);
	 	let mut pty: Option<PTY> =
			match conpty_instance {
				Ok(conpty) => {
					let pty_instance = PTY {
						backend: PTYBackend::ConPTY,
						winpty: None,
						conpty: conpty
					};
					Some(pty_instance)
				},
				Err(err) => {
					errors = format!("{} (ConPTY) -> {};", errors, err);
					None
				}
			}

		// Try to create a PTY instance using the WinPTY backend
		match pty {
			Some(pty) => Ok(pty),
			None => {
				let winpty_instance: Result<WinPTY, &str> = WinPTY::new(args);
				match winpty_instance {
					Ok(winpty) => {
						let pty_instance = PTY {
							backend: PTYBackend::WinPTY,
							winpty: winpty,
							conpty: None
						};
						Ok(pty_instance)
					},
					Err(err) => {
						errors = format!("{} (WinPTY) -> {}", errors, err);
						Err(errors)
					}
				}
			}
		}
	}

	/// Create a new pseudoterminal using a given backend
	pub fn new(args: PTYArgs, backend: PTYBackend) -> Result<PTY, &str> {
		match backend {
			PTYBackend::ConPTY => {
				match ConPTY::new(args) {
					Ok(conpty) => {
						let pty = PTY {
							backend: backend,
							winpty: None
							conpty: conpty,
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
							backend: backend,
							winpty: winpty
							conpty: None,
						};
						Ok(pty)
					},
					Err(err) => Err(err)
				}
			},
			PTYBackend::Auto => PTY::new(args),
			PTYBackend::NoBackend => Err("NoBackend is not a valid option")
		}
	}
};
