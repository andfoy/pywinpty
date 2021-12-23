/// This module provides a [`super::PTY`] backend that uses
/// [winpty](https://github.com/rprichard/winpty) as its implementation.
/// This backend is useful as a fallback implementation to the newer ConPTY
/// backend, which is only available on Windows 10 starting on build number 1809.

use bitflags::bitflags;
use enum_primitive_derive::Primitive;

// Actual implementation if winpty is available
#[cfg(feature="winpty")]
mod pty_impl;

#[cfg(feature="winpty")]
mod bindings;

#[cfg(feature="winpty")]
pub use pty_impl::WinPTY;

// Default implementation if winpty is not available
#[cfg(not(feature="winpty"))]
mod default_impl;

#[cfg(not(feature="winpty"))]
pub use default_impl::WinPTY;

///  Mouse capture settings for the winpty backend.
#[derive(Primitive)]
#[allow(non_camel_case_types)]
pub enum MouseMode {
    /// QuickEdit mode is initially disabled, and the agent does not send mouse
    /// mode sequences to the terminal.  If it receives mouse input, though, it
    // still writes MOUSE_EVENT_RECORD values into CONIN.
    WINPTY_MOUSE_MODE_NONE = 0,

    /// QuickEdit mode is initially enabled.  As CONIN enters or leaves mouse
    /// input mode (i.e. where ENABLE_MOUSE_INPUT is on and
    /// ENABLE_QUICK_EDIT_MODE is off), the agent enables or disables mouse
    /// input on the terminal.
    WINPTY_MOUSE_MODE_AUTO = 1,

    /// QuickEdit mode is initially disabled, and the agent enables the
    /// terminal's mouse input mode.  It does not disable terminal
    /// mouse mode (until exit).
    WINPTY_MOUSE_MODE_FORCE = 2,
}

bitflags! {
    /// General configuration settings for the winpty backend.
    pub struct AgentConfig: u64 {
        /// Create a new screen buffer (connected to the "conerr" terminal pipe) and
        /// pass it to child processes as the STDERR handle.  This flag also prevents
        /// the agent from reopening CONOUT$ when it polls -- regardless of whether
        /// the active screen buffer changes, winpty continues to monitor the
        /// original primary screen buffer.
        const WINPTY_FLAG_CONERR = 0b00000001;

        /// Don't output escape sequences.
        const WINPTY_FLAG_PLAIN_OUTPUT = 0b00000010;

        /// Do output color escape sequences.  These escapes are output by default,
        /// but are suppressed with WINPTY_FLAG_PLAIN_OUTPUT.
        /// Use this flag to reenable them.
        const WINPTY_FLAG_COLOR_ESCAPES = 0b00000100;
    }
}
