use std::future::Future;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};

struct Noop;

impl Wake for Noop {
    fn wake(self: Arc<Self>) {}
}

fn twice(value: i32) -> i32 {
    value * 2
}

async fn value() -> i32 {
    twice(async { 7 }.await)
}

fn main() {
    let waker = Waker::from(Arc::new(Noop));
    let mut context = Context::from_waker(&waker);
    let mut future = std::pin::pin!(value());
    match future.as_mut().poll(&mut context) {
        Poll::Ready(answer) => println!("{}", answer),
        Poll::Pending => panic!("unexpected suspension"),
    }
}
