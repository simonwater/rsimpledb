use std::fs;
pub struct TempFileGuard<'a> {
    dir: &'a str,
}

impl<'a> TempFileGuard<'a> {
    pub fn new(dir: &'a str) -> Self {
        Self { dir }
    }
}

impl<'a> Drop for TempFileGuard<'a> {
    fn drop(&mut self) {
        fs::remove_dir_all(self.dir).unwrap();
    }
}
