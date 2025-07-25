use crossbeam::channel;
use futures::task::{ArcWake, waker_ref};
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    thread,
};

/// A boxed and pinned future that is `Send` and `'static`.
type BoxFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// Represents a single task that wraps a future and can be scheduled for execution.
struct Task {
    /// The future to be executed, protected by a mutex for thread-safe access.
    future: Mutex<Option<BoxFuture>>,
    /// Channel sender to re-schedule the task when it is woken up.
    task_sender: channel::Sender<Arc<Task>>,
}

impl ArcWake for Task {
    /// Wakes the task by sending it back into the executor's channel.
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let _ = arc_self.task_sender.send(arc_self.clone());
    }
}

/// A simple multi-threaded executor for asynchronous tasks.
pub struct IoThreadPool {
    /// Channel sender used to submit new tasks to the executor.
    task_sender: channel::Sender<Arc<Task>>,
    /// Holds the handles to worker threads to keep them alive.
    _threads: Vec<thread::JoinHandle<()>>,
}

impl IoThreadPool {
    /// Creates a new thread pool with the specified number of worker threads.
    ///
    /// Each thread continuously polls and executes tasks from the task queue.
    pub fn new(num_threads: usize) -> Self {
        let (task_sender, task_receiver) = channel::unbounded::<Arc<Task>>();
        let task_receiver = Arc::new(task_receiver);

        let mut threads = Vec::with_capacity(num_threads);

        for _ in 0..num_threads {
            let task_receiver = Arc::clone(&task_receiver);

            // Spawns a thread that polls tasks from the queue and executes them.
            let handle = thread::spawn(move || {
                while let Ok(task) = task_receiver.recv() {
                    let mut future_slot = task.future.lock().unwrap();

                    // Take the future and poll it.
                    if let Some(mut future) = future_slot.take() {
                        let waker = waker_ref(&task);
                        let mut context = Context::from_waker(&waker);

                        match future.as_mut().poll(&mut context) {
                            // If not ready, put the future back to be polled again later.
                            Poll::Pending => {
                                *future_slot = Some(future);
                            }
                            // Future is complete; do nothing.
                            Poll::Ready(()) => {}
                        }
                    }
                }
            });

            threads.push(handle);
        }

        IoThreadPool {
            task_sender,
            _threads: threads,
        }
    }

    /// Spawns a new asynchronous task onto the thread pool.
    ///
    /// The task will be scheduled for execution on an available thread.
    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let future = Box::pin(future);
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        });

        let _ = self.task_sender.send(task);
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    /// Asynchronously waits for the specified number of milliseconds.
    async fn delay(ms: u64) {
        let (tx, rx) = std::sync::mpsc::channel();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(ms));
            let _ = tx.send(());
        });
        let _ = rx.recv();
    }

    /// Tests that tasks submitted to the thread pool are executed and completed.
    #[test]
    fn test_io_pool_runs_async_tasks() {
        let pool = IoThreadPool::new(2);
        let result = Arc::new(Mutex::new(vec![]));

        for i in 0..5 {
            let res = Arc::clone(&result);
            pool.spawn(async move {
                delay(10).await;
                res.lock().unwrap().push(i);
            });
        }

        // Wait to allow all tasks to complete.
        std::thread::sleep(Duration::from_millis(100));

        let mut res = result.lock().unwrap().clone();
        res.sort();
        assert_eq!(res, vec![0, 1, 2, 3, 4]);
    }

    /// Tests that multiple yielding tasks can run concurrently and complete correctly.
    #[test]
    fn test_multiple_yielding_tasks() {
        let pool = IoThreadPool::new(4);
        let output = Arc::new(Mutex::new(0));

        for _ in 0..10 {
            let out = Arc::clone(&output);
            pool.spawn(async move {
                for _ in 0..5 {
                    delay(1).await;
                    let mut v = out.lock().unwrap();
                    *v += 1;
                }
            });
        }

        thread::sleep(Duration::from_millis(200));
        assert_eq!(*output.lock().unwrap(), 50);
    }
}
