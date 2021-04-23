
mod native;

pub use crate::native::pywinptyrs;
use pyo3::prelude::*;
use cxx::Exception;
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::types::PyBytes;
//use std::char::{decode_utf16, REPLACEMENT_CHARACTER};


fn unwrap_bytes(value: Option<Vec<u8>>) -> Vec<u8> {
    let vec: Vec<u8> = Vec::new();
    value.unwrap_or(vec)
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
    #[args(encoding="\"utf-8\".to_owned()", backend = "None", input_mode = "512", output_mode = "4",
           override_pipes = "true", mouse_mode = "0", timeout = "1000", agent_config = "4")]
    fn new(
        cols: i32,
        rows: i32,
        encoding: String,
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
            agent_config,
            encoding
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
                Ok(PTY { pty })
			}  
            Err(error) => {
                let error_str: String = error.what().to_owned();
                Err(PyWinptyError::new_err(string_to_static_str(error_str)))
			}
		}
	}

    #[args(cmdline = "None", cwd = "None", env = "None")]
    fn spawn(&self, appname: Vec<u8>, cmdline: Option<Vec<u8>>, cwd: Option<Vec<u8>>, env: Option<Vec<u8>>) -> PyResult<bool> {
        let result: Result<bool, Exception> = pywinptyrs::spawn(
            &self.pty, appname, unwrap_bytes(cmdline), unwrap_bytes(cwd), unwrap_bytes(env));
        
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
    fn read<'p>(&self, length: u64, blocking: bool, py: Python<'p>) -> PyResult<&'p PyBytes> {
        let result: Result<Vec<u8>, Exception> = pywinptyrs::read(&self.pty, length, blocking);
        match result {
            Ok(bytes) => {
                Ok(PyBytes::new(py, &bytes[..]))
            },
            Err(error) => {
                let error_str: String = error.what().to_owned();
                Err(PyWinptyError::new_err(string_to_static_str(error_str)))
			}
		}
	}

    #[args(length = "1000", blocking = "false")]
    fn read_stderr<'p>(&self, length: u64, blocking: bool, py: Python<'p>) -> PyResult<&'p PyBytes> {
        let result: Result<Vec<u8>, Exception> = pywinptyrs::read_stderr(&self.pty, length, blocking);
        match result {
             Ok(bytes) => {
                Ok(PyBytes::new(py, &bytes[..]))
            },
            Err(error) => {
                let error_str: String = error.what().to_owned();
                Err(PyWinptyError::new_err(string_to_static_str(error_str)))
			}
		}
	}

    fn write(&self, to_write: Vec<u8>) -> PyResult<u32> {
        //let utf16_str: Vec<u16> = to_write.encode_utf16().collect();
        let result: Result<u32, Exception> = pywinptyrs::write(&self.pty, to_write);
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
