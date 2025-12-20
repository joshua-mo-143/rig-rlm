//! The LLM module.
//! We technically use a rig Agent here, but because we don't really care here about the finer details of the completions, it's fine. For now, anyway.

use rig::{
    OneOrMany,
    agent::{Agent, Text},
    client::CompletionClient,
    completion::Chat,
    message::{AssistantContent, Message, UserContent},
    providers::openai::responses_api::ResponsesCompletionModel,
};

use crate::repl::{Command, REPL};

pub struct RigRlm {
    agent: Agent<ResponsesCompletionModel>,
    repl: REPL,
}

impl RigRlm {
    pub fn new() -> Self {
        let agent = rig::providers::openai::Client::<reqwest::Client>::builder()
            .base_url("http://127.0.0.1:1234/v1")
            .api_key("")
            .http_client(reqwest::Client::new())
            .build()
            .unwrap();

        let agent = agent.agent("qwen/qwen3-4b").preamble(PREAMBLE).build();

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

        let mut message_history: Vec<Message> = Vec::new();

        loop {
            let prompt_result = self
                .agent
                .chat(prompt.clone(), message_history.clone())
                .await?;

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

                return Ok(res);
            } else {
                let cmd_result = self.repl.run_command(cmd)?;

                prompt = cmd_result;
            }
        }
    }
}

const PREAMBLE: &str = r#"""
You are tasked with answering a query with associated context. You can access, transform, and analyze this context interactively in a REPL environment that can recursively query sub-LLMs, which you are strongly encouraged to use as much as possible. You will be queried iteratively until you provide a final answer.

The REPL environment is initialized with:
1. A `context` variable that contains extremely important information about your query. You should check the content of the `context` variable to understand what you are working with. Make sure you look through it sufficiently as you answer your query.
2. A `llm_query` function that allows you to query an LLM (that can handle around 500K chars) inside your REPL environment.
3. The ability to use `print()` statements to view the output of your REPL code and continue your reasoning.

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
    summary = llm_query(f"Summarize this {{header}} section: {{info}}")
    buffers.append(f"{{header}}: {{summary}}")
final_answer = llm_query(f"Based on these summaries, answer the original query: {{query}}\\n\\nSummaries:\\n" + "\\n".join(buffers))
```
In the next step, we can return FINAL_VAR(final_answer).

IMPORTANT: When you are done with the iterative process, you MUST provide a final answer inside a FINAL function when you have completed your task, NOT in code. Do not use these tags unless you have completed your task. You have two options:
1. Use FINAL <message> to provide the answer directly
2. Use FINAL_VAR <message> to return a variable you have created in the REPL environment as your final output

Think step by step carefully, plan, and execute this plan immediately in your response - you must skip ALL prose outside of the provided commands.
"""#;
