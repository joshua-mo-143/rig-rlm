//! A module for showcasing different types of execution environments.
//! An execution environment within the confines of this program is simply defined as a place where code gets executed.
//! This demo primarily targets Python code, as Python is an easy language to use/run and can be iterated on very quickly.
//! Out of the box, you will be able to get to try two different envs: PyO3 (requires an installation of Python), and Firecracker (requires Firecracker installed).

use std::ffi::CString;

use pyo3::{
    Python,
    types::{PyAnyMethods, PyDict, PyInt, PyString},
};

pub trait ExecutionEnvironment {
    type Error: std::error::Error + Send + Sync + 'static;

    fn execute_code(&self, code: String) -> Result<String, Self::Error>;
}

/// Use native Python as an executor. Requires a shared lib Python installation.
pub struct Pyo3Executor;

impl ExecutionEnvironment for Pyo3Executor {
    type Error = pyo3::PyErr;

    fn execute_code(&self, code: String) -> Result<String, Self::Error> {
        let string: String = Python::attach(|py| {
            let io = py.import("io")?;
            let sys = py.import("sys")?;

            let string_io = io.call_method0("StringIO")?;
            sys.setattr("stdout", &string_io)?;

            let locals = PyDict::new(py);
            locals.set_item("context", py.None())?;

            let code = CString::new(code).unwrap();

            // If there are any errors, we need to return them back to the LLM (stopping execution makes no sense here).
            if let Err(e) = py.run(&code, None, Some(&locals)) {
                return Ok(e.to_string());
            };

            if let Ok(ret) = locals.get_item("my_answer") {
                if let Ok(result) = ret.cast::<PyInt>() {
                    return Ok::<String, pyo3::PyErr>(result.to_string());
                }
                if let Ok(result) = ret.cast::<PyString>() {
                    return Ok(result.to_string());
                }
            }

            let output = string_io.call_method0("getvalue")?;
            Ok(output.to_string())
        })?;

        Ok(string)
    }
}
