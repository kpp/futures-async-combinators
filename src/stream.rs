use futures::stream::Stream;
use futures::future::Future;

use core::pin::Pin;
use core::iter::IntoIterator;

use pin_utils::pin_mut;

pub async fn next<St>(stream: &mut St) -> Option<St::Item>
    where St: Stream + Unpin,
{
    use crate::future::poll_fn;
    let future_next = poll_fn(|waker| Pin::new(&mut *stream).poll_next(waker));
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

pub async fn into_future<St>(stream: St) -> (Option<St::Item>, impl Stream<Item = St::Item>)
    where St: Stream + Unpin,
{
    let mut stream = stream;
    let next_item = await!(next(&mut stream));
    (next_item, stream)
}

pub fn iter<I>(i: I) -> impl Stream<Item = I::Item>
    where I: IntoIterator,
{
    use core::task::Poll;
    let mut iter = i.into_iter();
    futures::stream::poll_fn(move |_| -> Poll<Option<I::Item>> {
        Poll::Ready(iter.next())
    })
}

pub async fn concat<St>(stream: St) -> St::Item
    where St: Stream,
          St::Item: Extend<<St::Item as IntoIterator>::Item>,
          St::Item: IntoIterator,
          St::Item: Default,
{
    pin_mut!(stream);
    let mut collection = <St::Item>::default();
    while let Some(item) = await!(next(&mut stream)) {
        collection.extend(item);
    }
    collection
}

pub async fn for_each<St, Fut, F>(stream: St, f: F) -> ()
    where St: Stream,
    F: FnMut(St::Item) -> Fut,
    Fut: Future<Output = ()>,
{
    pin_mut!(stream);
    let mut f = f;
    while let Some(item) = await!(next(&mut stream)) {
        f(item);
    }
}

pub fn take<St>(stream: St, n: u64) -> impl Stream<Item = St::Item>
    where St: Stream,
{
    let stream = Box::pin(stream);
    futures::stream::unfold((stream, n), async move | (mut stream, n)| {
        if n == 0 {
            None
        } else {
            if let Some(item) = await!(next(&mut stream)) {
                Some((item, (stream, n - 1)))
            } else {
                None
            }
        }
    })
}

pub fn repeat<T>(item: T) -> impl Stream<Item = T>
    where T: Clone,
{
    use core::task::Poll;
    futures::stream::poll_fn(move |_| -> Poll<Option<T>> {
        Poll::Ready(Some(item.clone()))
    })
}

pub fn flatten<St, SubSt, T>(stream: St) -> impl Stream<Item = T>
    where SubSt: Stream<Item = T>,
          St: Stream<Item = SubSt>,
{
    let stream = Box::pin(stream);
    futures::stream::unfold((Some(stream), None), async move | (mut state_stream, mut state_substream)| {
        loop {
            if let Some(mut substream) = state_substream.take() {
                if let Some(item) = await!(next(&mut substream)) {
                    return Some((item, (state_stream, Some(substream))))
                } else {
                    continue;
                }
            }
            if let Some(mut stream) = state_stream.take() {
                if let Some(substream) = await!(next(&mut stream)) {
                    let substream = Box::pin(substream);
                    state_stream = Some(stream);
                    state_substream = Some(substream);
                    continue;
                }
            }
            return None;
        }
    })
}

pub fn then<St, F, Fut>(stream: St, f: F) -> impl Stream<Item = St::Item>
    where St: Stream,
          F: FnMut(St::Item) -> Fut,
          Fut: Future<Output = St::Item>
{
    let stream = Box::pin(stream);
    futures::stream::unfold((stream, f), async move | (mut stream, mut f)| {
        let item = await!(next(&mut stream));
        if let Some(item) = item {
            let new_item = await!(f(item));
            Some((new_item, (stream, f)))
        } else {
            None
        }
    })
}

pub fn skip<St>(stream: St, n: u64) -> impl Stream<Item = St::Item>
    where St: Stream,
{
    let stream = Box::pin(stream);
    futures::stream::unfold((stream, n), async move | (mut stream, mut n)| {
        while n != 0 {
            if let Some(_) = await!(next(&mut stream)) {
                n = n - 1;
                continue
            } else {
                return None
            }
        }
        if let Some(item) = await!(next(&mut stream)) {
            Some((item, (stream, 0)))
        } else {
            None
        }
    })
}

pub fn zip<St1, St2>(stream: St1, other: St2) -> impl Stream<Item = (St1::Item, St2::Item)>
    where St1: Stream,
          St2: Stream,
{
    let stream = Box::pin(stream);
    let other = Box::pin(other);
    futures::stream::unfold((stream, other), async move | (mut stream, mut other)| {
        let left = await!(next(&mut stream));
        let right = await!(next(&mut other));
        match (left, right) {
            (Some(left), Some(right)) => Some(((left, right), (stream, other))),
            _ => None
        }
    })
}

pub fn chain<St>(stream: St, other: St) -> impl Stream<Item = St::Item>
    where St: Stream,
{
    let stream = Box::pin(stream);
    let other = Box::pin(other);
    let start_with_first = true;
    futures::stream::unfold((stream, other, start_with_first), async move | (mut stream, mut other, start_with_first)| {
        if start_with_first {
            if let Some(item) = await!(next(&mut stream)) {
                return Some((item, (stream, other, start_with_first)))
            }
        }
        if let Some(item) = await!(next(&mut other)) {
            Some((item, (stream, other, /* start_with_first */ false)))
        } else {
            None
        }
    })
}

pub fn take_while<St, F, Fut>(stream: St, f: F) -> impl Stream<Item = St::Item>
    where St: Stream,
          F: FnMut(&St::Item) -> Fut,
          Fut: Future<Output = bool>,
{
    let stream = Box::pin(stream);
    futures::stream::unfold((stream, f), async move | (mut stream, mut f)| {
        if let Some(item) = await!(next(&mut stream)) {
            if await!(f(&item)) {
                Some((item, (stream, f)))
            } else {
                None
            }
        } else {
            None
        }
    })
}

pub fn skip_while<St, F, Fut>(stream: St, f: F) -> impl Stream<Item = St::Item>
    where St: Stream,
          F: FnMut(&St::Item) -> Fut,
          Fut: Future<Output = bool>,
{
    let stream = Box::pin(stream);
    let should_skip = true;
    futures::stream::unfold((stream, f, should_skip), async move | (mut stream, mut f, should_skip)| {
        while should_skip {
            if let Some(item) = await!(next(&mut stream)) {
                if await!(f(&item)) {
                    continue;
                } else {
                    return Some((item, (stream, f, /* should_skip */ false)))
                }
            } else {
                return None
            }
        }
        if let Some(item) = await!(next(&mut stream)) {
            Some((item, (stream, f, /* should_skip */ false)))
        } else {
            None
        }
    })
}

pub async fn fold<St, T, F, Fut>(stream: St, init: T, f: F) -> T
    where St: Stream,
          F: FnMut(T, St::Item) -> Fut,
          Fut: Future<Output = T>,
{
    pin_mut!(stream);
    let mut f = f;
    let mut acc = init;
    while let Some(item) = await!(next(&mut stream)) {
        acc = await!(f(acc, item));
    }
    acc
}

#[cfg(test)]
mod tests {
    use futures::executor;
    use crate::stream::*;
    use crate::future::ready;

    #[test]
    fn test_next() {
        let mut stream = iter(1..=3);

        assert_eq!(executor::block_on(next(&mut stream)), Some(1));
        assert_eq!(executor::block_on(next(&mut stream)), Some(2));
        assert_eq!(executor::block_on(next(&mut stream)), Some(3));
        assert_eq!(executor::block_on(next(&mut stream)), None);
    }

    #[test]
    fn test_collect() {
        let stream = iter(1..=5);

        let collection : Vec<i32> = executor::block_on(collect(stream));
        assert_eq!(collection, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_map() {
        let stream = iter(1..=3);
        let stream = map(stream, |x| x * 2);

        assert_eq!(vec![2, 4, 6], executor::block_on(collect::<_, Vec<_>>(stream)));
    }

    #[test]
    fn test_filter() {
        let stream = iter(1..=10);
        let evens = filter(stream, |x| ready(x % 2 == 0));

        assert_eq!(vec![2, 4, 6, 8, 10], executor::block_on(collect::<_, Vec<_>>(evens)));
    }

    #[test]
    fn test_filter_map() {
        let stream = iter(1..=10);
        let evens = filter_map(stream, |x| {
            let ret = if x % 2 == 0 { Some(x + 1) } else { None };
            ready(ret)
        });

        assert_eq!(vec![3, 5, 7, 9, 11], executor::block_on(collect::<_, Vec<_>>(evens)));
    }

    #[test]
    fn test_into_future() {
        let stream = iter(1..=2);

        let (item, stream) = executor::block_on(into_future(stream));
        assert_eq!(Some(1), item);

        let (item, stream) = executor::block_on(into_future(stream));
        assert_eq!(Some(2), item);

        let (item, _) = executor::block_on(into_future(stream));
        assert_eq!(None, item);
    }

    #[test]
    fn test_iter() {
        let stream = iter(1..=5);

        let collection : Vec<i32> = executor::block_on(collect(stream));
        assert_eq!(collection, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_concat() {
        let stream = iter(vec![vec![1, 2], vec![3], vec![4, 5]]);

        let collection : Vec<i32> = executor::block_on(concat(stream));
        assert_eq!(collection, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_for_each() {
        let mut x = 0;

        {
            let stream = iter(1..=3);
            let future = for_each(stream, |item| {
                x += item;
                ready(())
            });
            executor::block_on(future);
        }

        assert_eq!(x, 6);
    }

    #[test]
    fn test_take() {
        let stream = iter(1..=10);
        let stream = take(stream, 3);

        assert_eq!(vec![1, 2, 3], executor::block_on(collect::<_, Vec<_>>(stream)));
    }

    #[test]
    fn test_repeat() {
        let stream = repeat(9);
        let stream = take(stream, 3);

        assert_eq!(vec![9, 9, 9], executor::block_on(collect::<_, Vec<_>>(stream)));
    }

    #[test]
    fn test_flatten() {
        let stream0 = iter(0..0);
        let stream1 = iter(1..4);
        let stream2 = iter(4..7);
        let stream3 = iter(7..10);
        let stream = iter(vec![stream0, stream1, stream2, stream3]);
        let stream = flatten(stream);

        assert_eq!(vec![1, 2, 3, 4, 5, 6, 7, 8, 9], executor::block_on(collect::<_, Vec<_>>(stream)));
    }

    #[test]
    fn test_then() {
        let stream = iter(1..=3);
        let stream = then(stream, |x| ready(x+3));

        assert_eq!(vec![4, 5, 6], executor::block_on(collect::<_, Vec<_>>(stream)));
    }

    #[test]
    fn test_skip() {
        let stream = iter(1..=10);
        let stream = skip(stream, 5);

        assert_eq!(vec![6, 7, 8, 9, 10], executor::block_on(collect::<_, Vec<_>>(stream)));
    }

    #[test]
    fn test_zip() {
        let stream1 = iter(1..=3);
        let stream2 = iter(5..=10);
        let stream = zip(stream1, stream2);

        assert_eq!(vec![(1, 5), (2, 6), (3, 7)], executor::block_on(collect::<_, Vec<_>>(stream)));
    }

    #[test]
    fn test_chain() {
        let stream1 = iter(1..=2);
        let stream2 = iter(3..=4);
        let stream = chain(stream1, stream2);

        assert_eq!(vec![1, 2, 3, 4], executor::block_on(collect::<_, Vec<_>>(stream)));
    }

    #[test]
    fn test_take_while() {
        let stream = iter(1..=10);
        let stream = take_while(stream, |x| ready(*x <= 5));

        assert_eq!(vec![1, 2, 3, 4, 5], executor::block_on(collect::<_, Vec<_>>(stream)));
    }

    #[test]
    fn test_skip_while() {
        let stream = iter(1..=10);
        let stream = skip_while(stream, |x| ready(*x <= 5));

        assert_eq!(vec![6, 7, 8, 9, 10], executor::block_on(collect::<_, Vec<_>>(stream)));
    }

    #[test]
    fn test_fold() {
        let stream = iter(0..6);
        let sum = fold(stream, 0, |acc, x| ready(acc + x));

        assert_eq!(15, executor::block_on(sum));
    }
}
