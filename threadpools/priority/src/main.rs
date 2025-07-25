use std::{
    cmp::Ordering,
    collections::BinaryHeap,
    sync::{Arc, Condvar, Mutex},
    thread,
};

// Type alias for a job that can be executed once and sent across threads
type Job = Box<dyn FnOnce() + Send + 'static>;

/// Represents a job with an associated priority.
///
/// Lower `priority` values indicate higher execution priority.
struct PrioritizedJob {
    priority: u32, // Lower is higher priority
    job: Job,
}

// Implement `Eq` for `PrioritizedJob` to allow comparison in `BinaryHeap`
impl Eq for PrioritizedJob {}

impl Ord for PrioritizedJob {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order to create a min-heap where lower numbers are higher priority
        other.priority.cmp(&self.priority)
    }
}

impl PartialOrd for PrioritizedJob {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PrioritizedJob {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

/// Shared queue structure used by the thread pool for job scheduling.
struct SharedQueue {
    queue: Mutex<BinaryHeap<PrioritizedJob>>,
    condvar: Condvar,
}

/// A simple thread pool that executes jobs according to their priority.
///
/// Lower numeric value means higher execution priority.
pub struct PriorityThreadPool {
    shared: Arc<SharedQueue>,
    _handles: Vec<thread::JoinHandle<()>>, // Keep thread handles to ensure threads live
}

impl PriorityThreadPool {
    /// Creates a new `PriorityThreadPool` with the specified number of threads.
    pub fn new(num_threads: usize) -> Self {
        let shared = Arc::new(SharedQueue {
            queue: Mutex::new(BinaryHeap::new()),
            condvar: Condvar::new(),
        });

        let mut handles = Vec::with_capacity(num_threads);

        for _ in 0..num_threads {
            let shared_clone = Arc::clone(&shared);
            let handle = thread::spawn(move || {
                loop {
                    let job = {
                        let mut queue = shared_clone.queue.lock().unwrap();

                        // Wait until there is at least one job in the queue
                        while queue.is_empty() {
                            queue = shared_clone.condvar.wait(queue).unwrap();
                        }

                        queue.pop()
                    };

                    // Execute the job if one was retrieved
                    if let Some(prio_job) = job {
                        (prio_job.job)();
                    }
                }
            });

            handles.push(handle);
        }

        Self {
            shared,
            _handles: handles,
        }
    }

    /// Submits a job to the thread pool with a given priority.
    ///
    /// Lower numeric values denote higher priority.
    pub fn spawn<F>(&self, job: F, priority: u32)
    where
        F: FnOnce() + Send + 'static,
    {
        let mut queue = self.shared.queue.lock().unwrap();
        queue.push(PrioritizedJob {
            priority,
            job: Box::new(job),
        });
        self.shared.condvar.notify_one(); // Notify a waiting thread
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::{Arc, Mutex},
        thread,
        time::Duration,
    };

    #[test]
    fn test_priority_order_execution() {
        let pool = PriorityThreadPool::new(2);
        let output = Arc::new(Mutex::new(vec![]));

        // Submits jobs with varying priorities. Lower priority number = higher importance.
        for (priority, label) in vec![(5, "low"), (1, "high"), (3, "medium")] {
            let out = Arc::clone(&output);
            pool.spawn(
                move || {
                    out.lock().unwrap().push(label);
                },
                priority,
            );
        }

        // Allow time for threads to execute
        thread::sleep(Duration::from_millis(100));

        // Check that jobs were executed in priority order
        let result = output.lock().unwrap().clone();
        assert_eq!(result, vec!["high", "medium", "low"]);
    }

    #[test]
    fn test_parallel_execution_with_priorities() {
        let pool = PriorityThreadPool::new(4);
        let counter = Arc::new(Mutex::new(0));

        // Submit 10 jobs with increasing priority values
        for i in 0..10 {
            let c = Arc::clone(&counter);
            pool.spawn(
                move || {
                    let mut n = c.lock().unwrap();
                    *n += i;
                },
                i,
            );
        }

        // Allow time for threads to execute all jobs
        thread::sleep(Duration::from_millis(200));

        // Validate the sum of all job inputs
        let total = *counter.lock().unwrap();
        assert_eq!(total, (0..10).sum());
    }
}
