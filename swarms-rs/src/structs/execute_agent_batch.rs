use std::thread::available_parallelism;

use dashmap::DashMap;
use futures::{StreamExt, stream};
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use crate::structs::{
    agent::{Agent, AgentError},
    conversation::AgentConversation,
};

/// Error type for batch execution
#[derive(Debug, thiserror::Error)]
pub enum BatchExecutionError {
    #[error("Agent error: {0}")]
    AgentError(#[from] AgentError),
    #[error("No agents provided")]
    NoAgents,
    #[error("No tasks provided")]
    NoTasks,
    #[error("Channel error: {0}")]
    ChannelError(#[from] mpsc::error::SendError<(String, Result<AgentConversation, AgentError>)>),
}

/// Configuration for batch execution
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum number of concurrent tasks per agent
    pub max_concurrent_tasks: Option<usize>,
    /// Whether to enable automatic CPU optimization
    pub auto_cpu_optimization: bool,
    /// Custom number of worker threads (overrides auto_cpu_optimization if set)
    pub worker_threads: Option<usize>,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: None,
            auto_cpu_optimization: true,
            worker_threads: None,
        }
    }
}

/// Builder for BatchConfig
#[derive(Default)]
pub struct BatchConfigBuilder {
    config: BatchConfig,
}

impl BatchConfigBuilder {
    pub fn max_concurrent_tasks(mut self, max: usize) -> Self {
        self.config.max_concurrent_tasks = Some(max);
        self
    }

    pub fn auto_cpu_optimization(mut self, enable: bool) -> Self {
        self.config.auto_cpu_optimization = enable;
        self
    }

    pub fn worker_threads(mut self, threads: usize) -> Self {
        self.config.worker_threads = Some(threads);
        self
    }

    pub fn build(self) -> BatchConfig {
        self.config
    }
}

/// Executes a batch of tasks across multiple agents concurrently
pub struct AgentBatchExecutor {
    agents: Vec<Box<dyn Agent>>,
    config: BatchConfig,
}

impl AgentBatchExecutor {
    /// Creates a new batch executor with the given agents and configuration
    pub fn new(agents: Vec<Box<dyn Agent>>, config: BatchConfig) -> Self {
        Self { agents, config }
    }

    /// Creates a new batch executor builder
    pub fn builder() -> AgentBatchExecutorBuilder {
        AgentBatchExecutorBuilder::default()
    }

    /// Calculates the optimal number of worker threads based on system resources
    fn calculate_optimal_threads(&self) -> usize {
        if let Some(threads) = self.config.worker_threads {
            return threads;
        }

        if !self.config.auto_cpu_optimization {
            return 4; // Default fallback
        }

        match available_parallelism() {
            Ok(num_cpus) => {
                let cpus = num_cpus.get();
                debug!("Detected {} CPU cores", cpus);
                cpus
            },
            Err(e) => {
                error!("Failed to determine CPU count: {}", e);
                4 // Default fallback
            },
        }
    }

    /// Executes a batch of tasks across all agents concurrently
    pub async fn execute_batch(
        &self,
        tasks: Vec<String>,
    ) -> Result<DashMap<String, AgentConversation>, BatchExecutionError> {
        if self.agents.is_empty() {
            return Err(BatchExecutionError::NoAgents);
        }
        if tasks.is_empty() {
            return Err(BatchExecutionError::NoTasks);
        }

        let results = DashMap::with_capacity(tasks.len());
        let (tx, mut rx) = mpsc::channel(tasks.len());
        let max_concurrent = self
            .config
            .max_concurrent_tasks
            .unwrap_or_else(|| self.calculate_optimal_threads());

        info!(
            "Starting batch execution with {} tasks across {} agents (max concurrent: {})",
            tasks.len(),
            self.agents.len(),
            max_concurrent
        );

        // Execute tasks concurrently
        stream::iter(tasks)
            .for_each_concurrent(max_concurrent, |task| {
                let tx = tx.clone();
                let agents = &self.agents;
                async move {
                    for agent in agents {
                        match agent.run(task.clone()).await {
                            Ok(response) => {
                                let mut conversation = AgentConversation::new(agent.name());
                                conversation.add(
                                    crate::structs::conversation::Role::Assistant(agent.name()),
                                    response,
                                );
                                tx.send((task.clone(), Ok(conversation))).await.unwrap();
                            },
                            Err(e) => {
                                error!(
                                    "Agent {} failed to process task '{}': {}",
                                    agent.name(),
                                    task,
                                    e
                                );
                                tx.send((task.clone(), Err(e))).await.unwrap();
                            },
                        }
                    }
                }
            })
            .await;

        drop(tx);

        // Collect results
        while let Some((task, result)) = rx.recv().await {
            match result {
                Ok(conversation) => {
                    results.insert(task, conversation);
                },
                Err(e) => {
                    error!("Task failed: {}", e);
                },
            }
        }

        info!("Batch execution completed with {} results", results.len());
        Ok(results)
    }
}

/// Builder for AgentBatchExecutor
#[derive(Default)]
pub struct AgentBatchExecutorBuilder {
    agents: Vec<Box<dyn Agent>>,
    config: BatchConfig,
}

impl AgentBatchExecutorBuilder {
    pub fn add_agent(mut self, agent: Box<dyn Agent>) -> Self {
        self.agents.push(agent);
        self
    }

    pub fn config(mut self, config: BatchConfig) -> Self {
        self.config = config;
        self
    }

    pub fn build(self) -> AgentBatchExecutor {
        AgentBatchExecutor::new(self.agents, self.config)
    }
}
