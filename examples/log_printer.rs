use rsimpledb::file::FileMgr;
use rsimpledb::file::Page;
use rsimpledb::log::LogMgr;
use rsimpledb::util::TempFileGuard;
use std::path::PathBuf;

pub fn main() {
    let db_dir = ".temp/lmdb";
    let _guard = TempFileGuard::new(db_dir);
    let fm = FileMgr::new(PathBuf::from(db_dir), 128).unwrap();
    let mut log_mgr = LogMgr::new(fm.clone(), "logfile".to_string()).unwrap();
    print_log_records("The initial empty log file:", &mut log_mgr);

    create_log_records(1, 35, &mut log_mgr);
    print_log_records("The log file now has these records:", &mut log_mgr);

    create_log_records(36, 70, &mut log_mgr);
    log_mgr.flush(65).unwrap();
    print_log_records(
        "The log file now has these records after flush 65:",
        &mut log_mgr,
    );
}

fn print_log_records(msg: &str, log_mgr: &mut LogMgr) {
    println!("{}", msg);
    let mut iter = log_mgr.iterator().unwrap();
    while let Some(rec) = iter.next() {
        let p = Page::from_bytes(rec.unwrap());
        let s = p.get_string(0);
        let ipos = 4 + s.len();
        let val = p.get_int(ipos);
        println!("[{s}, {val}]");
    }
}

fn create_log_records(start: i32, end: i32, log_mgr: &mut LogMgr) {
    println!("Creating log records from {} to {}", start, end);
    for i in start..=end {
        let rec = create_log_record(format!("record{i}").as_str(), i);
        let lsn = log_mgr.append(&rec).unwrap();
        print!("{lsn} ");
    }
    println!()
}

// Create a log record having two values: a string and an integer.
fn create_log_record(s: &str, n: i32) -> Vec<u8> {
    let spos = 0;
    let ipos = spos + 4 + s.len(); // 4 bytes for length of string
    let rec = vec![0u8; ipos + 4]; // 4 bytes for integer
    let mut p = Page::from_bytes(rec);
    // write the string
    p.set_string(spos, s);
    p.set_int(ipos, n);
    p.contents().to_vec()
}
