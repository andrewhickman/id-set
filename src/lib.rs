//! A bit-set implementation for use in id-map. Notable differences from bit-set are the IntoIter
//! struct and retain() methods.

#![deny(missing_docs, missing_debug_implementations)]

#[cfg(test)]
mod tests;

use std::{cmp, fmt, slice, usize, vec};
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

    #[cfg(test)]
    fn from_bytes(bytes: &[Block]) -> Self {
        let blocks = Vec::from(bytes);
        let len = bytes
            .iter()
            .map(|&word| word.count_ones() as usize)
            .sum();
        IdSet { blocks, len }
    }

    #[inline]
    /// Returns the number of distinct elements in the set.
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    /// Returns true if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    /// Returns capacity of the set. Inserting any elements less than this will not cause
    /// reallocation.
    pub fn capacity(&self) -> usize {
        self.blocks
            .capacity()
            .checked_mul(BITS)
            .unwrap_or(usize::MAX)
    }

    #[inline]
    /// Resizes the set such that `capacity() >= cap`.
    pub fn reserve(&mut self, cap: usize) {
        self.blocks.reserve(ceil_div(cap, BITS));
    }

    #[inline]
    /// Resizes the set such that `capacity()` is minimal.
    pub fn shrink_to_fit(&mut self) {
        while let Some(&block) = self.blocks.last() {
            if block == 0 {
                break;
            }
            self.blocks.pop();
        }
        self.blocks.shrink_to_fit();
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
    /// Returns a slice of the underlying blocks.
    pub fn as_blocks(&self) -> &[Block] {
        &self.blocks
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
        Blocks { inner: self.blocks.iter() }
    }

    #[inline]
    /// A consuming iterator over the blocks of the underlying representation.
    pub fn into_blocks(self) -> IntoBlocks {
        IntoBlocks { inner: self.blocks.into_iter() }
    }

    #[inline]
    /// Iterator over the union of two sets.
    /// Equivalent to `self.blocks().union(other.blocks()).into_id_iter()`.
    pub fn union<'a>(&'a self, other: &'a Self) -> IdIter<Union<Blocks<'a>, Blocks<'a>>> {
        self.blocks().union(other.blocks()).into_id_iter()
    }

    #[inline]
    /// Iterator over the intersection of two sets.
    /// Equivalent to `self.blocks().intersection(other.blocks()).into_id_iter()`.
    pub fn intersection<'a>(&'a self,
                            other: &'a Self)
                            -> IdIter<Intersection<Blocks<'a>, Blocks<'a>>> {
        self.blocks()
            .intersection(other.blocks())
            .into_id_iter()
    }

    #[inline]
    /// Iterator over the difference of two sets.
    /// Equivalent to `self.blocks().difference(other.blocks()).into_id_iter()`.
    pub fn difference<'a>(&'a self, other: &'a Self) -> IdIter<Difference<Blocks<'a>, Blocks<'a>>> {
        self.blocks().difference(other.blocks()).into_id_iter()
    }

    #[inline]
    /// Iterator over the symmetric difference of two sets.
    /// Equivalent to `self.blocks().symmetric_difference(other.blocks()).into_id_iter()`.
    pub fn symmetric_difference<'a>(&'a self,
                                    other: &'a Self)
                                    -> IdIter<SymmetricDifference<Blocks<'a>, Blocks<'a>>> {
        self.blocks()
            .symmetric_difference(other.blocks())
            .into_id_iter()
    }

    #[inline]
    /// Iterator over the complement of the set. This iterator will never return None.
    pub fn complement(&self) -> IdIter<Complement<Blocks>> {
        self.blocks().complement().into_id_iter()
    }

    #[inline]
    /// Take the union of the set with another set.
    pub fn union_with(&mut self, other: &Self) {
        let mut blocks = other.blocks();
        for lblock in self.blocks.iter_mut() {
            if let Some(rblock) = blocks.next() {
                self.len += (rblock & !*lblock).count_ones() as usize;
                *lblock |= rblock;
            } else {
                return;
            }
        }
        let len = &mut self.len;
        self.blocks.extend(blocks.inspect(|block| *len += block.count_ones() as usize));
    }

    #[inline]
    /// Take the intersection of the set with another set.
    pub fn intersect_with(&mut self, other: &Self) {
        let blocks = other.blocks();
        if blocks.len() < self.blocks.len() {
            for block in self.blocks.drain(blocks.len()..) {
                self.len -= block.count_ones() as usize;
            }
        }
        for (lblock, rblock) in self.blocks.iter_mut().zip(blocks) {
            self.len -= (*lblock & !rblock).count_ones() as usize;
            *lblock &= rblock;
        }
    }

    #[inline]
    /// Take the difference of the set with another set.
    pub fn difference_with(&mut self, other: &Self) {
        for (lblock, rblock) in self.blocks.iter_mut().zip(other.blocks()) {
            self.len -= (*lblock & rblock).count_ones() as usize;
            *lblock &= !rblock;
        }
    }

    #[inline]
    /// Take the symmetric difference of the set with another set.
    pub fn symmetric_difference_with(&mut self, other: &Self) {
        let mut blocks = other.blocks();
        for lblock in self.blocks.iter_mut() {
            if let Some(rblock) = blocks.next() {
                self.len -= lblock.count_ones() as usize;
                *lblock ^= rblock;
                self.len += lblock.count_ones() as usize;
            } else {
                return;
            }
        }
        let len = &mut self.len;
        self.blocks.extend(blocks.inspect(|block| *len += block.count_ones() as usize));
    }

    #[inline]
    /// Returns true if the sets are disjoint.
    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.len() + other.len() < cmp::max(self.capacity(), other.capacity()) &&
        self.intersection(other).count() == 0
    }

    #[inline]
    /// Returns true if self is a superset of other.
    pub fn is_superset(&self, other: &Self) -> bool {
        !other.is_subset(self)
    }

    #[inline]
    /// Returns true if self is a subset of other.
    pub fn is_subset(&self, other: &Self) -> bool {
        self.len() <= other.len() && self.difference(other).count() == 0
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
        let (mut lhs, mut rhs) = (self.blocks(), other.blocks());
        loop {
            match (lhs.next(), rhs.next()) {
                (Some(l), Some(r)) => {
                    if l != r {
                        return false;
                    }
                }
                (None, None) => return true,
                (Some(l), None) => return l == 0 && lhs.all(|block| block == 0),
                (None, Some(r)) => return r == 0 && rhs.all(|block| block == 0),
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
pub struct IdIter<B> {
    blocks: B,
    word: Block,
    idx: usize,
}

impl<B> Iterator for IdIter<B>
    where B: BlockIterator
{
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
        (ones, self.blocks.size_hint().1.map(|hi| hi * BITS + ones))
    }
}

/// An iterator over blocks of elements.
pub trait BlockIterator: Iterator<Item = Block> + Sized {
    /// Creates an iterator over elements in the blocks.
    fn into_id_iter(mut self) -> IdIter<Self> {
        let word = self.next().unwrap_or(0);
        IdIter {
            blocks: self,
            word,
            idx: 0,
        }
    }

    /// Take the union of two iterators.
    fn union<B: BlockIterator>(self, other: B) -> Union<Self, B> {
        Union {
            left: self,
            right: other,
        }
    }

    /// Take the intersection of two iterators.
    fn intersection<B: BlockIterator>(self, other: B) -> Intersection<Self, B> {
        Intersection {
            left: self,
            right: other,
        }
    }

    /// Take the difference of two iterators.
    fn difference<B: BlockIterator>(self, other: B) -> Difference<Self, B> {
        Difference {
            left: self,
            right: other,
        }
    }

    /// Take the symmetric difference of two iterators.
    fn symmetric_difference<B: BlockIterator>(self, other: B) -> SymmetricDifference<Self, B> {
        SymmetricDifference {
            left: self,
            right: other,
        }
    }

    /// Take the complement of the iterator.
    fn complement(self) -> Complement<Self> {
        Complement { blocks: self }
    }
}

impl<I> BlockIterator for I where I: Iterator<Item = Block> {}

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

#[derive(Clone, Debug)]
/// Takes the union of two block iterators.
pub struct Union<L, R> {
    left: L,
    right: R,
}

impl<L, R> Iterator for Union<L, R>
    where L: BlockIterator,
          R: BlockIterator
{
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.left.next(), self.right.next()) {
            (Some(l), Some(r)) => Some(l | r),
            (Some(l), None) => Some(l),
            (None, Some(r)) => Some(r),
            (None, None) => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (llo, lhi) = self.left.size_hint();
        let (rlo, rhi) = self.right.size_hint();
        let lo = cmp::max(llo, rlo);
        let hi = if let (Some(lhi), Some(rhi)) = (lhi, rhi) {
            Some(cmp::max(lhi, rhi))
        } else {
            None
        };
        (lo, hi)
    }
}

impl<L, R> ExactSizeIterator for Union<L, R>
    where L: BlockIterator + ExactSizeIterator,
          R: BlockIterator + ExactSizeIterator
{
    fn len(&self) -> usize {
        cmp::max(self.left.len(), self.right.len())
    }
}

#[derive(Clone, Debug)]
/// Takes the intersection of two block iterators.
pub struct Intersection<L, R> {
    left: L,
    right: R,
}

impl<L, R> Iterator for Intersection<L, R>
    where L: BlockIterator,
          R: BlockIterator
{
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        if let (Some(l), Some(r)) = (self.left.next(), self.right.next()) {
            Some(l & r)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (llo, lhi) = self.left.size_hint();
        let (rlo, rhi) = self.right.size_hint();
        let lo = cmp::max(llo, rlo);
        let hi = match (lhi, rhi) {
            (Some(lhi), Some(rhi)) => Some(cmp::min(lhi, rhi)),
            (Some(lhi), None) => Some(lhi),
            (None, Some(rhi)) => Some(rhi),
            (None, None) => None,
        };
        (lo, hi)
    }
}

impl<L, R> ExactSizeIterator for Intersection<L, R>
    where L: ExactSizeIterator<Item = Block>,
          R: ExactSizeIterator<Item = Block>
{
    fn len(&self) -> usize {
        cmp::max(self.left.len(), self.right.len())
    }
}

#[derive(Clone, Debug)]
/// Takes the difference of two block iterators.
pub struct Difference<L, R> {
    left: L,
    right: R,
}

impl<L, R> Iterator for Difference<L, R>
    where L: BlockIterator,
          R: BlockIterator
{
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        self.left
            .next()
            .map(|l| l & !self.right.next().unwrap_or(0))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.left.size_hint()
    }
}

impl<L, R> ExactSizeIterator for Difference<L, R>
    where L: ExactSizeIterator<Item = Block>,
          R: ExactSizeIterator<Item = Block>
{
    fn len(&self) -> usize {
        self.left.len()
    }
}

#[derive(Clone, Debug)]
/// Takes the symmetric difference of two block iterators.
pub struct SymmetricDifference<L, R> {
    left: L,
    right: R,
}

impl<L, R> Iterator for SymmetricDifference<L, R>
    where L: BlockIterator,
          R: BlockIterator
{
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.left.next(), self.right.next()) {
            (Some(l), Some(r)) => Some(l ^ r),
            (Some(l), None) => Some(l),
            (None, Some(r)) => Some(r),
            (None, None) => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (llo, lhi) = self.left.size_hint();
        let (rlo, rhi) = self.right.size_hint();
        let lo = cmp::max(llo, rlo);
        let hi = if let (Some(lhi), Some(rhi)) = (lhi, rhi) {
            Some(cmp::max(lhi, rhi))
        } else {
            None
        };
        (lo, hi)
    }
}

impl<L, R> ExactSizeIterator for SymmetricDifference<L, R>
    where L: ExactSizeIterator<Item = Block>,
          R: ExactSizeIterator<Item = Block>
{
    fn len(&self) -> usize {
        cmp::max(self.left.len(), self.right.len())
    }
}

#[derive(Clone, Debug)]
/// Takes the complement of a block iterator. This iterator will never return None.
pub struct Complement<B> {
    blocks: B,
}

impl<B> Iterator for Complement<B>
    where B: BlockIterator
{
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        Some(!self.blocks.next().unwrap_or(0))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}
