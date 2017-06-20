//! A bit-set implementation for use in id-map. Notable differences from bit-set are the IntoIter
//! struct and retain() methods.

#![deny(missing_docs)]

#[cfg(test)] mod tests;

use std::{fmt, slice, vec};
use std::iter::FromIterator;

/// The element type of the set.
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

/// A set of `usize` elements represented by a bit vector. Storage required is proportional to the
/// maximum element in the set.
pub struct IdSet {
    storage: Vec<Block>,
    len: usize,
}

impl IdSet {
    /// Creates an empty `IdSet`.
    pub fn new() -> Self {
        IdSet {
            storage: Vec::new(),
            len: 0,
        }
    }

    /// Creates a `IdSet` filled with all elements from 0 to n.
    pub fn new_filled(n: usize) -> Self {
        let (nwords, nbits) = (n / BITS, n % BITS);
        let mut storage = vec![!0; nwords];
        if nbits != 0 {
            storage.push((1u32 << nbits) - 1);
        }
        IdSet {
            storage,
            len: n,
        }
    }

    /// Creates a empty `IdSet` that can hold elements up to n before reallocating.
    pub fn with_capacity(n: usize) -> Self {
        IdSet {
            storage: Vec::with_capacity(ceil_div(n, BITS)),
            len: 0,
        }
    }

    /// Creates a set from a raw set of bytes.
    pub fn from_bytes(bytes: &[u32]) -> Self {
        let storage = Vec::from(bytes);
        let len = bytes.iter().map(|&word| word.count_ones() as usize).sum();
        IdSet { storage, len }
    }
    
    /// Returns the number of elements in the set.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Removes all elements from the set.
    pub fn clear(&mut self) {
        self.storage.clear();
        self.len = 0;
    }

    /// Inserts the given elements into the set, returning true if it was not already in the set.
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

    /// Removes the given element from the set, returning true if it was in the set.
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

    /// Returns true if the given element is in the set.
    pub fn contains(&self, id: Id) -> bool {
        let (word, bit) = (id / BITS, id % BITS);
        let mask = 1 << bit;

        if word < self.storage.len() {
            (self.storage[word] & mask) != 0
        } else {
            false
        }
    }

    /// Remove all elements that don't satisfy the predicate.
    pub fn retain<F: FnMut(Id) -> bool>(&mut self, mut pred: F) {
        let mut id = 0;
        for word in &mut self.storage {
            for bit in 0..BITS {
                let mask = 1 << bit;
                if (*word & mask) != 0 && !pred(id) {
                    self.len -= 1;
                    *word &= !mask;
                }
                id += 1;
            }
        }
    }

    /// An iterator over all elements in increasing order.
    pub fn iter(&self) -> Iter {
        let mut storage = self.storage.iter();
        let &word = storage.next().unwrap_or(&0);
        Iter {
            storage,
            len: self.len,
            word,
            idx: 0,
        }
    }

    /// A consuming iterator over all elements in increasing order.
    pub fn into_iter(self) -> IntoIter {
        let mut storage = self.storage.into_iter();
        let word = storage.next().unwrap_or(0);
        IntoIter {
            storage,
            len: self.len,
            word,
            idx: 0,
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
/// An iterator over all elements in increasing order.
pub struct Iter<'a> {
    storage: slice::Iter<'a, u32>,
    len: usize,
    word: u32, 
    idx: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        while self.word == 0 {
            match self.storage.next() {
                Some(&word) => self.word = word,
                None => return None,
            }
            self.idx += BITS;
        }
        // remove the LSB of the current word
        let bit = (self.word & (!self.word + 1)) - 1;
        self.word &= self.word - 1;
        self.len -= 1;
        Some(self.idx + bit.count_ones() as usize)
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

#[derive(Clone, Debug)]
/// A consuming iterator over all elements in increasing order.
pub struct IntoIter {
    storage: vec::IntoIter<u32>,
    len: usize,
    word: u32, 
    idx: usize,
}

impl Iterator for IntoIter {
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        while self.word == 0 {
            match self.storage.next() {
                Some(word) => self.word = word,
                None => return None,
            }
            self.idx += BITS;
        }
        // remove the LSB of the current word
        let bit = (self.word & (!self.word + 1)) - 1;
        self.word &= self.word - 1;
        self.len -= 1;
        Some(self.idx + bit.count_ones() as usize)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl ExactSizeIterator for IntoIter {
    fn len(&self) -> usize {
        self.len
    }
}