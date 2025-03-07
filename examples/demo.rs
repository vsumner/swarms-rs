use anyhow::Result;
use swarms_rust::{Agent, Config};

#[tokio::main]
async fn main() -> Result<()> {
    // Load the shared configuration.
    let config = Config::from_env()?;
    
    // Create an agent and override system prompt and temperature directly.
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
        llm: None, // Use default integrated LLM for analysis.
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
