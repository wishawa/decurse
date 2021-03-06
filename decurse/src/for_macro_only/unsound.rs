pub use super::pend_once::PendOnce;
pub use decurse_macro::decurse_unsound;
use pfn::PFnOnce;
use pinned_vec::PinnedVec;
use scoped_tls::scoped_thread_local;
use std::{cell::RefCell, future::Future, task::Poll};

pub struct Context<F: Future> {
	next: RefCell<Option<F>>,
	result: RefCell<Option<F::Output>>,
}

impl<F: Future> Context<F> {
	pub fn new() -> Self {
		Self {
			next: RefCell::new(None),
			result: RefCell::new(None),
		}
	}
	fn to_untyped(&self) -> *const () {
		self as *const Self as *const ()
	}
	pub unsafe fn set_next(self_ptr: *const (), fut: F) {
		let this: &Self = &*(self_ptr as *const Self);
		*this.next.borrow_mut() = Some(fut);
	}
	pub unsafe fn get_result(self_ptr: *const ()) -> F::Output {
		let this: &Self = &*(self_ptr as *const Self);
		this.result.borrow_mut().take().unwrap()
	}
}

scoped_thread_local! (static CONTEXT: *const ());

pub unsafe fn set_next<F: Future>(fut: F) {
	CONTEXT.with(|c| unsafe { Context::set_next(*c, fut) })
}

pub unsafe fn get_result<A, R, F>(_phantom: R) -> F::Output
where
	R: PFnOnce<A, PFnOutput = F>,
	F: Future,
{
	CONTEXT.with(|c| unsafe { Context::<F>::get_result(*c) })
}

pub fn execute<F>(fut: F) -> F::Output
where
	F: Future,
{
	let dummy_waker = waker_fn::waker_fn(|| {});
	let mut dummy_async_cx: std::task::Context = std::task::Context::from_waker(&dummy_waker);
	let ctx: Context<F> = Context::new();

	let output = CONTEXT.set(&ctx.to_untyped(), || {
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
macro_rules! for_macro_only_recurse_unsound {
    ($func:path, ($($args:expr),*)) => {
        ({
            unsafe { $crate::for_macro_only::unsound::set_next($func ($($args),*)) };
            $crate::for_macro_only::unsound::PendOnce::new().await;
            unsafe { $crate::for_macro_only::unsound::get_result($func) }
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
				for_macro_only_recurse_unsound!(factorial, (x - 1)) * x
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
				for_macro_only_recurse_unsound!(fibonacci, (x - 1))
					+ for_macro_only_recurse_unsound!(fibonacci, (x - 2))
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
					for_macro_only_recurse_unsound!(decurse_triangular, (x - 1)) + x
				}
			}
			execute(decurse_triangular(x))
		}
		assert_eq!(20000100000, triangular(200000));
	}
}
