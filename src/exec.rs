//! A module for showcasing different types of execution environments.
//! An execution environment within the confines of this program is simply defined as a place where code gets executed.
//! This demo primarily targets Python code, as Python is an easy language to use/run and can be iterated on very quickly.
//! Out of the box, you will be able to get to try using PyO3 (requires an installation of Python).

use std::ffi::CString;

use pyo3::{
    Python,
    types::{PyAnyMethods, PyDict, PyInt, PyModuleMethods, PyString},
    wrap_pyfunction,
};

use crate::llm::RigRlm;

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

            let globals = py.import("__main__")?.dict();
            let func = wrap_pyfunction!(query_llm, py)?;
            globals.set_item("query_llm", func)?;

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

/// A function to query an LLM (that can be called by an LLM).
#[pyo3::pyfunction]
fn query_llm(prompt: String) -> String {
    // SAFETY: This is a prototype, we can deal with fixing unwrap and making this impl less fragile later
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (tx, rx) = oneshot::channel();

    rt.block_on(async {
        let rlm = RigRlm::new_local();
        let res = rlm.query(&prompt).await.unwrap();

        tx.send(res).unwrap();
    });

    rx.try_recv().unwrap()
}
