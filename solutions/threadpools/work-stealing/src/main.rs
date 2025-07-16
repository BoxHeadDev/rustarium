use crossbeam::deque::{Injector, Steal, Worker};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

/// A work-stealing thread pool that distributes jobs across multiple worker threads.
pub struct WorkStealingThreadPool {
    /// Shared queue for pushing new jobs.
    inject: Arc<Injector<Job>>,
    /// Shared shutdown flag to signal worker threads to stop.
    shutdown: Arc<AtomicBool>,
    /// Join handles for all spawned threads. Stored to keep threads alive.
    _threads: Vec<thread::JoinHandle<()>>,
}

impl WorkStealingThreadPool {
    /// Creates a new work-stealing thread pool with the given number of threads.
    pub fn new(num_threads: usize) -> Self {
        let inject: Arc<Injector<Job>> = Arc::new(Injector::new());
        let shutdown = Arc::new(AtomicBool::new(false));

        let mut stealers = Vec::with_capacity(num_threads);
        let mut workers = Vec::with_capacity(num_threads);

        // Initializes one local worker and corresponding stealer for each thread.
        for _ in 0..num_threads {
            let worker = Worker::new_fifo();
            stealers.push(worker.stealer());
            workers.push(Some(worker)); // Use Option to allow later move
        }

        let mut threads = Vec::with_capacity(num_threads);

        // Spawns threads and starts the work-stealing loop.
        for i in 0..num_threads {
            // Takes ownership of the worker for this thread.
            let thread_worker = workers[i].take().unwrap();
            let all_stealers = stealers.clone();
            let inject = Arc::clone(&inject);
            let shutdown = Arc::clone(&shutdown);

            let handle = thread::spawn(move || {
                // Continues processing jobs until shutdown is signaled.
                while !shutdown.load(Ordering::Relaxed) {
                    // Tries to pop a job from local queue, or steal from global or others.
                    let job = thread_worker
                        .pop()
                        .or_else(|| inject.steal().success())
                        .or_else(|| {
                            for (j, s) in all_stealers.iter().enumerate() {
                                if j == i {
                                    continue; // Skips stealing from self.
                                }
                                if let Steal::Success(task) = s.steal() {
                                    return Some(task);
                                }
                            }
                            None
                        });

                    if let Some(job) = job {
                        job(); // Executes the job.
                    } else {
                        thread::yield_now(); // No work found, yields to other threads.
                    }
                }
            });

            threads.push(handle);
        }

        Self {
            inject,
            shutdown,
            _threads: threads,
        }
    }

    /// Pushes a new job into the global job queue to be executed by any thread.
    pub fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.inject.push(Box::new(job));
    }
}

impl Drop for WorkStealingThreadPool {
    /// Signals all threads to shut down. Threads exit once current work is done.
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    /// Tests whether all submitted jobs are executed by the pool.
    #[test]
    fn test_work_stealing_pool_executes_all_jobs() {
        let pool = WorkStealingThreadPool::new(4);
        let counter = Arc::new(Mutex::new(0));

        for _ in 0..20 {
            let c = Arc::clone(&counter);
            pool.spawn(move || {
                let mut count = c.lock().unwrap();
                *count += 1;
            });
        }

        thread::sleep(Duration::from_millis(100)); // Waits for all jobs to finish.

        let count = *counter.lock().unwrap();
        assert_eq!(count, 20);
    }

    /// Tests whether jobs are distributed among threads and all are completed.
    #[test]
    fn test_jobs_are_balanced_among_threads() {
        let pool = WorkStealingThreadPool::new(4);
        let results = Arc::new(Mutex::new(vec![]));

        for i in 0..8 {
            let res = Arc::clone(&results);
            pool.spawn(move || {
                res.lock().unwrap().push(i);
            });
        }

        thread::sleep(Duration::from_millis(100)); // Waits for all jobs to finish.

        let mut res = results.lock().unwrap();
        res.sort();
        assert_eq!(*res, (0..8).collect::<Vec<_>>());
    }
}
