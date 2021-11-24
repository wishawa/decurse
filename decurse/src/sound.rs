pub use super::pend_once::PendOnce;
pub use decurse_macro::decurse_sound;
use pfn::PFnOnce;
use pinned_vec::PinnedVec;
use scoped_tls::scoped_thread_local;
use std::{any::Any, cell::RefCell, future::Future, task::Poll};

pub struct Context<F: Future> {
    next: RefCell<Option<F>>,
    result: RefCell<Option<F::Output>>,
}

impl<F: Future + 'static> Context<F> {
    pub fn new() -> Self {
        Self {
            next: RefCell::new(None),
            result: RefCell::new(None),
        }
    }
    pub fn set_next(self_ptr: &Box<dyn Any>, fut: F) {
        let this: &Self = self_ptr.downcast_ref().unwrap();
        *this.next.borrow_mut() = Some(fut);
    }
    pub fn get_result(self_ptr: &Box<dyn Any>) -> F::Output {
        let this: &Self = self_ptr.downcast_ref().unwrap();
        this.result.borrow_mut().take().unwrap()
    }
}

scoped_thread_local! (static CONTEXT: Box<dyn Any>);

pub fn set_next<F: Future + 'static>(fut: F) {
    CONTEXT.with(|c| Context::set_next(c, fut))
}

pub fn get_result<A, R, F>(_phantom: R) -> F::Output
where
    R: PFnOnce<A, PFnOutput = F>,
    F: Future + 'static,
{
    CONTEXT.with(|c| Context::<F>::get_result(c))
}

pub fn execute<F>(fut: F) -> F::Output
where
    F: Future + 'static,
{
    let dummy_waker = waker_fn::waker_fn(|| {});
    let mut dummy_async_cx: std::task::Context = std::task::Context::from_waker(&dummy_waker);
    let ctx: Context<F> = Context::new();
    let any_ctx: Box<dyn Any> = Box::new(ctx);
    let ctx: &Context<F> = any_ctx.downcast_ref().unwrap();

    let output = CONTEXT.set(&any_ctx, || {
        let mut heap_stack: PinnedVec<F> = PinnedVec::new();
        heap_stack.push(fut);
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
                    // UNWRAP Safety: The decurse macro only yields when recursing,
                    // in which case `next` would be filled before Pending is returned (see ctx.set_next).
                    heap_stack.push(ctx.next.borrow_mut().take().unwrap());
                }
            }
        }
    });
    output
}

#[macro_export]
macro_rules! recurse_sound {
    ($fun:ident($($args:expr),*)) => {
        ({
            $crate::sound::set_next($fun($($args),*));
            $crate::sound::PendOnce::new().await;
            $crate::sound::get_result($fun)
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
        async fn factorial(x: u32) -> u32 {
            if x == 0 {
                1
            } else {
                recurse_sound!(factorial(x - 1)) * x
            }
        }
        assert_eq!(execute(factorial(6)), 720);
    }

    #[test]
    fn fibonacci() {
        async fn fibonacci(x: u32) -> u32 {
            if x == 0 || x == 1 {
                1
            } else {
                recurse_sound!(fibonacci(x - 1)) + recurse_sound!(fibonacci(x - 2))
            }
        }
        assert_eq!(execute(fibonacci(10)), 89);
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
            async fn decurse_triangular(x: u64) -> u64 {
                if x == 0 {
                    0
                } else {
                    recurse_sound!(decurse_triangular(x - 1)) + x
                }
            }
            execute(decurse_triangular(x))
        }
        assert_eq!(20000100000, triangular(200000));
    }
}
