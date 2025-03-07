use anyhow::Result;
use swarms_rust::{Agent, Config, DefaultLLM};

#[tokio::main]
async fn main() -> Result<()> {
    // Load the configuration from environment variables (.env file is supported)
    let mut config = Config::from_env()?;
    // Override the default model name to use a supported model (e.g., "gpt-3.5-turbo")
    config.openai_model = "gpt-3.5-turbo".to_string();
    
    // Create an agent instance.
    // The llm field is left as None so that the agent uses DefaultLLM automatically.
    let mut agent = Agent {
        config,
        user_name: "User".to_string(),
        agent_name: "AgentX".to_string(),
        plan_enabled: true,
        max_loops: 3,
        retry_attempts: 3,
        dynamic_temperature_enabled: false,
        autosave: false,
        interactive: false,
        custom_exit_command: "exit".to_string(),
        loop_interval: Some(1),
        output_type: "string".to_string(),
        short_memory: Vec::new(),
        llm: None, // No custom LLM provided; DefaultLLM will be used.
        system_prompt: Some("You are a helpful assistant with advanced reasoning.".to_string()),
        temperature: Some(0.9),
    };

    // Run the agent with a sample task prompt.
    let final_output = agent
        .run(
            Some("What is the capital of France?".to_string()),
            None,
            None,
            None,
            Some(true),
            Some(true),
            Some(false),
            None,
        )
        .await?;

    println!("Final Output:\n{}", final_output);
    Ok(())
}
