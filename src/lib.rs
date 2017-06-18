#[cfg(test)] mod tests;

type Block = u32;
const BITS: usize = 32;

fn div_ceil(div: usize, rem: usize) -> usize {
    if rem == 0 { div } else { div + 1 }
}

pub type Id = usize;

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
            storage: Vec::with_capacity(div_ceil(nbits / BITS, nbits % BITS)),
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn insert(&mut self, id: Id) -> bool {
        let (word, bit) = (id / BITS, id % BITS);
        let mask = 1 << bit;

        let len = div_ceil(word, bit);
        if self.storage.len() < len {
            self.storage.resize(len, 0);
            self.storage[word] = mask;
            false
        } else {
            if (self.storage[word] & mask) == 0 {
                self.storage[word] |= mask;
                self.len += 1;
                true
            } else {
                false
            }
        }
    }

    pub fn remove(&mut self, id: Id) -> bool {
        let (word, bit) = (id / BITS, id % BITS);
        let mask = 1 << bit;

        if self.storage.len() < div_ceil(word, bit) {
            false
        } else {
            if (self.storage[word] & mask) != 0 {
                self.storage[word] &= !mask;
                self.len -= 1;
                true
            } else {
                false
            }
        }
    }

    pub fn contains(&mut self, id: Id) -> bool {
        let (word, bit) = (id / BITS, id % BITS);
        let mask = 1 << bit;

        if self.storage.len() < div_ceil(word, bit) {
            false
        } else {
            (self.storage[word] & mask) != 0
        }
    }
}