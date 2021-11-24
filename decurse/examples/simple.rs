use decurse::decurse;

#[decurse]
fn factorial(x: u32) -> u32 {
	if x == 0 {
		1
	} else {
		x * factorial(x - 1)
	}
}

#[decurse]
fn fibonacci(x: u32) -> u32 {
	if x == 0 || x == 1 {
		1
	} else {
		fibonacci(x - 1) + fibonacci(x - 2)
	}
}

#[decurse]
fn triangular(x: u64) -> u64 {
	if x == 0 {
		0
	} else {
		triangular(x - 1) + x
	}
}

fn main() {
	assert_eq!(factorial(6), 720);
	assert_eq!(fibonacci(10), 89);
	assert_eq!(triangular(200000), 20000100000);
}
