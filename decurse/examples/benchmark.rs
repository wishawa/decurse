use owning_ref::VecRef;
use std::time::{Duration, Instant};

fn slow(v: u32) -> u32 {
	let mut k = v;
	let mut i = 0;
	loop {
		i += 1;
		if k == 1 {
			break i;
		} else if k % 2 == 0 {
			k /= 2;
		} else {
			k = k * 3 + 1
		}
	}
}

#[decurse::decurse_unsound]
fn linear_search<T>(slice: VecRef<T, [T]>, find: T) -> usize
where
	T: Ord + 'static,
{
	assert!(slow(8723) < 10000);
	if let Some(first) = slice.first() {
		match find.cmp(first) {
			std::cmp::Ordering::Greater => linear_search(slice.map(|s| &s[1..]), find) + 1,
			_ => 0,
		}
	} else {
		0
	}
}

fn stack_linear_search<T>(slice: VecRef<T, [T]>, find: T) -> usize
where
	T: Ord + 'static,
{
	assert!(slow(8723) < 10000);
	if let Some(first) = slice.first() {
		match find.cmp(first) {
			std::cmp::Ordering::Greater => stack_linear_search(slice.map(|s| &s[1..]), find) + 1,
			_ => 0,
		}
	} else {
		0
	}
}

fn run(h: i32) -> Duration {
	let vecs: Vec<_> = (0..1000)
		.map(|_| {
			let arr: Vec<i32> = (0..h).map(|x| x * 2).collect();
			arr
		})
		.collect();
	
	let start = Instant::now();
	for vec in vecs.into_iter() {
		let or = VecRef::new(vec);
		assert_eq!(linear_search(or, h * 8 / 5), (h * 4 / 5) as usize);
	}
	start.elapsed()
}

fn run_stack(h: i32) -> Duration {
	let vecs: Vec<_> = (0..1000)
		.map(|_| {
			let arr: Vec<i32> = (0..h).map(|x| x * 2).collect();
			arr
		})
		.collect();

	let start = Instant::now();
	for vec in vecs.into_iter() {
		let or = VecRef::new(vec);
		assert_eq!(stack_linear_search(or, h * 8 / 5), (h * 4 / 5) as usize);
	}
	start.elapsed()
}

fn main() {
	for i in 1..10 {
		if i <= 4 {
			// Adjust this number
			let d = run(i * 20000).as_secs_f64();
			let s = run_stack(i * 20000).as_secs_f64();

			println!("{}, {:.2}, {:.2}, {:.2}", i * 20000, d, s, d / s);
		} else {
			let d = run(i * 20000).as_secs_f64();
			println!("{}, {:.2}, Stack Overflow, N/A", i * 20000, d);
		}
	}
}
