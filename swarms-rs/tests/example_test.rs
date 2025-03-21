use anyhow::Result;
use swarms_rs::{llm::provider::openai::OpenAI, structs::agent::Agent};

#[tokio::test]
async fn test_basic_agent_functionality() -> Result<()> {
    // Skip test if environment variables are not set
    if std::env::var("DEEPSEEK_BASE_URL").is_err() || std::env::var("DEEPSEEK_API_KEY").is_err() {
        println!("Skipping test_basic_agent_functionality: Required environment variables not set");
        return Ok(());
    }

    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Set up the OpenAI client with DeepSeek configuration
    let base_url = std::env::var("DEEPSEEK_BASE_URL").unwrap();
    let api_key = std::env::var("DEEPSEEK_API_KEY").unwrap();
    let client = OpenAI::from_url(base_url, api_key).set_model("deepseek-chat");

    // Create the agent
    let agent = client
        .agent_builder()
        .system_prompt("You are a helpful assistant.")
        .agent_name("TestAgent")
        .user_name("TestUser")
        .enable_autosave()
        .max_loops(1)
        .save_sate_path("./temp/test_agent_state.json")
        .enable_plan("Split the task into subtasks.".to_owned())
        .build();

    // Test the agent with a simple query
    let response = agent.run("What is 2+2?".to_owned()).await?;

    // Basic assertions
    assert!(!response.is_empty(), "Response should not be empty");
    assert!(
        response.len() > 10,
        "Response should be reasonably detailed"
    );
    assert!(
        response.to_lowercase().contains("4"),
        "Response should contain the answer '4'"
    );

    Ok(())
}

#[tokio::test]
async fn test_agent_error_handling() -> Result<()> {
    // Test with invalid credentials
    let client = OpenAI::from_url(
        "https://invalid-url.com".to_string(),
        "invalid-key".to_string(),
    )
    .set_model("deepseek-chat");

    let agent = client
        .agent_builder()
        .system_prompt("You are a helpful assistant.")
        .agent_name("TestAgent")
        .user_name("TestUser")
        .build();

    // This should return an error
    let result = agent.run("Test query".to_owned()).await;

    // Check that we got an error, but don't be specific about the error type
    assert!(
        result.is_err(),
        "Expected an error with invalid credentials"
    );

    // Print the error for debugging purposes
    if let Err(e) = result {
        println!("Received expected error: {:?}", e);
    }

    Ok(())
}

// Helper function to create a mock agent for testing
#[cfg(test)]
fn create_mock_agent() -> impl Agent {
    let client = OpenAI::from_url("https://mock-url.com".to_string(), "mock-key".to_string())
        .set_model("mock-model");

    client
        .agent_builder()
        .system_prompt("You are a test assistant.")
        .agent_name("MockAgent")
        .user_name("TestUser")
        .build()
}

#[tokio::test]
async fn test_agent_creation() -> Result<()> {
    let agent = create_mock_agent();
    assert_eq!(agent.name(), "MockAgent");
    Ok(())
}
