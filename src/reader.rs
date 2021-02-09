use std::io::prelude::*;

pub type Range = std::ops::Range<usize>;

pub struct Reader {
    reader: Box<dyn BufRead>,
    buffer: Vec<char>,
    offset: usize,
    eof: bool,
}

impl Reader {
    /// Creates a new reader on buffer read.
    pub fn new(reader: Box<dyn BufRead>) -> Self {
        Self {
            reader,
            buffer: Vec::new(),
            offset: 0,
            eof: false,
        }
    }

    /// Internal function for reading a line.
    fn read_line(&mut self) -> Option<usize> {
        let mut s = String::new();
        if let Ok(n) = self.reader.read_line(&mut s) {
            if n == 0 {
                self.eof = true;
                return None;
            }

            for ch in s.chars() {
                self.buffer.push(ch);
            }

            Some(n)
        } else {
            self.eof = true;
            None
        }
    }

    pub fn next(&mut self) -> Option<char> {
        if self.offset < self.buffer.len() {
            self.offset += 1;
            return Some(self.buffer[self.offset - 1]);
        }

        if self.eof {
            return None;
        }

        self.read_line();
        self.next()
    }

    pub fn peek(&mut self) -> Option<char> {
        if self.offset < self.buffer.len() {
            return Some(self.buffer[self.offset]);
        }

        if self.eof {
            return None;
        }

        self.read_line();
        self.peek()
    }

    pub fn tell(&self) -> usize {
        self.offset
    }

    pub fn eof(&self) -> bool {
        self.eof
    }

    pub fn reset(&mut self, offset: usize) {
        self.offset = offset;
    }

    /// Capture last length characters.
    pub fn capture_last(&self, mut length: usize) -> Range {
        if length > self.offset {
            length = self.offset;
        }

        self.offset - length..self.offset
    }

    // Capture all characters from start to current offset.
    pub fn capture_from(&self, mut start: usize) -> Range {
        if start > self.offset {
            start = self.offset;
        }

        start..self.offset
    }

    pub fn print(&self, start: usize) {
        println!("{:?}", &self.buffer[start..self.offset])
    }

    pub fn extract(&self, range: &Range) -> String {
        self.buffer[range.start..range.end].iter().collect()
    }

    pub fn commit(&mut self) {
        self.buffer.drain(0..self.offset);
        self.reset(0);
    }
}
