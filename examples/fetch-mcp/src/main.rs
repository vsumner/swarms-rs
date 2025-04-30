use std::env;

use anyhow::Result;
use swarms_rs::{llm::provider::openai::OpenAI, structs::agent::Agent};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(
            tracing_subscriber::fmt::layer()
                .with_line_number(true)
                .with_file(true),
        )
        .init();

    let base_url = env::var("DEEPSEEK_BASE_URL").unwrap();
    let api_key = env::var("DEEPSEEK_API_KEY").unwrap();
    let client = OpenAI::from_url(base_url, api_key).set_model("deepseek-chat");
    let agent = client
        .agent_builder()
        .system_prompt("You are a helpful assistant.".to_owned())
        .agent_name("HelpfulAssistant")
        .user_name("Alice")
        .add_stdio_mcp_server(
            "uvx",
            [
                "mcp-server-fetch",
                "--ignore-robots-txt",
                "--user-agent",
                "swarms-agent",
                // Proxy, if needed
                // "--proxy-url",
                // "http://127.0.0.1:55535",
            ],
        )
        .await
        // .add_sse_mcp_server("binance api", "http://127.0.0.1:8000/sse").await
        .retry_attempts(3)
        .max_loops(300) // max loops for the agent
        .enable_autosave()
        .save_state_dir("./temp/fetch-agent")
        .build();

    let response = agent
        .run(
            r#"Fetch Hacker News, after that, fetch each news page, and get the title and url of the top 5 articles.
             Finally, return a json array of the top 5 articles.
             
             The format should be:
             [
                {
                    "title": "title",
                    "url": "url"
                    "metadata": "metadata"
                },
                ...
             ]
             "#
            .to_owned(),
        )
        .await
        .unwrap();

    println!("{response}");
    Ok(())
}
