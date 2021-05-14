use windows;

fn main() {
	windows::build!(
        Windows::Win32::System::SystemServices::{GetProcAddress, GetModuleHandleW}
    );
}
