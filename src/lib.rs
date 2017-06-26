//! A bit-set implementation for use in id-map. Notable differences from bit-set are the IntoIter
//! struct and retain() methods.

#![deny(missing_docs)]

#[cfg(test)]
mod tests;

use std::{fmt, slice, vec};
use std::iter::FromIterator;

/// The element type of the set.
pub type Id = usize;

/// The block type of the underlying representation.
pub type Block = u32;

/// The number of bits in the block type.
pub const BITS: usize = 32;

fn mask(bit: usize) -> Block {
    (1 as Block) << bit
}

/// Given n and k return the largest integer m such that m*k <= n
fn ceil_div(n: usize, k: usize) -> usize {
    if n % k == 0 { n / k } else { n / k + 1 }
}

/// A set of `usize` elements represented by a bit vector. blocks required is proportional to the
/// maximum element in the set.
pub struct IdSet {
    blocks: Vec<Block>,
    len: usize,
}

impl IdSet {
    #[inline]
    /// Creates an empty `IdSet`.
    pub fn new() -> Self {
        IdSet {
            blocks: Vec::new(),
            len: 0,
        }
    }

    #[inline]
    /// Creates a `IdSet` filled with all elements from 0 to n.
    pub fn new_filled(n: usize) -> Self {
        let (nwords, nbits) = (n / BITS, n % BITS);
        let mut blocks = vec![!0; nwords];
        if nbits != 0 {
            blocks.push(mask(nbits) - 1);
        }
        IdSet { blocks, len: n }
    }

    #[inline]
    /// Creates a empty `IdSet` that can hold elements up to n before reallocating.
    pub fn with_capacity(n: usize) -> Self {
        IdSet {
            blocks: Vec::with_capacity(ceil_div(n, BITS)),
            len: 0,
        }
    }

    #[inline]
    /// Creates a set from a raw set of bytes.
    pub fn from_bytes(bytes: &[Block]) -> Self {
        let blocks = Vec::from(bytes);
        let len = bytes
            .iter()
            .map(|&word| word.count_ones() as usize)
            .sum();
        IdSet { blocks, len }
    }

    #[inline]
    /// Returns the number of elements in the set.
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    /// Removes all elements from the set.
    pub fn clear(&mut self) {
        self.blocks.clear();
        self.len = 0;
    }

    #[inline]
    /// Inserts the given elements into the set, returning true if it was not already in the set.
    pub fn insert(&mut self, id: Id) -> bool {
        let (word, bit) = (id / BITS, id % BITS);
        let mask = mask(bit);

        if word < self.blocks.len() {
            if (self.blocks[word] & mask) == 0 {
                self.blocks[word] |= mask;
                self.len += 1;
                true
            } else {
                false
            }
        } else {
            self.blocks.resize(word + 1, 0);
            self.blocks[word] = mask;
            self.len += 1;
            true
        }
    }

    #[inline]
    /// Removes the given element from the set, returning true if it was in the set.
    pub fn remove(&mut self, id: Id) -> bool {
        let (word, bit) = (id / BITS, id % BITS);
        let mask = mask(bit);

        if word < self.blocks.len() {
            if (self.blocks[word] & mask) != 0 {
                self.blocks[word] &= !mask;
                self.len -= 1;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    #[inline]
    /// Returns true if the given element is in the set.
    pub fn contains(&self, id: Id) -> bool {
        let (word, bit) = (id / BITS, id % BITS);

        if word < self.blocks.len() {
            (self.blocks[word] & mask(bit)) != 0
        } else {
            false
        }
    }

    #[inline]
    /// Remove all elements that don't satisfy the predicate.
    pub fn retain<F: FnMut(Id) -> bool>(&mut self, mut pred: F) {
        let mut id = 0;
        for word in &mut self.blocks {
            for bit in 0..BITS {
                let mask = mask(bit);
                if (*word & mask) != 0 && !pred(id) {
                    self.len -= 1;
                    *word &= !mask;
                }
                id += 1;
            }
        }
    }

    #[inline]
    /// An iterator over all elements in increasing order.
    pub fn iter(&self) -> Iter {
        Iter {
            inner: self.blocks().into_id_iter(),
            len: self.len,
        }
    }

    #[inline]
    /// A consuming iterator over all elements in increasing order.
    pub fn into_iter(self) -> IntoIter {
        let len = self.len;
        IntoIter {
            inner: self.into_blocks().into_id_iter(),
            len,
        }
    }

    #[inline]
    /// An iterator over the blocks of the underlying representation.
    pub fn blocks(&self) -> Blocks {
        Blocks {
            inner: self.blocks.iter(),
        }
    }

    #[inline]
    /// A consuming iterator over the blocks of the underlying representation.
    pub fn into_blocks(self) -> IntoBlocks {
        IntoBlocks {
            inner: self.blocks.into_iter(),
        }
    }
}

impl Clone for IdSet {
    #[inline]
    fn clone(&self) -> Self {
        IdSet {
            blocks: self.blocks.clone(),
            len: self.len,
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.blocks.clone_from(&source.blocks);
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
    #[inline]
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
        let (mut lhs, mut rhs) = (self.blocks.iter(), other.blocks.iter());
        loop {
            match (lhs.next(), rhs.next()) {
                (Some(&l), Some(&r)) => {
                    if l != r {
                        return false;
                    }
                }
                (None, None) => return true,
                (Some(&l), None) => return l == 0 && lhs.all(|&word| word == 0),
                (None, Some(&r)) => return r == 0 && rhs.all(|&word| word == 0),
            }
        }
    }
}

impl Extend<Id> for IdSet {
    #[inline]
    fn extend<I: IntoIterator<Item = Id>>(&mut self, iter: I) {
        for id in iter {
            self.insert(id);
        }
    }
}

impl FromIterator<Id> for IdSet {
    #[inline]
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

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Clone, Debug)]
/// An iterator over all elements in increasing order.
pub struct Iter<'a> {
    inner: IdIter<Blocks<'a>>,
    len: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = Id;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let id = self.inner.next();
        if id.is_some() {
            self.len -= 1;
        }
        id
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.len
    }
}

#[derive(Clone, Debug)]
/// A consuming iterator over all elements in increasing order.
pub struct IntoIter {
    inner: IdIter<IntoBlocks>,
    len: usize,
}

impl Iterator for IntoIter {
    type Item = Id;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {

        let id = self.inner.next();
        if id.is_some() {
            self.len -= 1;
        }
        id
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl ExactSizeIterator for IntoIter {
    #[inline]
    fn len(&self) -> usize {
        self.len
    }
}

#[derive(Clone, Debug)]
/// Transforms an iterator over blocks into an iterator over elements.
pub struct IdIter<I> {
    blocks: I,
    word: Block,
    idx: usize,
}

impl<I> Iterator for IdIter<I> where I: ExactSizeIterator<Item = Block> {
    type Item = Id;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while self.word == 0 {
            match self.blocks.next() {
                Some(word) => self.word = word,
                None => return None,
            }
            self.idx += BITS;
        }
        // remove the LSB of the current word
        let bit = (self.word & (!self.word + 1)) - 1;
        self.word &= self.word - 1;
        Some(self.idx + bit.count_ones() as usize)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let ones = self.word.count_ones() as usize;
        (ones, Some(self.blocks.len() * BITS + ones))
    }
}

/// An iterator over blocks of elements.
pub trait BlockIterator: ExactSizeIterator<Item = Block> + Sized {
    /// Creates an iterator over elements in the blocks.
    fn into_id_iter(mut self) -> IdIter<Self> {
        let word = self.next().unwrap_or(0);
        IdIter {
            blocks: self,
            word,
            idx: 0,
        }
    }
}

impl<I> BlockIterator for I where I: ExactSizeIterator<Item = Block> {}

#[derive(Clone, Debug)]
/// An iterator over the blocks of the underlying representation.
pub struct Blocks<'a> {
    inner: slice::Iter<'a, Block>,
}

impl<'a> Iterator for Blocks<'a> {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|&block| block)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a> ExactSizeIterator for Blocks<'a> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

#[derive(Clone, Debug)]
/// A consuming iterator over the blocks of the underlying representation.
pub struct IntoBlocks {
    inner: vec::IntoIter<Block>,
}

impl Iterator for IntoBlocks {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for IntoBlocks {
    fn len(&self) -> usize {
        self.inner.len()
    }
}