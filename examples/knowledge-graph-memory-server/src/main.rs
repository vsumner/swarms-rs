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
        .system_prompt(r#"
        Follow these steps for each interaction:

        1. User Identification:
        - You should assume that you are interacting with default_user
        - If you have not identified default_user, proactively try to do so.

        2. Memory Retrieval:
        - Always begin your chat by saying only "Remembering..." and retrieve all relevant information from your knowledge graph
        - Always refer to your knowledge graph as your "memory"

        3. Memory
        - While conversing with the user, be attentive to any new information that falls into these categories:
            a) Basic Identity (age, gender, location, job title, education level, etc.)
            b) Behaviors (interests, habits, etc.)
            c) Preferences (communication style, preferred language, etc.)
            d) Goals (goals, targets, aspirations, etc.)
            e) Relationships (personal and professional relationships up to 3 degrees of separation)

        4. Memory Update:
        - If any new information was gathered during the interaction, update your memory as follows:
            a) Create entities for recurring organizations, people, and significant events
            b) Connect them to the current entities using relations
            b) Store facts about them as observations"#.to_owned())
        .agent_name("HelpfulAssistant")
        .user_name("Alice")
        .add_stdio_mcp_server("npx", ["-y", "@modelcontextprotocol/server-memory"])
        .await
        // .add_sse_mcp_server("binance api", "http://127.0.0.1:8000/sse").await
        .retry_attempts(3)
        .max_loops(300) // max loops for the agent
        .enable_autosave()
        .save_state_dir("./temp/memory")
        .disable_concurrent_tool_call()
        .build();

    let response = agent
        .run(
            "
        Ilya is a software engineer.
        He is a good friend of Alice.
        Alice is a good friend of Ilya.
        Alice is a good friend of Bob.
        Bob is a good friend of Ilya.
        Ilya likes to play chess.
        They all have a cat and a dog.
        They all speak English.
    "
            .to_owned(),
        )
        .await
        .unwrap();

    println!("{response}");
    Ok(())
}
