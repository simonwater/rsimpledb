use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use crate::file::BlockId;
use crate::file::Page;

pub struct FileMgr {
    db_directory: PathBuf,
    blocksize: usize,
    is_new: bool,
    open_files: HashMap<String, File>,
}

impl FileMgr {
    pub fn new(db_directory: PathBuf, blocksize: usize) -> Self {
        let is_new = !db_directory.exists();
        if is_new {
            std::fs::create_dir_all(&db_directory).expect("cannot create db directory");
        }

        // remove leftover temporary tables
        if let Ok(entries) = std::fs::read_dir(&db_directory) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with("temp") {
                        let _ = std::fs::remove_file(entry.path());
                    }
                }
            }
        }

        FileMgr {
            db_directory,
            blocksize,
            is_new,
            open_files: HashMap::new(),
        }
    }

    fn get_file_mut(&mut self, filename: &str) -> std::io::Result<&mut File> {
        if !self.open_files.contains_key(filename) {
            let mut db_table = self.db_directory.clone();
            db_table.push(filename);
            let f = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(db_table)?;
            self.open_files.insert(filename.to_string(), f);
        }
        Ok(self.open_files.get_mut(filename).unwrap())
    }

    pub fn read(&mut self, blk: &BlockId, p: &mut Page) {
        let offset = (blk.number() as usize) * self.blocksize;
        let f = self
            .get_file_mut(blk.file_name())
            .expect("cannot access file");
        f.seek(SeekFrom::Start(offset as u64)).expect("cannot seek");
        let buf = p.contents_mut();
        f.read_exact(buf)
            .unwrap_or_else(|_| panic!("cannot read block {}", blk));
    }

    pub fn write(&mut self, blk: &BlockId, p: &Page) {
        let offset = (blk.number() as usize) * self.blocksize;
        let f = self
            .get_file_mut(blk.file_name())
            .expect("cannot access file");
        f.seek(SeekFrom::Start(offset as u64)).expect("cannot seek");
        f.write_all(p.contents())
            .unwrap_or_else(|_| panic!("cannot write block {}", blk));
        f.flush().ok();
    }

    pub fn append(&mut self, filename: &str) -> BlockId {
        let newblknum = self.length(filename);
        let blk = BlockId::new(filename.to_string(), newblknum);
        let zeros = vec![0u8; self.blocksize];
        let offset = (blk.number() as usize) * self.blocksize;
        let f = self.get_file_mut(filename).expect("cannot access file");
        f.seek(SeekFrom::Start(offset as u64)).expect("cannot seek");
        f.write_all(&zeros)
            .unwrap_or_else(|_| panic!("cannot append block {}", blk));
        f.flush().ok();
        blk
    }

    pub fn length(&mut self, filename: &str) -> usize {
        let f = match self.get_file_mut(filename) {
            Ok(f) => f,
            Err(_) => panic!("cannot access {}", filename),
        };
        match f.metadata() {
            Ok(meta) => (meta.len() as usize) / self.blocksize,
            Err(_) => panic!("cannot access {}", filename),
        }
    }

    pub fn is_new(&self) -> bool {
        self.is_new
    }

    pub fn block_size(&self) -> usize {
        self.blocksize
    }
}

#[cfg(test)]
mod tests {
    use super::{BlockId, FileMgr, Page};
    use std::path::PathBuf;

    #[test]
    fn test_file_mgr() {
        let db_dir = PathBuf::from("testdb");
        let blocksize = 128;
        let mut fm = FileMgr::new(db_dir.clone(), blocksize);

        let filename = "testfile";
        let blk = fm.append(filename); // 在末尾追加

        let mut p1 = Page::new(blocksize);
        p1.set_int(0, 12345);
        p1.set_string(4, "hello");
        fm.write(&blk, &p1);

        let mut p2 = Page::new(blocksize);
        fm.read(&blk, &mut p2);
        assert_eq!(p2.get_int(0), 12345);
        assert_eq!(p2.get_string(4), "hello");

        let blk = BlockId::new(filename.to_string(), 10);
        let mut p3 = Page::new(blocksize);
        p3.set_string(88, "abcdefghijklm");
        p3.set_int(10, 123);
        fm.write(&blk, &p3);

        let mut p4 = Page::new(blocksize);
        fm.read(&blk, &mut p4);
        assert_eq!(p4.get_string(88), "abcdefghijklm");
        assert_eq!(p4.get_int(10), 123);

        // Clean up
        let mut db_table = db_dir.clone();
        db_table.push(filename);
        std::fs::remove_file(db_table).ok();
        std::fs::remove_dir(db_dir).ok();
    }
}
