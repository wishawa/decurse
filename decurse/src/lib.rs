mod pinned_vec;
mod pend_once;
pub use decurse_macro::decurse;
use pinned_vec::PinnedVec;
use pend_once::PendOnce;
use std::{cell::RefCell, future::Future, rc::Rc, task::Poll};

pub struct Context<'a, O> {
    next: &'a (),
    result: &'a RefCell<Option<O>>,
}

unsafe fn transmute_next_back<'a, F: 'a>(from: &'a ()) -> &'a RefCell<Option<F>> {
    std::mem::transmute(from)
}
fn transmute_next<'a, F: 'a>(from: &'a RefCell<Option<F>>) -> &'a () {
    unsafe { std::mem::transmute(from) }
}


impl<'a, O> Clone for Context<'a, O> {
    fn clone(&self) -> Self {
        Self {
            next: self.next,
            result: self.result,
        }
    }
}

impl<'a, O> Context<'a, O> {
    pub fn set_next<F: Future<Output = O> + 'a>(&self, fut: F) -> pend_once::PendOnce {
        // UNWRAP Safety: The decurse macro allows only one F type, so downcast should always succeed.
        let next: &RefCell<Option<F>> = unsafe {
            transmute_next_back(self.next)
        };
        let mut bm = next.borrow_mut();
        *bm = Some(fut);
        pend_once::PendOnce::new()
    }
    pub fn get_result(&self) -> Option<O> {
        let mut bm = self.result.borrow_mut();
        bm.take()
    }
}

pub fn execute<'a, 'c, F, R>(run: R) -> F::Output
where
    'a: 'c,
    F: Future + 'a,
    R: FnOnce(Context<'c, F::Output>) -> F,
    F::Output: 'static,
{
    let dummy_waker = waker_fn::waker_fn(|| {});
    let mut dummy_async_cx: std::task::Context = std::task::Context::from_waker(&dummy_waker);
    let next_cell = RefCell::new(Option::<F>::None);
    let result_cell = RefCell::new(Option::<F::Output>::None);
    let ctx: Context<'c, F::Output> = unsafe {
        std::mem::transmute(Context {
            next: transmute_next(&next_cell),
            result: &result_cell,
        })
    };
    let mut heap_stack: PinnedVec<F> = PinnedVec::new();
    heap_stack.push(run(ctx.clone()));
    loop {
        let len = heap_stack.len();
        // UNWRAP Safety: The only way len could go down is through the pop in the Poll::Ready case,
        // in which we return if len is 1. So len never gets to 0.
        let fut = heap_stack.get_mut(len - 1).unwrap();
        let polled = fut.poll(&mut dummy_async_cx);
        match polled {
            Poll::Ready(r) => {
                if len == 1 {
                    break r;
                } else {
                    let mut bm = ctx.result.borrow_mut();
                    *bm = Some(r);
                    heap_stack.pop();
                }
            }
            Poll::Pending => {
                // UNWRAP Safety: The decurse macro allows only one F type, so downcast should always succeed.
                let next: &RefCell<Option<F>> = unsafe { transmute_next_back(ctx.next) };
                // UNWRAP Safety: The decurse macro only yields when recursing,
                // in which case `next` would be filled before Pending is returned (see ctx.set_next).
                heap_stack.push(next.take().unwrap());
            }
        }
    }
}

#[macro_export]
macro_rules! recurse {
    ($ctx:ident, $fun:ident($($args:expr),*)) => {
        ({
            let f = $ctx.set_next($fun($ctx.clone(), $($args),*));
            f.await;
            // UNWRAP Safety: In the PendOnce.await above, the executor would execute the recursive call.
            // Only when the result of that is available would the executor re-poll this function.
            $ctx.get_result().unwrap()
        })
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stack_factorial() {
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
    fn stack_fibonacci() {
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
    fn factorial() {
        async fn factorial<'a>(ctx: Context<'a, u32>, x: u32) -> u32 {
            if x == 0 {
                1
            } else {
                recurse!(ctx, factorial(x - 1)) * x
            }
        }
        assert_eq!(execute(|ctx| { factorial(ctx, 6) }), 720);
    }

    #[test]
    fn fibonacci() {
        async fn fibonacci<'a>(ctx: Context<'a, u32>, x: u32) -> u32 {
            if x == 0 || x == 1 {
                1
            } else {
                recurse!(ctx, fibonacci(x - 1)) + recurse!(ctx, fibonacci(x - 2))
            }
        }
        assert_eq!(execute(|ctx| fibonacci(ctx, 10)), 89);
    }

    // This test cause stack overflow.
    // #[test]
    // fn stack_triangular() {
    //     fn stack_triangular(x: u64) -> u64 {
    //         if x == 0 {
    //             0
    //         } else {
    //             stack_triangular(x - 1) + x
    //         }
    //     }
    //     assert_eq!(20000100000, stack_triangular(200000));
    // }

    #[test]
    fn triangular() {
        fn triangular(x: u64) -> u64 {
            async fn decurse_triangular<'a>(ctx: Context<'a, u64>, x: u64) -> u64 {
                if x == 0 {
                    0
                } else {
                    recurse!(ctx, decurse_triangular(x - 1)) + x
                }
            }
            execute(|ctx| decurse_triangular(ctx, x))
        }
        assert_eq!(20000100000, triangular(200000));
    }
}
