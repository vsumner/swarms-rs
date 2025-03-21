# swarms-rs

**The Enterprise-Grade, Production-Ready Multi-Agent Orchestration Framework in Rust**

![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)

## Overview

`swarms-rs` is an enterprise-grade, production-ready multi-agent orchestration framework built in Rust, designed to handle the most demanding tasks with unparalleled speed and efficiency. Leveraging Rust's bleeding-edge performance and safety features, `swarms-rs` provides a powerful and scalable solution for orchestrating complex multi-agent systems across various industries.

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
swarms-rs = "0.1"
```

For development, clone the repository and build from source:

```bash
git clone https://github.com/The-Swarm-Corporation/swarms-rs
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

## Run Examples

In [swarms-rs/examples](swarms-rs/examples/) there is our sample code, which can provide a considerable degree of reference:

To run the graph workflow example:

```bash
cargo run --example graph_workflow
```

`DEEPSEEK_API_KEY` and `DEEPSEEK_BASE_URL` environment variables are read by default.

## Architecture

`swarms-rs` is built with a modular architecture that allows for easy extension and customization:

- **Agent Layer**: Core agent implementation with memory management and tool integration
- **LLM Provider Layer**: Abstraction for different LLM providers (OpenAI, DeepSeek, etc.)
- **Tool System**: Extensible tool framework for adding capabilities to agents
- **Swarm Orchestration**: Coordination of multiple agents for complex workflows
- **Persistence Layer**: State management and recovery mechanisms

### Development Setup

1. Clone the repository:

   ```bash
   git clone https://github.com/The-Swarm-Corporation/swarms-rs
   cd swarms-rs
   ```

2. Install development dependencies:

   ```bash
   cargo install cargo-nextest
   ```

3. Run tests:

   ```bash
   cargo nextest run
   ```

4. Run benchmarks:

   ```bash
   cargo bench
   ```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contact

For questions, suggestions, or feedback, please open an issue or contact us at [kye@swarms.world](mailto:kye@swarms.world).
