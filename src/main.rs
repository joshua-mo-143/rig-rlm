use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};

use crate::llm::RigRlm;

pub mod exec;
pub mod llm;
pub mod repl;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::TRACE.into())
                .parse("rlm=trace")
                .unwrap(),
        )
        .with(Layer::new())
        .init();

    let rlm = RigRlm::new_local();

    let prompt = "Please write a CRUD server using the Flask framework in Python and write the code to a file in my current folder. Please additionally provide a requirements.txt file with all required dependencies.";
    println!("Query: {prompt}");
    let response = rlm.query(prompt).await.unwrap();

    println!("Response: {response}");
}
