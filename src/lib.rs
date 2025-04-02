//! Basically just the flag struct
//! in the `atomic_waker` example.

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::Poll,
};

use atomic_waker::AtomicWaker;

struct FlagInner {
    waker: AtomicWaker,
    set: AtomicBool,
}

#[derive(Clone)]
pub struct Flag(Arc<FlagInner>);

impl Flag {
    pub fn new() -> Self {
        Flag(Arc::new(FlagInner {
            waker: AtomicWaker::new(),
            set: AtomicBool::new(false),
        }))
    }

    pub fn signal(&self) {
        self.0.set.store(true, Ordering::Relaxed);
        self.0.waker.wake();
    }

    pub fn reset(&mut self) -> bool {
        if let Some(inner) = Arc::get_mut(&mut self.0) {
            inner.waker.take();
            inner.set.store(false, Ordering::Relaxed);
            true
        } else {
            false
        }
    }
}

impl Future for Flag {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        // Quick check to avoid registration if already done.
        if self.0.set.load(Ordering::Relaxed) {
            return Poll::Ready(());
        }

        self.0.waker.register(cx.waker());

        // Need to check condition **after** `register` to avoid a race
        // condition that would result in lost notifications.
        if self.0.set.load(Ordering::Relaxed) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
