use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::DbResult;
use crate::file::BlockId;
use crate::file::FileError;
use crate::file::Page;

#[derive(Clone)]
pub struct FileMgr {
    state: Arc<Mutex<FileMgrState>>,
    blocksize: usize,
    is_new: bool,
}

impl FileMgr {
    pub fn new(db_directory: PathBuf, blocksize: usize) -> DbResult<Self> {
        let is_new = !db_directory.exists();
        let state = FileMgrState::new(db_directory, blocksize)?;
        Ok(FileMgr {
            state: Arc::new(Mutex::new(state)),
            blocksize,
            is_new,
        })
    }

    pub fn read(&mut self, blk: &BlockId, p: &mut Page) -> DbResult<()> {
        let mut state = self.state.lock().unwrap();
        state.read(blk, p)
    }

    pub fn write(&mut self, blk: &BlockId, p: &Page) -> DbResult<()> {
        let mut state = self.state.lock().unwrap();
        state.write(blk, p)
    }

    pub fn append(&mut self, filename: &str) -> DbResult<BlockId> {
        let mut state = self.state.lock().unwrap();
        state.append(filename)
    }

    pub fn length(&mut self, filename: &str) -> DbResult<usize> {
        let mut state = self.state.lock().unwrap();
        state.length(filename)
    }

    pub fn is_new(&self) -> bool {
        self.is_new
    }

    pub fn block_size(&self) -> usize {
        self.blocksize
    }
}

struct FileMgrState {
    db_directory: PathBuf,
    blocksize: usize,
    open_files: HashMap<String, File>,
}

impl FileMgrState {
    pub fn new(db_directory: PathBuf, blocksize: usize) -> DbResult<Self> {
        let is_new = !db_directory.exists();
        if is_new {
            fs::create_dir_all(&db_directory).map_err(|io_err| FileError::Io {
                io_err,
                message: String::from("Failed to create database directory"),
            })?;
        }

        // remove leftover temporary tables
        if let Ok(entries) = fs::read_dir(&db_directory) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with("temp") {
                        fs::remove_file(entry.path()).map_err(|io_err| FileError::Io {
                            io_err,
                            message: format!("Failed to remove temporary table file: {name}"),
                        })?;
                    }
                }
            }
        }

        Ok(FileMgrState {
            db_directory,
            blocksize,
            open_files: HashMap::new(),
        })
    }

    fn get_file_mut(&mut self, filename: &str) -> DbResult<&mut File> {
        if !self.open_files.contains_key(filename) {
            let mut db_table = self.db_directory.clone();
            db_table.push(filename);
            let f = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(db_table)
                .map_err(|io_err| FileError::Io {
                    io_err,
                    message: format!("Failed to access file: {filename}"),
                })?;
            self.open_files.insert(filename.to_string(), f);
        }
        Ok(self.open_files.get_mut(filename).unwrap())
    }

    pub fn read(&mut self, blk: &BlockId, p: &mut Page) -> DbResult<()> {
        let offset = (blk.number() as usize) * self.blocksize;
        let f = self.get_file_mut(blk.file_name())?;
        f.seek(SeekFrom::Start(offset as u64))
            .map_err(|io_err| FileError::Io {
                io_err,
                message: format!("Cannot seek file for block {}", blk),
            })?;
        let buf = p.contents_mut();
        f.read_exact(buf).map_err(|io_err| FileError::Io {
            io_err,
            message: format!("Cannot read block {}", blk),
        })?;
        Ok(())
    }

    pub fn write(&mut self, blk: &BlockId, p: &Page) -> DbResult<()> {
        let offset = (blk.number() as usize) * self.blocksize;
        let f = self.get_file_mut(blk.file_name())?;
        f.seek(SeekFrom::Start(offset as u64))
            .map_err(|io_err| FileError::Io {
                io_err,
                message: format!("Cannot seek file for block {}", blk),
            })?;
        f.write_all(p.contents()).map_err(|io_err| FileError::Io {
            io_err,
            message: format!("Cannot write block {}", blk),
        })?;
        f.flush().map_err(|io_err| FileError::Io {
            io_err,
            message: format!("Cannot flush block {}", blk),
        })?;
        Ok(())
    }

    pub fn append(&mut self, filename: &str) -> DbResult<BlockId> {
        let newblknum = self.length(filename)?;
        let blk = BlockId::new(filename.to_string(), newblknum as i32);
        let zeros = vec![0u8; self.blocksize];
        let offset = (blk.number() as usize) * self.blocksize;
        let f = self.get_file_mut(filename)?;
        f.seek(SeekFrom::Start(offset as u64))
            .map_err(|io_err| FileError::Io {
                io_err,
                message: format!("Cannot seek file for block {}", blk),
            })?;
        f.write_all(&zeros).map_err(|io_err| FileError::Io {
            io_err,
            message: format!("Cannot write block {}", blk),
        })?;
        f.flush().map_err(|io_err| FileError::Io {
            io_err,
            message: format!("Cannot flush block {}", blk),
        })?;
        Ok(blk)
    }

    pub fn length(&mut self, filename: &str) -> DbResult<usize> {
        let f = self.get_file_mut(filename)?;
        f.metadata()
            .map(|meta| (meta.len() as usize) / self.blocksize)
            .map_err(|io_err| {
                FileError::Io {
                    io_err,
                    message: format!("Cannot access file metadata for {}", filename),
                }
                .into()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{BlockId, Page};
    use crate::DataBase;
    use crate::util::TempFileGuard;

    #[test]
    fn file_mgr_test() {
        let db_dir = ".temp/fmdb";
        let _guard = TempFileGuard::new(db_dir);
        let blocksize = 400;
        let db: DataBase = DataBase::new_with_size(db_dir, blocksize, 10).unwrap();
        let mut fm = db.file_mgr();

        let filename = "testfile";
        let blk = fm.append(filename).unwrap(); // 在末尾追加

        let mut p1 = Page::new(blocksize);
        p1.set_int(0, 12345);
        p1.set_string(4, "hello");
        fm.write(&blk, &p1).unwrap();

        let mut p2 = Page::new(blocksize);
        fm.read(&blk, &mut p2).unwrap();
        assert_eq!(p2.get_int(0), 12345);
        assert_eq!(p2.get_string(4), "hello");

        let blk = BlockId::new(filename.to_string(), 10);
        let mut p3 = Page::new(blocksize);
        p3.set_string(88, "abcdefghijklm");
        p3.set_int(10, 123);
        fm.write(&blk, &p3).unwrap();

        let mut p4 = Page::new(blocksize);
        fm.read(&blk, &mut p4).unwrap();
        assert_eq!(p4.get_string(88), "abcdefghijklm");
        assert_eq!(p4.get_int(10), 123);
    }
}
