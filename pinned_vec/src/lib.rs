//! # PinnedVec
//! Vec-like structure whose elements never move.
//!
//! Normal Vec holds all its content in one contigious region, and moves when it needs to expand.
//! PinnedVec holds several smaller sub-vector, each of which never moves.
//! The first sub-vector is of capacity 1, the second 2, the third 4, the nth 2^(n-2).
//!
//! ## Example Usage
//! ```rust
//! use pinned_vec::PinnedVec;
//! use std::pin::Pin;
//! let mut v = PinnedVec::new();
//! v.push(5);
//! {
//! 	let r: Pin<&i32> = v.get(0).unwrap();
//! 	assert_eq!(*r, 5);
//! }
//! {
//! 	let r: Pin<&mut i32> = v.get_mut(0).unwrap();
//! 	assert_eq!(*r, 5);
//! }
//! assert_eq!(v.len(), 1);
//! v.pop();
//! v.push(7);
//! v.push(8);
//! v.replace(0, 6);
//! assert_eq!(*v.get(0).unwrap(), 6);
//! assert_eq!(*v.get(1).unwrap(), 8);
//! ```

use std::pin::Pin;

// A block never grows, so it never moves, so Pinning is safe.
struct Block<T> {
    vec: Vec<T>,
}

impl<T> Block<T> {
    fn new(capacity: usize) -> Self {
        Self {
            vec: Vec::with_capacity(capacity),
        }
    }
    fn get(&self, index: usize) -> Option<Pin<&T>> {
        // SAFETY: Since the sub-vector's allocation never move, all its contents are pinned.
        self.vec
            .get(index)
            .map(|p| unsafe { Pin::new_unchecked(p) })
    }
    fn get_mut(&mut self, index: usize) -> Option<Pin<&mut T>> {
        // SAFETY: Since the sub-vector's allocation never move, all its contents are pinned.
        self.vec
            .get_mut(index)
            .map(|p| unsafe { Pin::new_unchecked(p) })
    }
    fn push(&mut self, item: T) {
        assert!(self.vec.len() < self.vec.capacity());
        self.vec.push(item);
    }
    fn pop(&mut self) {
        self.vec.truncate(self.vec.len() - 1);
    }
    fn replace(&mut self, index: usize, item: T) {
        *self.vec.get_mut(index).unwrap() = item;
    }
}

/// Vec-like structure whose elements never move.
pub struct PinnedVec<T> {
    blocks: Vec<Block<T>>,
    len: usize,
}

impl<T> PinnedVec<T> {
    fn outter_idx(index: usize) -> usize {
        (usize::BITS - (index + 1).leading_zeros() - 1) as usize
    }
    fn split_idx(index: usize) -> (usize, usize) {
        let m = index + 1;
        let outter_idx = (usize::BITS - m.leading_zeros() - 1) as usize;
        let inner_idx: usize = m & (!(1 << outter_idx));
        (outter_idx, inner_idx)
    }
    /// Create a new, empty PinnedVec.
    /// This method does not allocate.
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            len: 0,
        }
    }
    /// Get the length of the PinnedVec (the number of elements inside).
    pub fn len(&self) -> usize {
        self.len
    }
    /// Get the current capacity of the PinnedVec
    /// Pushing within capacity means no extra allocation.
    /// Pushing over capacity will cause allocation, increasing capacity.
    pub fn capacity(&self) -> usize {
        let outter_idx = Self::outter_idx(self.len);
        (1 << outter_idx) - 1
    }
    /// Get a pinned reference to the element at the specified index, if it exists.
    pub fn get(&self, index: usize) -> Option<Pin<&T>> {
        if index >= self.len {
            None
        } else {
            let (outter, inner) = Self::split_idx(index);
            let block = self.blocks.get(outter).unwrap();
            let item = block.get(inner).unwrap();
            Some(item)
        }
    }
    /// Get a pinned mutable reference to the element at the specified index, if it exists.
    pub fn get_mut(&mut self, index: usize) -> Option<Pin<&mut T>> {
        if index >= self.len {
            None
        } else {
            let (outter, inner) = Self::split_idx(index);
            let block = self.blocks.get_mut(outter).unwrap();
            let item = block.get_mut(inner).unwrap();
            Some(item)
        }
    }
    /// Push an element to the end of the PinnedVec.
    /// Might cause the PinnedVec to allocate a new sub-vector.
    pub fn push(&mut self, item: T) {
        let outter_idx = Self::outter_idx(self.len);
        if self.blocks.len() <= outter_idx {
            let new_block = Block::new(1 << outter_idx);
            self.blocks.push(new_block);
        }
        self.blocks[outter_idx].push(item);
        self.len += 1;
    }
    /// Remove the last element in the PinnedVec.
    /// The element is not returned because that would violate Pin invariant.
    /// ### Panics
    /// Panics if the vec is empty.
    pub fn pop(&mut self) {
        assert!(self.len > 0);
        self.len -= 1;
        let outter_idx = Self::outter_idx(self.len);
        self.blocks[outter_idx].pop();
    }
    /// Replace the element at the specified index with another one.
    /// The element is not returned because that would violate Pin invariant.
    /// ### Panics
    /// Panics if index is not in the vec (i.e. len >= index).
    pub fn replace(&mut self, index: usize, item: T) {
        let (outter, inner) = Self::split_idx(index);
        self.blocks[outter].replace(inner, item);
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use super::*;

    struct Both<T> {
        normal: Vec<T>,
        pinned: PinnedVec<T>,
    }

    impl<T> Both<T> {
        fn new() -> Self {
            Self {
                normal: Vec::new(),
                pinned: PinnedVec::new(),
            }
        }
    }

    impl<T: PartialEq + Debug> Both<T> {
        fn check(&self) {
            let len = self.normal.len();
            assert_eq!(len, self.pinned.len());
            for i in 0..len {
                assert_eq!(self.normal.get(i), self.pinned.get(i).as_deref());
            }
            assert_eq!(self.pinned.get(len), None);
        }
    }
    impl<T: Clone> Both<T> {
        fn push(&mut self, item: T) {
            self.normal.push(item.clone());
            self.pinned.push(item);
        }
        fn pop(&mut self) {
            self.normal.pop();
            self.pinned.pop();
        }
        fn replace(&mut self, index: usize, item: T) {
            self.normal[index] = item.clone();
            self.pinned.replace(index, item);
        }
    }

    #[test]
    fn one() {
        let mut b: Both<i32> = Both::new();
        for i in 0..200 {
            for j in 0..i {
                b.push(j);
            }
            b.check();
            for _ in 0..(i - 2) {
                b.pop();
            }
            b.check();
            for j in (0..(i / 5)).map(|x| x * 3) {
                b.replace(j as usize, -j);
            }
            b.check();
        }
    }

    #[test]
    fn two() {
        let mut b: Both<i32> = Both::new();
        b.push(1);
        b.push(2);
        b.push(3);
        b.push(4);
        b.check();
        b.push(5);
        b.push(6);
        b.push(7);
        b.check();
        b.pop();
        b.check();
        b.pop();
        b.check();
        b.pop();
        b.check();
        b.pop();
        b.check();
        b.push(5);
        b.check()
    }
}
