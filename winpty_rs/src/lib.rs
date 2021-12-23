//! Create and spawn processes inside a pseudoterminal in Windows.
//!
//! This crate provides an abstraction over different backend implementations to spawn PTY processes in Windows.
//! Right now this library supports using [`WinPTY`] and [`ConPTY`].
//!
//! The abstraction is represented through the [`PTY`] struct, which declares methods to initialize, spawn, read,
//! write and get diverse information about the state of a process that is running inside a pseudoterminal.
//!
//! [`WinPTY`]: https://github.com/rprichard/winpty
//! [`ConPTY`]: https://docs.microsoft.com/en-us/windows/console/creating-a-pseudoconsole-session


#[macro_use]
extern crate enum_primitive_derive;
extern crate num_traits;

pub mod pty;
pub use pty::{PTY, PTYArgs, PTYBackend, MouseMode, AgentConfig};
