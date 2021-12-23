/// This module provides a [`super::PTY`] backend that uses
/// [conpty](https://docs.microsoft.com/en-us/windows/console/creating-a-pseudoconsole-session) as its implementation.
/// This backend is available on Windows 10 starting from build number 1809.

// Actual implementation if winpty is available
#[cfg(feature="conpty")]
mod pty_impl;

#[cfg(feature="conpty")]
pub use pty_impl::ConPTY;

// Default implementation if winpty is not available
#[cfg(not(feature="conpty"))]
mod default_impl;

#[cfg(not(feature="conpty"))]
pub use default_impl::ConPTY;