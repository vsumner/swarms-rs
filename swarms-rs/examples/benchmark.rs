use criterion::{black_box, criterion_group, criterion_main, Criterion};
use crossbeam::deque::{Injector, Steal, Worker};
use std::sync::Arc;
use std::thread;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use log::{info, error};
use env_logger;

/// Number of tasks for the benchmark
const TASK_COUNT: usize = 1_000_000;
const THREAD_COUNT: usize = 16;
const BATCH_SIZE: usize = 10_000;

/// Global counter to track completed tasks
static COMPLETED_TASKS: AtomicUsize = AtomicUsize::new(0);

/// A simple task that increments an atomic counter
fn simple_task() {
    COMPLETED_TASKS.fetch_add(1, Ordering::Relaxed);
}

/// Work-Stealing Executor
fn work_stealing_executor(task_count: usize, thread_count: usize, batch_size: usize) {
    let injector = Arc::new(Injector::new());
    let workers: Vec<_> = (0..thread_count).map(|_| Worker::new_fifo()).collect();
    let stealers: Vec<_> = workers.iter().map(|w| w.stealer()).collect();

    // Populate the task queue
    for _ in 0..task_count {
        injector.push(simple_task);
    }

    let mut handles = vec![];
    let start_time = Instant::now();

    for (id, worker) in workers.into_iter().enumerate() {
        let injector = Arc::clone(&injector);
        let stealers = stealers.clone();

        handles.push(thread::spawn(move || {
            loop {
                let mut batch = Vec::new();
                for _ in 0..batch_size {
                    if let Some(task) = worker.pop() {
                        batch.push(task);
                    } else if let Steal::Success(task) = stealers[id].steal() {
                        batch.push(task);
                    } else if let Steal::Success(task) = injector.steal() {
                        batch.push(task);
                    } else {
                        break;
                    }
                }

                if batch.is_empty() {
                    break;
                }

                for task in batch {
                    task();
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start_time.elapsed();
    let completed = COMPLETED_TASKS.load(Ordering::Relaxed);
    if completed != task_count {
        error!("Mismatch: Expected {}, but completed {} tasks!", task_count, completed);
    } else {
        info!("Successfully executed {} tasks in {:.4} seconds with {} threads.", completed, duration.as_secs_f64(), thread_count);
    }
}

/// Benchmark function
fn benchmark(c: &mut Criterion) {
    env_logger::init();

    println!("Starting benchmark for work_stealing_executor with {} tasks, {} threads, and batch size {}...",
             TASK_COUNT, THREAD_COUNT, BATCH_SIZE);

    c.bench_function("work_stealing_executor", |b| {
        b.iter(|| work_stealing_executor(black_box(TASK_COUNT), THREAD_COUNT, BATCH_SIZE))
    });

    println!("Benchmark completed!");
}

criterion_group!(benches, benchmark);
criterion_main!(benches);

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_task_execution() {
        env_logger::init();
        let executor_threads = 8;
        let total_tasks = 1_000_000;
        let counter = Arc::new(Mutex::new(0));
        
        let tasks: Vec<Box<dyn FnOnce() + Send>> = (0..total_tasks)
            .map(|_| {
                let counter = Arc::clone(&counter);
                Box::new(move || {
                    let mut num = counter.lock().unwrap();
                    *num += 1;
                })
            })
            .collect();
        
        let start = Instant::now();
        work_stealing_executor(total_tasks, executor_threads, BATCH_SIZE);
        let duration = start.elapsed();
        
        thread::sleep(Duration::from_secs(1));
        let final_count = *counter.lock().unwrap();
        assert_eq!(final_count, total_tasks, "Not all tasks completed!");
        info!("Test completed {} tasks in {:.4} seconds with {} threads.", total_tasks, duration.as_secs_f64(), executor_threads);
    }
}
