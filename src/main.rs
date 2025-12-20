use crate::llm::RigRlm;

pub mod llm;
pub mod repl;

#[tokio::main]
async fn main() {
    let rlm = RigRlm::new();

    let prompt = "Please write a CRUD server using the Flask framework in Python and write the code to a file in my current folder";
    println!("Query: {prompt}");
    let response = rlm.query(prompt).await.unwrap();

    println!("Response: {response}");
}
