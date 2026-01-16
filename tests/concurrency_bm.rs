use rsimpledb::buffer::BufferMgr;
use rsimpledb::file::BlockId;
use rsimpledb::file::FileMgr;
use rsimpledb::log::LogMgr;
use rsimpledb::thread::MultiThreadRunner;
use rsimpledb::util::TempFileGuard;
use std::path::PathBuf;

#[test]
fn test_buffer_manager_concurrency() {
    // 1. 初始化文件管理器和缓冲区管理器
    let db_dir = ".temp/bmdb2";
    let _guard = TempFileGuard::new(db_dir);
    let mut fm = FileMgr::new(PathBuf::from(db_dir), 400).unwrap();
    let lm = LogMgr::new(fm.clone(), "testlog.log".to_string()).unwrap();
    let bm = BufferMgr::new(fm.clone(), lm.clone(), 10);

    // 2. 创建测试文件和块
    let filename = "testfile";
    if fm.length(filename).unwrap() == 0 {
        for _ in 0..5 {
            fm.append(filename).unwrap();
        }
    }

    // 3. 启动多个线程进行并发测试
    let headers = vec![format!("{:<12}", "Block Num")];
    let runner = MultiThreadRunner::new(10, headers);
    runner.excute(move |id| {
        let blk = BlockId::new(
            filename.to_string(),
            (id % 5) as i32, // 故意让线程竞争 blk
        );
        // 1. Pin 一个页面
        let buff_arc = bm.pin(&blk).unwrap();
        // 2. 获取锁修改内容
        {
            let mut buf = buff_arc.lock().unwrap();
            let msg = format!("Data from thread {}", id);
            buf.contents_mut().set_string(0, &msg);
            buf.set_modified(id as i32, -1);
        } // 释放锁

        // 3. Unpin
        bm.unpin(buff_arc);
        vec![format!("{:<12}", blk.number())]
    });
}
