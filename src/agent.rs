use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    error::Error,
    fmt,
    fs::{self, File},
    io::Write,
    path::Path,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use thiserror::Error;
use tokio::{
    sync::{Mutex, RwLock},
    time::{sleep, Duration},
};
use tracing::{error, info, instrument};
use uuid::Uuid;

// Custom error types
#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Execution error: {0}")]
    ExecutionError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

// Agent output types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputType {
    String,
    List,
    Json,
    Dict,
    Yaml,
}

// Core agent trait that any LLM implementation must fulfill
#[async_trait]
pub trait LanguageModel: Send + Sync {
    async fn run(&self, task: &str) -> Result<String, Box<dyn Error + Send + Sync>>;
    fn name(&self) -> &str;
}

// Response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent_id: String,
    pub agent_name: String,
    pub max_loops: usize,
    pub loop_interval: u64,
    pub retry_attempts: usize,
    pub retry_interval: u64,
    pub output_type: OutputType,
    pub temperature: f32,
    pub max_tokens: usize,
    pub system_prompt: Option<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            agent_id: Uuid::new_v4().to_string(),
            agent_name: "default-agent".to_string(),
            max_loops: 1,
            loop_interval: 0,
            retry_attempts: 3,
            retry_interval: 1,
            output_type: OutputType::String,
            temperature: 0.7,
            max_tokens: 2048,
            system_prompt: None,
        }
    }
}

// Main Agent struct
pub struct Agent {
    config: AgentConfig,
    llm: Arc<Box<dyn LanguageModel>>,
    response_history: Arc<RwLock<Vec<AgentResponse>>>,
    workspace_dir: String,
}

impl Agent {
    pub fn new(
        config: AgentConfig,
        llm: Box<dyn LanguageModel>,
        workspace_dir: Option<String>,
    ) -> Result<Self, AgentError> {
        // Validate configuration
        Self::validate_config(&config)?;

        // Create workspace directory if it doesn't exist
        let workspace = workspace_dir.unwrap_or_else(|| "agent_workspace".to_string());
        fs::create_dir_all(&workspace).map_err(AgentError::IoError)?;

        Ok(Self {
            config,
            llm: Arc::new(llm),
            response_history: Arc::new(RwLock::new(Vec::new())),
            workspace_dir: workspace,
        })
    }

    fn validate_config(config: &AgentConfig) -> Result<(), AgentError> {
        if config.max_loops == 0 {
            return Err(AgentError::ConfigError("max_loops must be greater than 0".into()));
        }
        if config.max_tokens == 0 {
            return Err(AgentError::ConfigError("max_tokens must be greater than 0".into()));
        }
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn run(&self, task: &str) -> Result<String, AgentError> {
        info!("Starting agent execution for task: {}", task);
        
        let mut final_response = String::new();
        let mut loop_count = 0;

        while loop_count < self.config.max_loops {
            loop_count += 1;
            info!("Executing loop {} of {}", loop_count, self.config.max_loops);

            let response = self.execute_with_retry(task).await?;
            
            // Store response in history
            let agent_response = AgentResponse {
                content: response.clone(),
                timestamp: Utc::now(),
                metadata: HashMap::new(),
            };
            
            self.response_history.write().await.push(agent_response);
            
            final_response = response;

            if self.config.loop_interval > 0 {
                sleep(Duration::from_secs(self.config.loop_interval)).await;
            }
        }

        Ok(self.format_output(final_response).await?)
    }

    async fn execute_with_retry(&self, task: &str) -> Result<String, AgentError> {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.config.retry_attempts {
            match self.llm.run(task).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    attempts += 1;
                    last_error = Some(e);
                    if attempts < self.config.retry_attempts {
                        sleep(Duration::from_secs(self.config.retry_interval)).await;
                    }
                }
            }
        }

        Err(AgentError::ExecutionError(format!(
            "Failed after {} attempts. Last error: {}",
            attempts,
            last_error.unwrap()
        )))
    }

    async fn format_output(&self, response: String) -> Result<String, AgentError> {
        match self.config.output_type {
            OutputType::String => Ok(response),
            OutputType::List => Ok(format!("[{}]", response)),
            OutputType::Json => serde_json::to_string(&response)
                .map_err(|e| AgentError::SerializationError(e.to_string())),
            OutputType::Dict => serde_json::to_string(&response)
                .map_err(|e| AgentError::SerializationError(e.to_string())),
            OutputType::Yaml => serde_yaml::to_string(&response)
                .map_err(|e| AgentError::SerializationError(e.to_string())),
        }
    }

    pub async fn save_state(&self) -> Result<(), AgentError> {
        let state_path = Path::new(&self.workspace_dir).join("agent_state.json");
        let history = self.response_history.read().await;
        
        let serialized = serde_json::to_string_pretty(&*history)
            .map_err(|e| AgentError::SerializationError(e.to_string()))?;
            
        let mut file = File::create(state_path)
            .map_err(AgentError::IoError)?;
            
        file.write_all(serialized.as_bytes())
            .map_err(AgentError::IoError)?;
            
        Ok(())
    }

    pub async fn load_state(&self) -> Result<(), AgentError> {
        let state_path = Path::new(&self.workspace_dir).join("agent_state.json");
        
        let contents = fs::read_to_string(state_path)
            .map_err(AgentError::IoError)?;
            
        let history: Vec<AgentResponse> = serde_json::from_str(&contents)
            .map_err(|e| AgentError::SerializationError(e.to_string()))?;
            
        *self.response_history.write().await = history;
        
        Ok(())
    }

    // Batch processing functionality
    pub async fn run_batch(&self, tasks: Vec<String>) -> Result<Vec<String>, AgentError> {
        let futures: Vec<_> = tasks.iter()
            .map(|task| self.run(task))
            .collect();
            
        let results = join_all(futures).await;
        
        results.into_iter().collect()
    }

    // Graceful shutdown
    pub async fn shutdown(&self) -> Result<(), AgentError> {
        info!("Initiating agent shutdown");
        self.save_state().await?;
        info!("Agent shutdown complete");
        Ok(())
    }
}

// Example implementation of a simple LLM
pub struct SimpleLLM {
    name: String,
}

#[async_trait]
impl LanguageModel for SimpleLLM {
    async fn run(&self, task: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        // Simple echo implementation - replace with actual LLM logic
        Ok(format!("Processed task: {}", task))
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_agent_basic_execution() {
        let config = AgentConfig {
            agent_name: "test-agent".to_string(),
            ..AgentConfig::default()
        };

        let llm = SimpleLLM {
            name: "test-llm".to_string(),
        };

        let agent = Agent::new(config, Box::new(llm), None).unwrap();
        let result = agent.run("test task").await.unwrap();
        assert!(result.contains("test task"));
    }

    #[test]
    async fn test_agent_batch_processing() {
        let config = AgentConfig::default();
        let llm = SimpleLLM {
            name: "test-llm".to_string(),
        };

        let agent = Agent::new(config, Box::new(llm), None).unwrap();
        let tasks = vec![
            "task1".to_string(),
            "task2".to_string(),
            "task3".to_string(),
        ];

        let results = agent.run_batch(tasks).await.unwrap();
        assert_eq!(results.len(), 3);
    }
}
