use std::thread;
use std::sync::{mpsc, Mutex, Arc};

//Parse User Input
pub struct UserOptions {
    pub thread_count: usize,
}

impl UserOptions {
    pub fn new(mut args: std::env::Args) -> Result<UserOptions, &'static str> {
        args.next();
        let thread_count = match args.next() {
            Some(arg) => arg,
            None => return Err("No thread count provided. \n This is the number of threads the application should create"),
        };
        
        let thread_count = thread_count.trim().parse().expect("Thread count must be a number");
        
        Ok(UserOptions{thread_count})
    }
}

// ThreadPool System
enum Message {
    NewJob(Job),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool.
    /// 
    /// The size is the number of threads in the pool
    /// 
    /// # Panics
    /// 
    /// The `new` function will panic if the size is zero
    pub fn new (size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender,
        }
    }

    /// Run a function on the next available thread in the pool.
    /// 
    /// f is the function you want to run.
    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        } 

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shuuting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) ->
        Worker {

        let thread = thread::spawn(move ||{
            loop {
                let message = receiver.lock().unwrap().recv().unwrap();

                match message {
                    Message::NewJob(job) => {
                        println!("Worker {} got a job; executing.", id);

                        job();
                    },
                    Message::Terminate => {
                        println!("Worker {} was told to terminate.", id);

                        break;
                    },
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}