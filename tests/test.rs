use decurse_macro::decurse;

#[test]
fn test_factorial() {
	#[decurse]
	fn factorial(x: u32) -> u32 {
		if x == 0 {
			1
		} else {
			x * factorial(x - 1)
		}
	}
	assert_eq!(::decurse::execute(|ctx| { factorial(ctx, 6) }), 720);
}
