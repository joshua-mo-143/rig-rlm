//! The LLM module.
//! We technically use a rig Agent here, but because we don't really care here about the finer details of the completions, it's fine. For now, anyway.

use rig::{
    OneOrMany,
    agent::{Agent, Text},
    client::{CompletionClient, ProviderClient},
    completion::Chat,
    message::{AssistantContent, Message, UserContent},
    providers::openai::CompletionModel,
};

use crate::{
    exec::{ExecutionEnvironment, Pyo3Executor},
    repl::{Command, REPL},
};

pub struct RigRlm<T>
where
    T: ExecutionEnvironment,
{
    agent: Agent<CompletionModel>,
    repl: REPL<T>,
}

impl RigRlm<Pyo3Executor> {
    pub fn new_local() -> Self {
        let agent = rig::providers::openai::Client::<reqwest::Client>::builder()
            .base_url("http://127.0.0.1:1234/v1")
            .api_key("")
            .http_client(reqwest::Client::new())
            .build()
            .unwrap();

        let agent = agent
            .completion_model("qwen/qwen3-8b")
            .completions_api()
            .into_agent_builder()
            .preamble(PREAMBLE)
            .build();

        Self {
            agent,
            repl: REPL::new(),
        }
    }

    pub fn new() -> Self {
        let agent = rig::providers::openai::Client::from_env();

        let agent = agent
            .completion_model("gpt-5.2")
            .completions_api()
            .into_agent_builder()
            .preamble(PREAMBLE)
            .build();

        Self {
            agent,
            repl: REPL::new(),
        }
    }

    pub async fn query(&self, input: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut prompt = format!(
            r#"
            Think step-by-step on what to do using the REPL environment (which contains the context) to answer the original query: "{input}".

            Continue using the REPL environment, which has the context variable, and querying sub-LLMs by writing to repl tags, and determine your answer. Your next action:"#
        );

        println!("Prompt: {prompt}");

        let mut message_history: Vec<Message> = Vec::new();

        loop {
            let prompt_result = self
                .agent
                .chat(prompt.clone(), message_history.clone())
                .await?;

            println!("Prompt result: {prompt_result}");

            message_history.push(Message::User {
                content: OneOrMany::one(UserContent::Text(Text {
                    text: prompt.clone(),
                })),
            });
            message_history.push(Message::Assistant {
                content: OneOrMany::one(AssistantContent::Text(Text {
                    text: prompt_result.clone(),
                })),
                id: None,
            });

            let cmd = Command::parse(&prompt_result);

            if let Some(output) = cmd.get_final_command() {
                let prompt = format!(
                    "Please send a message to the user in regular text format using the following output:\n\n{output}"
                );
                let res = self.agent.chat(prompt, message_history).await?;

                return Ok(res.trim().to_string());
            } else {
                prompt = match self.repl.run_command(cmd) {
                    Ok(res) => res,
                    Err(e) => e.to_string(),
                };

                println!("Prompt: {prompt}");
            }
        }
    }
}

const PREAMBLE: &str = r#"""
You are tasked with answering a query with associated context. You can access, transform, and analyze this context interactively in a REPL environment that can recursively query sub-LLMs, which you are strongly encouraged to use as much as possible. You will be queried iteratively until you provide a final answer.

Commands available to you:
1. `RUN <command>` - Run a Bash command. This will operate over the user's entire file system.
2. `FINAL <message>` - Return a final message that will also be the signal for you to return your final response.
3. You can also run Python code by using a triple-backtick marked code snippet, with the `repl` tag (see below for notes on the REPL environment).

The REPL environment is initialized with:
1. A `context` variable that may or may not contain extremely important information about your query. You should check the content of the `context` variable to understand what you are working with. Make sure you look through it sufficiently as you answer your query.
2. The ability to use `print()` statements to view the output of your REPL code and continue your reasoning.

You will only be able to see truncated outputs from the REPL environment, so you should use the query LLM function on variables you want to analyze. You will find this function especially useful when you have to analyze the semantics of the context. Use these variables as buffers to build up your final answer.
Make sure to explicitly look through the entire context in REPL before answering your query. An example strategy is to first look at the context and figure out a chunking strategy, then break up the context into smart chunks, and query an LLM per chunk with a particular question and save the answers to a buffer, then query an LLM with all the buffers to produce your final answer.

You can use the REPL environment to help you understand your context, especially if it is huge. Remember that your sub LLMs are powerful -- they can fit around 500K characters in their context window, so don't be afraid to put a lot of context into them. For example, a viable strategy is to feed 10 documents per sub-LLM query. Analyze your input data and see if it is sufficient to just fit it in a few sub-LLM calls!

When you want to execute Python code in the REPL environment, wrap it in triple backticks with 'repl' language identifier. For example, say we want our recursive model to search for the magic number in the context (assuming the context is a string), and the context is very long, so we want to chunk it:
```repl
chunk = context[:10000]
answer = llm_query(f"What is the magic number in the context? Here is the chunk: {{chunk}}")
print(answer)
```

As an example, after analyzing the context and realizing its separated by Markdown headers, we can maintain state through buffers by chunking the context by headers, and iteratively querying an LLM over it:
```repl
# After finding out the context is separated by Markdown headers, we can chunk, summarize, and answer
import re
sections = re.split(r'### (.+)', context["content"])
buffers = []
for i in range(1, len(sections), 2):
    header = sections[i]
    info = sections[i+1]
    summary = query_llm(f"Summarize this {{header}} section: {{info}}")
    buffers.append(f"{{header}}: {{summary}}")
my_answer = query_llm(f"Based on these summaries, answer the original query: {{query}}\\n\\nSummaries:\\n" + "\\n".join(buffers))
```
In the next step, we can return `my_answer`.

IMPORTANT: When you are done with the iterative process, you MUST provide a final answer inside a FINAL function when you have completed your task, NOT in code. Do not use these tags unless you have completed your task. You have two options:
1. Use FINAL <message> to provide the answer directly back to the user.
2. To return a variable as an output from any REPL script, you must assign it to the `my_answer` variable. Printing a variable will NOT return it.
3. The final output of any REPL MUST be an integer or a string!

Think step by step carefully, plan, and execute this plan immediately in your response - you must skip ALL prose outside of the provided commands.
"""#;
