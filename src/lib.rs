#[cfg(test)] mod tests;

use std::iter::FromIterator;

pub type Id = usize;

type Block = u32;
const BITS: usize = 32;

/// Given return the smallest integer m such that m*k <= n
fn num_blocks(bits: usize) -> usize {
    if bits % BITS == 0 { 
        bits / BITS
    } else { 
        bits / BITS + 1 
    }
}

pub struct IdSet {
    storage: Vec<Block>,
    len: usize,
}

impl IdSet {
    pub fn new() -> Self {
        IdSet {
            storage: Vec::new(),
            len: 0,
        }
    }

    pub fn with_capacity(nbits: usize) -> Self {
        IdSet {
            storage: Vec::with_capacity(num_blocks(nbits)),
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn insert(&mut self, id: Id) -> bool {
        let (word, bit) = (id / BITS, id % BITS);
        let mask = 1 << bit;

        if word < self.storage.len() {
            if (self.storage[word] & mask) == 0 {
                self.storage[word] |= mask;
                self.len += 1;
                true
            } else {
                false
            }
        } else {
            self.storage.resize(word + 1, 0);
            self.storage[word] = mask;
            self.len += 1;
            true
        }
    }

    pub fn remove(&mut self, id: Id) -> bool {
        let (word, bit) = (id / BITS, id % BITS);
        let mask = 1 << bit;

        if word < self.storage.len() {
            if (self.storage[word] & mask) != 0 {
                self.storage[word] &= !mask;
                self.len -= 1;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn contains(&self, id: Id) -> bool {
        let (word, bit) = (id / BITS, id % BITS);
        let mask = 1 << bit;

        if word < self.storage.len() {
            (self.storage[word] & mask) != 0
        } else {
            false
        }
    }

    pub fn iter(&self) -> Iter {
        Iter {
            storage: &self.storage,
            word: 0,
            bit: 0,
        }
    }
}

impl FromIterator<Id> for IdSet {
    fn from_iter<I: IntoIterator<Item = Id>>(iter: I) -> Self {
        let mut set = IdSet::new();
        for id in iter {
            set.insert(id);
        }
        set
    }
}

impl<'a> IntoIterator for &'a IdSet {
    type Item = Id;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct Iter<'a> {
    storage: &'a [u32],
    word: usize,
    bit: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.bit == BITS {
                self.word += 1;
                if self.word >= self.storage.len() {
                    return None;
                }
                self.bit = 0;
            }
            let bit = self.bit;
            self.bit += 1;
            if (self.storage[self.word] & (1 << bit)) != 0 {
                return Some(self.word * BITS + bit)
            }
        }
    }
}