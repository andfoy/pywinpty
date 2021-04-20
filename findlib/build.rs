use windows;

fn main() {
	windows::build!(
        Windows::Win32::SystemServices::{GetProcAddress, GetModuleHandleW}
    );
}
