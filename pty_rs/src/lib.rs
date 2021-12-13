
#[macro_use]
extern crate enum_primitive_derive;
extern crate num_traits;

use num_traits::FromPrimitive;

pub mod pty;
pub use pty::{PTY, PTYArgs, PTYBackend, MouseMode, AgentConfig};

#[cfg(test)]
mod tests {
    use crate::pty::{PTY, PTYArgs, PTYBackend, MouseMode, AgentConfig};

    #[cfg(feature="winpty")]
    #[test]
    fn it_works() {
        let mut pty_args = PTYArgs {
            cols: 80,
            rows: 25,
            mouse_mode: MouseMode::WINPTY_MOUSE_MODE_NONE,
            timeout: 10000,
            agent_config: AgentConfig::WINPTY_FLAG_COLOR_ESCAPES
        };

        match PTY::new_with_backend(&mut pty_args, PTYBackend::WinPTY) {
            Ok(_) => {assert!(true)},
            Err(err) => {panic!("{:?}", err)}
        }

        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
