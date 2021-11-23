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
    assert_eq!(factorial(6), 720);
}

#[test]
fn test_fibonacci() {
    #[decurse]
    fn fibonacci(x: u32) -> u32 {
        if x == 0 || x == 1 {
            1
        } else {
            fibonacci(x - 1) + fibonacci(x - 2)
        }
    }
    assert_eq!(fibonacci(10), 89);
}
