use crossbeam::deque::{Injector, Steal, Stealer, Worker};
use std::panic;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tokio::runtime::Builder;
use log::{debug, error, info, warn};

/// Type alias for a generic task.
type Task = Box<dyn FnOnce() + Send + 'static>;

/// Configuration for the executor. This can be extended in the future.
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Maximum number of tasks to process in one batch.
    pub batch_size: usize,
    /// Sleep duration when no tasks are found (in milliseconds).
    pub sleep_duration_ms: u64,
    /// Timeout to wait for the task queue to drain before shutdown (in milliseconds).
    pub drain_timeout_ms: u64,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            batch_size: 10,
            sleep_duration_ms: 1,
            drain_timeout_ms: 5000,
        }
    }
}

/// Metrics to track the executor's performance.
#[derive(Debug, Default)]
pub struct ExecutorMetrics {
    /// Total number of tasks successfully executed.
    pub tasks_executed: AtomicUsize,
    /// Total number of tasks that failed (panicked).
    pub tasks_failed: AtomicUsize,
    /// Total number of tasks submitted.
    pub tasks_submitted: AtomicUsize,
}

/// A production-grade executor that uses lock-free work stealing,
/// graceful shutdown, and supports both synchronous and asynchronous tasks.
pub struct HighThroughputExecutor {
    // Use Arc to make the injector shareable across threads
    injector: Arc<Injector<Task>>,
    stealers: Vec<Stealer<Task>>,
    thread_count: usize,
    tokio_runtime: tokio::runtime::Runtime,
    shutdown: Arc<AtomicBool>,
    metrics: Arc<ExecutorMetrics>,
    config: ExecutorConfig,
}

impl HighThroughputExecutor {
    /// Creates a new executor instance with the given configuration.
    pub fn new(config: ExecutorConfig) -> Self {
        // Determine the available thread count using Rust's built-in parallelism API.
        let thread_count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        info!("Initializing HighThroughputExecutor with {} threads", thread_count);

        // Wrap the injector in an Arc
        let injector = Arc::new(Injector::new());
        let mut stealers = Vec::with_capacity(thread_count);

        // Create placeholder workers to generate stealers.
        for _ in 0..thread_count {
            let worker = Worker::new_fifo();
            stealers.push(worker.stealer());
        }

        // Build a multi-threaded Tokio runtime for async tasks.
        // Using new_current_thread instead of new_multi_thread for compatibility
        let tokio_runtime = Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build Tokio runtime");

        Self {
            injector,
            stealers,
            thread_count,
            tokio_runtime,
            shutdown: Arc::new(AtomicBool::new(false)),
            metrics: Arc::new(ExecutorMetrics::default()),
            config,
        }
    }

    /// Spawns a synchronous (CPU-bound) task.
    pub fn spawn<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // Increment submitted tasks counter.
        self.metrics.tasks_submitted.fetch_add(1, Ordering::Relaxed);
        self.injector.push(Box::new(task));
    }

    /// Spawns a collection of synchronous tasks.
    pub fn spawn_all<F, I>(&self, tasks: I)
    where
        F: FnOnce() + Send + 'static,
        I: IntoIterator<Item = F>,
    {
        for task in tasks {
            self.spawn(task);
        }
    }

    /// Spawns an asynchronous (I/O-bound) task.
    pub fn spawn_async<F>(&self, fut: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        // Fixed the panic handling for async tasks
        self.tokio_runtime.spawn(async move {
            // Using a simpler approach for panic handling
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                futures::executor::block_on(fut)
            })) {
                Ok(_) => {},
                Err(e) => error!("Async task panicked: {:?}", e),
            }
        });
    }

    /// Runs the synchronous worker loop.
    pub fn run_sync(&self) {
        // Now we can clone the Arc<Injector>
        let injector = Arc::clone(&self.injector);
        let stealers = self.stealers.clone();
        let shutdown_flag = Arc::clone(&self.shutdown);
        let metrics = Arc::clone(&self.metrics);
        let batch_size = self.config.batch_size;
        let sleep_duration = Duration::from_millis(self.config.sleep_duration_ms);

        let mut handles = Vec::with_capacity(self.thread_count);

        for thread_id in 0..self.thread_count {
            let injector = Arc::clone(&injector);
            let stealers_clone = stealers.clone();
            let shutdown_flag = Arc::clone(&shutdown_flag);
            let metrics = Arc::clone(&metrics);

            let handle = thread::Builder::new()
                .name(format!("worker-{}", thread_id))
                .spawn(move || {
                    info!("Worker {} starting.", thread_id);
                    let local_worker = Worker::new_fifo();
                    while !shutdown_flag.load(Ordering::Relaxed) {
                        let mut found_task = false;
                        // Process up to 'batch_size' tasks in one iteration.
                        for _ in 0..batch_size {
                            // Try to get a task from the local worker, then global, then steal from peers.
                            let task_option = local_worker.pop().or_else(|| {
                                match injector.steal() {
                                    Steal::Success(task) => Some(task),
                                    _ => None,
                                }
                            }).or_else(|| {
                                let mut stolen = None;
                                for stealer in stealers_clone.iter() {
                                    if let Steal::Success(task) = stealer.steal() {
                                        stolen = Some(task);
                                        break;
                                    }
                                }
                                stolen
                            });

                            if let Some(task) = task_option {
                                found_task = true;
                                // Wrap the task execution in a panic catcher.
                                let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                                    task()
                                }));
                                match result {
                                    Ok(_) => {
                                        metrics.tasks_executed.fetch_add(1, Ordering::Relaxed);
                                    }
                                    Err(e) => {
                                        metrics.tasks_failed.fetch_add(1, Ordering::Relaxed);
                                        error!("Worker {} encountered a panic: {:?}", thread_id, e);
                                    }
                                }
                            } else {
                                break;
                            }
                        }
                        // If no tasks were found in the current batch, yield or sleep briefly.
                        if !found_task {
                            thread::sleep(sleep_duration);
                        }
                    }
                    info!("Worker {} exiting.", thread_id);
                })
                .expect("Failed to spawn worker thread");
            handles.push(handle);
        }

        // Wait for all worker threads to finish.
        for handle in handles {
            if let Err(e) = handle.join() {
                error!("Worker thread join failed: {:?}", e);
            }
        }
    }

    /// Blocks until the global task queue appears empty or a timeout is reached.
    pub fn wait_for_drain(&self) {
        let timeout = Duration::from_millis(self.config.drain_timeout_ms);
        let start = Instant::now();
        while !self.injector.is_empty() && start.elapsed() < timeout {
            debug!("Waiting for task queue to drain. Remaining tasks: {}", self.metrics.tasks_submitted.load(Ordering::Relaxed) - self.metrics.tasks_executed.load(Ordering::Relaxed));
            thread::sleep(Duration::from_millis(10));
        }
        if self.injector.is_empty() {
            info!("Task queue drained successfully.");
        } else {
            warn!("Timeout reached before task queue drained completely.");
        }
    }

    /// Signals the executor to shut down gracefully.
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
        info!("Shutdown signal issued.");
    }

    /// Returns a tuple of metrics: (tasks_submitted, tasks_executed, tasks_failed).
    pub fn metrics(&self) -> (usize, usize, usize) {
        (
            self.metrics.tasks_submitted.load(Ordering::Relaxed),
            self.metrics.tasks_executed.load(Ordering::Relaxed),
            self.metrics.tasks_failed.load(Ordering::Relaxed),
        )
    }
}

fn main() {
    // Initialize logging using env_logger. In production you might configure a more robust logger.
    env_logger::init();

    // Create an executor configuration; defaults can be overridden here.
    let config = ExecutorConfig::default();
    let executor = HighThroughputExecutor::new(config);

    // Example: Spawn several individual CPU-bound tasks.
    for i in 0..10_000 {
        executor.spawn(move || {
            // Simulate a small computation.
            let result: usize = (0..100).sum();
            if i % 1000 == 0 {
                info!("Task {} computed result: {}", i, result);
            }
        });
    }

    // Example: Create a list of callables.
    let tasks: Vec<Box<dyn FnOnce() + Send>> = vec![
        Box::new(|| println!("Callable Task 1 executed")),
        Box::new(|| println!("Callable Task 2 executed")),
        Box::new(|| println!("Callable Task 3 executed")),
        Box::new(|| println!("Callable Task 2 executed")),
        Box::new(|| println!("Callable Task 2 executed")),
        Box::new(|| println!("Callable Task 2 executed")),
        Box::new(|| println!("Callable Task 2 executed")),
        Box::new(|| println!("Callable Task 2 executed")),
    ];

    // Upload the list of callables.
    executor.spawn_all(tasks.into_iter().map(|task| move || task()));

    // Example: Spawn an asynchronous (I/O-bound) task.
    executor.spawn_async(async {
        info!("Starting async I/O task.");
        tokio::time::sleep(Duration::from_secs(1)).await;
        info!("Async I/O task complete.");
    });

    // Run synchronous tasks in a separate thread.
    let executor_arc = Arc::new(executor);
    let sync_executor = Arc::clone(&executor_arc);
    let sync_handle = thread::spawn(move || {
        sync_executor.run_sync();
    });

    // Let the executor run for a while.
    thread::sleep(Duration::from_secs(3));

    // Wait for the task queue to drain (optional) before shutdown.
    executor_arc.wait_for_drain();

    // Signal shutdown.
    executor_arc.shutdown();

    // Wait for synchronous worker threads to finish.
    if let Err(e) = sync_handle.join() {
        error!("Failed to join sync executor thread: {:?}", e);
    }

    let (submitted, executed, failed) = executor_arc.metrics();
    info!("Executor shutdown complete. Metrics: Submitted: {}, Executed: {}, Failed: {}", submitted, executed, failed);
}