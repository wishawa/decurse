mod pinned_vec;
use std::{cell::RefCell, future::Future, pin::Pin, rc::Rc, task::Poll};

use pinned_vec::PinnedVec;

pub struct Context<F: Future> {
    inner: Rc<RefCell<ContextInner<F>>>,
}

impl<F: Future> Clone for Context<F> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

struct ContextInner<F>
where
    F: Future,
{
    next: Option<F>,
    result: Option<F::Output>,
}

pub struct LinkedFuture<F>
where
    F: Future,
{
    ctx: Context<F>,
}

impl<F: Future> Future for LinkedFuture<F> {
    type Output = F::Output;
    fn poll(self: std::pin::Pin<&mut Self>, _: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        match self.ctx.inner.borrow_mut().result.take() {
            Some(r) => Poll::Ready(r),
            None => Poll::Pending,
        }
    }
}

impl<F: Future> Context<F> {
    fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(ContextInner {
                next: None,
                result: None,
            })),
        }
    }
    pub fn add_next(&self, fut: F) -> LinkedFuture<F> {
        let mut bm = self.inner.borrow_mut();
        bm.next = Some(fut);
        LinkedFuture { ctx: self.clone() }
    }
}

pub fn execute<F, R>(run: R) -> F::Output
where
    F: Future,
    R: FnOnce(Context<F>) -> F,
{
    let waker = waker_fn::waker_fn(|| {});
    let mut cx = std::task::Context::from_waker(&waker);
    let mut heap_stack: PinnedVec<F> = PinnedVec::new();
    let ctx: Context<F> = Context::new();
    heap_stack.push(run(ctx.clone()));
    while heap_stack.len() > 0 {
        let len = heap_stack.len();
        let fut = heap_stack.get_mut(len - 1).unwrap();
        let polled = fut.poll(&mut cx);
        let mut inn = ctx.inner.borrow_mut();
        match polled {
            Poll::Ready(r) => {
                inn.result = Some(r);
                heap_stack.pop();
            }
            Poll::Pending => heap_stack.push(inn.next.take().unwrap()),
        }
    }
    let mut bm = ctx.inner.borrow_mut();
    bm.result.take().unwrap()
}

pub type BoxedFuture<T> = Pin<Box<dyn Future<Output = T>>>;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_basic_factorial() {
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
    fn test_basic_fibonacci() {
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
    fn test_boxed() {
        fn factorial(ctx: Context<BoxedFuture<u32>>, x: u32) -> BoxedFuture<u32> {
            Box::pin(async move {
                if x == 0 {
                    1
                } else {
                    ctx.add_next(factorial(ctx.clone(), x - 1)).await * x
                }
            })
        }
        assert_eq!(execute(|ctx| { factorial(ctx, 6) }), 720);
    }

    #[test]
    fn test_boxed_branching() {
        fn fibonacci(ctx: Context<BoxedFuture<u32>>, x: u32) -> BoxedFuture<u32> {
            Box::pin(async move {
                if x == 0 || x == 1 {
                    1
                } else {
                    ctx.add_next(fibonacci(ctx.clone(), x - 1)).await
                        + ctx.add_next(fibonacci(ctx.clone(), x - 2)).await
                }
            })
        }
        assert_eq!(execute(|ctx| { fibonacci(ctx, 10) }), 89);
    }

    #[test]
    fn test_unboxed() {
        struct FactorialFuture {
            c: Context<FactorialFuture>,
            x: u32,
            f: Option<LinkedFuture<FactorialFuture>>,
        }
        impl Future for FactorialFuture {
            type Output = u32;
            fn poll(
                mut self: Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> Poll<Self::Output> {
                let Self { c, x, f } = &mut *self;
                let x = *x;
                if x == 0 {
                    Poll::Ready(1)
                } else {
                    let f = f.get_or_insert_with(|| {
                        c.add_next(Self {
                            c: c.clone(),
                            x: x - 1,
                            f: None,
                        })
                    });
                    Pin::new(f).poll(cx).map(|r| x * r)
                }
            }
        }

        assert_eq!(
            execute(|ctx| {
                FactorialFuture {
                    c: ctx,
                    x: 6,
                    f: None,
                }
            }),
            720
        );
    }
}
