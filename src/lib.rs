
mod native;

pub use crate::native::pywinptyrs;
use pyo3::prelude::*;
use cxx::Exception;
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use std::char::{decode_utf16, REPLACEMENT_CHARACTER};


fn unwrap_option_utf16(value: Option<&str>) -> Vec<u16> {
    let str_value;
    match value {
        Some(str_val) => {
            str_value = str_val;  
		},
        None => {
            str_value = "";  
		}
	}

    str_value.encode_utf16().collect()
}


fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}


create_exception!(pywinpty, PyWinptyError, PyException);

#[pyclass]
struct PTY {
    pty: pywinptyrs::PTYRef
}

#[pymethods]
impl PTY {
    #[new]
    #[args(backend = "None", input_mode = "512", output_mode = "4",
           override_pipes = "true", mouse_mode = "0", timeout = "1000", agent_config = "4")]
    fn new(
        cols: i32,
        rows: i32,
        backend: Option<i32>,
        input_mode: i32,
        output_mode: i32,
        override_pipes: bool,
        mouse_mode: i32,
		timeout: i32,
        agent_config: i32) -> PyResult<Self> {
        
        let config = pywinptyrs::PTYConfig {
            input_mode,
            output_mode,
            override_pipes,
            mouse_mode,
	    	timeout,
            agent_config  
		};

        let pty: Result<pywinptyrs::PTYRef, Exception>;
        match backend {
            Some(backend_value) => {
                pty = pywinptyrs::create_pty_with_backend_and_config(cols, rows, backend_value, config);
            },
            None => {
                pty = pywinptyrs::create_pty_with_config(cols, rows, config);
			}
		}

        match pty {
            Ok(pty) => {
                println!("PTY created");
                Ok(PTY { pty })
			}  
            Err(error) => {
                let error_str: String = error.what().to_owned();
                Err(PyWinptyError::new_err(string_to_static_str(error_str)))
			}
		}
	}

    #[args(cmdline = "None", cwd = "None", env = "None")]
    fn spawn(&self, appname: &str, cmdline: Option<&str>, cwd: Option<&str>, env: Option<&str>) -> PyResult<bool> {
        let utf16_appname: Vec<u16> = appname.encode_utf16().collect();
        
        let utf16_cmdline: Vec<u16> = unwrap_option_utf16(cmdline);
        let utf16_cwd: Vec<u16> = unwrap_option_utf16(cwd);
        let utf16_env: Vec<u16> = unwrap_option_utf16(env);

        let result: Result<bool, Exception> = pywinptyrs::spawn(
            &self.pty, utf16_appname, utf16_cmdline, utf16_cwd, utf16_env);
        
        match result {
            Ok(bool_result) => Ok(bool_result),
            Err(error) => {
                let error_str: String = error.what().to_owned();
                Err(PyWinptyError::new_err(string_to_static_str(error_str)))
			}
		}
    }

    fn set_size(&self, cols: i32, rows: i32) -> PyResult<()> {
       let result: Result<(), Exception> = pywinptyrs::set_size(&self.pty, cols, rows);
       match result {
            Ok(()) => Ok(()),
            Err(error) => {
                let error_str: String = error.what().to_owned();
                Err(PyWinptyError::new_err(string_to_static_str(error_str)))
			}
		}
	}

    #[args(length = "1000", blocking = "false")]
    fn read(&self, length: u64, blocking: bool) -> PyResult<String> {
        let result: Result<Vec<u16>, Exception> = pywinptyrs::read(&self.pty, length, blocking);
        match result {
            Ok(utf16_vec) => {
                let encoded_string =
                    decode_utf16(utf16_vec.iter().cloned())
                    .map(|r| r.unwrap_or(REPLACEMENT_CHARACTER))
                    .collect::<String>();
                Ok(encoded_string)
            },
            Err(error) => {
                let error_str: String = error.what().to_owned();
                Err(PyWinptyError::new_err(string_to_static_str(error_str)))
			}
		}
	}

    #[args(length = "1000", blocking = "false")]
    fn read_stderr(&self, length: u64, blocking: bool) -> PyResult<String> {
        let result: Result<Vec<u16>, Exception> = pywinptyrs::read_stderr(&self.pty, length, blocking);
        match result {
            Ok(utf16_vec) => {
                let encoded_string =
                    decode_utf16(utf16_vec.iter().cloned())
                    .map(|r| r.unwrap_or(REPLACEMENT_CHARACTER))
                    .collect::<String>();
                Ok(encoded_string)
            },
            Err(error) => {
                let error_str: String = error.what().to_owned();
                Err(PyWinptyError::new_err(string_to_static_str(error_str)))
			}
		}
	}

    fn write(&self, to_write: &str) -> PyResult<u32> {
        let utf16_str: Vec<u16> = to_write.encode_utf16().collect();
        let result: Result<u32, Exception> = pywinptyrs::write(&self.pty, utf16_str);
        match result {
            Ok(bytes) => Ok(bytes),
            Err(error) => {
                let error_str: String = error.what().to_owned();
                Err(PyWinptyError::new_err(string_to_static_str(error_str)))
			}
		}
	}

}


#[pymodule]
fn winpty(py: Python, m: &PyModule) -> PyResult<()> {
    m.add("PyWinptyError", py.get_type::<PyWinptyError>())?;
    m.add_class::<PTY>()?;
    Ok(())
}
