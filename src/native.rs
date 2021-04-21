extern crate cxx;

/// Native bindings for ConPTY/WinPTY
#[cxx::bridge]
pub mod pywinptyrs {
    struct PTYRef {
        pty: SharedPtr<PTY>,
	}

    struct PTYConfig {
        input_mode: i32,
        output_mode: i32,
        override_pipes: bool,
        mouse_mode: i32,
		timeout: i32,
        agent_config: i32
	}

    extern "Rust" {}

    unsafe extern "C++" {
        include!("wrapper.h");

        /// Reference to a torch tensor in memory
        type PTY;

        /// Create an automatic-backend pseudo terminal with given columns and rows
        fn create_pty(cols: i32, rows: i32) -> PTYRef;

        /// Create a manual-backend pseudo terminal with given columns and rows
        #[rust_name = "create_pty_with_backend"]
        fn create_pty(cols: i32, rows: i32, backend: i32) -> PTYRef;
        
        /// Create an automatic-backend pseudo terminal with given columns, rows and settings
        #[rust_name = "create_pty_with_config"]
        fn create_pty(cols: i32, rows: i32, config: PTYConfig) -> Result<PTYRef>;

        /// Create a manual-backend pseudo terminal with given columns, rows and settings
        #[rust_name = "create_pty_with_backend_and_config"]
        fn create_pty(cols: i32, rows: i32, backend: i32, config: PTYConfig) -> Result<PTYRef>;

        /// Spawn a program in a given pseudo terminal
        fn spawn(
            pty: &PTYRef,
            appname: Vec<u16>,
            cmdline: Vec<u16>,
	        cwd: Vec<u16>,
            env: Vec<u16>
        ) -> Result<bool>;

        /// Resize a given pseudo terminal
        fn set_size(pty: &PTYRef, cols: i32, rows: i32) -> Result<()>;

        /// Read n UTF-16 characters from the stdout stream of the PTY process
        fn read(pty: &PTYRef, length: u64, blocking: bool) -> Result<Vec<u16>>;

        /// Read n UTF-16 characters from the stderr stream of the PTY process
        fn read_stderr(pty: &PTYRef, length: u64, blocking: bool) -> Result<Vec<u16>>;

        /// Write a stream of UTF-16 characters into the stdin stream of the PTY process
        fn write(pty: &PTYRef, in_str: Vec<u16>) -> Result<u32>;

        /// Determine if the process spawned by the PTY is alive
        fn is_alive(pty: PTYRef) -> bool;

        /// Retrieve the exit status code of the process spawned by the PTY
        fn get_exitstatus(pty: PTYRef) -> i64;
    }
}

unsafe impl std::marker::Send for pywinptyrs::PTYRef {}
unsafe impl std::marker::Sync for pywinptyrs::PTYRef {}
