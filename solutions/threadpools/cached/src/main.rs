use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

// Type alias for a job: a boxed closure that can be executed once.
// The closure must be thread-safe (`Send`) and have a `'static` lifetime.
type Job = Box<dyn FnOnce() + Send + 'static>;

/// A thread pool that spawns threads elastically and terminates them after an idle timeout.
pub struct ElasticThreadPool {
    sender: mpsc::Sender<Job>,
    inner: Arc<Inner>,
}

/// Shared state across all threads in the pool.
struct Inner {
    receiver: Mutex<mpsc::Receiver<Job>>, // Receiver for incoming jobs.
    active_threads: Mutex<usize>,         // Tracks the number of active threads.
    max_idle_time: Duration,              // Maximum idle duration before a thread exits.
}

impl ElasticThreadPool {
    /// Creates a new elastic thread pool.
    ///
    /// # Arguments
    ///
    /// * `max_idle_time` - The duration a worker thread can stay idle before terminating.
    pub fn new(max_idle_time: Duration) -> Self {
        let (sender, receiver) = mpsc::channel();
        let inner = Arc::new(Inner {
            receiver: Mutex::new(receiver),
            active_threads: Mutex::new(0),
            max_idle_time,
        });

        Self { sender, inner }
    }

    /// Submits a job to the thread pool.
    ///
    /// If no thread is available to pick up the job, a new thread is spawned.
    ///
    /// # Arguments
    ///
    /// * `job` - The closure to execute.
    pub fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // Send the job to the job queue.
        self.sender.send(Box::new(job)).expect("Failed to send job");

        // Always increment and spawn a new thread for this simplified elastic model.
        let should_spawn = {
            let mut active = self.inner.active_threads.lock().unwrap();
            *active += 1;
            true
        };

        if should_spawn {
            let inner_clone = Arc::clone(&self.inner);
            thread::spawn(move || worker_thread(inner_clone));
        }
    }
}

/// The function executed by each worker thread.
///
/// Waits for a job or exits after being idle for longer than `max_idle_time`.
fn worker_thread(inner: Arc<Inner>) {
    let receiver = &inner.receiver;
    let max_idle = inner.max_idle_time;
    let idle_timeout = Duration::from_millis(10); // Polling interval.

    let start = Instant::now();

    loop {
        // Wait for a job or timeout.
        let job_opt = {
            let lock = receiver.lock().unwrap();
            lock.recv_timeout(idle_timeout)
        };

        match job_opt {
            Ok(job) => {
                job(); // Execute the job.
                continue; // Reset idle timer on work.
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Exit if idle time exceeds the threshold.
                if start.elapsed() > max_idle {
                    break;
                }
                continue;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    // Decrement the active thread count on exit.
    let mut active = inner.active_threads.lock().unwrap();
    *active -= 1;
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_elastic_thread_pool_runs_jobs() {
        // Create a thread pool with a short idle timeout.
        let pool = ElasticThreadPool::new(Duration::from_millis(50));
        let counter = Arc::new(Mutex::new(0));

        // Submit 10 jobs that increment a shared counter.
        for _ in 0..10 {
            let c = Arc::clone(&counter);
            pool.spawn(move || {
                let mut val = c.lock().unwrap();
                *val += 1;
            });
        }

        // Give time for all jobs to complete.
        thread::sleep(Duration::from_millis(100));
        let result = *counter.lock().unwrap();

        // Verify all jobs have executed.
        assert_eq!(result, 10);
    }

    #[test]
    fn test_threads_terminate_after_idle() {
        // Create a thread pool with a short idle timeout.
        let pool = ElasticThreadPool::new(Duration::from_millis(50));
        let counter = Arc::new(Mutex::new(0));

        // Submit one job to trigger thread creation.
        pool.spawn({
            let c = Arc::clone(&counter);
            move || {
                let mut val = c.lock().unwrap();
                *val += 1;
            }
        });

        // Wait longer than the idle timeout to allow threads to terminate.
        thread::sleep(Duration::from_millis(100));

        // Ensure all worker threads have exited.
        let active = pool.inner.active_threads.lock().unwrap();
        assert_eq!(*active, 0);
    }
}
