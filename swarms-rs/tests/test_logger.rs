use dotenv::dotenv;
use std::env;
use swarms_rs::logging::{
    init_logger, log_agent_init, log_agent_state_change, log_agent_task_completion,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_logger_initialization() {
        env::set_var("SWARMS_LOG_LEVEL", "DEBUG");
        init_logger();

        log::info!("Test log message");
        log::debug!("Test debug message");
        log::error!("Test error message");
    }

    #[test]
    pub unsafe fn test_agent_logging_macros() {
        env::set_var("SWARMS_LOG_LEVEL", "TRACE");
        init_logger();

        log_agent!(info, "TestAgent", "agent-123", "Agent started successfully");
        log_task!(
            info,
            "TestAgent",
            "agent-123",
            "Process data",
            "Task initialized"
        );
        log_tool!(
            debug,
            "TestAgent",
            "agent-123",
            "WebScraper",
            "Tool execution started"
        );
        log_memory!(
            trace,
            "TestAgent",
            "agent-123",
            "Save",
            "Saving to long-term memory"
        );
        log_llm!(
            info,
            "TestAgent",
            "agent-123",
            "GPT-4",
            "Sending completion request"
        );
    }
}
