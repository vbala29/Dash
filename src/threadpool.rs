use crate::threadpoolerror::ThreadPoolError;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

type Result<T> = std::result::Result<T, ThreadPoolError>;

pub trait ThreadPoolJob {
    fn run_job(&self);
}

pub struct Statistics {
    number_of_jobs_serviced: usize,
}

pub struct Worker<Job>
where
    Job: ThreadPoolJob + Send + 'static,
{

    id: usize,
    thread: thread::JoinHandle<()>,
}

impl<Job> Worker<Job>
where
    Job: ThreadPoolJob + Send + 'static,
{
    fn new (id: usize, rx: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker<Job> {
        let thread = thread::spawn(move || loop {
            let job = rx.lock().unwrap().recv().unwrap();

            job.run_job();
        });

        Worker { id, thread }
    }
}

pub struct ThreadPool<Job>
where
    Job: ThreadPoolJob + Send + 'static,
{
    // Periodically reset statistics to 0, used to check for dynamic resizing of resources.
    worker_statistics: HashMap<usize, Statistics>,
    workers: Vec<Worker<Job>>,
    send_queue: mpsc::Sender<Job>,
    min_pool_size: usize,
    max_pool_size: usize,
    max_exec_time: Duration,
}

impl<Job> ThreadPool<Job>
where
    Job: ThreadPoolJob + Send + 'static,
{
    pub fn new(
        pool_size: usize,
        min_pool_size: usize,
        max_pool_size: usize,
        max_exec_time: Duration,
    ) -> Result<ThreadPool<Job>> {
        let (tx, rx) = mpsc::channel(); // Tokio MPSC
        let rx_arc = Arc::new(Mutex::new(rx));

        let mut workers = Vec::with_capacity(pool_size);
        let worker_statistics = HashMap::new();
        let worker_statistics_arc = Arc::new(Mutex::new(worker_statistics));
        for id in 0..pool_size {
            workers.push(Worker::new(id, Arc::clone(&rx_arc), Arc::clone(&worker_statistics_arc)));
            worker_statistics.insert(
                id,
                Statistics {
                    number_of_jobs_serviced: 0,
                },
            );
        }

        Ok(ThreadPool {
            worker_statistics,
            workers,
            send_queue: tx,
            min_pool_size,
            max_pool_size,
            max_exec_time,
        })
    }


    pub fn submit_job(&self, job : Job) {
       let boxed_job : Box<Job> = Box::new(job);

       self.send_queue.send(job).unwrap();
    }
}
