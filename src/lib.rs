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

pub struct LinkedFuture<O> {
    ctx: Context<O>,
}

impl<O> Future for LinkedFuture<O> {
    type Output = O;
    fn poll(self: std::pin::Pin<&mut Self>, _: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        match self.ctx.result.borrow_mut().take() {
            Some(r) => Poll::Ready(r),
            None => Poll::Pending,
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
    pub fn set_next<F: Future<Output = O> + 'static>(&self, fut: F) -> LinkedFuture<O> {
        let next: &RefCell<Option<F>> = self.next.downcast_ref().unwrap();
        let mut bm = next.borrow_mut();
        *bm = Some(fut);
        LinkedFuture { ctx: self.clone() }
    }
}

pub fn execute<F, R>(run: R) -> F::Output
where
    F: Future + 'static,
    R: FnOnce(Context<F::Output>) -> F,
{
    let waker = waker_fn::waker_fn(|| {});
    let mut cx = std::task::Context::from_waker(&waker);
    let mut heap_stack: PinnedVec<F> = PinnedVec::new();
    let ctx: Context<F::Output> = Context::new::<F>();
    heap_stack.push(run(ctx.clone()));
    loop {
        let len = heap_stack.len();
        if len == 0 {
            break;
        }
        let fut = heap_stack.get_mut(len - 1).unwrap();
        let polled = fut.poll(&mut cx);
        match polled {
            Poll::Ready(r) => {
                let mut bm = ctx.result.borrow_mut();
                *bm = Some(r);
                heap_stack.pop();
            }
            Poll::Pending => {
                let next: &RefCell<Option<F>> = ctx.next.downcast_ref().unwrap();
                heap_stack.push(next.take().unwrap());
            }
        }
    }
    let mut bm = ctx.result.borrow_mut();
    bm.take().unwrap()
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
                let lf = ctx.set_next(factorial(ctx.clone(), x - 1));
                x * lf.await
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
                ({
                    let lf = ctx.set_next(fibonacci(ctx.clone(), x - 1));
                    lf.await
                }) + ({
                    let lf = ctx.set_next(fibonacci(ctx.clone(), x - 2));
                    lf.await
                })
            }
        }
        assert_eq!(execute(|ctx| fibonacci(ctx, 10)), 89);
    }
}
