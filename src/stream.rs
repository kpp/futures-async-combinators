use futures_core::future::Future;
pub use futures_core::stream::Stream;
use futures_async_stream::async_stream_block;

use core::iter::IntoIterator;
use core::pin::Pin;
use core::task::{Context, Poll};

use pin_utils::pin_mut;

/// Creates a future that resolves to the next item in the stream.
///
/// Note that because `next` doesn't take ownership over the stream,
/// the [`Stream`] type must be [`Unpin`]. If you want to use `next` with a
/// [`!Unpin`](Unpin) stream, you'll first have to pin the stream. This can
/// be done by boxing the stream using [`Box::pin`] or
/// pinning it to the stack using the `pin_mut!` macro from the `pin_utils`
/// crate.
///
/// # Examples
///
/// ```
/// # futures::executor::block_on(async {
/// use futures_async_combinators::stream::{iter, next};
///
/// let mut stream = iter(1..=3);
///
/// assert_eq!(next(&mut stream).await, Some(1));
/// assert_eq!(next(&mut stream).await, Some(2));
/// assert_eq!(next(&mut stream).await, Some(3));
/// assert_eq!(next(&mut stream).await, None);
/// # });
/// ```
pub fn next<'a, St>(mut stream: &'a mut St) -> impl Future<Output = Option<St::Item>> + 'a
where
    St: Stream + Unpin,
{
    use crate::future::poll_fn;
    poll_fn(move |context| Pin::new(&mut stream).poll_next(context))
}

/// Collect all of the values of this stream into a vector, returning a
/// future representing the result of that computation.
///
/// The returned future will be resolved when the stream terminates.
///
/// # Examples
///
/// ```
/// # futures::executor::block_on(async {
/// use futures_async_combinators::stream::{iter, collect};
///
/// let stream = iter(1..=5);
///
/// let collection: Vec<i32> = collect(stream).await;
/// assert_eq!(collection, vec![1, 2, 3, 4, 5]);
/// # });
/// ```
pub async fn collect<St, C>(stream: St) -> C
where
    St: Stream,
    C: Default + Extend<St::Item>,
{
    pin_mut!(stream);
    let mut collection = C::default();
    while let Some(item) = next(&mut stream).await {
        collection.extend(Some(item));
    }
    collection
}

/// Maps this stream's items to a different type, returning a new stream of
/// the resulting type.
///
/// The provided closure is executed over all elements of this stream as
/// they are made available. It is executed inline with calls to
/// [`poll_next`](Stream::poll_next).
///
/// Note that this function consumes the stream passed into it and returns a
/// wrapped version of it, similar to the existing `map` methods in the
/// standard library.
///
/// # Examples
///
/// ```
/// # futures::executor::block_on(async {
/// use futures_async_combinators::stream::{iter, map, collect};
///
/// let stream = iter(1..=3);
/// let stream = map(stream, |x| x + 3);
///
/// let result: Vec<_> = collect(stream).await;
/// assert_eq!(vec![4, 5, 6], result);
/// # });
/// ```
pub fn map<St, U, F>(stream: St, f: F) -> impl Stream<Item = U>
where
    St: Stream,
    F: FnMut(St::Item) -> U,
{
    let mut f = f;
    async_stream_block! {
        #[for_await]
        for item in stream {
            yield f(item)
        }
    }
}

/// Filters the values produced by this stream according to the provided
/// asynchronous predicate.
///
/// As values of this stream are made available, the provided predicate `f`
/// will be run against them. If the predicate returns a `Future` which
/// resolves to `true`, then the stream will yield the value, but if the
/// predicate returns a `Future` which resolves to `false`, then the value
/// will be discarded and the next value will be produced.
///
/// Note that this function consumes the stream passed into it and returns a
/// wrapped version of it, similar to the existing `filter` methods in the
/// standard library.
///
/// # Examples
///
/// ```
/// # futures::executor::block_on(async {
/// use futures_async_combinators::{future::ready, stream::{iter, filter, collect}};
///
/// let stream = iter(1..=10);
/// let evens = filter(stream, |x| ready(x % 2 == 0));
///
/// let result: Vec<_> = collect(evens).await;
/// assert_eq!(vec![2, 4, 6, 8, 10], result);
/// # });
/// ```
pub fn filter<St, Fut, F>(stream: St, f: F) -> impl Stream<Item = St::Item>
where
    St: Stream,
    F: FnMut(&St::Item) -> Fut,
    Fut: Future<Output = bool>,
{
    let mut f = f;
    async_stream_block! {
        #[for_await]
        for item in stream {
            if f(&item).await {
                yield item
            }
        }
    }
}

/// Filters the values produced by this stream while simultaneously mapping
/// them to a different type according to the provided asynchronous closure.
///
/// As values of this stream are made available, the provided function will
/// be run on them. If the future returned by the predicate `f` resolves to
/// [`Some(item)`](Some) then the stream will yield the value `item`, but if
/// it resolves to [`None`] then the next value will be produced.
///
/// Note that this function consumes the stream passed into it and returns a
/// wrapped version of it, similar to the existing `filter_map` methods in
/// the standard library.
///
/// # Examples
/// ```
/// # futures::executor::block_on(async {
/// use futures_async_combinators::{future::ready, stream::{iter, filter_map, collect}};
///
/// let stream = iter(1..=10);
/// let evens = filter_map(stream, |x| {
///     let ret = if x % 2 == 0 { Some(x + 1) } else { None };
///     ready(ret)
/// });
///
/// let result: Vec<_> = collect(evens).await;
/// assert_eq!(vec![3, 5, 7, 9, 11], result);
/// # });
/// ```
pub fn filter_map<St, Fut, F, U>(stream: St, f: F) -> impl Stream<Item = U>
where
    St: Stream,
    F: FnMut(St::Item) -> Fut,
    Fut: Future<Output = Option<U>>,
{
    let mut f = f;
    async_stream_block! {
        #[for_await]
        for item in stream {
            if let Some(item) = f(item).await {
                yield item
            }
        }
    }
}

/// Converts this stream into a future of `(next_item, tail_of_stream)`.
/// If the stream terminates, then the next item is [`None`].
///
/// The returned future can be used to compose streams and futures together
/// by placing everything into the "world of futures".
///
/// Note that because `into_future` moves the stream, the [`Stream`] type
/// must be [`Unpin`]. If you want to use `into_future` with a
/// [`!Unpin`](Unpin) stream, you'll first have to pin the stream. This can
/// be done by boxing the stream using [`Box::pin`] or
/// pinning it to the stack using the `pin_mut!` macro from the `pin_utils`
/// crate.
///
/// # Examples
///
/// ```
/// # futures::executor::block_on(async {
/// use futures_async_combinators::stream::{iter, into_future};
///
/// let stream = iter(1..3);
///
/// let (item, stream) = into_future(stream).await;
/// assert_eq!(Some(1), item);
///
/// let (item, stream) = into_future(stream).await;
/// assert_eq!(Some(2), item);
///
/// let (item, stream) = into_future(stream).await;
/// assert_eq!(None, item);
/// # });
/// ```
pub async fn into_future<St>(stream: St) -> (Option<St::Item>, impl Stream<Item = St::Item>)
where
    St: Stream + Unpin,
{
    let mut stream = stream;
    let next_item = next(&mut stream).await;
    (next_item, stream)
}

/// Converts an `Iterator` into a `Stream` which is always ready
/// to yield the next value.
///
/// Iterators in Rust don't express the ability to block, so this adapter
/// simply always calls `iter.next()` and returns that.
///
/// ```
/// # futures::executor::block_on(async {
/// use futures_async_combinators::stream::{iter, collect};
///
/// let stream = iter(vec![17, 19]);
/// let result: Vec<_> = collect(stream).await;
/// assert_eq!(vec![17, 19], result);
/// # });
/// ```
pub fn iter<I>(i: I) -> impl Stream<Item = I::Item>
where
    I: IntoIterator,
{
    let mut iter = i.into_iter();
    crate::stream::poll_fn(move |_| -> Poll<Option<I::Item>> { Poll::Ready(iter.next()) })
}

/// Concatenate all items of a stream into a single extendable
/// destination, returning a future representing the end result.
///
/// This combinator will extend the first item with the contents
/// of all the subsequent results of the stream. If the stream is
/// empty, the default value will be returned.
///
/// Works with all collections that implement the
/// [`Extend`](std::iter::Extend) trait.
///
/// # Examples
///
/// ```
/// # futures::executor::block_on(async {
/// use futures_async_combinators::stream::{iter, concat};
///
/// let stream = iter(vec![vec![1, 2], vec![3], vec![4, 5]]);
///
/// let result: Vec<_> = concat(stream).await;
/// assert_eq!(result, vec![1, 2, 3, 4, 5]);
/// # });
/// ```
pub async fn concat<St>(stream: St) -> St::Item
where
    St: Stream,
    St::Item: Extend<<St::Item as IntoIterator>::Item>,
    St::Item: IntoIterator,
    St::Item: Default,
{
    pin_mut!(stream);
    let mut collection = <St::Item>::default();
    while let Some(item) = next(&mut stream).await {
        collection.extend(item);
    }
    collection
}

/// Runs this stream to completion, executing the provided asynchronous
/// closure for each element on the stream.
///
/// The closure provided will be called for each item this stream produces,
/// yielding a future. That future will then be executed to completion
/// before moving on to the next item.
///
/// The returned value is a `Future` where the `Output` type is `()`; it is
/// executed entirely for its side effects.
///
/// To process each item in the stream and produce another stream instead
/// of a single future, use `then` instead.
///
/// # Examples
///
/// ```
/// # futures::executor::block_on(async {
/// use futures::future;
/// use futures_async_combinators::{future::ready, stream::{repeat, take, for_each}};
///
/// let mut x = 0;
///
/// {
///     let stream = repeat(1);
///     let stream = take(stream, 3);
///     let fut = for_each(stream, |item| {
///         x += item;
///         ready(())
///     });
///     fut.await;
/// }
///
/// assert_eq!(x, 3);
/// # });
/// ```
pub async fn for_each<St, Fut, F>(stream: St, f: F) -> ()
where
    St: Stream,
    F: FnMut(St::Item) -> Fut,
    Fut: Future<Output = ()>,
{
    pin_mut!(stream);
    let mut f = f;
    while let Some(item) = next(&mut stream).await {
        f(item);
    }
}

/// Creates a new stream of at most `n` items of the underlying stream.
///
/// Once `n` items have been yielded from this stream then it will always
/// return that the stream is done.
///
/// # Examples
///
/// ```
/// # futures::executor::block_on(async {
/// use futures_async_combinators::stream::{iter, take, collect};
///
/// let stream = iter(1..=10);
/// let stream = take(stream, 3);
///
/// let result: Vec<_> = collect(stream).await;
/// assert_eq!(vec![1, 2, 3], result);
/// # });
/// ```
pub fn take<St>(stream: St, n: u64) -> impl Stream<Item = St::Item>
where
    St: Stream,
{
    let mut n = n;
    async_stream_block! {
        #[for_await]
        for item in stream {
            if n == 0 {
                break;
            } else {
                n = n - 1;
                yield item
            }
        }
    }
}

/// Create a stream which produces the same item repeatedly.
///
/// The stream never terminates. Note that you likely want to avoid
/// usage of `collect` or such on the returned stream as it will exhaust
/// available memory as it tries to just fill up all RAM.
///
/// ```
/// # futures::executor::block_on(async {
/// use futures_async_combinators::stream::{repeat, take, collect};
///
/// let stream = repeat(9);
/// let stream = take(stream, 3);
///
/// let result: Vec<_> = collect(stream).await;
/// assert_eq!(vec![9, 9, 9], result);
/// # });
/// ```
pub fn repeat<T>(item: T) -> impl Stream<Item = T>
where
    T: Clone,
{
    crate::stream::poll_fn(move |_| -> Poll<Option<T>> { Poll::Ready(Some(item.clone())) })
}

/// Flattens a stream of streams into just one continuous stream.
///
/// # Examples
///
/// ```
/// # futures::executor::block_on(async {
/// use futures_async_combinators::stream::{iter, flatten, collect};
///
/// let stream0 = iter(0..0);
/// let stream1 = iter(1..4);
/// let stream2 = iter(4..7);
/// let stream3 = iter(7..10);
/// let stream = iter(vec![stream0, stream1, stream2, stream3]);
/// let stream = flatten(stream);
///
/// let result: Vec<_> = collect(stream).await;
/// assert_eq!(vec![1, 2, 3, 4, 5, 6, 7, 8, 9], result);
/// # });
/// ```
pub fn flatten<St, SubSt, T>(stream: St) -> impl Stream<Item = T>
where
    SubSt: Stream<Item = T>,
    St: Stream<Item = SubSt>,
{
    async_stream_block! {
        #[for_await]
        for substream in stream {
            #[for_await]
            for item in substream {
                yield item
            }
        }
    }
}

/// Computes from this stream's items new items of a different type using
/// an asynchronous closure.
///
/// The provided closure `f` will be called with an `Item` once a value is
/// ready, it returns a future which will then be run to completion
/// to produce the next value on this stream.
///
/// Note that this function consumes the stream passed into it and returns a
/// wrapped version of it.
///
/// # Examples
///
/// ```
/// # futures::executor::block_on(async {
/// use futures_async_combinators::{future::ready, stream::{iter, then, collect}};
///
/// let stream = iter(1..=3);
/// let stream = then(stream, |x| ready(x + 3));
///
/// let result: Vec<_> = collect(stream).await;
/// assert_eq!(vec![4, 5, 6], result);
/// # });
/// ```
pub fn then<St, F, Fut>(stream: St, f: F) -> impl Stream<Item = St::Item>
where
    St: Stream,
    F: FnMut(St::Item) -> Fut,
    Fut: Future<Output = St::Item>,
{
    let mut f = f;
    async_stream_block! {
        #[for_await]
        for item in stream {
            let new_item = f(item).await;
            yield new_item
        }
    }
}

/// Creates a new stream which skips `n` items of the underlying stream.
///
/// Once `n` items have been skipped from this stream then it will always
/// return the remaining items on this stream.
///
/// # Examples
///
/// ```
/// # futures::executor::block_on(async {
/// use futures_async_combinators::stream::{iter, skip, collect};
///
/// let stream = iter(1..=10);
/// let stream = skip(stream, 5);
///
/// let result: Vec<_> = collect(stream).await;
/// assert_eq!(vec![6, 7, 8, 9, 10], result);
/// # });
/// ```
pub fn skip<St>(stream: St, n: u64) -> impl Stream<Item = St::Item>
where
    St: Stream,
{
    let mut n = n;
    async_stream_block! {
        #[for_await]
        for item in stream {
            if n == 0 {
                yield item
            } else {
                n = n - 1;
                continue;
            }
        }
    }
}

/// An adapter for zipping two streams together.
///
/// The zipped stream waits for both streams to produce an item, and then
/// returns that pair. If either stream ends then the zipped stream will
/// also end.
///
/// # Examples
///
/// ```
/// # futures::executor::block_on(async {
/// use futures_async_combinators::stream::{iter, zip, collect};
///
/// let stream1 = iter(1..=3);
/// let stream2 = iter(5..=10);
///
/// let stream = zip(stream1, stream2);
/// let result: Vec<_> = collect(stream).await;
/// assert_eq!(vec![(1, 5), (2, 6), (3, 7)], result);
/// # });
/// ```
///
pub fn zip<St1, St2>(stream: St1, other: St2) -> impl Stream<Item = (St1::Item, St2::Item)>
where
    St1: Stream,
    St2: Stream,
{
    let mut stream = Box::pin(stream);
    let mut other = Box::pin(other);
    async_stream_block! {
        loop {
            let left = next(&mut stream).await;
            let right = next(&mut other).await;
            match (left, right) {
                (Some(left), Some(right)) => yield (left, right),
                _ => break,
            }
        }
    }
}

/// Adapter for chaining two stream.
///
/// The resulting stream emits elements from the first stream, and when
/// first stream reaches the end, emits the elements from the second stream.
///
/// ```
/// # futures::executor::block_on(async {
/// use futures_async_combinators::stream::{iter, chain, collect};
///
/// let stream1 = iter(vec![Ok(10), Err(false)]);
/// let stream2 = iter(vec![Err(true), Ok(20)]);
///
/// let stream = chain(stream1, stream2);
///
/// let result: Vec<_> = collect(stream).await;
/// assert_eq!(result, vec![
///     Ok(10),
///     Err(false),
///     Err(true),
///     Ok(20),
/// ]);
/// # });
/// ```
pub fn chain<St>(stream: St, other: St) -> impl Stream<Item = St::Item>
where
    St: Stream,
{
    async_stream_block! {
        #[for_await]
        for item in stream {
            yield item
        }
        #[for_await]
        for item in other {
            yield item
        }
    }
}

/// Take elements from this stream while the provided asynchronous predicate
/// resolves to `true`.
///
/// This function, like `Iterator::take_while`, will take elements from the
/// stream until the predicate `f` resolves to `false`. Once one element
/// returns false it will always return that the stream is done.
///
/// # Examples
///
/// ```
/// # futures::executor::block_on(async {
/// use futures_async_combinators::{future::ready, stream::{iter, take_while, collect}};
///
/// let stream = iter(1..=10);
/// let stream = take_while(stream, |x| ready(*x <= 5));
///
/// let result: Vec<_> = collect(stream).await;
/// assert_eq!(vec![1, 2, 3, 4, 5], result);
/// # });
/// ```
pub fn take_while<St, F, Fut>(stream: St, f: F) -> impl Stream<Item = St::Item>
where
    St: Stream,
    F: FnMut(&St::Item) -> Fut,
    Fut: Future<Output = bool>,
{
    let mut f = f;
    async_stream_block! {
        #[for_await]
        for item in stream {
            if f(&item).await {
                yield item
            } else {
                break;
            }
        }
    }
}

/// Skip elements on this stream while the provided asynchronous predicate
/// resolves to `true`.
///
/// This function, like `Iterator::skip_while`, will skip elements on the
/// stream until the predicate `f` resolves to `false`. Once one element
/// returns false all future elements will be returned from the underlying
/// stream.
///
/// # Examples
///
/// ```
/// # futures::executor::block_on(async {
/// use futures_async_combinators::{future::ready, stream::{iter, skip_while, collect}};
///
/// let stream = iter(1..=10);
/// let stream = skip_while(stream, |x| ready(*x <= 5));
///
/// let result: Vec<_> = collect(stream).await;
/// assert_eq!(vec![6, 7, 8, 9, 10], result);
/// # });
/// ```
pub fn skip_while<St, F, Fut>(stream: St, f: F) -> impl Stream<Item = St::Item>
where
    St: Stream,
    F: FnMut(&St::Item) -> Fut,
    Fut: Future<Output = bool>,
{
    let mut f = f;
    let mut should_skip = true;
    async_stream_block! {
        #[for_await]
        for item in stream {
            if should_skip {
                if f(&item).await {
                    continue;
                } else {
                    should_skip = false;
                    yield item
                }
            } else {
                yield item
            }
        }
    }
}

/// Execute an accumulating asynchronous computation over a stream,
/// collecting all the values into one final result.
///
/// This combinator will accumulate all values returned by this stream
/// according to the closure provided. The initial state is also provided to
/// this method and then is returned again by each execution of the closure.
/// Once the entire stream has been exhausted the returned future will
/// resolve to this value.
///
/// # Examples
///
/// ```
/// # futures::executor::block_on(async {
/// use futures_async_combinators::{future::ready, stream::{iter, fold}};
///
/// let number_stream = iter(0..6);
/// let sum = fold(number_stream, 0, |acc, x| ready(acc + x));
/// assert_eq!(sum.await, 15);
/// # });
/// ```
pub async fn fold<St, T, F, Fut>(stream: St, init: T, f: F) -> T
where
    St: Stream,
    F: FnMut(T, St::Item) -> Fut,
    Fut: Future<Output = T>,
{
    pin_mut!(stream);
    let mut f = f;
    let mut acc = init;
    while let Some(item) = next(&mut stream).await {
        acc = f(acc, item).await;
    }
    acc
}

/// Creates a `Stream` from a seed and a closure returning a `Future`.
///
/// This function is the dual for the [`fold()`] adapter: while
/// [`fold()`] reduces a `Stream` to one single value, `unfold()` creates a
/// `Stream` from a seed value.
///
/// `unfold()` will call the provided closure with the provided seed, then wait
/// for the returned `Future` to complete with `(a, b)`. It will then yield the
/// value `a`, and use `b` as the next internal state.
///
/// If the closure returns `None` instead of `Some(Future)`, then the `unfold()`
/// will stop producing items and return `Poll::Ready(None)` in future
/// calls to `poll()`.
///
/// In case of error generated by the returned `Future`, the error will be
/// returned by the `Stream`.  The `Stream` will then yield
/// `Poll::Ready(None)` in future calls to `poll()`.
///
/// This function can typically be used when wanting to go from the "world of
/// futures" to the "world of streams": the provided closure can build a
/// `Future` using other library functions working on futures, and `unfold()`
/// will turn it into a `Stream` by repeating the operation.
///
/// # Example
///
/// ```
/// # futures::executor::block_on(async {
/// use futures_async_combinators::{future::ready, stream::{unfold, collect}};
///
/// let stream = unfold(0, |state| {
///     if state <= 2 {
///         let next_state = state + 1;
///         let yielded = state * 2;
///         ready(Some((yielded, next_state)))
///     } else {
///         ready(None)
///     }
/// });
///
/// let result: Vec<_> = collect(stream).await;
/// assert_eq!(result, vec![0, 2, 4]);
/// # });
/// ```
pub fn unfold<T, F, Fut, It>(init: T, mut f: F) -> impl Stream<Item = It>
where
    F: FnMut(T) -> Fut,
    Fut: Future<Output = Option<(It, T)>>,
{
    let mut state = Some(init);
    let mut running = Box::pin(None);

    crate::stream::poll_fn(move |context| -> Poll<Option<It>> {
        if let Some(state) = state.take() {
            let fut = f(state);
            running.set(Some(fut));
        }

        let fut = Option::as_pin_mut(running.as_mut()).expect("this stream must not be polled any more");
        let step = futures_core::ready!(fut.poll(context));

        match step {
            None => {
                Poll::Ready(None)
            },
            Some((item, new_state)) => {
                state = Some(new_state);
                running.set(None);
                Poll::Ready(Some(item))
            },
        }
    })
}

/// Creates a new stream wrapping a function returning `Poll<Option<T>>`.
///
/// Polling the returned stream calls the wrapped function.
///
/// # Examples
///
/// ```
/// use futures_async_combinators::stream::poll_fn;
/// use core::task::Poll;
///
/// let mut counter = 1usize;
///
/// let read_stream = poll_fn(move |_| -> Poll<Option<String>> {
///     if counter == 0 { return Poll::Ready(None); }
///     counter -= 1;
///     Poll::Ready(Some("Hello, World!".to_owned()))
/// });
/// ```
pub fn poll_fn<T, F>(f: F) -> impl Stream<Item = T>
where
    F: FnMut(&mut Context<'_>) -> Poll<Option<T>>,
{
    pub struct PollFn<F> {
        f: F,
    }

    impl<F> Unpin for PollFn<F> {}

    impl<T, F> Stream for PollFn<F>
    where
        F: FnMut(&mut Context<'_>) -> Poll<Option<T>>,
    {
        type Item = T;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
            (&mut self.f)(cx)
        }
    }
    PollFn { f }
}

#[cfg(test)]
mod tests {
    use crate::future::ready;
    use crate::stream::*;
    use futures::executor;

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

        let collection: Vec<i32> = executor::block_on(collect(stream));
        assert_eq!(collection, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_map() {
        let stream = iter(1..=3);
        let stream = map(stream, |x| x * 2);

        assert_eq!(
            vec![2, 4, 6],
            executor::block_on(collect::<_, Vec<_>>(stream))
        );
    }

    #[test]
    fn test_filter() {
        let stream = iter(1..=10);
        let evens = filter(stream, |x| ready(x % 2 == 0));

        assert_eq!(
            vec![2, 4, 6, 8, 10],
            executor::block_on(collect::<_, Vec<_>>(evens))
        );
    }

    #[test]
    fn test_filter_map() {
        let stream = iter(1..=10);
        let evens = filter_map(stream, |x| {
            let ret = if x % 2 == 0 { Some(x + 1) } else { None };
            ready(ret)
        });

        assert_eq!(
            vec![3, 5, 7, 9, 11],
            executor::block_on(collect::<_, Vec<_>>(evens))
        );
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

        let collection: Vec<i32> = executor::block_on(collect(stream));
        assert_eq!(collection, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_concat() {
        let stream = iter(vec![vec![1, 2], vec![3], vec![4, 5]]);

        let collection: Vec<i32> = executor::block_on(concat(stream));
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

        assert_eq!(
            vec![1, 2, 3],
            executor::block_on(collect::<_, Vec<_>>(stream))
        );
    }

    #[test]
    fn test_take_more_than_size() {
        let stream = iter(1..=3);
        let stream = take(stream, 10);

        assert_eq!(
            vec![1, 2, 3],
            executor::block_on(collect::<_, Vec<_>>(stream))
        );
    }

    #[test]
    fn test_repeat() {
        let stream = repeat(9);
        let stream = take(stream, 3);

        assert_eq!(
            vec![9, 9, 9],
            executor::block_on(collect::<_, Vec<_>>(stream))
        );
    }

    #[test]
    fn test_flatten() {
        let stream0 = iter(0..0);
        let stream1 = iter(1..4);
        let stream2 = iter(4..7);
        let stream3 = iter(7..10);
        let stream = iter(vec![stream0, stream1, stream2, stream3]);
        let stream = flatten(stream);

        assert_eq!(
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
            executor::block_on(collect::<_, Vec<_>>(stream))
        );
    }

    #[test]
    fn test_then() {
        let stream = iter(1..=3);
        let stream = then(stream, |x| ready(x + 3));

        assert_eq!(
            vec![4, 5, 6],
            executor::block_on(collect::<_, Vec<_>>(stream))
        );
    }

    #[test]
    fn test_skip() {
        let stream = iter(1..=10);
        let stream = skip(stream, 5);

        assert_eq!(
            vec![6, 7, 8, 9, 10],
            executor::block_on(collect::<_, Vec<_>>(stream))
        );
    }

    #[test]
    fn test_skip_more_than_size() {
        let stream = iter(1..=10);
        let stream = skip(stream, 15);

        assert!(executor::block_on(collect::<_, Vec<_>>(stream)).is_empty());
    }

    #[test]
    fn test_zip() {
        let stream1 = iter(1..=3);
        let stream2 = iter(5..=10);
        let stream = zip(stream1, stream2);

        assert_eq!(
            vec![(1, 5), (2, 6), (3, 7)],
            executor::block_on(collect::<_, Vec<_>>(stream))
        );
    }

    #[test]
    fn test_chain() {
        let stream1 = iter(1..=2);
        let stream2 = iter(3..=4);
        let stream = chain(stream1, stream2);

        assert_eq!(
            vec![1, 2, 3, 4],
            executor::block_on(collect::<_, Vec<_>>(stream))
        );
    }

    #[test]
    fn test_take_while() {
        let stream = iter(1..=10);
        let stream = take_while(stream, |x| ready(*x <= 5));

        assert_eq!(
            vec![1, 2, 3, 4, 5],
            executor::block_on(collect::<_, Vec<_>>(stream))
        );
    }

    #[test]
    fn test_take_while_more_than_size() {
        let stream = iter(1..=3);
        let stream = take_while(stream, |x| ready(*x <= 5));

        assert_eq!(
            vec![1, 2, 3],
            executor::block_on(collect::<_, Vec<_>>(stream))
        );
    }

    #[test]
    fn test_skip_while() {
        let stream = iter(1..=10);
        let stream = skip_while(stream, |x| ready(*x <= 5));

        assert_eq!(
            vec![6, 7, 8, 9, 10],
            executor::block_on(collect::<_, Vec<_>>(stream))
        );
    }

    #[test]
    fn test_skip_while_more_than_size() {
        let stream = iter(1..=3);
        let stream = skip_while(stream, |x| ready(*x <= 5));

        assert!(executor::block_on(collect::<_, Vec<_>>(stream)).is_empty());
    }

    #[test]
    fn test_fold() {
        let stream = iter(0..6);
        let sum = fold(stream, 0, |acc, x| ready(acc + x));

        assert_eq!(15, executor::block_on(sum));
    }

    #[test]
    fn test_unfold() {
        let stream = unfold(0, |state| {
            if state <= 2 {
                let next_state = state + 1;
                let yielded = state * 2;
                ready(Some((yielded, next_state)))
            } else {
                ready(None)
            }
        });
        assert_eq!(
            vec![0, 2, 4],
            executor::block_on(collect::<_, Vec<_>>(stream))
        );
    }
}
