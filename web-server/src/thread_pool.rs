use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use log::info;

/// A Worker struct
///
/// The Worker struct is responsible for holding the id and thread of a worker
///
/// # Fields
///
/// * `id` - The id of the worker
/// * `thread` - The thread of the worker
///
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    info!("Worker {} got a job; executing.", id);
                    job();
                }
                Err(_) => {
                    info!("Worker {} is shutting down.", id);
                    break;
                }
            };
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

/// A ThreadPool struct
///
/// The ThreadPool struct is responsible for holding the workers and the sender of the thread pool
///
/// # Fields
///
/// * `workers` - A vector of the workers
/// * `sender` - The sender of the thread pool
///
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Create a new ThreadPool
    ///
    /// The size is the number of threads in the pool
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            info!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
