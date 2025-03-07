// examples/demo.rs

use swarms_rust::{Config, ModelProvider, call_model_api};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration from environment variables (.env file is supported)
    let config = Config::from_env()?;
    println!("Loaded config: {:?}", config);

    // Call the OpenAI API using the chat completions endpoint.
    let response = call_model_api(ModelProvider::OpenAI, "Hello, world!", &config).await?;
    println!("Response: {}", response);

    Ok(())
}
