use std::io::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Offset {
    // fixme: Hold source filename information as well in the future?
    pub offset: usize,
    pub row: u32,
    pub col: u32,
}

pub type Range = std::ops::Range<usize>;

pub struct Reader {
    reader: Box<dyn BufRead>,
    buffer: Vec<char>,
    offset: Offset,
    eof: bool,
}

impl Reader {
    /// Creates a new reader on buffer read.
    pub fn new(reader: Box<dyn BufRead>) -> Self {
        Self {
            reader,
            buffer: Vec::new(),
            offset: Offset {
                offset: 0,
                row: 1,
                col: 1,
            },
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
        if self.offset.offset < self.buffer.len() {
            self.offset.offset += 1;

            let ch = self.buffer[self.offset.offset - 1];
            if ch == '\n' {
                self.offset.row += 1;
                self.offset.col = 1;
            } else {
                self.offset.col += 1;
            }

            return Some(ch);
        }

        if self.eof {
            return None;
        }

        self.read_line();
        self.next()
    }

    pub fn peek(&mut self) -> Option<char> {
        if self.offset.offset < self.buffer.len() {
            return Some(self.buffer[self.offset.offset]);
        }

        if self.eof {
            return None;
        }

        self.read_line();
        self.peek()
    }

    pub fn tell(&self) -> Offset {
        self.offset
    }

    pub fn eof(&self) -> bool {
        self.eof
    }

    pub fn reset(&mut self, offset: Offset) {
        self.offset = offset;
    }

    /// Capture last length characters.
    pub fn capture_last(&self, mut length: usize) -> Range {
        if length > self.offset.offset {
            length = self.offset.offset;
        }

        self.offset.offset - length..self.offset.offset
    }

    // Capture all characters from start to current offset.
    pub fn capture_from(&self, start: &Offset) -> Range {
        let mut start = start.offset;

        if start > self.offset.offset {
            start = self.offset.offset;
        }

        start..self.offset.offset
    }

    pub fn print(&self, start: usize) {
        println!("{:?}", &self.buffer[start..self.offset.offset])
    }

    pub fn extract(&self, range: &Range) -> String {
        self.buffer[range.start..range.end].iter().collect()
    }

    /// Commits current input buffer and removes cached content
    pub fn commit(&mut self) {
        self.buffer.drain(0..self.offset.offset);
        self.offset.offset = 0;
    }
}
