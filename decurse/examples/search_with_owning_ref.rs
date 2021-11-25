use owning_ref::VecRef;

// Binary search is efficient enough that even without decurse,
// it is almost impossible to overflow the stack.
#[decurse::decurse]
fn binary_search<T>(slice: VecRef<T, [T]>, find: T) -> usize
where
	T: Ord + 'static,
{
	let len = slice.len();
	if len == 0 {
		0
	} else {
		let midpoint = len / 2;
		match find.cmp(&slice[midpoint]) {
			std::cmp::Ordering::Less => binary_search(slice.map(|s| &s[..midpoint]), find),
			std::cmp::Ordering::Equal => midpoint,
			std::cmp::Ordering::Greater => {
				binary_search(slice.map(|s| &s[(midpoint + 1)..]), find) + (midpoint + 1)
			}
		}
	}
}

// ↓↓ Try removing this, you will get stack overflow.
#[decurse::decurse]
fn linear_search<T>(slice: VecRef<T, [T]>, find: T) -> usize
where
	T: Ord + 'static,
{
	if let Some(first) = slice.first() {
		match find.cmp(first) {
			std::cmp::Ordering::Greater => linear_search(slice.map(|s| &s[1..]), find) + 1,
			_ => 0,
		}
	} else {
		0
	}
}

fn main() {
	{
		let arr: Vec<i32> = (0..2000000).map(|x| x * 2).collect();
		let or = VecRef::new(arr);
		assert_eq!(binary_search(or, 1333333), 666667);
	}
	{
		let arr: Vec<i32> = (0..2000000).map(|x| x * 2).collect();
		let or = VecRef::new(arr);
		assert_eq!(linear_search(or, 1333333), 666667);
	}
}
