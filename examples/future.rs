use futures::executor;
use futures_async_combinators::future::*;

fn main() {
    let future = ready(Ok::<i32, i32>(1));
    let future = and_then(future, |x| ready(Ok::<i32, i32>(x + 3)));
    let future = inspect(future, |x| {
        dbg!(x);
    });

    let res = executor::block_on(future);
    assert_eq!(res, Ok(4));
}
