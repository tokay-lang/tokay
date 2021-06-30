//! Handling of character-classes used by Token::Char and Token::Chars

use std::cmp::Ordering;

type CclRange = std::ops::RangeInclusive<char>;

/// Representation of a character-class
#[derive(Clone, PartialEq)]
pub struct Ccl {
    ranges: Vec<CclRange>,
}

impl Ccl {
    /** Create new empty character class. */
    pub fn new() -> Self {
        Self { ranges: Vec::new() }
    }

    /** Internal function for sorting ranges. */
    fn sort_ranges(&mut self) {
        self.ranges.sort_by(|a, b| a.start().cmp(b.start()));
    }

    /** Retrieve total number of characters in class */
    pub fn len(&self) -> u32 {
        self.ranges
            .iter()
            .map(|r| *r.end() as u32 - *r.start() as u32 + 1)
            .sum()
    }

    /** Dump the character ranges */
    pub fn dump(&self) {
        println!("{:p} ranges={}", self, self.ranges.len());

        for i in 0..self.ranges.len() {
            println!("  {:?}", self.ranges[i]);
        }
    }

    /** Normalize character-class by removing intersections and coherent ranges. */
    pub fn normalize(&mut self) {
        let mut prev_count: usize = 0;

        while self.ranges.len() != prev_count {
            prev_count = self.ranges.len();

            // First sort all ranges
            self.sort_ranges();

            for i in 0..self.ranges.len() - 1 {
                let a = &self.ranges[i];
                let b = &self.ranges[i + 1];

                // Remove intersections
                if b.start() <= a.end() && b.end() >= a.start() {
                    if b.end() > a.end() {
                        self.ranges[i] = *a.start()..=*b.end();
                    }

                    self.ranges.remove(i + 1);
                    break;
                }
                // Merge coherent ranges
                else if *a.end() as u32 + 1 == *b.start() as u32 {
                    self.ranges[i] = *a.start()..=*b.end();
                    self.ranges.remove(i + 1);
                    break;
                }
            }
        }
    }

    /** Negate entire character class */
    pub fn negate(&mut self) {
        let mut prev_count: usize = 0;
        let mut start = '\0';
        let mut end = '\0';

        while self.ranges.len() != prev_count {
            prev_count = self.ranges.len();

            for i in 0..self.ranges.len() {
                let irange = self.ranges[i].clone();

                if end < *irange.start() {
                    end = if *irange.start() > '\0' {
                        std::char::from_u32(*irange.start() as u32 - 1).unwrap()
                    } else {
                        '\0'
                    };

                    self.ranges[i] = start..=end;

                    start = if *irange.end() < std::char::MAX {
                        std::char::from_u32(*irange.end() as u32 + 1).unwrap()
                    } else {
                        std::char::MAX
                    };

                    end = start;
                } else {
                    end = if *irange.end() < std::char::MAX {
                        std::char::from_u32(*irange.end() as u32 + 1).unwrap()
                    } else {
                        std::char::MAX
                    };

                    self.ranges.remove(i);
                    break;
                }
            }
        }

        if end < std::char::MAX {
            self.ranges.push(end..=std::char::MAX);
        }

        self.normalize();
    }

    /** Add range to character class. */
    pub fn add(&mut self, range: CclRange) -> u32 {
        let len = self.len();
        self.ranges.push(range);
        self.normalize();
        self.len() - len
    }

    /** Clears entire range to be empty. */
    pub fn clear(&mut self) {
        self.ranges.clear();
    }

    /** Unions two character-classes; Returns the numer of characters added. */
    pub fn union(&mut self, merge: &Ccl) -> u32 {
        let len = self.len();

        for range in &merge.ranges {
            self.ranges.push(range.clone());
        }

        self.normalize();
        self.len() - len
    }

    /** Test */
    pub fn test(&self, range: &CclRange) -> bool {
        self.ranges
            .binary_search_by(|r| {
                if r.start() > range.end() {
                    Ordering::Greater
                } else if r.end() < range.start() {
                    Ordering::Less
                } else {
                    if range.start() >= r.start() && range.end() <= r.end() {
                        Ordering::Equal
                    } else {
                        Ordering::Less // fixme: Is here also a Greater-case?
                    }
                }
            })
            .is_ok()
    }

    /** Does this range fit all chars? */
    fn is_any(&self) -> bool {
        self.ranges.len() == 1
            && *self.ranges[0].start() == 0 as char
            && *self.ranges[0].end() == std::char::MAX
    }
}

impl std::fmt::Debug for Ccl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn escape(ch: char) -> String {
            match ch {
                '\x07' => "\\a".to_string(),
                '\x08' => "\\b".to_string(),
                '\x0c' => "\\f".to_string(),
                '\n' => "\\n".to_string(),
                '\r' => "\\r".to_string(),
                '\t' => "\\t".to_string(),
                '\x0b' => "\\v".to_string(),
                _ => format!("{}", ch),
            }
        }

        if self.is_any() {
            write!(f, ".")?;
        } else {
            write!(f, "[")?;
            for range in &self.ranges {
                if range.start() < range.end() {
                    write!(f, "{}-{}", escape(*range.start()), escape(*range.end()))?;
                } else {
                    write!(f, "{}", escape(*range.start()))?;
                }
            }
            write!(f, "]")?;
        }

        Ok(())
    }
}

impl PartialOrd for Ccl {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.ranges.len() == other.ranges.len() {
            for (mine, other) in self.ranges.iter().zip(other.ranges.iter()) {
                if other.end() > mine.end() || other.start() > mine.start() {
                    return Some(std::cmp::Ordering::Less);
                } else if other.end() < mine.end() {
                    return Some(std::cmp::Ordering::Greater);
                }
            }

            Some(std::cmp::Ordering::Equal)
        } else {
            None
        }
    }
}

/** Character-class construction helper-macro

Example:
```
use tokay::ccl;

let ccl = ccl!['A'..='Z', 'a'..='z', '_'..='_'];
```
*/
#[macro_export]
macro_rules! ccl {
    [$($range:expr),*] => {
        {
            let mut ccl = $crate::ccl::Ccl::new();
            $( ccl.add($range); )*
            ccl
        }
    }
}

pub fn ccl_test() {
    let mut ccl = Ccl::new();
    ccl.add('a'..='c');
    ccl.add('€'..='€');
    ccl.add('k'..='v');
    ccl.normalize();
    //ccl.dump();
    //ccl.negate();
    ccl.dump();

    //ccl.add('a'..='z');
    ccl.normalize();
    ccl.dump();

    for c in b'a'..=b'z' {
        let c = char::from(c);
        println!("{}: {}", c, ccl.test(&(c..=c)));
    }

    for rg in vec!['k'..='v', 'l'..='o', 'a'..='d', 'a'..='b', 'k'..='x'] {
        println!("{:?} {}", &rg, ccl.test(&rg));
    }

    let mut t = Ccl::new();
    t.add('A'..='D');

    ccl.union(&t);
    ccl.dump();
}
