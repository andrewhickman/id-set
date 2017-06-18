#[cfg(test)] mod tests;

type Block = u32;
const BITS: usize = 32;

pub struct IdSet {
    storage: Vec<Block>,
    len: usize,
}