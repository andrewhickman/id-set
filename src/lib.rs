#[cfg(test)] mod tests;

use std::{fmt, slice};
use std::iter::FromIterator;

pub type Id = usize;

type Block = u32;
const BITS: usize = 32;

/// Given n and k return the largest integer m such that m*k <= n
fn ceil_div(n: usize, k: usize) -> usize {
    if n % k == 0 {
        n / k
    } else {
        n / k + 1
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

    pub fn new_filled(len: usize) -> Self {
        let (nwords, nbits) = (len / BITS, len % BITS);
        let mut storage = vec![!0; nwords];
        if nbits != 0 {
            storage.push((1u32 << nbits) - 1);
        }
        IdSet {
            storage,
            len,
        }
    }

    pub fn with_capacity(nbits: usize) -> Self {
        IdSet {
            storage: Vec::with_capacity(ceil_div(nbits, BITS)),
            len: 0,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut storage = Vec::with_capacity(ceil_div(bytes.len(), 8));
        let mut len = 0;
        for chunk in bytes.chunks(4) {
            let mut word = 0;
            for &byte in chunk {
                word <<= 8;
                word |= byte as u32;
            }
            len += word.count_ones() as usize;
            storage.push(word);
        }
        IdSet { storage, len }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn clear(&mut self) {
        self.storage.clear();
        self.len = 0;
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
            storage: self.storage.iter(),
            len: self.len,
            word: 0,
            mask: 0,
            id: 0,
        }
    }
}

impl Clone for IdSet {
    fn clone(&self) -> Self {
        IdSet {
            storage: self.storage.clone(),
            len: self.len,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.storage.clone_from(&source.storage);
        self.len = source.len;
    }
}

impl fmt::Debug for IdSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{")?;
        let mut iter = self.iter();
        if let Some(id) = iter.next() {
            write!(f, "{:?}", id)?;
            for id in iter {
                write!(f, ", {:?}", id)?;
            }
        }
        write!(f, "}}")
    }
}

impl Default for IdSet {
    fn default() -> Self {
        IdSet::new()
    }
}

impl Eq for IdSet {}

impl PartialEq for IdSet {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }
        let (mut lhs, mut rhs) = (self.storage.iter(), other.storage.iter());
        loop {
            match (lhs.next(), rhs.next()) {
                (Some(&l), Some(&r)) => {
                    if l != r {
                        return false;
                    }
                },
                (None, None) => return true,
                (Some(&l), None) => return l == 0 && lhs.all(|&word| word == 0),
                (None, Some(&r)) => return r == 0 && rhs.all(|&word| word == 0),
            }
        }
    }
}

impl Extend<Id> for IdSet {
    fn extend<I: IntoIterator<Item = Id>>(&mut self, iter: I) {
        for id in iter {
            self.insert(id);
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

#[derive(Clone, Debug)]
pub struct Iter<'a> {
    storage: slice::Iter<'a, u32>,
    len: usize, // the number of remaining set bits.
    word: u32,  // 
    mask: u32,
    id: Id,
}

impl<'a> Iterator for Iter<'a> {
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.mask == 0 {
                loop {
                    self.word = match self.storage.next() {
                        Some(&word) => word,
                        None => return None,
                    };
                    if self.word != 0 {
                        break;
                    }
                    self.id += BITS;
                }
                self.mask = 1;
            }
            let bit = self.word & self.mask;
            self.mask <<= 1;
            self.id += 1;
            if bit != 0 {
                self.len -= 1;
                return Some(self.id - 1);
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {
    fn len(&self) -> usize {
        self.len
    }
}