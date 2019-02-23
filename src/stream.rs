use futures::stream::Stream;
use futures::future::Future;
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
    where St: Stream,
          F: FnMut(St::Item) -> U,
{
    let stream = Box::pin(stream);
    futures::stream::unfold((stream, f), async move | (mut stream, mut f)| {
        let item = await!(next(&mut stream));
        item.map(|item| (f(item), (stream, f)))
    })
}

pub fn filter<St, Fut, F>(stream: St, f: F) -> impl Stream<Item = St::Item>
    where St: Stream,
          F: FnMut(&St::Item) -> Fut,
          Fut: Future<Output = bool>
{
    let stream = Box::pin(stream);
    futures::stream::unfold((stream, f), async move | (mut stream, mut f)| {
        while let Some(item) = await!(next(&mut stream)) {
            let matched = await!(f(&item));
            if matched {
                return Some((item, (stream, f)))
            } else {
                continue;
            }
        };
        None
    })
}

pub fn filter_map<St, Fut, F, U>(stream: St, f: F) -> impl Stream<Item = U>
    where St: Stream,
          F: FnMut(St::Item) -> Fut,
          Fut: Future<Output = Option<U>>
{
    let stream = Box::pin(stream);
    futures::stream::unfold((stream, f), async move | (mut stream, mut f)| {
        while let Some(item) = await!(next(&mut stream)) {
            if let Some(item) = await!(f(item)) {
                return Some((item, (stream, f)))
            } else {
                continue;
            }
        };
        None
    })
}

#[cfg(test)]
mod tests {
    use futures::{executor, stream};
    use crate::stream::*;
    use crate::future::ready;

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

    #[test]
    fn test_filter() {
        let stream = stream::iter(1..=10);
        let evens = filter(stream, |x| ready(x % 2 == 0));

        assert_eq!(vec![2, 4, 6, 8, 10], executor::block_on(collect::<_, Vec<_>>(evens)));
    }

    #[test]
    fn test_filter_map() {
        let stream = stream::iter(1..=10);
        let evens = filter_map(stream, |x| {
            let ret = if x % 2 == 0 { Some(x + 1) } else { None };
            ready(ret)
        });

        assert_eq!(vec![3, 5, 7, 9, 11], executor::block_on(collect::<_, Vec<_>>(evens)));
    }
}
