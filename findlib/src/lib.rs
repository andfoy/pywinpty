extern crate windows;

mod win_calls {
	windows::include_bindings!();
}

pub use win_calls::{Windows::Win32::System::LibraryLoader::{GetProcAddress, GetModuleHandleW}};
