use pinned_vec::PinnedVec;
use std::pin::Pin;

#[test]
fn test() {
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
}
