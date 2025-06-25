// Arc + Mutex for safe sharing of reciever
// mpsc for job queue
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

// Public ThreadPool struct to manage a set of worker threads
pub struct ThreadPool {
    workers: Vec<Worker>,
    // Optional so we can take() and drop on shutdown
    sender: Option<mpsc::Sender<Job>>,
}

// Job is a trait object to allow sending any `FnOnce` task
type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    // Create a new ThreadPool with the given size
    pub fn new(size: usize) -> ThreadPool {
        // Panic if size is zero
        assert!(size > 0);

        // Channel for sending jobs
        let (sender, receiver) = mpsc::channel();

        // Wrap in Arc<Mutex<>> to share safely across threads
        let receiver = Arc::new(Mutex::new(receiver));

        // Preallocate the vector
        let mut workers = Vec::with_capacity(size);

        // Spawn worker threads
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    // Execute a task by sending it to the job queue
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // Box the task to fit the Job type
        let job = Box::new(f);

        // Send job to the workers
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

// Implement Drop to gracefully shut down all workers
impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Drop the sender to signal no more jobs will be sent
        drop(self.sender.take());

        // Join all worker threads
        for worker in &mut self.workers {
            // Print message of which worker is shutting down
            println!("Shutting down worker {}", worker.id);

            // If the worker has an active thread, join it to ensure cleanup
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

// A worker thread in the thread pool
struct Worker {
    id: usize,
    // Option so we can take() during cleanup
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    // Create and start a new worker thread
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        // Spawn a new thread that will continuously receive and execute jobs
        let thread = thread::spawn(move || {
            // Continuously listen for incoming jobs
            loop {
                // Lock the receiver and wait for a job
                let message = receiver.lock().unwrap().recv();

                // Handle the received message
                match message {
                    // Execute the job
                    Ok(job) => {
                        println!("Worker {id} got a job; executing.");

                        job();
                    }
                    // Exit if disconnected
                    Err(_) => {
                        // Exit loop if channel is closed (e.g., ThreadPool is dropped)
                        println!("Worker {id} disconnected; shutting down.");
                        break;
                    }
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
