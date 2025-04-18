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
            You are a Crypto Trading Strategist, an expert in digital asset markets with deep expertise in technical analysis (TA), on-chain analytics, and macroeconomic factors.
            Your role is to provide data-driven insights, trading strategies, and risk-aware guidance")
        .agent_name("Crypto Trading Strategist")
        .user_name("User")
        .add_stdio_mcp_server("cargo", ["run", "--package", "binance-tools"])
        .await
        // .add_sse_mcp_server("binance api", "http://127.0.0.1:8000/sse").await
        .retry_attempts(3)
        .max_loops(300) // max loops for the agent
        .enable_autosave()
        .save_state_dir("./temp/binance-swarms-agent")
        .build();

    let response = agent
        .run("
            I currently hold BTC and ETH and want to evaluate short-term market trends for trading opportunities. Please provide analysis covering:

            Market Structure
            What are the key support/resistance levels based on recent price action?
            How does current order book liquidity distribution reinforce these levels?
            Sentiment & Capital Flows
            Show me volume patterns over 24hrs - any signs of institutional accumulation/distribution?
            How tight are bid-ask spreads currently? Does market depth suggest trend continuation risks?
            Actionable Strategy
            Based on confluence: Would you favor buying dips or waiting for breakout confirmation in the next 48hrs?
            Define specific alert levels: Where would you set stop-loss and take-profit targets?
            Risk parameters: Medium tolerance (max 10% drawdown per trade).
        ".to_owned())
        .await
        .unwrap();

    println!("{response}");
    Ok(())
}
