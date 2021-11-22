use std::pin::Pin;

struct Block<T> {
    vec: Vec<T>,
}

impl<T> Block<T> {
    fn new(capacity: usize) -> Self {
        Self {
            vec: Vec::with_capacity(capacity),
        }
    }
    #[allow(dead_code)]
    fn get(&self, index: usize) -> Option<&T> {
        self.vec.get(index)
    }
    fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.vec.get_mut(index)
    }
    fn push(&mut self, item: T) {
        assert!(self.vec.len() < self.vec.capacity());
        self.vec.push(item);
    }
    fn pop(&mut self) {
        self.vec.truncate(self.vec.len() - 1);
    }
}

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
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            len: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    #[allow(dead_code)]
    pub fn get(&self, index: usize) -> Option<Pin<&T>> {
        if index >= self.len {
            None
        } else {
            let (outter, inner) = Self::split_idx(index);
            let block = self.blocks.get(outter).unwrap();
            let item = block.get(inner).unwrap();
            Some(unsafe { Pin::new_unchecked(item) })
        }
    }
    pub fn get_mut(&mut self, index: usize) -> Option<Pin<&mut T>> {
        if index >= self.len {
            None
        } else {
            let (outter, inner) = Self::split_idx(index);
            let block = self.blocks.get_mut(outter).unwrap();
            let item = block.get_mut(inner).unwrap();
            Some(unsafe { Pin::new_unchecked(item) })
        }
    }
    pub fn push(&mut self, item: T) {
        let outter_idx = Self::outter_idx(self.len);
        if self.blocks.len() <= outter_idx {
            let new_block = Block::new(1 << outter_idx);
            self.blocks.push(new_block);
        }
        self.blocks[outter_idx].push(item);
        self.len += 1;
    }
    pub fn pop(&mut self) {
        if self.len > 0 {
            self.len -= 1;
            let outter_idx = Self::outter_idx(self.len);
            self.blocks[outter_idx].pop();
        }
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
    }

    #[test]
    fn basic() {
        let mut a = PinnedVec::new();
        a.push(1);
        a.push(2);
        a.push(3);
        a.push(4);
        assert_eq!(a.len(), 4);
        assert_eq!(*a.get(0).unwrap(), 1);
        assert_eq!(*a.get(1).unwrap(), 2);
        assert_eq!(*a.get(2).unwrap(), 3);
        assert_eq!(*a.get(3).unwrap(), 4);
        assert_eq!(a.get(4), None);
        a.push(5);
        a.push(6);
        a.push(7);
        assert_eq!(a.len(), 7);
        assert_eq!(*a.get(4).unwrap(), 5);
        assert_eq!(*a.get(5).unwrap(), 6);
        assert_eq!(*a.get(6).unwrap(), 7);
        a.pop();
        assert_eq!(a.len(), 6);
        a.pop();
        assert_eq!(a.len(), 5);
        a.pop();
        assert_eq!(a.len(), 4);
        assert_eq!(a.get(4), None);
        assert_eq!(*a.get(0).unwrap(), 1);
        assert_eq!(*a.get(1).unwrap(), 2);
        assert_eq!(*a.get(2).unwrap(), 3);
        assert_eq!(*a.get(3).unwrap(), 4);
    }

    #[test]
    fn one() {
        let mut b: Both<i32> = Both::new();
        for i in 0..100 {
            for j in 0..i {
                b.push(j);
            }
            b.check();
            for _ in 0..(i - 2) {
                b.pop();
            }
            b.check();
        }
    }
}
