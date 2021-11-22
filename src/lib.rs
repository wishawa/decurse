mod pinned_vec;
use std::{any::Any, cell::RefCell, future::Future, rc::Rc, task::Poll};

use pinned_vec::PinnedVec;

pub struct Context<O> {
    next: Rc<dyn Any>,
    result: Rc<RefCell<Option<O>>>,
}

impl<O> Clone for Context<O> {
    fn clone(&self) -> Self {
        Self {
            next: self.next.clone(),
            result: self.result.clone(),
        }
    }
}

pub struct PendOnce {
    pended: bool,
}

impl PendOnce {
    pub fn new() -> Self {
        Self { pended: false }
    }
}

impl Future for PendOnce {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        if self.pended {
            Poll::Ready(())
        } else {
            self.pended = true;
            Poll::Pending
        }
    }
}

impl<O> Context<O> {
    fn new<F: Future<Output = O> + 'static>() -> Self {
        Self {
            next: Rc::new(RefCell::new(Option::<F>::None)),
            result: Rc::new(RefCell::new(None)),
        }
    }
    pub fn set_next<F: Future<Output = O> + 'static>(&self, fut: F) {
        // UNWRAP: The decurse macro allows only one F type, so downcast should always succeed.
        let next: &RefCell<Option<F>> = self.next.downcast_ref().unwrap();
        let mut bm = next.borrow_mut();
        *bm = Some(fut);
    }
}

pub fn execute<F, R>(run: R) -> F::Output
where
    F: Future + 'static,
    R: FnOnce(Context<F::Output>) -> F,
{
    let dummy_waker = waker_fn::waker_fn(|| {});
    let mut dummy_async_cx: std::task::Context = std::task::Context::from_waker(&dummy_waker);
    let ctx: Context<F::Output> = Context::new::<F>();
    let mut heap_stack: PinnedVec<F> = PinnedVec::new();
    heap_stack.push(run(ctx.clone()));
    loop {
        let len = heap_stack.len();
        // UNWRAP: The only way len could go down is through the pop in the Poll::Ready case,
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
                // UNWRAP: The decurse macro allows only one F type, so downcast should always succeed.
                let next: &RefCell<Option<F>> = ctx.next.downcast_ref().unwrap();
                // UNWRAP: The decurse macro only yields when recursing,
                // in which case `next` would be filled before Pending is returned (see ctx.set_next).
                heap_stack.push(next.take().unwrap());
            }
        }
    }
}

#[macro_export]
macro_rules! recurse {
    ($ctx:ident, $fun:ident($($args:expr),*)) => {
        {
            $ctx.set_next($fun($ctx.clone(), $($args),*));
            PendOnce::new().await;
            let res = {
                $ctx.result.borrow_mut().take().unwrap()
            };
            res
        }
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
        async fn factorial(ctx: Context<u32>, x: u32) -> u32 {
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
        async fn fibonacci(ctx: Context<u32>, x: u32) -> u32 {
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
            async fn decurse_triangular(ctx: Context<u64>, x: u64) -> u64 {
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
