#![feature(async_await, await_macro)]

use futures_async_combinators::future::*;
use futures::executor;

fn main() {
    executor::block_on(async {
        let future = ready(Ok::<i32, i32>(1));
        let future = and_then(future, |x| ready(Ok::<i32, i32>(x + 3)));
        let future = inspect(future, |x| { dbg!(x); });
        assert_eq!(future.await, Ok(4));
    });
}
