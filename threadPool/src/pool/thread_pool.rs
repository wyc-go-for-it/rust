use std::fmt::Display;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

#[derive(Debug)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("waiting worker {} shutting down.", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap()
            }
            println!("worker {} has shut down.", worker.id);
        }
        println!("ThreadPool shut down...");
    }
}

impl ThreadPool {
    pub fn new(n: usize) -> ThreadPool {
        let mut workers: Vec<Worker> = Vec::with_capacity(n);
        let (s, r) = mpsc::channel::<Job>();

        let receiver = Arc::new(Mutex::new(r));

        for id in 0..n {
            let worker = Worker::new(id, receiver.clone());

            workers.push(worker);
        }

        ThreadPool {
            workers: workers,
            sender: Some(s),
        }
    }

    pub fn execute(&self, fun: impl FnOnce() + Send + 'static) {
        match self.sender.as_ref() {
            Some(sender) => {
                let _ = sender.send(Box::new(fun));
            }
            None => (),
        }
    }
}

#[derive(Debug)]
pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Display for Worker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "id-{}", self.id)
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let thread_job = thread::spawn(move || {
            loop {
                let job = receiver.lock().unwrap().recv();
                match job {
                    Ok(job) => {
                        println!(
                            "Worker {id} at thread {:?} got a job, executing.",
                            thread::current().name()
                        );

                        job();

                        println!("Worker {id} has executed.");
                    }
                    Err(_) => {
                        println!("Worker {id} has disconnected");
                        break;
                    }
                }
            }
            println!("Worker {id} has exited");
        });
        Worker {
            id: id,
            thread: Some(thread_job),
        }
    }
}
