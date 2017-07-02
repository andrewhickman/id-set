use std::{iter, ops, slice, vec};

use super::{Block, BITS};
use BlockStore::{Stack, Heap};

/// The number of blocks that fit into the 196-bit footprint of a vector.
const SIZE: usize = 196 / BITS;

#[derive(Clone, Debug)]
pub enum BlockStore {
    Stack([Block; SIZE]),
    Heap(Vec<Block>),
}

impl BlockStore {
    pub fn new() -> Self {
        Stack([0; SIZE])
    }

    pub fn with_capacity(cap: usize) -> Self {
        if cap < SIZE {
            Stack([0; SIZE])
        } else {
            Heap(Vec::with_capacity(cap))
        }
    }

    pub fn from_elem(elem: Block, len: usize) -> Self {
        if len < SIZE {
            let mut arr = [0; SIZE];
            for i in 0..len {
                arr[i] = elem;
            }
            Stack(arr)
        } else {
            Heap(vec![elem; len])
        }
    }

    pub fn len(&self) -> usize {
        self.map_vec(Vec::len).unwrap_or(SIZE)
    }

    pub fn clear(&mut self) {
        if let Heap(ref mut vec) = *self {
            vec.clear()
        } else {
            *self = Stack([0; SIZE])
        }
    }

    pub fn capacity(&self) -> usize {
        self.map_vec(Vec::capacity).unwrap_or(SIZE)
    }

    pub fn reserve(&mut self, cap: usize) {
        if cap >= SIZE {
            let vec = match *self {
                Stack(ref arr) => {
                    let mut vec = Vec::with_capacity(cap);
                    vec.extend(arr);
                    vec
                }
                Heap(ref mut vec) => {
                    vec.reserve(cap);
                    return;
                }
            };
            *self = Heap(vec);
        }
    }

    pub fn shrink_to_fit(&mut self) {
        let arr = match *self {
            Stack(_) => return,
            Heap(ref mut vec) => {
                while let Some(&block) = vec.last() {
                    if block == 0 {
                        vec.pop();
                    } else {
                        break;
                    }
                }
                if vec.len() < SIZE {
                    let mut arr = [0; SIZE];
                    for i in 0..vec.len() {
                        arr[i] = vec[i];
                    }
                    arr
                } else {
                    vec.shrink_to_fit();
                    return;
                }
            }
        };
        *self = Stack(arr);
    }

    fn map_vec<T, F: FnOnce(&Vec<Block>) -> T>(&self, f: F) -> Option<T> {
        if let Heap(ref vec) = *self {
            Some(f(vec))
        } else {
            None
        }
    }

    pub fn drain(&mut self, idx: usize) -> Drain {
        match *self {
            Stack(ref mut data) => {
                assert!(idx <= SIZE);
                Drain::Stack {
                    data,
                    idx: idx as u8,
                }
            }
            Heap(ref mut vec) => Drain::Heap(vec.drain(idx..)),
        }
    }

    pub fn resize(&mut self, new_len: usize) {
        let vec = match *self {
            Stack(ref mut arr) => {
                if new_len < SIZE {
                    for i in new_len..SIZE {
                        arr[i] = 0;
                    }
                    return;
                } else {
                    let mut vec = Vec::with_capacity(new_len);
                    vec.extend(&*arr);
                    vec.resize(new_len, 0);
                    vec
                }
            }
            Heap(ref mut vec) => {
                vec.resize(new_len, 0);
                return;
            }
        };
        *self = Heap(vec);
    }

    pub fn iter(&self) -> Iter {
        Iter { inner: ops::Deref::deref(self).iter() }
    }

    pub fn iter_mut(&mut self) -> slice::IterMut<Block> {
        ops::DerefMut::deref_mut(self).iter_mut()
    }

    pub fn into_iter(self) -> IntoIter {
        match self {
            Stack(data) => IntoIter::Stack { data, idx: 0 },
            Heap(vec) => IntoIter::Heap(vec.into_iter()),
        }
    }
}

impl Extend<Block> for BlockStore {
    fn extend<I>(&mut self, iter: I)
        where I: IntoIterator<Item = Block>
    {
        let iter = iter.into_iter();
        let arr = match *self {
            Stack(arr) => arr,
            Heap(ref mut vec) => return vec.extend(iter),
        };
        let mut vec = Vec::with_capacity(SIZE + iter.size_hint().0);
        vec.extend(&arr);
        vec.extend(iter);
        *self = Heap(vec);
    }
}

impl ops::Deref for BlockStore {
    type Target = [Block];

    fn deref(&self) -> &Self::Target {
        match *self {
            Stack(ref arr) => arr,
            Heap(ref vec) => vec,
        }
    }
}

impl ops::DerefMut for BlockStore {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match *self {
            Stack(ref mut arr) => arr,
            Heap(ref mut vec) => vec,
        }
    }
}

impl<'a> iter::FromIterator<&'a Block> for BlockStore {
    fn from_iter<I>(iter: I) -> Self
        where I: IntoIterator<Item = &'a Block>
    {
        BlockStore::from_iter(iter.into_iter().cloned())
    }
}

impl iter::FromIterator<Block> for BlockStore {
    fn from_iter<I>(iter: I) -> Self
        where I: IntoIterator<Item = Block>
    {
        let mut iter = iter.into_iter();
        if iter.size_hint().0 < SIZE {
            let mut arr = [0; SIZE];
            for i in 0..SIZE {
                if let Some(block) = iter.next() {
                    arr[i] = block;
                } else {
                    return Stack(arr);
                }
            }
            let mut vec = Vec::with_capacity(SIZE + iter.size_hint().0);
            vec.extend(&arr);
            vec.extend(iter);
            Heap(vec)
        } else {
            Heap(Vec::from_iter(iter))
        }
    }
}

impl<'a> IntoIterator for &'a BlockStore {
    type Item = Block;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for BlockStore {
    type Item = Block;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

#[derive(Clone, Debug)]
pub enum IntoIter {
    Stack { data: [Block; SIZE], idx: u8 },
    Heap(vec::IntoIter<Block>),
}

impl Iterator for IntoIter {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            IntoIter::Stack {
                ref data,
                ref mut idx,
            } => {
                if *idx as usize == SIZE {
                    None
                } else {
                    let ret = data[*idx as usize];
                    *idx += 1;
                    Some(ret)
                }
            }
            IntoIter::Heap(ref mut vec) => vec.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match *self {
            IntoIter::Stack { idx, .. } => (SIZE - idx as usize, Some(SIZE - idx as usize)),
            IntoIter::Heap(ref vec) => vec.size_hint(),
        }
    }
}

impl ExactSizeIterator for IntoIter {}

#[derive(Debug)]
pub enum Drain<'a> {
    Stack {
        data: &'a mut [Block; SIZE],
        idx: u8,
    },
    Heap(vec::Drain<'a, Block>),
}

impl<'a> Iterator for Drain<'a> {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            Drain::Stack {
                ref mut data,
                ref mut idx,
            } => {
                if *idx as usize == SIZE {
                    None
                } else {
                    let ret = data[*idx as usize];
                    data[*idx as usize] = 0;
                    *idx += 1;
                    Some(ret)
                }
            }
            Drain::Heap(ref mut vec) => vec.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match *self {
            Drain::Stack { idx, .. } => (SIZE - idx as usize, Some(SIZE - idx as usize)),
            Drain::Heap(ref vec) => vec.size_hint(),
        }
    }
}

impl<'a> ExactSizeIterator for Drain<'a> {}

#[derive(Clone, Debug)]
/// An iterator over the blocks of the underlying representation.
pub struct Iter<'a> {
    inner: slice::Iter<'a, Block>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|&block| block)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

#[test]
fn size() {
    use std::{mem, u8};

    assert_eq!(mem::size_of::<[Block; SIZE]>(),
               mem::size_of::<Vec<Block>>());
    assert!(SIZE <= u8::MAX as usize);
}
