use crate::threadpoolerror::ThreadPoolError;
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, SystemTime};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

type Result<T> = std::result::Result<T, ThreadPoolError>;
type Job = dyn ThreadPoolJob + Send + 'static;

pub trait ThreadPoolJob {
    fn run_job(&self);
}

pub struct Statistics {
    number_of_jobs_serviced: usize,
}

pub struct Worker
{

    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker
{
    fn new (id: usize, rx: Arc<Mutex<mpsc::Receiver<Box<Job>>>>, statistics: Arc<Mutex<HashMap<usize, Statistics>>>) -> Worker {
        const COUNT_RESET_TIME_SECS : u64 = 5;
        let thread = thread::spawn(move || {
            let mut job_count = 0;
            let start = SystemTime::now();

            loop {
                let job = rx.lock().unwrap().recv().unwrap();

                if job_count % 10 == 0 {
                    let statistics = statistics.lock().unwrap().get(&id).unwrap();
                    statistics.number_of_jobs_serviced = job_count;
                }

                match start.elapsed() {
                    Ok(t) => {
                        if t.as_secs() > COUNT_RESET_TIME_SECS {
                            job_count = 0;
                        }
                    }
                    Err(e) => ()
                }

                job_count += 1;
                job.run_job();
            }
        });

        Worker { id, thread }
    }
}

pub struct ThreadPool
{
    // Periodically reset statistics to 0, used to check for dynamic resizing of resources.
    worker_statistics: HashMap<usize, Statistics>,
    workers: Vec<Worker>,
    send_queue: mpsc::Sender<Box<Job>>,
    min_pool_size: usize,
    max_pool_size: usize,
    max_exec_time: Duration,
}

impl ThreadPool
{
    pub fn new(
        pool_size: usize,
        min_pool_size: usize,
        max_pool_size: usize,
        max_exec_time: Duration,
    ) -> Result<ThreadPool> {
        let (tx, rx) = mpsc::channel(); // Tokio MPSC
        let rx_arc = Arc::new(Mutex::new(rx));

        let mut workers = Vec::with_capacity(pool_size);
        let mut worker_statistics = HashMap::new();
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


    pub fn submit_job(&self, job : Box<Job>) {
       let boxed_job = Box::new(job);

       self.send_queue.send(job).unwrap();
    }
}