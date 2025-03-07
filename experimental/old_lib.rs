// use pyo3::prelude::*;
// use pyo3::wrap_pyfunction;
// use std::sync::mpsc;
// use std::fs;
// use std::io::Write;
// use threadpool::ThreadPool;
// use reqwest::Client;
// use futures::future::join_all;

// /// Executes a Python callable concurrently using a thread pool.
// ///
// /// This function spawns a number of tasks that each call the provided Python callable. The number
// /// of worker threads is determined by `worker_count` (or defaults to the number of CPU cores if `None`).
// ///
// /// # Arguments
// /// * `py_callable` - A Python callable (e.g., a function) to execute.
// /// * `task_count` - The number of times to execute the callable concurrently.
// /// * `worker_count` - Optional number of worker threads to use; if `None`, uses the number of CPU cores.
// ///
// /// # Returns
// /// A list of results from each execution of the Python callable.
// ///
// /// # Errors
// /// Propagates any Python errors encountered during the callable execution.
// #[pyfunction]
// fn run_callable_concurrently(
//     py_callable: PyObject,
//     task_count: usize,
//     worker_count: Option<usize>,
// ) -> PyResult<Vec<PyObject>> {
//     let workers = worker_count.unwrap_or_else(num_cpus::get);
//     let pool = ThreadPool::new(workers);
//     let (tx, rx) = mpsc::channel();

//     // Spawn tasks into the thread pool.
//     for _ in 0..task_count {
//         let tx = tx.clone();
//         let callable = py_callable.clone();
//         pool.execute(move || {
//             // Each thread must re-acquire the GIL to safely interact with Python.
//             let result = Python::with_gil(|py| {
//                 // Call the provided Python callable without arguments.
//                 callable.call0(py).map(|res| res.to_object(py))
//             });
//             tx.send(result).expect("Failed to send result");
//         });
//     }
//     drop(tx);

//     // Collect the results from all threads.
//     let mut results = Vec::with_capacity(task_count);
//     for result in rx.into_iter().take(task_count) {
//         results.push(result?);
//     }
    
//     Ok(results)
// }

// /// Creates a file with the specified filename and contents.
// ///
// /// # Arguments
// /// * `filename` - The path to the file that will be created.
// /// * `contents` - The string content to write into the file.
// ///
// /// # Returns
// /// Returns `None` on success.
// ///
// /// # Errors
// /// Returns a Python IOError if the file cannot be created or written.
// #[pyfunction]
// fn create_file(filename: &str, contents: &str) -> PyResult<()> {
//     let mut file = fs::File::create(filename)
//         .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to create file: {}", e)))?;
//     file.write_all(contents.as_bytes())
//         .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to write to file: {}", e)))?;
//     Ok(())
// }

// /// Asynchronously sends a single API request and returns the response body as a string.
// ///
// /// This function leverages Tokio and Reqwest to perform the HTTP GET request concurrently.
// ///
// /// # Arguments
// /// * `url` - The URL to send the GET request to.
// ///
// /// # Returns
// /// The response body as a string.
// ///
// /// # Errors
// /// Returns a Python RuntimeError if the request or response processing fails.
// // #[pyfunction]
// // async fn send_api_request(url: String) -> PyResult<String> {
// //     let client = Client::new();
// //     let resp = client.get(&url)
// //         .send()
// //         .await
// //         .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Request error: {}", e)))?;
// //     let text = resp.text().await
// //         .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to read response: {}", e)))?;
// //     Ok(text)
// // }

// /// Asynchronously sends multiple API requests concurrently and returns a list of response bodies.
// ///
// /// This function uses Tokio and Reqwest to dispatch all requests concurrently, then awaits their completion.
// ///
// /// # Arguments
// /// * `urls` - A list of URLs to which GET requests will be sent concurrently.
// ///
// /// # Returns
// /// A list of response bodies as strings.
// ///
// /// # Errors
// /// Returns a Python RuntimeError if any of the requests fail.
// // #[pyfunction]
// // async fn send_multiple_api_requests(urls: Vec<String>) -> PyResult<Vec<String>> {
// //     let client = Client::new();
// //     let futures = urls.into_iter().map(|url| {
// //         let client = client.clone();
// //         async move {
// //             let resp = client.get(&url).send().await?;
// //             let text = resp.text().await?;
// //             Ok(text)
// //         }
// //     });
// //     let results: Vec<Result<String, reqwest::Error>> = join_all(futures).await;
// //     let mut responses = Vec::new();
// //     for res in results {
// //         match res {
// //             Ok(text) => responses.push(text),
// //             Err(e) => return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Error in request: {}", e))),
// //         }
// //     }
// //     Ok(responses)
// // }

// /// The `my_rust_module` module provides several utilities:
// /// - Concurrent execution of Python callables.
// /// - File creation functionality.
// /// - Asynchronous API request wrappers using Tokio.
// ///
// /// This module demonstrates integrating multithreading, file I/O, and async networking in Rust,
// /// and exposes these capabilities to Python.
// #[pymodule]
// fn swarms_rust(_py: Python, m: &PyModule) -> PyResult<()> {
//     m.add_function(wrap_pyfunction!(run_callable_concurrently, m)?)?;
//     m.add_function(wrap_pyfunction!(create_file, m)?)?;
//     // m.add_function(wrap_pyfunction!(send_api_request, m)?)?;
//     // m.add_function(wrap_pyfunction!(send_multiple_api_requests, m)?)?;
//     Ok(())
// }


//! # Application Configuration and Enhanced Model API Integration Module
//!
//! This module provides a production-ready implementation for loading application configuration
//! from a `.env` file and for dispatching API calls to multiple model providers (e.g., OpenAI, Anthropic,
//! Google, HuggingFace, and custom providers). It includes:
//!
//! - Loading environment variables using the [`dotenv`](https://crates.io/crates/dotenv) crate.
//! - A `Config` struct with type-safe fields and fallback defaults.
//! - Robust, context-aware error handling using [`anyhow`](https://crates.io/crates/anyhow).
//! - A unified API caller function that supports multiple providers with a simple retry mechanism.
//! - Detailed Rustdoc documentation and unit tests.
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! # #[tokio::main]
//! # async fn main() -> Result<(), anyhow::Error> {
//! use your_module::{Config, ModelProvider, call_model_api};
//!
//! // Load configuration from .env file and environment variables
//! let config = Config::from_env()?;
//!
//! // Example API call to OpenAI's Chat Completion API with a sample prompt
//! let response = call_model_api(ModelProvider::OpenAI, "Write a haiku about recursion in programming.", &config).await?;
//! println!("Response: {}", response);
//! # Ok(())
//! # }
//! ```
