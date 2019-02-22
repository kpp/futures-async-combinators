use futures::stream::Stream;
use core::pin::Pin;
use pin_utils::pin_mut;

pub async fn next<St>(stream: &mut St) -> Option<St::Item>
    where St: Stream + Unpin,
{
    use futures::future::poll_fn;
    let poll_next = |waker: &_| Pin::new(&mut *stream).poll_next(waker);
    let future_next = poll_fn(poll_next);
    await!(future_next)
}

pub async fn collect<St, C>(stream: St) -> C
    where St: Stream,
          C: Default + Extend<St::Item>
{
    pin_mut!(stream);
    let mut collection = C::default();
    while let Some(item) = await!(next(&mut stream)) {
        collection.extend(Some(item));
    }
    collection
}

pub fn map<St, U, F>(stream: St, f: F) -> impl Stream<Item = U>
    where F: FnMut(St::Item) -> U,
          St: Stream + Unpin,
{
    futures::stream::unfold((stream, f), async move |(mut stream, mut f)| {
        let item = await!(next(&mut stream));
        item.map(|item| (f(item), (stream, f)))
    })
}

#[cfg(test)]
mod tests {
    use futures::{stream, executor};
    use crate::stream::*;

    #[test]
    fn test_next() {
        let mut stream = stream::iter(1..=3);

        assert_eq!(executor::block_on(next(&mut stream)), Some(1));
        assert_eq!(executor::block_on(next(&mut stream)), Some(2));
        assert_eq!(executor::block_on(next(&mut stream)), Some(3));
        assert_eq!(executor::block_on(next(&mut stream)), None);
    }

    #[test]
    fn test_collect() {
        let stream = stream::iter(1..=5);

        let collection : Vec<i32> = executor::block_on(collect(stream));
        assert_eq!(collection, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_map() {
        let stream = stream::iter(1..=3);
        let stream = map(stream, |x| x * 2);

        assert_eq!(vec![2, 4, 6], executor::block_on(collect::<_, Vec<_>>(stream)));
    }
}
