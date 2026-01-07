pub struct Page {
    buf: Vec<u8>,
}

impl Page {
    pub fn new(size: usize) -> Self {
        Page { buf: vec![0; size] }
    }

    pub fn from_bytes(b: Vec<u8>) -> Self {
        Page { buf: b }
    }

    pub fn get_int(&self, offset: usize) -> i32 {
        let bytes = &self.buf[offset..offset + 4];
        i32::from_be_bytes(bytes.try_into().expect("get_int: invalid integer bytes"))
    }

    pub fn set_int(&mut self, offset: usize, val: i32) {
        let bytes = val.to_be_bytes();
        self.buf[offset..offset + 4].copy_from_slice(&bytes);
    }

    pub fn get_bytes(&self, offset: usize) -> Vec<u8> {
        let length = self.get_int(offset) as usize;
        let start = offset + 4;
        let end = start + length;
        self.buf[start..end].to_vec()
    }

    pub fn set_bytes(&mut self, offset: usize, val: &[u8]) {
        let length = val.len() as i32;
        self.set_int(offset, length);
        let start = offset + 4;
        let end = start + length as usize;
        self.buf[start..end].copy_from_slice(val);
    }

    pub fn get_string(&self, offset: usize) -> String {
        let bytes = self.get_bytes(offset);
        String::from_utf8(bytes).expect("get_string: invalid UTF-8 string")
    }

    pub fn set_string(&mut self, offset: usize, val: &str) {
        self.set_bytes(offset, val.as_bytes())
    }

    pub fn contents_mut(&mut self) -> &mut [u8] {
        &mut self.buf[..]
    }

    pub fn contents(&self) -> &[u8] {
        &self.buf
    }

    pub fn max_length(s: &str) -> usize {
        4 + s.len() // 4 bytes for length + string bytes
    }
}

#[cfg(test)]
mod tests {
    use super::Page;
    #[test]
    fn page_test() {
        let mut page = Page::new(100);
        page.set_int(0, 42);
        page.set_string(4, "hello");
        assert_eq!(page.get_int(0), 42);
        assert_eq!(page.get_string(4), "hello");
    }
}
