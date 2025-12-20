use std::collections::HashMap;

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

            Ok(String::from_utf8_lossy(&thing.stdout).to_string())
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
}

pub struct RunArgs {
    program: String,
    args: Vec<String>,
}

impl Command {
    pub fn parse(input: &str) -> Self {
        if input.starts_with("RUN") {
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

        if input.starts_with("FINAL") {
            let result = input
                .split_ascii_whitespace()
                .skip(1)
                .map(|x| x.to_owned())
                .collect::<Vec<String>>()
                .join(" ");

            return Self::Final(result);
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
