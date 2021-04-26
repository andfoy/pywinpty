extern crate cxx;

/// Native bindings for ConPTY/WinPTY
#[cxx::bridge]
pub mod pywinptyrs {
    // Shared struct that contains a shared pointer to a pseudo terminal
    struct PTYRef {
        pty: SharedPtr<PTY>,
        encoding: i32,
    }

    // Shared struct that contains the configuration values for the pseudo terminal
    struct PTYConfig {
        mouse_mode: i32,
        timeout: i32,
        agent_config: i32,
        encoding: String,
    }

    extern "Rust" {}

    unsafe extern "C++" {
        include!("wrapper.h");

        /// Reference to a pseudoterminal
        type PTY;

        /// Create an automatic-backend pseudo terminal with given columns, rows and settings
        #[rust_name = "create_pty_with_config"]
        fn create_pty(cols: i32, rows: i32, config: PTYConfig) -> Result<PTYRef>;

        /// Create a manual-backend pseudo terminal with given columns, rows and settings
        #[rust_name = "create_pty_with_backend_and_config"]
        fn create_pty(cols: i32, rows: i32, backend: i32, config: PTYConfig) -> Result<PTYRef>;

        /// Spawn a program in a given pseudo terminal
        fn spawn(
            pty: &PTYRef,
            appname: Vec<u8>,
            cmdline: Vec<u8>,
            cwd: Vec<u8>,
            env: Vec<u8>,
        ) -> Result<bool>;

        /// Resize a given pseudo terminal
        fn set_size(pty: &PTYRef, cols: i32, rows: i32) -> Result<()>;

        /// Read n UTF-16 characters from the stdout stream of the PTY process
        fn read(pty: &PTYRef, length: u64, blocking: bool) -> Result<Vec<u8>>;

        /// Read n UTF-16 characters from the stderr stream of the PTY process
        fn read_stderr(pty: &PTYRef, length: u64, blocking: bool) -> Result<Vec<u8>>;

        /// Write a stream of UTF-16 characters into the stdin stream of the PTY process
        fn write(pty: &PTYRef, in_str: Vec<u8>) -> Result<u32>;

        /// Determine if the process spawned by the PTY is alive
        fn is_alive(pty: &PTYRef) -> Result<bool>;

        /// Retrieve the exit status code of the process spawned by the PTY
        fn get_exitstatus(pty: &PTYRef) -> Result<i64>;

        /// Determine if the process spawned by the PTY reached the end of file
        fn is_eof(pty: &PTYRef) -> Result<bool>;

        /// Retrieve the PID of the PTY process
        fn pid(pty: &PTYRef) -> u32;
    }
}

unsafe impl std::marker::Send for pywinptyrs::PTYRef {}
unsafe impl std::marker::Sync for pywinptyrs::PTYRef {}
