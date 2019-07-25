use futures::future::Future;
use futures::stream::Stream;

use core::task::{Poll, Context};

/// Create a future that is immediately ready with a value.
///
/// # Examples
///
/// ```
/// #![feature(async_await)]
/// # futures::executor::block_on(async {
/// use futures_async_combinators::future;
///
/// let a = future::ready(1);
/// assert_eq!(a.await, 1);
/// # });
/// ```
pub async fn ready<T>(value: T) -> T {
    value
}

/// Map this future's output to a different type, returning a new future of
/// the resulting type.
///
/// This function is similar to the `Option::map` or `Iterator::map` where
/// it will change the type of the underlying future. This is useful to
/// chain along a computation once a future has been resolved.
///
/// Note that this function consumes the receiving future and returns a
/// wrapped version of it, similar to the existing `map` methods in the
/// standard library.
///
/// # Examples
///
/// ```
/// #![feature(async_await)]
/// # futures::executor::block_on(async {
/// use futures_async_combinators::future::{ready, map};
///
/// let future = ready(1);
/// let new_future = map(future, |x| x + 3);
/// assert_eq!(new_future.await, 4);
/// # });
/// ```
pub async fn map<Fut, U, F>(future: Fut, f: F) -> U
    where F: FnOnce(Fut::Output) -> U,
          Fut: Future,
{
    f(future.await)
}

/// Chain on a computation for when a future finished, passing the result of
/// the future to the provided closure `f`.
///
/// The returned value of the closure must implement the `Future` trait
/// and can represent some more work to be done before the composed future
/// is finished.
///
/// The closure `f` is only run *after* successful completion of the `self`
/// future.
///
/// Note that this function consumes the receiving future and returns a
/// wrapped version of it.
///
/// # Examples
///
/// ```
/// #![feature(async_await)]
/// # futures::executor::block_on(async {
/// use futures_async_combinators::future::{ready, then};
///
/// let future_of_1 = ready(1);
/// let future_of_4 = then(future_of_1, |x| ready(x + 3));
/// assert_eq!(future_of_4.await, 4);
/// # });
/// ```
pub async fn then<FutA, FutB, F>(future: FutA, f: F) -> FutB::Output
    where F: FnOnce(FutA::Output) -> FutB,
          FutA: Future,
          FutB: Future,
{
    let new_future = f(future.await);
    new_future.await
}

/// Executes another future after this one resolves successfully. The
/// success value is passed to a closure to create this subsequent future.
///
/// The provided closure `f` will only be called if this future is resolved
/// to an [`Ok`]. If this future resolves to an [`Err`], panics, or is
/// dropped, then the provided closure will never be invoked. The
/// `Error` type of this future and the future
/// returned by `f` have to match.
///
/// Note that this method consumes the future it is called on and returns a
/// wrapped version of it.
///
/// # Examples
///
/// ```
/// #![feature(async_await)]
/// use futures::future::{self, TryFutureExt};
///
/// # futures::executor::block_on(async {
/// let future = future::ready(Ok::<i32, i32>(1));
/// let future = future.and_then(|x| future::ready(Ok::<i32, i32>(x + 3)));
/// assert_eq!(future.await, Ok(4));
/// # });
/// ```
///
/// Calling [`and_then`] on an errored future has no
/// effect:
///
/// ```
/// #![feature(async_await)]
/// use futures_async_combinators::future::{ready, and_then};
///
/// # futures::executor::block_on(async {
/// let future = ready(Err::<i32, i32>(1));
/// let future = and_then(future, |x| ready(Err::<i32, i32>(x + 3)));
/// assert_eq!(future.await, Err(1));
/// # });
/// ```
pub async fn and_then<FutA, FutB, F, T, U, E>(future: FutA, f: F) -> Result<U, E>
    where F: FnOnce(T) -> FutB,
          FutA: Future<Output = Result<T,E>>,
          FutB: Future<Output = Result<U,E>>,
{
    match future.await {
        Ok(ok) => {
            let new_future = f(ok);
            new_future.await
        },
        Err(err) => Err(err),
    }
}

/// Executes another future if this one resolves to an error. The
/// error value is passed to a closure to create this subsequent future.
///
/// The provided closure `f` will only be called if this future is resolved
/// to an [`Err`]. If this future resolves to an [`Ok`], panics, or is
/// dropped, then the provided closure will never be invoked. The
/// `Ok` type of this future and the future returned by `f`
/// have to match.
///
/// Note that this method consumes the future it is called on and returns a
/// wrapped version of it.
///
/// # Examples
///
/// ```
/// #![feature(async_await)]
/// use futures_async_combinators::future::{ready, or_else};
///
/// # futures::executor::block_on(async {
/// let future = ready(Err::<i32, i32>(1));
/// let future = or_else(future, |x| ready(Err::<i32, i32>(x + 3)));
/// assert_eq!(future.await, Err(4));
/// # });
/// ```
///
/// Calling [`or_else`] on a successful future has
/// no effect:
///
/// ```
/// #![feature(async_await)]
/// use futures_async_combinators::future::{ready, or_else};
///
/// # futures::executor::block_on(async {
/// let future = ready(Ok::<i32, i32>(1));
/// let future = or_else(future, |x| ready(Ok::<i32, i32>(x + 3)));
/// assert_eq!(future.await, Ok(1));
/// # });
/// ```
pub async fn or_else<FutA, FutB, F, T, E, U>(future: FutA, f: F) -> Result<T, U>
    where F: FnOnce(E) -> FutB,
          FutA: Future<Output = Result<T,E>>,
          FutB: Future<Output = Result<T,U>>,
{
    match future.await {
        Ok(ok) => Ok(ok),
        Err(err) => {
            let new_future = f(err);
            new_future.await
        },
    }
}

/// Maps this future's success value to a different value.
///
/// This method can be used to change the `Ok` type of the
/// future into a different type. It is similar to the [`Result::map`]
/// method. You can use this method to chain along a computation once the
/// future has been resolved.
///
/// The provided closure `f` will only be called if this future is resolved
/// to an `Ok`. If it resolves to an `Err`, panics, or is dropped, then
/// the provided closure will never be invoked.
///
/// Note that this method consumes the future it is called on and returns a
/// wrapped version of it.
///
/// # Examples
///
/// ```
/// #![feature(async_await)]
/// use futures_async_combinators::future::{ready, map_ok};
///
/// # futures::executor::block_on(async {
/// let future = ready(Ok::<i32, i32>(1));
/// let future = map_ok(future, |x| x + 3);
/// assert_eq!(future.await, Ok(4));
/// # });
/// ```
///
/// Calling [`map_ok`] on an errored future has no
/// effect:
///
/// ```
/// #![feature(async_await)]
/// use futures_async_combinators::future::{ready, map_ok};
///
/// # futures::executor::block_on(async {
/// let future = ready(Err::<i32, i32>(1));
/// let future = map_ok(future, |x| x + 3);
/// assert_eq!(future.await, Err(1));
/// # });
/// ```
pub async fn map_ok<Fut, F, T, U, E>(future: Fut, f: F) -> Result<U, E>
    where F: FnOnce(T) -> U,
          Fut: Future<Output = Result<T,E>>,
{
    future.await.map(f)
}

/// Maps this future's error value to a different value.
///
/// This method can be used to change the `Error` type
/// of the future into a different type. It is similar to the
/// [`Result::map_err`] method. You can use this method for example to
/// ensure that futures have the same `Error` type when
/// using `select!` or `join!`.
///
/// The provided closure `f` will only be called if this future is resolved
/// to an [`Err`]. If it resolves to an [`Ok`], panics, or is dropped, then
/// the provided closure will never be invoked.
///
/// Note that this method consumes the future it is called on and returns a
/// wrapped version of it.
///
/// # Examples
///
/// ```
/// #![feature(async_await)]
/// use futures_async_combinators::future::{ready, map_err};
///
/// # futures::executor::block_on(async {
/// let future = ready(Err::<i32, i32>(1));
/// let future = map_err(future, |x| x + 3);
/// assert_eq!(future.await, Err(4));
/// # });
/// ```
///
/// Calling [`map_err`] on a successful future has
/// no effect:
///
/// ```
/// #![feature(async_await)]
/// use futures_async_combinators::future::{ready, map_err};
///
/// # futures::executor::block_on(async {
/// let future = ready(Ok::<i32, i32>(1));
/// let future = map_err(future, |x| x + 3);
/// assert_eq!(future.await, Ok(1));
/// # });
/// ```
pub async fn map_err<Fut, F, T, E, U>(future: Fut, f: F) -> Result<T, U>
    where F: FnOnce(E) -> U,
          Fut: Future<Output = Result<T,E>>,
{
    future.await.map_err(f)
}

/// Flatten the execution of this future when the successful result of this
/// future is itself another future.
///
/// This can be useful when combining futures together to flatten the
/// computation out the final result. This method can only be called
/// when the successful result of this future itself implements the
/// `IntoFuture` trait and the error can be created from this future's error
/// type.
///
/// This method is roughly equivalent to `and_then(self, |x| x)`.
///
/// Note that this function consumes the receiving future and returns a
/// wrapped version of it.
///
/// # Examples
///
/// ```
/// #![feature(async_await)]
/// # futures::executor::block_on(async {
/// use futures_async_combinators::future::{ready, flatten};
///
/// let nested_future = ready(ready(1));
/// let future = flatten(nested_future);
/// assert_eq!(future.await, 1);
/// # });
/// ```
pub async fn flatten<FutA, FutB>(future: FutA) -> FutB::Output
    where FutA: Future<Output = FutB>,
          FutB: Future,
{
    let nested_future = future.await;
    nested_future.await
}

/// Do something with the output of a future before passing it on.
///
/// When using futures, you'll often chain several of them together.  While
/// working on such code, you might want to check out what's happening at
/// various parts in the pipeline, without consuming the intermediate
/// value. To do that, insert a call to `inspect`.
///
/// # Examples
///
/// ```
/// #![feature(async_await)]
/// # futures::executor::block_on(async {
/// use futures_async_combinators::future::{ready, inspect};
///
/// let future = ready(1);
/// let new_future = inspect(future, |&x| println!("about to resolve: {}", x));
/// assert_eq!(new_future.await, 1);
/// # });
/// ```
pub async fn inspect<Fut, F>(future: Fut, f: F) -> Fut::Output
    where Fut: Future,
          F: FnOnce(&Fut::Output),
{
    let future_result = future.await;
    f(&future_result);
    future_result
}

/// Maps this future's `Error` to a new error type
/// using the [`Into`](std::convert::Into) trait.
///
/// This method does for futures what the `?`-operator does for
/// [`Result`]: It lets the compiler infer the type of the resulting
/// error. Just as [`map_err`](map_err), this is useful for
/// example to ensure that futures have the same `Error`
/// type when using `select!` or `join!`.
///
/// Note that this method consumes the future it is called on and returns a
/// wrapped version of it.
///
/// # Examples
///
/// ```
/// #![feature(async_await)]
/// use futures_async_combinators::future::{ready, err_into};
///
/// # futures::executor::block_on(async {
/// let future_err_u8 = ready(Err::<(), u8>(1));
/// let future_err_i32 = err_into::<i32, _, _, _>(future_err_u8);
/// # });
/// ```
pub async fn err_into<U, Fut, T, E>(future: Fut) -> Result<T,U>
    where Fut: Future<Output = Result<T,E>>,
          E: Into<U>,
{
    future.await.map_err(Into::into)
}

/// Unwraps this future's ouput, producing a future with this future's
/// `Ok` type as its [`Output`](std::future::Future::Output) type.
///
/// If this future is resolved successfully, the returned future will
/// contain the original future's success value as output. Otherwise, the
/// closure `f` is called with the error value to produce an alternate
/// success value.
///
/// This method is similar to the [`Result::unwrap_or_else`] method.
///
/// # Examples
///
/// ```
/// #![feature(async_await)]
/// use futures_async_combinators::future::{ready, unwrap_or_else};
///
/// # futures::executor::block_on(async {
/// let future = ready(Err::<(), &str>("Boom!"));
/// let future = unwrap_or_else(future, |_| ());
/// assert_eq!(future.await, ());
/// # });
/// ```
pub async fn unwrap_or_else<Fut, T, E, F>(future: Fut, f: F) -> T
    where Fut: Future<Output = Result<T,E>>,
          F: FnOnce(E) -> T,
{
    future.await.unwrap_or_else(f)
}

/// Flatten the execution of this future when the successful result of this
/// future is a stream.
///
/// This can be useful when stream initialization is deferred, and it is
/// convenient to work with that stream as if stream was available at the
/// call site.
///
/// Note that this function consumes this future and returns a wrapped
/// version of it.
///
/// # Examples
///
/// ```
/// #![feature(async_await)]
/// # futures::executor::block_on(async {
/// use futures_async_combinators::future::{ready, flatten_stream};
/// use futures_async_combinators::stream::{collect, iter};
///
/// let stream_items = vec![17, 18, 19];
/// let future_of_a_stream = ready(iter(stream_items));
///
/// let stream = flatten_stream(future_of_a_stream);
/// let list: Vec<_> = collect(stream).await;
/// assert_eq!(list, vec![17, 18, 19]);
/// # });
/// ```
pub fn flatten_stream<Fut, St, T>(future: Fut) -> impl Stream<Item = T>
    where Fut: Future<Output = St>,
          St: Stream<Item = T>,
{
    use crate::stream::next;
    futures::stream::unfold((Some(future), None), async move | (future, stream)| {
        match (future, stream) {
            (Some(future), None) => {
                let stream = future.await;
                let mut stream = Box::pin(stream);
                let item = next(&mut stream).await;
                item.map(|item| (item, (None, Some(stream))))
            },
            (None, Some(mut stream)) => {
                let item = next(&mut stream).await;
                item.map(|item| (item, (None, Some(stream))))
            },
            _ => unreachable!()
        }
    })
}

/// Convert this future into a single element stream.
///
/// The returned stream contains single success if this future resolves to
/// success or single error if this future resolves into error.
///
/// # Examples
///
/// ```
/// #![feature(async_await)]
/// # futures::executor::block_on(async {
/// use futures_async_combinators::future::{ready, into_stream};
/// use futures_async_combinators::stream::collect;
///
/// let future = ready(17);
/// let stream = into_stream(future);
/// let collected: Vec<_> = collect(stream).await;
/// assert_eq!(collected, vec![17]);
/// # });
/// ```
pub fn into_stream<Fut>(future: Fut) -> impl Stream<Item = Fut::Output>
    where Fut: Future,
{
    futures::stream::unfold(Some(future), async move |future| {
        if let Some(future) = future {
            let item = future.await;
            Some((item, (None)))
        } else {
            None
        }
    })
}

/// Creates a new future wrapping around a function returning [`Poll`](core::task::Poll).
///
/// Polling the returned future delegates to the wrapped function.
///
/// # Examples
///
/// ```
/// #![feature(async_await)]
/// # futures::executor::block_on(async {
/// use futures_async_combinators::future::poll_fn;
/// use core::task::{Poll, Context};
///
/// fn read_line(_cx: &mut Context<'_>) -> Poll<String> {
///     Poll::Ready("Hello, World!".into())
/// }
///
/// let read_future = poll_fn(read_line);
/// assert_eq!(read_future.await, "Hello, World!".to_owned());
/// # });
/// ```
pub fn poll_fn<F, T>(f: F) -> impl Future<Output = T>
    where F: FnMut(&mut Context) -> Poll<T>,
{
    use std::future::from_generator;
    use std::future::get_task_context;

    from_generator(|| {
        let mut f = f;
        loop {
            match get_task_context(|context: &mut Context| f(context)) {
                Poll::Pending => yield,
                Poll::Ready(value) => return value,
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use futures::executor;
    use crate::future::*;

    #[test]
    fn test_ready() {
        executor::block_on(async {
            let future = ready(1);
            assert_eq!(future.await, 1);
        });
    }

    #[test]
    fn test_map() {
        executor::block_on(async {
            let future = ready(1);
            let new_future = map(future, |x| x + 3);
            assert_eq!(new_future.await, 4);
        });
    }

    #[test]
    fn test_then() {
        executor::block_on(async {
            let future = ready(1);
            let new_future = then(future, |x| ready(x + 3));
            assert_eq!(new_future.await, 4);
        });
    }

    #[test]
    fn test_and_then_ok() {
        executor::block_on(async {
            let future = ready(Ok::<i32, i32>(1));
            let new_future = and_then(future, |x| ready(Ok::<i32, i32>(x + 3)));
            assert_eq!(new_future.await, Ok(4));
        });
    }

    #[test]
    fn test_and_then_err() {
        executor::block_on(async {
            let future = ready(Err::<i32, i32>(1));
            let new_future = and_then(future, |x| ready(Ok::<i32, i32>(x + 3)));
            assert_eq!(new_future.await, Err(1));
        });
    }

    #[test]
    fn test_or_else() {
        executor::block_on(async {
            let future = ready(Err::<i32, i32>(1));
            let new_future = or_else(future, |x| ready(Err::<i32, i32>(x + 3)));
            assert_eq!(new_future.await, Err(4));
        });
    }

    #[test]
    fn test_map_ok() {
        executor::block_on(async {
            let future = ready(Ok::<i32, i32>(1));
            let new_future = map_ok(future, |x| x + 3);
            assert_eq!(new_future.await, Ok(4));
        });
    }

    #[test]
    fn test_map_err() {
        executor::block_on(async {
            let future = ready(Err::<i32, i32>(1));
            let new_future = map_err(future, |x| x + 3);
            assert_eq!(new_future.await, Err(4));
        });
    }

    #[test]
    fn test_flatten() {
        executor::block_on(async {
            let nested_future = ready(ready(1));
            let future = flatten(nested_future);
            assert_eq!(future.await, 1);
        });
    }

    #[test]
    fn test_inspect() {
        executor::block_on(async {
            let future = ready(1);
            let new_future = inspect(future, |&x| assert_eq!(x, 1));
            assert_eq!(new_future.await, 1);
        });
    }

    #[test]
    fn test_err_into() {
        executor::block_on(async {
            let future_err_u8 = ready(Err::<(), u8>(1));
            let future_err_i32 = err_into::<i32, _, _, _>(future_err_u8);

            assert_eq!(future_err_i32.await, Err::<(), i32>(1));
        });
    }

    #[test]
    fn test_unwrap_or_else() {
        executor::block_on(async {
            let future = ready(Err::<(), &str>("Boom!"));
            let new_future = unwrap_or_else(future, |_| ());
            assert_eq!(new_future.await, ());
        });
    }

    #[test]
    fn test_flatten_stream() {
        use crate::stream::{collect, iter};
        executor::block_on(async {
            let stream_items = vec![17, 18, 19];
            let future_of_a_stream = ready(iter(stream_items));

            let stream = flatten_stream(future_of_a_stream);
            let list: Vec<_> = collect(stream).await;
            assert_eq!(list, vec![17, 18, 19]);
        });
    }

    #[test]
    fn test_into_stream() {
        use crate::stream::collect;
        executor::block_on(async {
            let future = ready(17);
            let stream = into_stream(future);
            let collected: Vec<_> = collect(stream).await;
            assert_eq!(collected, vec![17]);
        });
    }

    #[test]
    fn test_poll_fn() {
        executor::block_on(async {
            let read_line = |_context: &mut Context| -> Poll<String> {
                Poll::Ready("Hello, World!".into())
            };

            let read_future = poll_fn(read_line);
            assert_eq!(read_future.await, "Hello, World!".to_owned());
        });
    }

}
