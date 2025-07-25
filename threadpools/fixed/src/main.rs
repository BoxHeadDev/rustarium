use anyhow::Result;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

// Type alias for a job: a boxed closure that takes no input and returns nothing.
// The closure must be thread-safe (`Send`) and have a static lifetime.
type Job = Box<dyn FnOnce() + Send + 'static>;

/// A simple thread pool that allows job scheduling across multiple worker threads.
pub struct ThreadPool {
    sender: Sender<Job>,
    _workers: Vec<thread::JoinHandle<()>>, // Keeps thread handles alive to prevent premature termination.
}

impl ThreadPool {
    /// Creates a new thread pool with the specified number of threads.
    ///
    /// # Arguments
    ///
    /// * `threads` - The number of worker threads to spawn.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread count is zero.
    pub fn new(threads: u32) -> Result<Self> {
        assert!(threads > 0);

        // Create a channel for job distribution.
        let (sender, receiver): (Sender<Job>, Receiver<Job>) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(threads as usize);

        // Spawn worker threads.
        for _ in 0..threads {
            let rx = Arc::clone(&receiver);

            let handle = thread::spawn(move || {
                loop {
                    // Acquire lock and receive job from the channel.
                    let job = {
                        let lock = rx.lock().unwrap();
                        lock.recv()
                    };

                    match job {
                        Ok(job) => {
                            // Execute the job.
                            job();
                        }
                        Err(_) => {
                            // If the channel is closed, terminate the thread.
                            break;
                        }
                    }
                }
            });

            workers.push(handle);
        }

        Ok(ThreadPool {
            sender,
            _workers: workers,
        })
    }

    /// Submits a job to the thread pool.
    ///
    /// # Arguments
    ///
    /// * `job` - A closure to be executed by a worker thread.
    pub fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(job);

        // Send the job to an available worker.
        self.sender.send(job).expect("Failed to send job to worker");
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[test]
    fn test_thread_pool_executes_jobs() {
        // Create a thread pool with 4 threads.
        let pool = ThreadPool::new(4).unwrap();
        let counter = Arc::new(Mutex::new(0));

        // Spawn 10 jobs that each increment a shared counter.
        for _ in 0..10 {
            let counter_clone = Arc::clone(&counter);
            pool.spawn(move || {
                let mut num = counter_clone.lock().unwrap();
                *num += 1;
            });
        }

        // Wait briefly to allow all jobs to complete.
        // Note: In production, use proper synchronization instead of sleeping.
        thread::sleep(Duration::from_millis(100));

        // Verify that all jobs were executed.
        let final_count = *counter.lock().unwrap();
        assert_eq!(final_count, 10);
    }
}
