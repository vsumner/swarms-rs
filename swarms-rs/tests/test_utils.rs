#[cfg(test)]
use futures::future::{self, BoxFuture};
#[cfg(test)]
use mockall::mock;

#[cfg(test)]
use swarms_rs::structs::agent::{Agent, AgentError};

#[cfg(test)]
mock! {
    /// Mock Agent
    pub Agent{}

    impl Agent for Agent {
        fn run(&self, task: String) -> BoxFuture<'static, Result<String, AgentError>> {
            Box::pin(future::ready(Ok(String::new())))
        }
        fn run_multiple_tasks(&mut self, tasks: Vec<String>) -> BoxFuture<'static, Result<Vec<String>, AgentError>> {
            Box::pin(future::ready(Ok(vec![])))
        }
        fn plan(&self, task: String) -> BoxFuture<'static, Result<(), AgentError>> {
            Box::pin(future::ready(Ok(())))
        }
        fn query_long_term_memory(&self, task: String) -> BoxFuture<'static, Result<(), AgentError>> {
            Box::pin(future::ready(Ok(())))
        }
        fn save_task_state(&self, task: String) -> BoxFuture<'static, Result<(), AgentError>> {
            Box::pin(future::ready(Ok(())))
        }
        fn is_response_complete(&self, response: String) -> bool {
            true
        }
        fn id(&self) -> String {
            String::new()
        }
        fn name(&self) -> String {
            String::new()
        }
        fn description(&self) -> String {
            String::new()
        }
        fn clone_box(&self) -> Box<dyn Agent> {
            panic!("clone_box not implemented for MockAgent")
        }
    }
}

#[cfg(test)]
pub fn create_mock_agent(id: &str, name: &str, desc: &str, response: &str) -> Box<MockAgent> {
    let mut agent = Box::new(MockAgent::new());

    let id_str = id.to_string();
    agent.expect_id().return_const(id_str);

    let name_str = name.to_string();
    agent.expect_name().return_const(name_str);

    let desc_str = desc.to_string();
    agent.expect_description().return_const(desc_str);

    let response_str = response.to_string();
    let response_str_clone = response_str.clone();
    agent.expect_run().returning(move |_| {
        let res = response_str_clone.clone();
        Box::pin(future::ready(Ok(res)))
    });

    agent.expect_is_response_complete().returning(|_| true);

    let response_str_clone = response_str.clone();
    agent.expect_run_multiple_tasks().returning(move |tasks| {
        let responses = tasks.iter().map(|_| response_str_clone.clone()).collect();
        Box::pin(future::ready(Ok(responses)))
    });

    agent
        .expect_plan()
        .returning(|_| Box::pin(future::ready(Ok(()))));

    agent
        .expect_query_long_term_memory()
        .returning(|_| Box::pin(future::ready(Ok(()))));

    agent
        .expect_save_task_state()
        .returning(|_| Box::pin(future::ready(Ok(()))));

    agent
}

#[cfg(test)]
pub fn create_failing_agent(id: &str, name: &str, error_msg: &str) -> Box<MockAgent> {
    let mut agent = Box::new(MockAgent::new());

    let id_str = id.to_string();
    agent.expect_id().return_const(id_str);

    let name_str = name.to_string();
    agent.expect_name().return_const(name_str);

    agent
        .expect_description()
        .return_const("Failing agent".to_string());

    let error_str = error_msg.to_string();
    let error_str_for_run = error_str.clone();
    agent.expect_run().returning(move |_| {
        let err = AgentError::ToolNotFound(error_str_for_run.clone());
        Box::pin(future::ready(Err(err)))
    });

    agent.expect_is_response_complete().returning(|_| false);

    agent.expect_run_multiple_tasks().returning(move |_| {
        let err = AgentError::ToolNotFound(error_str.clone());
        Box::pin(future::ready(Err(err)))
    });

    agent
}
