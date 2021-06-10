use windows;

fn main() {
	windows::build!(
        Windows::Win32::System::LibraryLoader::{GetProcAddress, GetModuleHandleW}
    );
}
