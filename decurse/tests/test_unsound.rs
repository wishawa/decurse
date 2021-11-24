use std::{cell::RefCell, rc::Rc};

use decurse::unsound::decurse_unsound;

#[test]
fn test_factorial() {
    #[decurse_unsound]
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
    #[decurse_unsound]
    fn fibonacci(x: u32) -> u32 {
        if x == 0 || x == 1 {
            1
        } else {
            fibonacci(x - 1) + fibonacci(x - 2)
        }
    }
    assert_eq!(fibonacci(10), 89);
}

#[test]
fn test_no_return() {
    #[decurse_unsound]
    fn no_return(modify_me: Rc<RefCell<String>>, iter: u32) {
        if iter == 0 {
        } else {
            modify_me.borrow_mut().push_str(&format!("{}, ", iter));
            no_return(modify_me, iter - 1);
        }
    }
    let cell = RefCell::new(String::new());
    let rc = Rc::new(cell);
    no_return(rc.clone(), 5);
    assert_eq!(*(rc.borrow()), "5, 4, 3, 2, 1, ");
}

#[test]
fn test_no_arg() {
    thread_local! {
        static CHANGE_ME: RefCell<usize> = RefCell::new(0);
    };

    #[decurse_unsound]
    fn no_arg() {
        if CHANGE_ME.with(|f: &RefCell<usize>| {
            let mut bm = f.borrow_mut();
            if *bm == 5 {
                false
            } else {
                *bm += 1;
                true
            }
        }) {
            no_arg();
        }
    }

    CHANGE_ME.with(|f| {
        *f.borrow_mut() = 0;
    });
    no_arg();
    CHANGE_ME.with(|f| {
        assert_eq!(*f.borrow(), 5);
    });
}

#[test]
fn test_borrow() {
    #[decurse_unsound]
    fn binary_search(s: &[i32], f: i32) -> usize {
        let len = s.len();
        if len == 0 {
            0
        } else {
            let midpoint = len / 2;
            let mid = s[midpoint];
            match f.cmp(&mid) {
                std::cmp::Ordering::Less => binary_search(&s[..midpoint], f),
                std::cmp::Ordering::Equal => midpoint,
                std::cmp::Ordering::Greater => {
                    binary_search(&s[(midpoint + 1)..], f) + (midpoint + 1)
                }
            }
        }
    }

    let arr: Vec<i32> = (0..2000).map(|x| x * 2).collect();
    assert_eq!(binary_search(&arr, 1333), 667);
}

#[test]
fn test_borrow_current() {
    #[decurse_unsound]
    fn borrow_current(a: &str) {
        let mut idxs = a.char_indices();
        if let Some((loc, _ch)) = idxs.nth(1) {
            let s = String::from(&a[loc..]);
            borrow_current(&s);
        }
    }
    borrow_current("asdf hello world lkjh qwer");
}

// Macro error
// fn macro_error() {
// 	#[decurse_unsound]
// 	fn clos() {
// 		|| {
// 			// Should error: "Decurse: recursive call inside closure not supported."
// 			clos();
// 		};
// 	}
// 	#[decurse_unsound]
// 	fn asy() {
// 		async {
// 			// Should error: "Decurse: recursive call inside async block not supported."
// 			asy();
// 		};
// 	}
// }
