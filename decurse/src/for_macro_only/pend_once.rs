use std::task::Poll;

use std::future::Future;

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
