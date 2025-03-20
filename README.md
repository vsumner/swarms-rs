# swarms-rs

**The Enterprise-Grade, Production-Ready Multi-Agent Orchestration Framework in Rust**

![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)

## Overview

`Swarms-rs` is an enterprise-grade, production-ready multi-agent orchestration framework built in Rust, designed to handle the most demanding tasks with unparalleled speed and efficiency. Leveraging Rust's bleeding-edge performance and safety features, `swarms-rs` provides a powerful and scalable solution for orchestrating complex multi-agent systems across various industries.

## Key Benefits

### ⚡ **Extreme Performance**
- **Multi-Threaded Architecture**: Utilize the full potential of modern multi-core processors with Rust's zero-cost abstractions and fearless concurrency. `Swarms-rs` ensures that your agents run with minimal overhead, achieving maximum throughput and efficiency.
- **Bleeding-Edge Speed**: Written in Rust, `swarms-rs` delivers near-zero latency and lightning-fast execution, making it the ideal choice for high-frequency and real-time applications.

### 🛡 **Enterprise-Grade Reliability**
- **Memory Safety**: Rust's ownership model guarantees memory safety without the need for a garbage collector, ensuring that your multi-agent systems are free from data races and memory leaks.
- **Production-Ready**: Designed for real-world deployment, `swarms-rs` is ready to handle mission-critical tasks with robustness and reliability that you can depend on.

### 🧠 **Powerful Orchestration**
- **Advanced Agent Coordination**: Seamlessly manage and coordinate thousands of agents, allowing them to communicate and collaborate efficiently to achieve complex goals.
- **Extensible and Modular**: `Swarms-rs` is highly modular, allowing developers to easily extend and customize the framework to suit specific use cases.

### 🚀 **Scalable and Efficient**
- **Optimized for Scale**: Whether you're orchestrating a handful of agents or scaling up to millions, `swarms-rs` is designed to grow with your needs, maintaining top-tier performance at every level.
- **Resource Efficiency**: Maximize the use of system resources with Rust's fine-grained control over memory and processing power, ensuring that your agents run optimally even under heavy loads.

## Getting Started

### Prerequisites

- Rust (latest stable version recommended)
- Cargo package manager
- An API key for your LLM provider (OpenAI, DeepSeek, etc.)

### Installation

Add `swarms-rs` to your `Cargo.toml`:

```toml
[dependencies]
swarms-rs = "1.0"
```

For development, clone the repository and build from source:

```bash
git clone https://github.com/yourusername/swarms-rs.git
cd swarms-rs
cargo build --release
```

### Environment Setup

Create a `.env` file in your project root with your API credentials:

```
OPENAI_API_KEY=your_openai_key_here
OPENAI_BASE_URL=https://api.openai.com/v1

# Or for DeepSeek
DEEPSEEK_API_KEY=your_deepseek_key_here
DEEPSEEK_BASE_URL=https://api.deepseek.com/v1
```

## Usage Examples

### Basic Agent Example

```rust
use std::env;
use anyhow::Result;
use swarms_rs::{llm::provider::openai::OpenAI, structs::agent::Agent};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables and set up logging
    dotenv::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(
            tracing_subscriber::fmt::layer()
                .with_line_number(true)
                .with_file(true),
        )
        .init();

    // Initialize the LLM client
    let base_url = env::var("DEEPSEEK_BASE_URL").unwrap();
    let api_key = env::var("DEEPSEEK_API_KEY").unwrap();
    let client = OpenAI::from_url(base_url, api_key).set_model("deepseek-chat");
    
    // Create an agent with the builder pattern
    let agent = client
        .agent_builder()
        .system_prompt("You are a helpful assistant.")
        .agent_name("SwarmsAgent")
        .user_name("User")
        .enable_autosave()
        .max_loops(1)
        .save_sate_path("./temp/agent1_state.json")
        .enable_plan("Split the task into subtasks.".to_owned())
        .build();
    
    // Run the agent with a task
    let response = agent
        .run("What is the meaning of life?".to_owned())
        .await
        .unwrap();
    
    println!("{response}");

    Ok(())
}
```

### Multi-Agent Collaboration

```rust
use swarms_rs::{
    llm::provider::openai::OpenAI,
    structs::agent::Agent,
    swarm::Swarm,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize OpenAI client
    let client = OpenAI::new(env::var("OPENAI_API_KEY").unwrap())
        .set_model("gpt-4");
    
    // Create specialized agents
    let researcher = client
        .agent_builder()
        .system_prompt("You are a research specialist who finds accurate information.")
        .agent_name("Researcher")
        .build();
        
    let writer = client
        .agent_builder()
        .system_prompt("You are a skilled writer who creates engaging content.")
        .agent_name("Writer")
        .build();
        
    let editor = client
        .agent_builder()
        .system_prompt("You are a detail-oriented editor who improves text quality.")
        .agent_name("Editor")
        .build();
    
    // Create a swarm with the agents
    let mut swarm = Swarm::new()
        .add_agent(researcher)
        .add_agent(writer)
        .add_agent(editor);
    
    // Execute a workflow with the swarm
    let result = swarm
        .execute_workflow("Write a comprehensive article about quantum computing", |task, agents| {
            // Define your workflow logic here
            // For example: research → write → edit
        })
        .await?;
        
    println!("Final result: {}", result);
    
    Ok(())
}
```

### Adding Custom Tools

```rust
use swarms_rs::{
    llm::provider::openai::OpenAI,
    structs::agent::Agent,
    structs::tool::{Tool, ToolResult},
};

// Define a custom tool
struct WebSearchTool;

impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }
    
    fn description(&self) -> &str {
        "Search the web for up-to-date information on a topic"
    }
    
    async fn call(&self, args: String) -> ToolResult {
        // Implement web search functionality
        // This is a simplified example
        let query = serde_json::from_str::<serde_json::Value>(&args)?;
        let search_term = query["query"].as_str().unwrap_or_default();
        
        // In a real implementation, you would call a search API here
        Ok(format!("Search results for: {}", search_term))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = OpenAI::new(env::var("OPENAI_API_KEY").unwrap())
        .set_model("gpt-4");
    
    // Create an agent with the custom tool
    let agent = client
        .agent_builder()
        .system_prompt("You are a research assistant with web search capabilities.")
        .agent_name("ResearchAgent")
        .add_tool(WebSearchTool)
        .build();
    
    let response = agent
        .run("Find the latest information about Rust programming language")
        .await?;
    
    println!("{}", response);
    
    Ok(())
}
```

## Advanced Configuration

### Agent Configuration Options

```rust
// Create a highly customized agent
let agent = client
    .agent_builder()
    .system_prompt("You are an AI assistant specialized in financial analysis.")
    .agent_name("FinanceGPT")
    .user_name("Analyst")
    .description("Financial analysis and investment recommendation agent")
    .temperature(0.3)  // Lower temperature for more deterministic outputs
    .max_tokens(2000)  // Longer responses for detailed analysis
    .max_loops(5)      // Allow multiple thinking steps
    .enable_plan("Break down the financial analysis into clear steps.".to_owned())
    .enable_autosave()
    .retry_attempts(3)
    .save_sate_path("./data/finance_agent_state.json")
    .add_stop_word("ANALYSIS COMPLETE")
    .build();
```

### Parallel Task Processing

```rust
// Process multiple tasks in parallel
let tasks = vec![
    "Analyze Tesla stock performance".to_string(),
    "Evaluate Bitcoin investment potential".to_string(),
    "Compare S&P 500 vs NASDAQ returns".to_string(),
];

let results = agent.run_multiple_tasks(tasks).await?;

for (i, result) in results.iter().enumerate() {
    println!("Task {} result: {}", i+1, result);
}
```

## Architecture

`swarms-rs` is built with a modular architecture that allows for easy extension and customization:

- **Agent Layer**: Core agent implementation with memory management and tool integration
- **LLM Provider Layer**: Abstraction for different LLM providers (OpenAI, DeepSeek, etc.)
- **Tool System**: Extensible tool framework for adding capabilities to agents
- **Swarm Orchestration**: Coordination of multiple agents for complex workflows
- **Persistence Layer**: State management and recovery mechanisms

## Contributing

We welcome contributions from the community! Please see our [CONTRIBUTING.md](link_to_contributing.md) for guidelines on how to get involved.

### Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/swarms-rs.git
   cd swarms-rs
   ```

2. Install development dependencies:
   ```bash
   cargo install cargo-watch cargo-expand cargo-audit
   ```

3. Run tests:
   ```bash
   cargo test
   ```

4. Start development with auto-reload:
   ```bash
   cargo watch -x 'test -- --nocapture'
   ```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contact

For questions, suggestions, or feedback, please open an issue or contact us at [kye@swarms.world](mailto:kye@swarms.world).
