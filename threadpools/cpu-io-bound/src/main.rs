use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    thread,
};

use crossbeam::channel;
use futures::task::{ArcWake, waker_ref};

/// Type alias for boxed future with 'static lifetime and Send bound.
type BoxFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// CPU-bound pool (basic fixed-size thread pool)
pub struct CpuThreadPool {
    /// Task sender used to distribute jobs to worker threads.
    sender: channel::Sender<Box<dyn FnOnce() + Send + 'static>>,
    /// Holds worker thread handles to keep them alive.
    _workers: Vec<thread::JoinHandle<()>>,
}

impl CpuThreadPool {
    /// Creates a new CPU thread pool with a fixed number of worker threads.
    pub fn new(num_threads: usize) -> Self {
        let (sender, receiver): (
            channel::Sender<Box<dyn FnOnce() + Send + 'static>>,
            channel::Receiver<Box<dyn FnOnce() + Send + 'static>>,
        ) = channel::unbounded();
        let receiver = Arc::new(receiver);

        let mut workers = Vec::new();
        for _ in 0..num_threads {
            let rx = Arc::clone(&receiver);
            // Each worker runs in a separate thread and executes tasks from the queue.
            workers.push(thread::spawn(move || {
                while let Ok(task) = rx.recv() {
                    task();
                }
            }));
        }

        Self {
            sender,
            _workers: workers,
        }
    }

    /// Spawns a task into the CPU thread pool.
    pub fn spawn<F: FnOnce() + Send + 'static>(&self, task: F) {
        self.sender.send(Box::new(task)).unwrap();
    }
}

/// Represents an asynchronous task to be executed by the IO thread pool.
struct IoTask {
    /// Holds the future to be executed. Wrapped in a mutex for interior mutability.
    future: Mutex<Option<BoxFuture>>,
    /// Sender used to re-schedule the task when it needs to be polled again.
    sender: channel::Sender<Arc<IoTask>>,
}

impl ArcWake for IoTask {
    /// Re-schedules the task by sending it back into the executor's queue.
    fn wake_by_ref(task: &Arc<Self>) {
        task.sender.send(task.clone()).unwrap();
    }
}

/// IO-bound thread pool implementing a simple event-driven executor.
pub struct IoThreadPool {
    /// Channel to send tasks to worker threads for polling.
    sender: channel::Sender<Arc<IoTask>>,
    /// Thread handles to keep the executor threads alive.
    _threads: Vec<thread::JoinHandle<()>>,
}

impl IoThreadPool {
    /// Creates a new IO thread pool with a specified number of threads.
    pub fn new(num_threads: usize) -> Self {
        let (sender, receiver): (channel::Sender<Arc<IoTask>>, channel::Receiver<Arc<IoTask>>) =
            channel::unbounded();
        let receiver = Arc::new(receiver);

        let mut threads = Vec::new();
        for _ in 0..num_threads {
            let rx = Arc::clone(&receiver);
            // Spawn thread to run the IO event loop.
            let handle = thread::spawn(move || {
                while let Ok(task) = rx.recv() {
                    let mut fut = task.future.lock().unwrap();
                    if let Some(mut future) = fut.take() {
                        let waker = waker_ref(&task);
                        let mut cx = Context::from_waker(&waker);

                        // Poll the future and requeue if still pending.
                        if let Poll::Pending = future.as_mut().poll(&mut cx) {
                            *fut = Some(future);
                        }
                    }
                }
            });

            threads.push(handle);
        }

        Self {
            sender,
            _threads: threads,
        }
    }

    /// Spawns an asynchronous task into the IO thread pool.
    pub fn spawn<Fut: Future<Output = ()> + Send + 'static>(&self, fut: Fut) {
        let future = Box::pin(fut);
        let task = Arc::new(IoTask {
            future: Mutex::new(Some(future)),
            sender: self.sender.clone(),
        });
        self.sender.send(task).unwrap();
    }
}

/// Hybrid thread pool managing both CPU-bound and IO-bound tasks.
pub struct HybridThreadPool {
    /// CPU-bound task executor.
    cpu: CpuThreadPool,
    /// IO-bound task executor.
    io: IoThreadPool,
}

impl HybridThreadPool {
    /// Creates a new hybrid thread pool with separate CPU and IO thread pools.
    pub fn new(cpu_threads: usize, io_threads: usize) -> Self {
        Self {
            cpu: CpuThreadPool::new(cpu_threads),
            io: IoThreadPool::new(io_threads),
        }
    }

    /// Spawns a CPU-bound task into the CPU thread pool.
    pub fn spawn_cpu<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.cpu.spawn(task);
    }

    /// Spawns an IO-bound task into the IO thread pool.
    pub fn spawn_io<Fut>(&self, future: Fut)
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.io.spawn(future);
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    /// Simulates an async delay using a thread and channel.
    async fn delay(ms: u64) {
        let (tx, rx) = std::sync::mpsc::channel();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(ms));
            tx.send(()).unwrap();
        });
        rx.recv().unwrap();
    }

    /// Tests that CPU and IO tasks can run concurrently.
    #[test]
    fn test_cpu_and_io_run_concurrently() {
        let pool = HybridThreadPool::new(2, 2);
        let result = Arc::new(Mutex::new(vec![]));

        let r1 = Arc::clone(&result);
        pool.spawn_cpu(move || {
            thread::sleep(Duration::from_millis(50));
            r1.lock().unwrap().push("CPU done");
        });

        let r2 = Arc::clone(&result);
        pool.spawn_io(async move {
            delay(10).await;
            r2.lock().unwrap().push("IO done");
        });

        thread::sleep(Duration::from_millis(100));

        let output = result.lock().unwrap().clone();
        assert!(output.contains(&"CPU done"));
        assert!(output.contains(&"IO done"));
    }

    /// Ensures that a long-running CPU task does not block IO task execution.
    #[test]
    fn test_cpu_tasks_do_not_block_io_tasks() {
        let pool = HybridThreadPool::new(1, 1);
        let result = Arc::new(Mutex::new(vec![]));

        // CPU task sleeps for long time
        pool.spawn_cpu({
            let r = Arc::clone(&result);
            move || {
                thread::sleep(Duration::from_millis(200));
                r.lock().unwrap().push("CPU finished");
            }
        });

        // IO task should still finish early
        pool.spawn_io({
            let r = Arc::clone(&result);
            async move {
                delay(20).await;
                r.lock().unwrap().push("IO finished");
            }
        });

        thread::sleep(Duration::from_millis(100));
        let interim = result.lock().unwrap().clone();
        assert!(interim.contains(&"IO finished"));
        assert!(!interim.contains(&"CPU finished"));

        thread::sleep(Duration::from_millis(150));
        let final_state = result.lock().unwrap().clone();
        assert!(final_state.contains(&"CPU finished"));
    }
}
