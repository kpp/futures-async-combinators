use futures::executor;
use futures_async_combinators::future::*;

fn main() {
    executor::block_on(async {
        let future = ready(Ok::<i32, i32>(1));
        let future = future.and_then(|x| ready(Ok::<i32, i32>(x + 3)));
        let future = future.inspect(|x| {
            dbg!(x);
        });
        assert_eq!(future.await, Ok(4));
    });
}
