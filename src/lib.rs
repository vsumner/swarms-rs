use anyhow::{Context, Result};
use futures::future::BoxFuture;
use reqwest::Client;
use serde_json::{json, Value};
use std::env;
use tokio::time::{sleep, Duration};

/// Holds the application configuration parameters.
///
/// All values are hard-coded defaults except for the OpenAI API key,
/// which is loaded from the environment.
#[derive(Debug, Clone)]
pub struct Config {
    /// The port on which the application will run.
    pub port: u16,
    /// API key for OpenAI (loaded from environment).
    pub openai_api_key: String,
    /// The model identifier to be used with OpenAI's chat completions.
    pub openai_model: String,
    /// API key for Anthropic (default: None).
    pub anthropic_api_key: Option<String>,
    /// API key for Google (default: None).
    pub google_api_key: Option<String>,
    /// API key for HuggingFace (default: None).
    pub huggingface_api_key: Option<String>,
    /// API key for another model provider (default: None).
    pub other_api_key: Option<String>,
    /// Optional system prompt (default: None).
    pub system_prompt: Option<String>,
    /// Temperature setting for model outputs.
    pub temperature: f32,
}

impl Config {
    /// Creates a new configuration.
    ///
    /// The OpenAI API key is loaded from the environment variable `OPENAI_API_KEY`.
    /// All other settings use default values.
    pub fn from_env() -> Result<Self> {
        let openai_api_key = env::var("OPENAI_API_KEY")
            .context("OPENAI_API_KEY must be set in the environment")?;

        Ok(Config {
            port: 8080,
            openai_api_key,
            openai_model: "gpt-4o".to_string(), // default model name
            anthropic_api_key: None,
            google_api_key: None,
            huggingface_api_key: None,
            other_api_key: None,
            system_prompt: None,
            temperature: 0.7,
        })
    }
}

/// Enum representing supported model providers.
#[derive(Debug)]
pub enum ModelProvider {
    OpenAI,
    Anthropic,
    Google,
    HuggingFace,
    Other(String),
}

/// Constants for retrying the API call.
const MAX_RETRIES: usize = 3;
const RETRY_DELAY_MS: u64 = 500;

/// Calls the API of the given model provider with the specified prompt.
///
/// For OpenAI, it sends a POST request to the chat completions endpoint and extracts the assistant's response.
/// Extensive logging is added to display the URL, payload, and response details for debugging.
pub async fn call_model_api(
    provider: ModelProvider,
    prompt: &str,
    config: &Config,
) -> Result<String> {
    println!("------------------------------");
    println!("Calling model API with prompt:\n{}", prompt);

    let client = Client::new();

    let (url, api_key) = match provider {
        ModelProvider::OpenAI => {
            let url = "https://api.openai.com/v1/chat/completions".to_string();
            println!("Using URL: {}", url);
            (url, config.openai_api_key.clone())
        }
        ModelProvider::Anthropic => {
            let key = config
                .anthropic_api_key
                .clone()
                .context("ANTHROPIC_API_KEY is not set")?;
            let url = "https://api.anthropic.com/v1/complete".to_string();
            println!("Using URL: {}", url);
            (url, key)
        }
        ModelProvider::Google => {
            let key = config
                .google_api_key
                .clone()
                .context("GOOGLE_API_KEY is not set")?;
            let url = "https://api.google.com/v1/generate".to_string();
            println!("Using URL: {}", url);
            (url, key)
        }
        ModelProvider::HuggingFace => {
            let key = config
                .huggingface_api_key
                .clone()
                .context("HUGGINGFACE_API_KEY is not set")?;
            let url = "https://api-inference.huggingface.co/models/your-model".to_string();
            println!("Using URL: {}", url);
            (url, key)
        }
        ModelProvider::Other(ref id) => {
            let key = config
                .other_api_key
                .clone()
                .context(format!("OTHER_API_KEY is not set for provider {}", id))?;
            let url = format!("https://api.{}.com/v1/complete", id);
            println!("Using URL: {}", url);
            (url, key)
        }
    };

    let payload: Value = match provider {
        ModelProvider::OpenAI => {
            // Build messages for chat completions.
            let mut messages = vec![];
            if let Some(system_prompt) = &config.system_prompt {
                messages.push(json!({"role": "developer", "content": system_prompt}));
            }
            messages.push(json!({"role": "user", "content": prompt}));
            let payload = json!({
                "model": config.openai_model,
                "messages": messages,
                "temperature": config.temperature,
                "max_tokens": 50
            });
            println!("Payload: {}", payload);
            payload
        }
        _ => {
            // Fallback payload for other providers.
            let mut payload = json!({
                "prompt": prompt,
                "temperature": config.temperature,
                "max_tokens": 50
            });
            if let Some(system_prompt) = &config.system_prompt {
                payload["system_prompt"] = json!(system_prompt);
            }
            println!("Payload: {}", payload);
            payload
        }
    };

    let mut attempt = 0;
    let response_text = loop {
        attempt += 1;
        println!("API call attempt {}/{}", attempt, MAX_RETRIES);
        let result = async {
            let resp = client
                .post(&url)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&payload)
                .send()
                .await
                .with_context(|| format!("Failed to send request to {}", url))?;
            println!("Response status: {}", resp.status());
            let resp = resp
                .error_for_status()
                .with_context(|| format!("Request to {} returned an error status", url))?;
            let text = resp.text().await.context("Failed to extract response text")?;
            println!("Raw response text: {}", text);
            Ok(text)
        }
        .await;

        match result {
            Ok(text) => break text,
            Err(err) if attempt < MAX_RETRIES => {
                eprintln!(
                    "Attempt {}/{} failed: {}. Retrying...",
                    attempt, MAX_RETRIES, err
                );
                sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
            }
            Err(err) => return Err(err),
        }
    };

    if let ModelProvider::OpenAI = provider {
        let json_response: Value =
            serde_json::from_str(&response_text).context("Failed to parse JSON response")?;
        println!("Parsed JSON response: {}", json_response);
        if let Some(content) = json_response
            .get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|c| c.as_str())
        {
            println!("API call succeeded. Extracted content: {}", content);
            return Ok(content.to_string());
        } else {
            return Err(anyhow::anyhow!(
                "Failed to extract assistant's response from JSON"
            ));
        }
    }

    println!("API call succeeded. Full response: {}", response_text);
    Ok(response_text)
}

/// Object-safe LLM trait that returns a boxed future.
pub trait LLM: Send + Sync {
    fn run(&self, task: &str) -> BoxFuture<'_, Result<String>>;
}

/// Default LLM implementation that uses the integrated model API to analyze a response.
pub struct DefaultLLM {
    pub config: Config,
}

impl LLM for DefaultLLM {
    fn run(&self, task: &str) -> BoxFuture<'_, Result<String>> {
        let config = self.config.clone();
        let prompt = format!("Please analyze the following response: {}", task);
        Box::pin(async move {
            println!("DefaultLLM analyzing with prompt:\n{}", prompt);
            call_model_api(ModelProvider::OpenAI, &prompt, &config).await
        })
    }
}

/// Represents an agent that performs tasks by querying the model API.
pub struct Agent {
    /// Configuration.
    pub config: Config,
    /// The user's name.
    pub user_name: String,
    /// The agent's name.
    pub agent_name: String,
    /// Whether planning is enabled.
    pub plan_enabled: bool,
    /// Maximum number of loops.
    pub max_loops: u32,
    /// Maximum retry attempts per loop.
    pub retry_attempts: u32,
    /// Whether dynamic temperature adjustment is enabled.
    pub dynamic_temperature_enabled: bool,
    /// Whether autosave is enabled.
    pub autosave: bool,
    /// Whether interactive mode is enabled.
    pub interactive: bool,
    /// Custom exit command.
    pub custom_exit_command: String,
    /// Optional loop interval (in seconds).
    pub loop_interval: Option<u64>,
    /// Output type (for demo, "string").
    pub output_type: String,
    /// Short-term memory (role, content) pairs.
    pub short_memory: Vec<(String, String)>,
    /// Optional LLM callable for further analysis.
    /// If None, the default integrated model API (DefaultLLM) is used.
    pub llm: Option<Box<dyn LLM>>,
    /// Agent-specific system prompt (overrides config.system_prompt if provided).
    pub system_prompt: Option<String>,
    /// Agent-specific temperature (overrides config.temperature if provided).
    pub temperature: Option<f32>,
}

impl Agent {
    /// Runs the agent.
    pub async fn run(
        &mut self,
        task: Option<String>,
        _img: Option<String>,
        _speech: Option<String>,
        _video: Option<String>,
        _is_last: Option<bool>,
        _print_task: Option<bool>,
        _generate_speech: Option<bool>,
        _correct_answer: Option<String>,
    ) -> Result<String> {
        println!("==============================");
        println!("Starting agent run...");
        // 1. Validate or auto-generate the task.
        let task = match task {
            Some(t) if !t.trim().is_empty() => t,
            _ => "Auto-generated task prompt".to_string(),
        };
        println!("Task: {}", task);

        // 2. Add the task to short-term memory.
        self.short_memory.push((self.user_name.clone(), task.clone()));

        // 3. Perform planning if enabled.
        if self.plan_enabled {
            println!("Planning enabled. Planning for task: {}", task);
        }

        // 4. Create an updated configuration overriding system_prompt and temperature if provided.
        let mut updated_config = self.config.clone();
        if let Some(prompt) = &self.system_prompt {
            updated_config.system_prompt = Some(prompt.clone());
            println!("Overriding system prompt: {}", prompt);
        }
        if let Some(temp) = self.temperature {
            updated_config.temperature = temp;
            println!("Overriding temperature: {}", temp);
        }

        // 5. Main loop: build prompt from memory, call API, and analyze response.
        let mut loop_count = 0;
        let mut all_responses: Vec<String> = Vec::new();

        while loop_count < self.max_loops {
            loop_count += 1;
            println!("--- Loop {} of {} ---", loop_count, self.max_loops);

            if self.dynamic_temperature_enabled {
                println!("Dynamic temperature adjustment enabled (simulated).");
            }

            let task_prompt = self
                .short_memory
                .iter()
                .map(|(role, content)| format!("{}: {}", role, content))
                .collect::<Vec<_>>()
                .join("\n");
            println!("Constructed prompt:\n{}", task_prompt);

            let mut attempt = 0;
            let mut success = false;
            let mut response = String::new();
            while attempt < self.retry_attempts && !success {
                attempt += 1;
                println!("  API call attempt {}/{}", attempt, self.retry_attempts);
                match call_model_api(ModelProvider::OpenAI, &task_prompt, &updated_config).await {
                    Ok(resp) => {
                        response = resp;
                        success = true;
                    }
                    Err(e) => {
                        eprintln!("  Error on attempt {}: {}", attempt, e);
                        sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
                    }
                }
            }

            if !success {
                eprintln!(
                    "Failed to generate a response after {} attempts.",
                    self.retry_attempts
                );
                break;
            }

            println!("{}: {}", self.agent_name, response);
            self.short_memory.push((self.agent_name.clone(), response.clone()));
            all_responses.push(response.clone());

            // 6. Invoke the LLM for further analysis.
            let analysis = if let Some(ref llm) = self.llm {
                println!("Using custom LLM for analysis.");
                llm.run(&response).await?
            } else {
                println!("Using default LLM for analysis.");
                let default_llm = DefaultLLM {
                    config: updated_config.clone(),
                };
                default_llm.run(&response).await?
            };
            println!("LLM Analysis: {}", analysis);
            self.short_memory.push((self.agent_name.clone(), analysis));

            // 7. Check for a stopping condition.
            if response.to_lowercase().contains("stop") {
                println!("Stopping condition met (response contains 'stop').");
                break;
            }

            // 8. Simulate interactive mode if enabled.
            if self.interactive {
                println!("Interactive mode enabled (simulation).");
            }

            if let Some(interval) = self.loop_interval {
                println!("Sleeping for {} seconds...", interval);
                sleep(Duration::from_secs(interval)).await;
            }
        }

        let output = all_responses.join("\n");
        println!("Agent run complete. Final output:\n{}", output);
        Ok(output)
    }
}
