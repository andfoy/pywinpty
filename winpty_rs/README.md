# WinPTY-rs

Create and spawn processes inside a pseudoterminal in Windows.

This crate provides an abstraction over different backend implementations to spawn PTY processes in Windows.
Right now this library supports using [`WinPTY`] and [`ConPTY`].

The abstraction is represented through the [`PTY`] struct, which declares methods to initialize, spawn, read,
write and get diverse information about the state of a process that is running inside a pseudoterminal.

[`WinPTY`]: https://github.com/rprichard/winpty
[`ConPTY`]: https://docs.microsoft.com/en-us/windows/console/creating-a-pseudoconsole-session

## Installation
In order to use Rust in your library/program, you need to add `winpty-rs` to your `Cargo.toml`:

```toml
[dependencies]
winpty-rs = "0.1"
```

In order to enable winpty compatibility, you will need the winpty redistributable binaries available in your PATH.
You can download them from the official [winpty repository releases](https://github.com/rprichard/winpty/releases/tag/0.4.3), or using any known package manager in Windows.

## Usage
This library offers two modes of operation, one that selects the PTY backend automatically and other that picks an specific backend that the user
prefers.

### Creating a PTY setting the backend automatically
```rust
use std::ffi::OsString;
use winptyrs::{PTY, PTYArgs, MouseMode, AgentConfig};

let cmd = OsString::from("c:\\windows\\system32\\cmd.exe");
let pty_args = PTYArgs {
    cols: 80,
    rows: 25,
    mouse_mode: MouseMode::WINPTY_MOUSE_MODE_NONE,
    timeout: 10000,
    agent_config: AgentConfig::WINPTY_FLAG_COLOR_ESCAPES
};

// Initialize a pseudoterminal.
let mut pty = PTY::new(&pty_args).unwrap();
```

## Creating a pseudoterminal using a specific backend.
```rust
use std::ffi::OsString;
use winptyrs::{PTY, PTYArgs, MouseMode, AgentConfig, PTYBackend};

let cmd = OsString::from("c:\\windows\\system32\\cmd.exe");
let pty_args = PTYArgs {
    cols: 80,
    rows: 25,
    mouse_mode: MouseMode::WINPTY_MOUSE_MODE_NONE,
    timeout: 10000,
    agent_config: AgentConfig::WINPTY_FLAG_COLOR_ESCAPES
};

// Initialize a winpty and a conpty pseudoterminal.
let winpty = PTY::new_with_backend(&pty_args, PTYBackend::WinPTY).unwrap();
let conpty = PTY::new_with_backend(&pty_args, PTYBackend::ConPTY).unwrap();
```

## General PTY operations
The `PTY` provides a set of operations to spawn and communicating with a process inside the PTY,
as well to get information about its status.

```rust
// Spawn a process inside the pseudoterminal.
pty.spawn(cmd, None, None, None).unwrap();

// Read the spawned process standard output (non-blocking).
let output = pty.read(1000, false);

// Write to the spawned process standard input.
let to_write = OsString::from("echo \"some str\"\r\n");
let num_bytes = pty.write(to_write).unwrap();

// Change the PTY size.
pty.set_size(80, 45).unwrap();

// Know if the process running inside the PTY is alive.
let is_alive = pty.is_alive().unwrap();

// Get the process exit status (if the process has stopped).
let exit_status = pty.get_exitstatus().unwrap();
```

## Examples
Please checkout the examples provided under the [examples](src/examples) folder, we provide examples for both
ConPTY and WinPTY. In order to compile these examples, you can enable the `conpty_example` and `winpty-example`
features when calling `cargo build`

## Changelog
Visit our [CHANGELOG](CHANGELOG.md) file to learn more about our new features and improvements.

## Contribution guidelines
We use `cargo clippy` to lint this project and `cargo test` to test its functionality. Feel free to send a PR or create an issue if you have any problem/question.
