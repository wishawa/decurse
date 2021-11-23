use std::{cell::RefCell, rc::Rc};

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

#[test]
fn test_no_return() {
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

    #[decurse]
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
