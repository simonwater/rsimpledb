use rsimpledb::thread_pool;
fn main() {
    let _pool = thread_pool::ThreadPool::new(4);
}
