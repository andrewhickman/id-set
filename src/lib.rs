//! A bitset implementation that stores data on the stack for small sizes (elements less than 196)
//! and keeps track of the element count.
//!
//! # Examples
//!
//! The API is generally similar to the of the bit-set crate.
//!
//! ```
//! use id_set::IdSet;
//!
//! let mut set = IdSet::new();
//! set.insert(42);
//! assert!(set.contains(42));
//! set.remove(42);
//! assert_eq!(set.len(), 0);
//! ```
//!
//! Additionally the `IdIter` struct provides iteration over the bits of any iterator over `Block`
//! values, allowing iteration over unions, intersections, and differences of arbitrarily many sets.
//!
//! ```
//! use id_set::IdSet;
//!
//! let a: IdSet = (0..15).collect();
//! let b: IdSet = (10..20).collect();
//! let c: IdSet = (0..5).collect();
//!
//! let expected: IdSet = (0..5).chain(10..15).collect();
//! let actual: IdSet = a.intersection(&b).union(&c).collect();
//! assert_eq!(actual, expected);
//! ```

#![deny(missing_docs, missing_debug_implementations)]

#[cfg(test)]
mod tests;
mod store;

pub use store::{Iter as Blocks, IntoIter as IntoBlocks};

use std::{cmp, fmt, iter, ops, usize};
use std::iter::FromIterator;

use store::BlockStore;

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

/// A set of `usize` elements represented by a bit vector. Storage required is proportional to the
/// maximum element in the set.
pub struct IdSet {
    blocks: BlockStore,
    len: usize,
}

impl IdSet {
    #[inline]
    /// Creates an empty `IdSet`.
    pub fn new() -> Self {
        IdSet {
            blocks: BlockStore::new(),
            len: 0,
        }
    }

    #[inline]
    /// Creates a `IdSet` filled with all elements from 0 to n.
    pub fn new_filled(n: usize) -> Self {
        let (nwords, nbits) = (n / BITS, n % BITS);
        let blocks: BlockStore = if nbits != 0 {
            iter::repeat(!0)
                .take(nwords)
                .chain(iter::once(mask(nbits) - 1))
                .collect()
        } else {
            iter::repeat(!0).take(nwords).collect()
        };
        IdSet { blocks, len: n }
    }

    #[inline]
    /// Creates a empty `IdSet` that can hold elements up to n before reallocating.
    pub fn with_capacity(n: usize) -> Self {
        IdSet {
            blocks: BlockStore::with_capacity(ceil_div(n, BITS)),
            len: 0,
        }
    }

    #[cfg(test)]
    fn from_bytes(bytes: &[Block]) -> Self {
        let blocks = BlockStore::from_iter(bytes);
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
    /// Resizes the set to minimise allocations.
    pub fn shrink_to_fit(&mut self) {
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
            self.blocks.resize(word + 1);
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
        for word in self.blocks.iter_mut() {
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
            inner: IdIter::new(self.blocks.iter()),
            len: self.len,
        }
    }

    #[inline]
    /// Returns an iterator over the blocks of the underlying representation.
    pub fn blocks(&self) -> Blocks {
        self.blocks.iter()
    }

    #[inline]
    /// Returns a consuming iterator over the blocks of the underlying representation.
    pub fn into_blocks(self) -> IntoBlocks {
        self.blocks.into_iter()
    }

    #[inline]
    /// Takes the union of the set with another. Equivalent to `self | other`.
    pub fn union<I: IntoBlockIter>(&self, other: I) -> BlockIter<Union<Blocks, I::Blocks>> {
        self | other
    }

    #[inline]
    /// Takes the intersection of the set with another. Equivalent to `self & other`.
    pub fn intersection<I: IntoBlockIter>(&self,
                                          other: I)
                                          -> BlockIter<Intersection<Blocks, I::Blocks>> {
        self & other
    }

    #[inline]
    /// Takes the difference of the set with another. Equivalent to `self - other`.
    pub fn difference<I: IntoBlockIter>(&self,
                                        other: I)
                                        -> BlockIter<Difference<Blocks, I::Blocks>> {
        self - other
    }

    #[inline]
    /// Takes the symmetric difference of the set with another. Equivalent to `self ^ other`.
    pub fn symmetric_difference<I: IntoBlockIter>
        (&self,
         other: I)
         -> BlockIter<SymmetricDifference<Blocks, I::Blocks>> {
        self ^ other
    }

    #[inline]
    /// Consumes the set and takes the union with another.
    pub fn into_union<I: IntoBlockIter>(self, other: I) -> BlockIter<Union<IntoBlocks, I::Blocks>> {
        self | other
    }

    #[inline]
    /// Consumes the set and takes the intersection with another.
    pub fn into_intersection<I: IntoBlockIter>
        (self,
         other: I)
         -> BlockIter<Intersection<IntoBlocks, I::Blocks>> {
        self & other
    }

    #[inline]
    /// Consumes the set and takes the difference with another.
    pub fn into_difference<I: IntoBlockIter>(self,
                                             other: I)
                                             -> BlockIter<Difference<IntoBlocks, I::Blocks>> {
        self - other
    }

    #[inline]
    /// Consumes the set and takes the symmetric difference with another.
    pub fn into_symmetric_difference<I: IntoBlockIter>
        (self,
         other: I)
         -> BlockIter<SymmetricDifference<IntoBlocks, I::Blocks>> {
        self ^ other
    }

    #[inline]
    /// Take the union of the set inplace with another set. Equivalent to `*self |= other`.
    pub fn inplace_union(&mut self, other: &Self) {
        *self |= other
    }

    #[inline]
    /// Take the intersection of the set inplace with another set. Equivalent to `*self &= other`.
    pub fn inplace_intersection(&mut self, other: &Self) {
        *self &= other
    }

    #[inline]
    /// Take the difference of the set inplace with another set. Equivalent to `*self -= other`.
    pub fn inplace_difference(&mut self, other: &Self) {
        *self -= other
    }

    #[inline]
    /// Take the symmetric difference of the set inplace with another set. Equivalent to
    /// `*self ^= other`.
    pub fn inplace_symmetric_difference(&mut self, other: &Self) {
        *self ^= other
    }

    #[inline]
    /// Returns true if the sets are disjoint.
    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.len() + other.len() < cmp::max(self.capacity(), other.capacity()) &&
        self.intersection(other).into_iter().count() == 0
    }

    #[inline]
    /// Returns true if self is a superset of other.
    pub fn is_superset(&self, other: &Self) -> bool {
        !other.is_subset(self)
    }

    #[inline]
    /// Returns true if self is a subset of other.
    pub fn is_subset(&self, other: &Self) -> bool {
        self.len() <= other.len() && self.difference(other).into_iter().count() == 0
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

impl IntoIterator for IdSet {
    type Item = Id;
    type IntoIter = IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let len = self.len;
        IntoIter {
            inner: IdIter::new(self.blocks.into_iter()),
            len,
        }
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

impl<B> IdIter<B>
    where B: ExactSizeIterator<Item = Block>
{
    /// Creates a new iterator over elements of a block iterator.
    pub fn new<I: IntoBlockIter<Blocks = B>>(iter: I) -> Self {
        let mut blocks = iter.into_block_iter().into_inner();
        let word = blocks.next().unwrap_or(0);
        IdIter {
            blocks,
            word,
            idx: 0,
        }
    }
}

impl<B> Iterator for IdIter<B>
    where B: ExactSizeIterator<Item = Block>
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
        (ones, Some(self.blocks.len() * BITS + ones))
    }
}

#[derive(Clone, Debug)]
/// Represents a view into the blocks of a set or combination of sets. An iterator over the elements
/// can be obtained with `into_iter()`.
pub struct BlockIter<B> {
    inner: B,
}

impl<B> BlockIter<B> {
    /// Returns the iterator over raw blocks.
    pub fn into_inner(self) -> B {
        self.inner
    }
}

impl<B> BlockIter<B>
    where B: ExactSizeIterator<Item = Block>
{
    /// Equivalent to `self.into_iter().collect()`.
    pub fn collect<T>(self) -> T
        where T: iter::FromIterator<Id>
    {
        self.into_iter().collect()
    }

    /// Takes the union of the blocks with another block iterator. Equivalent to `self | other`.
    pub fn union<I: IntoBlockIter>(self, other: I) -> BlockIter<Union<B, I::Blocks>> {
        self | other
    }

    /// Takes the intersection of the blocks with another block iterator. Equivalent to 
    /// `self & other`.
    pub fn intersection<I: IntoBlockIter>(self, other: I) -> BlockIter<Intersection<B, I::Blocks>> {
        self & other
    }

    /// Takes the difference of the blocks with another block iterator. Equivalent to 
    /// `self - other`.
    pub fn difference<I: IntoBlockIter>(self, other: I) -> BlockIter<Difference<B, I::Blocks>> {
        self - other
    }

    /// Takes the symmetric difference of the blocks with another block iterator. Equivalent to 
    /// `self ^ other`.
    pub fn symmetric_difference<I: IntoBlockIter>
        (self,
         other: I)
         -> BlockIter<SymmetricDifference<B, I::Blocks>> {
        self ^ other
    }
}

impl<B> IntoIterator for BlockIter<B>
    where B: ExactSizeIterator<Item = Block>
{
    type Item = Id;
    type IntoIter = IdIter<B>;

    fn into_iter(self) -> Self::IntoIter {
        IdIter::new(self.inner)
    }
}

/// Conversion into an iterator over blocks.
pub trait IntoBlockIter {
    /// The raw iterator type.
    type Blocks: ExactSizeIterator<Item = Block>;

    /// Creates a block iterator.
    fn into_block_iter(self) -> BlockIter<Self::Blocks>;
}

impl<B> IntoBlockIter for B
    where B: ExactSizeIterator<Item = Block>
{
    type Blocks = B;

    fn into_block_iter(self) -> BlockIter<Self::Blocks> {
        BlockIter { inner: self }
    }
}

impl<B> IntoBlockIter for BlockIter<B>
    where B: ExactSizeIterator<Item = Block>
{
    type Blocks = B;

    #[inline]
    fn into_block_iter(self) -> BlockIter<Self::Blocks> {
        self
    }
}

impl<'a> IntoBlockIter for &'a IdSet {
    type Blocks = Blocks<'a>;

    #[inline]
    fn into_block_iter(self) -> BlockIter<Self::Blocks> {
        self.blocks().into_block_iter()
    }
}

impl IntoBlockIter for IdSet {
    type Blocks = IntoBlocks;

    #[inline]
    fn into_block_iter(self) -> BlockIter<Self::Blocks> {
        self.into_blocks().into_block_iter()
    }
}

impl<B, I> ops::BitAnd<I> for BlockIter<B>
    where B: ExactSizeIterator<Item = Block>,
          I: IntoBlockIter
{
    type Output = BlockIter<Intersection<B, I::Blocks>>;

    #[inline]
    /// Takes the intersection of two objects.
    fn bitand(self, other: I) -> Self::Output {
        BlockIter {
            inner: Intersection {
                left: self.inner,
                right: other.into_block_iter().inner,
            },
        }
    }
}

impl<B, I> ops::BitOr<I> for BlockIter<B>
    where B: ExactSizeIterator<Item = Block>,
          I: IntoBlockIter
{
    type Output = BlockIter<Union<B, I::Blocks>>;

    #[inline]
    /// Takes the union of two objects.
    fn bitor(self, other: I) -> Self::Output {
        BlockIter {
            inner: Union {
                left: self.inner,
                right: other.into_block_iter().inner,
            },
        }
    }
}

impl<B, I> ops::BitXor<I> for BlockIter<B>
    where B: ExactSizeIterator<Item = Block>,
          I: IntoBlockIter
{
    type Output = BlockIter<SymmetricDifference<B, I::Blocks>>;

    #[inline]
    /// Takes the symmetric difference of two objects.
    fn bitxor(self, other: I) -> Self::Output {
        BlockIter {
            inner: SymmetricDifference {
                left: self.inner,
                right: other.into_block_iter().inner,
            },
        }
    }
}

impl<B, I> ops::Sub<I> for BlockIter<B>
    where B: ExactSizeIterator<Item = Block>,
          I: IntoBlockIter
{
    type Output = BlockIter<Difference<B, I::Blocks>>;

    #[inline]
    /// Takes the difference of two objects.
    fn sub(self, other: I) -> Self::Output {
        BlockIter {
            inner: Difference {
                left: self.inner,
                right: other.into_block_iter().inner,
            },
        }
    }
}

impl<'a, I> ops::BitAnd<I> for &'a IdSet
    where I: IntoBlockIter
{
    type Output = BlockIter<Intersection<Blocks<'a>, I::Blocks>>;

    #[inline]
    /// Takes the intersection of two objects.
    fn bitand(self, other: I) -> Self::Output {
        self.into_block_iter() & other
    }
}

impl<'a, I> ops::BitOr<I> for &'a IdSet
    where I: IntoBlockIter
{
    type Output = BlockIter<Union<Blocks<'a>, I::Blocks>>;

    #[inline]
    /// Takes the union of two objects.
    fn bitor(self, other: I) -> Self::Output {
        self.into_block_iter() | other
    }
}

impl<'a, I> ops::BitXor<I> for &'a IdSet
    where I: IntoBlockIter
{
    type Output = BlockIter<SymmetricDifference<Blocks<'a>, I::Blocks>>;

    #[inline]
    /// Takes the symmetric difference of two objects.
    fn bitxor(self, other: I) -> Self::Output {
        self.into_block_iter() ^ other
    }
}

impl<'a, I> ops::Sub<I> for &'a IdSet
    where I: IntoBlockIter
{
    type Output = BlockIter<Difference<Blocks<'a>, I::Blocks>>;

    #[inline]
    /// Takes the difference of two objects.
    fn sub(self, other: I) -> Self::Output {
        self.into_block_iter() - other
    }
}

impl<I> ops::BitAnd<I> for IdSet
    where I: IntoBlockIter
{
    type Output = BlockIter<Intersection<IntoBlocks, I::Blocks>>;

    #[inline]
    /// Takes the intersection of two objects.
    fn bitand(self, other: I) -> Self::Output {
        self.into_block_iter() & other
    }
}

impl<I> ops::BitOr<I> for IdSet
    where I: IntoBlockIter
{
    type Output = BlockIter<Union<IntoBlocks, I::Blocks>>;

    #[inline]
    /// Takes the union of two objects.
    fn bitor(self, other: I) -> Self::Output {
        self.into_block_iter() | other
    }
}

impl<I> ops::BitXor<I> for IdSet
    where I: IntoBlockIter
{
    type Output = BlockIter<SymmetricDifference<IntoBlocks, I::Blocks>>;

    #[inline]
    /// Takes the symmetric difference of two objects.
    fn bitxor(self, other: I) -> Self::Output {
        self.into_block_iter() ^ other
    }
}

impl<I> ops::Sub<I> for IdSet
    where I: IntoBlockIter
{
    type Output = BlockIter<Difference<IntoBlocks, I::Blocks>>;

    #[inline]
    /// Takes the difference of two objects.
    fn sub(self, other: I) -> Self::Output {
        self.into_block_iter() - other
    }
}

impl<I> ops::BitAndAssign<I> for IdSet
    where I: IntoBlockIter
{
    #[inline]
    /// Takes the inplace intersection of the set with another.
    fn bitand_assign(&mut self, other: I) {
        let blocks = other.into_block_iter().into_inner();
        if blocks.len() < self.blocks.len() {
            for block in self.blocks.drain(blocks.len()) {
                self.len -= block.count_ones() as usize;
            }
        }
        for (lblock, rblock) in self.blocks.iter_mut().zip(blocks) {
            self.len -= (*lblock & !rblock).count_ones() as usize;
            *lblock &= rblock;
        }
    }
}

impl<I> ops::BitOrAssign<I> for IdSet
    where I: IntoBlockIter
{
    #[inline]
    /// Takes the inplace union of the set with another.
    fn bitor_assign(&mut self, other: I) {
        let mut blocks = other.into_block_iter().into_inner();
        for lblock in self.blocks.iter_mut() {
            if let Some(rblock) = blocks.next() {
                self.len += (rblock & !*lblock).count_ones() as usize;
                *lblock |= rblock;
            } else {
                return;
            }
        }
        let len = &mut self.len;
        self.blocks
            .extend(blocks.inspect(|block| *len += block.count_ones() as usize));
    }
}

impl<I> ops::BitXorAssign<I> for IdSet
    where I: IntoBlockIter
{
    #[inline]
    /// Takes the inplace symmetric difference of the set with another.
    fn bitxor_assign(&mut self, other: I) {
        let mut blocks = other.into_block_iter().into_inner();
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
        self.blocks
            .extend(blocks.inspect(|block| *len += block.count_ones() as usize));
    }
}

impl<I> ops::SubAssign<I> for IdSet
    where I: IntoBlockIter
{
    #[inline]
    /// Takes the inplace difference of the set with another.
    fn sub_assign(&mut self, other: I) {
        for (lblock, rblock) in self.blocks
                .iter_mut()
                .zip(other.into_block_iter().into_inner()) {
            self.len -= (*lblock & rblock).count_ones() as usize;
            *lblock &= !rblock;
        }
    }
}

#[derive(Clone, Debug)]
/// Takes the intersection of two block iterators.
pub struct Intersection<L, R> {
    left: L,
    right: R,
}

impl<L, R> Iterator for Intersection<L, R>
    where L: ExactSizeIterator<Item = Block>,
          R: ExactSizeIterator<Item = Block>
{
    type Item = Block;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let (Some(l), Some(r)) = (self.left.next(), self.right.next()) {
            Some(l & r)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<L, R> ExactSizeIterator for Intersection<L, R>
    where L: ExactSizeIterator<Item = Block>,
          R: ExactSizeIterator<Item = Block>
{
    #[inline]
    fn len(&self) -> usize {
        cmp::min(self.left.len(), self.right.len())
    }
}

#[derive(Clone, Debug)]
/// Takes the union of two block iterators.
pub struct Union<L, R> {
    left: L,
    right: R,
}

impl<L, R> Iterator for Union<L, R>
    where L: ExactSizeIterator<Item = Block>,
          R: ExactSizeIterator<Item = Block>
{
    type Item = Block;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match (self.left.next(), self.right.next()) {
            (Some(l), Some(r)) => Some(l | r),
            (Some(l), None) => Some(l),
            (None, Some(r)) => Some(r),
            (None, None) => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<L, R> ExactSizeIterator for Union<L, R>
    where L: ExactSizeIterator<Item = Block>,
          R: ExactSizeIterator<Item = Block>
{
    #[inline]
    fn len(&self) -> usize {
        cmp::max(self.left.len(), self.right.len())
    }
}

#[derive(Clone, Debug)]
/// Takes the symmetric difference of two block iterators.
pub struct SymmetricDifference<L, R> {
    left: L,
    right: R,
}

impl<L, R> Iterator for SymmetricDifference<L, R>
    where L: ExactSizeIterator<Item = Block>,
          R: ExactSizeIterator<Item = Block>
{
    type Item = Block;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match (self.left.next(), self.right.next()) {
            (Some(l), Some(r)) => Some(l ^ r),
            (Some(l), None) => Some(l),
            (None, Some(r)) => Some(r),
            (None, None) => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<L, R> ExactSizeIterator for SymmetricDifference<L, R>
    where L: ExactSizeIterator<Item = Block>,
          R: ExactSizeIterator<Item = Block>
{
    #[inline]
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
    where L: ExactSizeIterator<Item = Block>,
          R: ExactSizeIterator<Item = Block>
{
    type Item = Block;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.left
            .next()
            .map(|l| l & !self.right.next().unwrap_or(0))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<L, R> ExactSizeIterator for Difference<L, R>
    where L: ExactSizeIterator<Item = Block>,
          R: ExactSizeIterator<Item = Block>
{
    #[inline]
    fn len(&self) -> usize {
        self.left.len()
    }
}
