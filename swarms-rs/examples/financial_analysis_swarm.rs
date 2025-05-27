use dotenv::dotenv;
use std::env;
use std::error::Error;
use swarms_rs::structs::swarms_client::{SwarmType, SwarmsClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load .env file
    dotenv().ok();

    // Get API key from environment variable
    let api_key = env::var("SWARMS_API_KEY").map_err(|_| {
        eprintln!("Error: SWARMS_API_KEY not found in environment or .env file");
        eprintln!("Please either:");
        eprintln!("1. Set it in your .env file: SWARMS_API_KEY='your-api-key'");
        eprintln!("2. Or export it: export SWARMS_API_KEY='your-api-key'");
        "SWARMS_API_KEY not set"
    })?;

    // Initialize the client
    let client = SwarmsClient::builder()
        .unwrap()
        .api_key(api_key)
        .timeout(std::time::Duration::from_secs(120))
        .max_retries(3)
        .build()?;

    // Create a financial analysis swarm
    let response = client
        .swarm()
        .completion()
        .name("Financial Health Analysis Swarm")
        .description("A sequential workflow of specialized financial agents analyzing company health")
        .swarm_type(SwarmType::ConcurrentWorkflow)
        .task("Analyze the financial health of Apple Inc. (AAPL) based on their latest quarterly report")
        // Financial Data Collector Agent
        .agent(|agent| {
            agent
                .name("Financial Data Collector")
                .description("Specializes in gathering and organizing financial data from various sources")
                .model("gpt-4o")
                .system_prompt("You are a financial data collection specialist. Your role is to gather and organize relevant financial data, including revenue, expenses, profit margins, and key financial ratios. Present the data in a clear, structured format.")
                .temperature(0.7)
                .max_tokens(2000)
        })
        // Financial Ratio Analyzer Agent
        .agent(|agent| {
            agent
                .name("Ratio Analyzer")
                .description("Analyzes key financial ratios and metrics")
                .model("gpt-4o")
                .system_prompt("You are a financial ratio analysis expert. Your role is to calculate and interpret key financial ratios such as P/E ratio, debt-to-equity, current ratio, and return on equity. Provide insights on what these ratios indicate about the company's financial health.")
                .temperature(0.7)
                .max_tokens(2000)
        })
        // Trend Analysis Agent
        .agent(|agent| {
            agent
                .name("Trend Analyst")
                .description("Identifies and analyzes financial trends and patterns")
                .model("gpt-4o")
                .system_prompt("You are a financial trend analysis specialist. Your role is to identify patterns and trends in the financial data, compare with historical performance, and predict potential future developments. Focus on revenue growth, profit margins, and market position trends.")
                .temperature(0.7)
                .max_tokens(2000)
        })
        // Risk Assessment Agent
        .agent(|agent| {
            agent
                .name("Risk Assessor")
                .description("Evaluates financial risks and potential concerns")
                .model("gpt-4o")
                .system_prompt("You are a financial risk assessment expert. Your role is to identify potential risks, including market risks, operational risks, and financial risks. Provide a detailed analysis of risk factors and their potential impact on the company's future performance.")
                .temperature(0.7)
                .max_tokens(2000)
        })
        // Investment Recommendation Agent
        .agent(|agent| {
            agent
                .name("Investment Advisor")
                .description("Provides investment recommendations based on analysis")
                .model("gpt-4o")
                .system_prompt("You are an investment advisory specialist. Your role is to synthesize the analysis from previous agents and provide clear, actionable investment recommendations. Consider both short-term and long-term investment perspectives.")
                .temperature(0.7)
                .max_tokens(2000)
        })
        .max_loops(1)
        .service_tier("standard")
        .send()
        .await?;

    // Print the swarm's output
    println!("Financial Analysis Results:");
    println!("{}", response.output);

    Ok(())
}
