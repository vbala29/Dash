use crate::threadpoolerror::{Result, ThreadPoolError, ThreadPoolErrorReason};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};

type Job = dyn ThreadPoolJob + Send + 'static;

pub trait ThreadPoolJob {
    fn run_job(&self);
}

struct Statistics {
    number_of_jobs_serviced: Option<usize>,
}

#[allow(dead_code)]
struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
    stop_execution: Arc<AtomicBool>,
}

impl Worker {
    fn new(
        id: usize,
        rx: Arc<Mutex<mpsc::Receiver<Box<Job>>>>,
        statistics: Arc<Mutex<HashMap<usize, Statistics>>>,
        _max_exec_time: Duration,
    ) -> Worker {
        const COUNT_RESET_TIME_SECS: u64 = 60;
        let atomic_bool = Arc::new(AtomicBool::new(false));
        let cloned_bool = Arc::clone(&atomic_bool);

        let thread = thread::spawn(move || {
            let mut job_count = 0;
            let mut start = SystemTime::now();

            loop {
                if cloned_bool.load(Ordering::SeqCst) {
                    break;
                }

                let job = rx.lock().unwrap().recv().unwrap();

                if let Ok(t) = start.elapsed() {
                    if t.as_secs() > COUNT_RESET_TIME_SECS {
                        let mut statistics_info = statistics.lock().unwrap();
                        statistics_info
                            .get_mut(&id)
                            .unwrap()
                            .number_of_jobs_serviced = Some(job_count);
                        job_count = 0;
                        start = SystemTime::now();
                    }
                }

                job_count += 1;
                /*
                if let Err(_) = rt.block_on(tokio::time::timeout(
                    max_exec_time,
                    tokio::task::spawn_blocking(move || job.run_job()),
                )){
                    // Todo do something else here
                    println!("Job did not finish executing within time limit");
                }
                */
                job.run_job();
            }
        });

        Worker {
            id,
            thread,
            stop_execution: atomic_bool,
        }
    }
}

pub struct ThreadPool {
    // Periodically reset statistics to 0, used to check for dynamic resizing of resources.
    worker_statistics: Arc<Mutex<HashMap<usize, Statistics>>>,
    workers: Vec<Worker>,
    send_queue: mpsc::Sender<Box<Job>>,
    min_pool_size: usize,
    max_pool_size: usize,
    max_exec_time: Duration,
    rx_queue: Arc<Mutex<mpsc::Receiver<Box<Job>>>>,
    next_id: usize,
}

impl ThreadPool {
    pub fn new(
        pool_size: usize,
        min_pool_size: usize,
        max_pool_size: usize,
        max_exec_time: Duration,
    ) -> Result<ThreadPool> {
        const MAX_POOL_SIZE: usize = 20480; // Configuration for my Mac found via sysctl kern.num_threads

        if pool_size > MAX_POOL_SIZE {
            return Err(ThreadPoolError::new(ThreadPoolErrorReason::InvalidPoolSize));
        } else if max_pool_size > MAX_POOL_SIZE {
            return Err(ThreadPoolError::new(
                ThreadPoolErrorReason::InvalidDynamicPoolBounds,
            ));
        }

        let (tx, rx) = mpsc::channel(); // Tokio MPSC
        let rx_arc = Arc::new(Mutex::new(rx));

        let mut workers = Vec::with_capacity(pool_size);
        let mut worker_statistics = HashMap::new();

        for id in 0..pool_size {
            worker_statistics.insert(
                id,
                Statistics {
                    number_of_jobs_serviced: None,
                },
            );
        }

        let worker_statistics_arc = Arc::new(Mutex::new(worker_statistics));
        for id in 0..pool_size {
            workers.push(Worker::new(
                id,
                Arc::clone(&rx_arc),
                Arc::clone(&worker_statistics_arc),
                max_exec_time,
            ));
        }

        Ok(ThreadPool {
            worker_statistics: worker_statistics_arc,
            workers,
            send_queue: tx,
            min_pool_size,
            max_pool_size,
            max_exec_time,
            rx_queue: rx_arc,
            next_id: pool_size,
        })
    }

    pub fn submit_job(&self, job: Box<Job>) {
        self.send_queue.send(job).unwrap();
    }

    fn get_next_id(&mut self) -> usize {
        let next_id = self.next_id;
        self.next_id += 1;
        next_id
    }

    pub fn dynamic_resizing(
        &mut self,
        jobs_per_duration_lower_bound: usize,
        jobs_per_duration_upper_bound: usize,
    ) -> Result<i32> {
        let mut reallocation: i32 = 0;
        let mut statistics = self.worker_statistics.lock().unwrap();
        for (_, s) in statistics.iter_mut() {
            if let Some(count) = s.number_of_jobs_serviced {
                if count < jobs_per_duration_lower_bound {
                    reallocation -= 1;
                } else if count > jobs_per_duration_upper_bound {
                    reallocation += 1;
                }
            }
        }

        let new_pool_size = reallocation + (self.workers.len() as i32);
        if new_pool_size < (self.min_pool_size as i32) {
            reallocation = (self.min_pool_size as i32) - (self.workers.len() as i32);
        } else if new_pool_size > (self.max_pool_size as i32) {
            reallocation = (self.max_pool_size as i32) - (self.workers.len() as i32);
        }

        match reallocation.cmp(&0) {
            std::cmp::Ordering::Less => {
                let stop_execution_count = -reallocation as usize;
                for i in 0..stop_execution_count {
                    let w = self.workers.get(i);
                    match w {
                        Some(worker) => worker.stop_execution.store(true, Ordering::SeqCst),
                        None => {
                            return Err(ThreadPoolError::new(
                                ThreadPoolErrorReason::DynamicResizingError,
                            ))
                        }
                    }
                }
            }
            std::cmp::Ordering::Greater => {
                drop(statistics);
                for _ in 0..(reallocation as usize) {
                    let new_id = self.get_next_id();
                    self.worker_statistics.lock().unwrap().insert(
                        new_id,
                        Statistics {
                            number_of_jobs_serviced: None,
                        },
                    );

                    self.workers.push(Worker::new(
                        new_id,
                        Arc::clone(&self.rx_queue),
                        Arc::clone(&self.worker_statistics),
                        self.max_exec_time,
                    ));
                }
            }
            _ => (),
        }

        Ok(reallocation)
    }
}
