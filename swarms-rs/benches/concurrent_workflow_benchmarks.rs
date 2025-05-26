use criterion::{black_box, criterion_group, criterion_main, Criterion};
use futures::future::{self, BoxFuture};
use tokio::runtime::Runtime;

use swarms_rs::structs::{
    agent::{Agent, AgentError},
    concurrent_workflow::ConcurrentWorkflow,
};

fn run_async<F: std::future::Future<Output = T>, T>(future: F) -> T {
    let rt = Runtime::new().unwrap();
    rt.block_on(future)
}

fn create_mock_agent(id: &str, name: &str, description: &str, response: &str) -> Box<dyn Agent> {
    let agent = MockAgent::new(id, name, description, response);
    Box::new(agent)
}

struct MockAgent {
    id: String,
    name: String,
    description: String,
    response: String,
}

impl MockAgent {
    fn new(id: &str, name: &str, description: &str, response: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            response: response.to_string(),
        }
    }
}

impl Agent for MockAgent {
    fn run(&self, _task: String) -> BoxFuture<'static, Result<String, AgentError>> {
        let response = self.response.clone();
        Box::pin(future::ready(Ok(response)))
    }

    fn run_multiple_tasks(
        &mut self,
        tasks: Vec<String>,
    ) -> BoxFuture<'static, Result<Vec<String>, AgentError>> {
        let response = self.response.clone();
        let responses = vec![response; tasks.len()];
        Box::pin(future::ready(Ok(responses)))
    }

    fn plan(&self, _task: String) -> BoxFuture<'static, Result<(), AgentError>> {
        Box::pin(future::ready(Ok(())))
    }

    fn query_long_term_memory(&self, _task: String) -> BoxFuture<'static, Result<(), AgentError>> {
        Box::pin(future::ready(Ok(())))
    }

    fn save_task_state(&self, _task: String) -> BoxFuture<'static, Result<(), AgentError>> {
        Box::pin(future::ready(Ok(())))
    }

    fn is_response_complete(&self, _response: String) -> bool {
        true
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn description(&self) -> String {
        format!("Mock agent: {}", self.name)
    }

    fn clone_box(&self) -> Box<dyn Agent> {
        Box::new(MockAgent {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            response: self.response.clone(),
        })
    }
}

// Different number of agents
fn bench_concurrent_workflow_agents(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_workflow_agents");

    for num_agents in [3, 5, 10, 20] {
        group.bench_function(format!("{}_agents", num_agents), |b| {
            b.iter(|| {
                run_async(async {
                    let mut builder = ConcurrentWorkflow::builder()
                        .name("benchmark")
                        .description("Concurrent workflow benchmark")
                        .metadata_output_dir("./temp/bench/concurrent");

                    for i in 1..=num_agents {
                        builder = builder.add_agent(create_mock_agent(
                            &i.to_string(),
                            &format!("agent{}", i),
                            &format!("description{}", i),
                            &format!("response{}", i),
                        ));
                    }

                    let workflow = builder.build();
                    let _ = workflow.run(black_box("benchmark task")).await.unwrap();
                })
            });
        });
    }

    group.finish();
}

// Different batch sizes
fn bench_concurrent_workflow_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_workflow_batch");

    for batch_size in [1, 5, 10, 20, 50] {
        group.bench_function(format!("batch_size_{}", batch_size), |b| {
            b.iter(|| {
                run_async(async {
                    let workflow = ConcurrentWorkflow::builder()
                        .name("benchmark")
                        .description("Concurrent workflow batch benchmark")
                        .metadata_output_dir("./temp/bench/concurrent")
                        .add_agent(create_mock_agent("1", "agent1", "desc1", "resp1"))
                        .add_agent(create_mock_agent("2", "agent2", "desc2", "resp2"))
                        .add_agent(create_mock_agent("3", "agent3", "desc3", "resp3"))
                        .build();

                    let tasks = (1..=batch_size)
                        .map(|i| format!("task {}", i))
                        .collect::<Vec<_>>();

                    let _ = workflow.run_batch(black_box(tasks)).await.unwrap();
                })
            });
        });
    }

    group.finish();
}

fn bench_concurrent_workflow_with_delay(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_workflow_delay");

    struct DelayedMockAgent {
        id: String,
        name: String,
        description: String,
        response: String,
        delay_ms: u64,
    }

    impl DelayedMockAgent {
        fn new(id: &str, name: &str, description: &str, response: &str, delay_ms: u64) -> Self {
            Self {
                id: id.to_string(),
                name: name.to_string(),
                description: description.to_string(),
                response: response.to_string(),
                delay_ms,
            }
        }
    }

    impl Agent for DelayedMockAgent {
        fn run(&self, _task: String) -> BoxFuture<'static, Result<String, AgentError>> {
            let response = self.response.clone();
            let delay = self.delay_ms;
            Box::pin(async move {
                if delay > 0 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                }
                Ok(response)
            })
        }

        fn run_multiple_tasks(
            &mut self,
            tasks: Vec<String>,
        ) -> BoxFuture<'static, Result<Vec<String>, AgentError>> {
            let response = self.response.clone();
            let responses = vec![response; tasks.len()];
            Box::pin(future::ready(Ok(responses)))
        }

        fn plan(&self, _task: String) -> BoxFuture<'static, Result<(), AgentError>> {
            Box::pin(future::ready(Ok(())))
        }

        fn query_long_term_memory(
            &self,
            _task: String,
        ) -> BoxFuture<'static, Result<(), AgentError>> {
            Box::pin(future::ready(Ok(())))
        }

        fn save_task_state(&self, _task: String) -> BoxFuture<'static, Result<(), AgentError>> {
            Box::pin(future::ready(Ok(())))
        }

        fn is_response_complete(&self, _response: String) -> bool {
            true
        }

        fn id(&self) -> String {
            self.id.clone()
        }

        fn name(&self) -> String {
            self.name.clone()
        }

        fn description(&self) -> String {
            format!("Delayed mock agent: {}", self.name)
        }

        fn clone_box(&self) -> Box<dyn Agent> {
            Box::new(DelayedMockAgent {
                id: self.id.clone(),
                name: self.name.clone(),
                description: self.description.clone(),
                response: self.response.clone(),
                delay_ms: self.delay_ms,
            })
        }
    }

    // Test agents with different delays
    for delay_pattern in ["uniform", "increasing", "one_slow"] {
        group.bench_function(format!("delay_{}", delay_pattern), |b| {
            b.iter(|| {
                run_async(async {
                    let mut builder = ConcurrentWorkflow::builder()
                        .name("benchmark")
                        .description("Concurrent workflow delay benchmark")
                        .metadata_output_dir("./temp/bench/concurrent");

                    for i in 1..=5 {
                        let delay_ms = match delay_pattern {
                            "uniform" => 50,
                            "increasing" => i * 20,
                            "one_slow" => {
                                if i == 3 {
                                    200
                                } else {
                                    10
                                }
                            },
                            _ => 0,
                        };

                        builder = builder.add_agent(Box::new(DelayedMockAgent::new(
                            &i.to_string(),
                            &format!("agent{}", i),
                            &format!("description{}", i),
                            &format!("response{}", i),
                            delay_ms,
                        )));
                    }

                    let workflow = builder.build();
                    let _ = workflow.run(black_box("benchmark task")).await.unwrap();
                })
            });
        });
    }

    group.finish();
}

// Testing concurrent workflows with different output sizes
fn bench_concurrent_workflow_output_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_workflow_output_size");

    for size_kb in [1, 10, 100] {
        group.bench_function(format!("output_{}_kb", size_kb), |b| {
            b.iter(|| {
                run_async(async {
                    // Create proxies with different size outputs
                    let kb_response = "X".repeat(size_kb * 1024);

                    let workflow = ConcurrentWorkflow::builder()
                        .name("benchmark")
                        .description("Output size benchmark")
                        .metadata_output_dir("./temp/bench/concurrent")
                        .add_agent(create_mock_agent("1", "agent1", "desc1", &kb_response))
                        .add_agent(create_mock_agent("2", "agent2", "desc2", &kb_response))
                        .add_agent(create_mock_agent("3", "agent3", "desc3", &kb_response))
                        .build();

                    let _ = workflow.run(black_box("benchmark task")).await.unwrap();
                })
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_concurrent_workflow_agents,
    bench_concurrent_workflow_batch,
    bench_concurrent_workflow_with_delay,
    bench_concurrent_workflow_output_size,
);
criterion_main!(benches);
