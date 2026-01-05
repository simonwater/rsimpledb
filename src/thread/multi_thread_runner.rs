use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

pub struct MultiThreadRunner {
    headers: Vec<String>,
    thread_count: usize,
}

impl MultiThreadRunner {
    pub fn new(thread_count: usize, mut headers: Vec<String>) -> Self {
        headers.insert(0, format!("{:<10}", "Duration (ms)"));
        headers.insert(0, format!("{:<12}", "Thread ID"));
        MultiThreadRunner {
            headers,
            thread_count,
        }
    }

    pub fn excute<F>(&self, task: F)
    where
        F: Fn(usize) -> Vec<String> + Send + Sync + 'static,
    {
        let barrier = Arc::new(Barrier::new(self.thread_count));
        let mut handles = vec![];
        let task_arc = Arc::new(task);

        for i in 0..self.thread_count {
            let barrier = barrier.clone();
            let f = task_arc.clone();
            let handle = thread::spawn(move || {
                barrier.wait();
                let start_time = Instant::now();

                let result = f(i);

                let duration = start_time.elapsed();
                let execution_info = (i, duration, result);
                execution_info
            });
            handles.push(handle);
        }

        let mut results: Vec<(usize, Duration, Vec<String>)> =
            handles.into_iter().map(|h| h.join().unwrap()).collect();
        results.sort_by(|a, b| a.1.cmp(&b.1));

        self.print_results(results);
    }

    fn print_results(&self, results: Vec<(usize, Duration, Vec<String>)>) {
        println!("{}", self.headers.join(" | "));
        println!("{:-<40}", "");
        for (id, duration, result) in results {
            println!("{:<12} | {:<10.2?} | {}", id, duration, result.join(" | "));
        }
        println!("{:-<40}", "");
        println!("所有线程执行完毕。");
    }
}
