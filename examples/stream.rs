#![type_length_limit="2111998"]

use futures::executor;
use futures_async_combinators::stream::*;

fn main() {
    let stream = iter(1..=3);
    let stream = map(stream, |x| x + 1);
    let stream = map(stream, |x| x * 2);

    let collect_future = collect(stream);
    let collection: Vec<_> = executor::block_on(collect_future);

    assert_eq!(vec![4, 6, 8], collection);
}
