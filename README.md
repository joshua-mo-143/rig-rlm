# rig-rlm
This repository is intended to be a Rust rewrite of [the RLM demo repository.](https://github.com/alexzhang13/rlm)

[Here is the original blogpost.](https://alexzhang13.github.io/blog/2025/rlm/)

## What actually is this repository supposed to showcase?
Recursive Language Models (RLMs for short) are a conceptually new agentic architecture where instead of tool calling, models will instead return commands that will then interact with some form of REPL (whether you're just parsing commands, writing code in an REPL or something else.) The model should also be able to call itself.

A REPL is simply defined as an execution boundary for code and other things to run in. There's multiple ways to define this, but this particular demo will use pyo3 to run Python functions.

There are numerous advantages to this:
- You can call directly into a sandbox, which allows for severely constraining agentic capabilities (making the program safer!)
- You can store context files within the sandbox and have the agent read them, then absorb the information into its context
- The LLM can make its own sub-LLM calls, with each sub-LLM also being an RLM (... you can see where this is going)

## How to run

### Using OpenAI
Change `fn main()` to use `RigRlm::new` rather than `RigRlm::new_local`. Then simply use `cargo run`.

Make sure you have `OPENAI_API_KEY` set, then simply use `cargo run`!

### Locally
Make sure you have LM Studio and the `qwen/qwen3-8b` model loaded, then use `cargo run`.

## TODO
- Set up `impl ExecutionEnvironment` for Firecracker
- Make binary more usable (ie switching between local and cloud)
