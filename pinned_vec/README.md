# PinnedVec

[<img alt="crates.io" src="https://img.shields.io/crates/v/pinned_vec?style=for-the-badge" height="20">](https://crates.io/crates/pinned_vec)
[<img alt="crates.io" src="https://img.shields.io/docsrs/pinned_vec?style=for-the-badge" height="20">](https://docs.rs/pinned_vec)

Vec-like structure whose elements never move.

Normal Vec holds all its content in one contigious region, and moves when it needs to expand.
PinnedVec holds several smaller sub-vector, each of which never moves.
The first sub-vector is of capacity 1, the second 2, the third 4, the nth 2^(n-2).

## Example Usage
```rust
use pinned_vec::PinnedVec;
use std::pin::Pin;
let mut v = PinnedVec::new();
v.push(5);
{
	let r: Pin<&i32> = v.get(0).unwrap();
	assert_eq!(*r, 5);
}
{
	let r: Pin<&mut i32> = v.get_mut(0).unwrap();
	assert_eq!(*r, 5);
}
assert_eq!(v.len(), 1);
v.pop();
v.push(7);
v.push(8);
v.replace(0, 6);
assert_eq!(*v.get(0).unwrap(), 6);
assert_eq!(*v.get(1).unwrap(), 8);
```