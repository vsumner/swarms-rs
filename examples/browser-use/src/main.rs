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
        .system_prompt("
            You are an AI-powered Financial Research Analyst with expertise in real-time market data analysis,
            regulatory compliance tracking, and cross-border capital flow pattern recognition.
            Your core capability lies in conducting evidence-based financial investigations through advanced web search and multi-source validation.
        ")
        .agent_name("Financial Researcher")
        .user_name("User")
        // this repo is a fork, but fix some bugs
        .add_stdio_mcp_server("uvx", ["--from", "git+https://github.com/deciduus/mcp-browser-use","mcp-server-browser-use"])
        .await
        // .add_sse_mcp_server("binance api", "http://127.0.0.1:8000/sse").await
        .retry_attempts(3)
        .max_loops(300) // max loops for the agent
        .enable_autosave()
        .save_state_dir("./temp/browser-use-agent")
        .build();

    let response = agent
        .run("
            Analyze the impact of Federal Reserve interest rate policy on U.S. technology stocks (2019-2025), focusing on:

            1. Historical correlation between rate cycles and NASDAQ volatility
            2. Differential impacts across tech subsectors (Semiconductors, Cloud Services, AI Startups)
            3. Corporate treasury management strategies (Apple, Microsoft, NVIDIA case studies)
            4. Institutional investor positioning changes (13F filings analysis)
            5. Emerging regulatory risks in capital allocation patterns
        ".to_owned())
        .await
        .unwrap();

    println!("{response}");
    Ok(())
}
