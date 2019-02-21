#![feature(futures_api, async_await, await_macro)]

use futures_async_combinators::future::*;
use futures::{future, executor};

fn main() {
    executor::block_on(async {
        let future = future::ready(Ok::<i32, i32>(1));
        let future = and_then(future, |x| future::ready(Ok::<i32, i32>(x + 3)));
        let future = inspect(future, |x| { dbg!(x); });
        assert_eq!(await!(future), Ok(4));
    });
}
