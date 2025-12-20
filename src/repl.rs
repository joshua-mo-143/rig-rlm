use std::{collections::HashMap, ffi::CString, num::NonZeroI32};

use pyo3::{
    Bound, Python,
    types::{PyAnyMethods, PyDict, PyInt, PyNone, PyString, PyStringMethods},
};

pub struct REPL {
    pub context: HashMap<String, String>,
}

impl REPL {
    pub fn new() -> Self {
        Self {
            context: HashMap::new(),
        }
    }

    pub fn get_variable(&self, name: &str) -> Option<String> {
        self.context.get(name).cloned()
    }

    pub fn run_command(&self, command: Command) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(input) = command.get_run_command() {
            let thing = std::process::Command::new(input.program.clone())
                .args(input.args.clone())
                .output()
                .unwrap();

            if !thing.status.success() {
                return Ok(String::from_utf8_lossy(&thing.stderr).to_string());
            }

            Ok(String::from_utf8_lossy(&thing.stdout).to_string())
        } else if let Some(output) = command.get_final_command() {
            Ok(output)
        } else if let Some(code) = command.get_code_to_run() {
            let string: String = Python::attach(|py| {
                let io = py.import("io")?;
                let sys = py.import("sys")?;

                // Capture stdout
                let string_io = io.call_method0("StringIO")?;
                sys.setattr("stdout", &string_io)?;

                let locals = PyDict::new(py);
                locals.set_item("context", py.None())?;

                let code = CString::new(code).unwrap();

                // If there are any errors, we need to return them back to the LLM.
                if let Err(e) = py.run(&code, None, Some(&locals)) {
                    return Ok(e.to_string());
                };

                // Check for final_var first
                if let Ok(ret) = locals.get_item("my_answer") {
                    if let Ok(result) = ret.cast::<PyInt>() {
                        return Ok::<String, pyo3::PyErr>(result.to_string());
                    }
                    if let Ok(result) = ret.cast::<PyString>() {
                        return Ok(result.to_string());
                    }
                }

                // If no final_var, return whatever was printed
                let output = string_io.call_method0("getvalue")?;
                Ok(output.to_string())
            })?;

            Ok(string)
        } else {
            Err("Could not find command.".into())
        }
    }
}

impl Default for REPL {
    fn default() -> Self {
        Self::new()
    }
}

pub enum Command {
    Run(RunArgs),
    Final(String),
    RunCode(String),
    InvalidCommand,
}

impl Command {
    fn get_run_command(&self) -> Option<&RunArgs> {
        if let Self::Run(args) = self {
            Some(args.to_owned())
        } else {
            None
        }
    }

    pub fn get_final_command(&self) -> Option<String> {
        if let Self::Final(str) = self {
            Some(str.to_owned())
        } else {
            None
        }
    }

    pub fn get_code_to_run(&self) -> Option<String> {
        if let Self::RunCode(str) = self {
            Some(str.to_owned())
        } else {
            None
        }
    }
}

pub struct RunArgs {
    program: String,
    args: Vec<String>,
}

impl Command {
    pub fn parse(input: &str) -> Self {
        if input.trim_start().starts_with("RUN") {
            println!(
                "Attempting to run command: {input}",
                input = input.trim().trim_start_matches("RUN ")
            );
            let mut iter = input.split_ascii_whitespace().skip(1);
            let Some(program) = iter.next() else {
                panic!("There's no arguments here!")
            };
            let args: Vec<String> = iter.map(|x| x.to_owned()).collect();

            let args = RunArgs {
                program: program.to_string(),
                args,
            };
            return Self::Run(args);
        }

        if input.trim_start().starts_with("FINAL") {
            let result = input
                .split_ascii_whitespace()
                .skip(1)
                .map(|x| x.to_owned())
                .collect::<Vec<String>>()
                .join(" ");

            return Self::Final(result);
        }

        if input.trim_start().starts_with("```repl") {
            let input = input
                .trim()
                .trim_start_matches("```repl\n")
                .trim_end_matches("\n```");

            println!("Trimmed code input: {input}");

            return Self::RunCode(input.to_string());
        }

        unimplemented!("Handle more branches")
    }
}

#[cfg(test)]
mod test {
    use crate::repl::{Command, REPL, RunArgs};

    #[test]
    fn it_works() {
        let repl = REPL::new();

        let cmd = Command::Run(RunArgs {
            program: "ls".to_string(),
            args: vec![".".to_string()],
        });

        let res = repl.run_command(cmd).unwrap();

        assert_eq!(
            res,
            "Cargo.lock\nCargo.toml\nREADME.md\nsrc\ntarget\n".to_string()
        );
    }
}
