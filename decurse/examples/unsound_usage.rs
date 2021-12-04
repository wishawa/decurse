#[decurse::decurse_unsound]
fn boom(slice: &[i32], first: bool) -> &[i32] {
	if first {
		let sl = {
			let v: Vec<i32> = (1..=10).collect();
			boom(&v, false)
		};
		println!("{:?}", sl);
		&[1, 2, 3]
	} else {
		slice
	}
}

fn main() {
	boom(&[42, 43, 44, 45, 46], true);
}
