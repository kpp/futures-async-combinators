use core::future::Future;
use futures::stream::Stream;

use async_trait::async_trait;
use core::task::{Context, Poll};

/// Create a future that is immediately ready with a value.
///
/// # Examples
///
/// ```
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

impl<T: ?Sized> FutureExt for T where T: Future {}

#[async_trait]
pub trait FutureExt: Future {
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
    /// # futures::executor::block_on(async {
    /// use futures_async_combinators::future::{ready, FutureExt};
    ///
    /// let future = ready(1);
    /// let new_future = future.map(|x| x + 3);
    /// assert_eq!(new_future.await, 4);
    /// # });
    /// ```
    async fn map<U, F>(self, f: F) -> U
    where
        F: FnOnce(Self::Output) -> U + Send,
        Self: Sized,
    {
        f(self.await)
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
    /// # futures::executor::block_on(async {
    /// use futures_async_combinators::future::{ready, FutureExt};
    ///
    /// let future_of_1 = ready(1);
    /// let future_of_4 = future_of_1.then(|x| ready(x + 3));
    /// assert_eq!(future_of_4.await, 4);
    /// # });
    /// ```
    async fn then<Fut, F>(self, f: F) -> Fut::Output
    where
        F: FnOnce(Self::Output) -> Fut + Send,
        Fut: Future + Send,
        Self: Sized,
    {
        let new_future = f(self.await);
        new_future.await
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
    /// This method is roughly equivalent to `future.and_then(|x| x)`.
    ///
    /// Note that this function consumes the receiving future and returns a
    /// wrapped version of it.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use futures_async_combinators::future::{ready, FutureExt};
    ///
    /// let nested_future = ready(ready(1));
    /// let future = nested_future.flatten();
    /// assert_eq!(future.await, 1);
    /// # });
    /// ```
    async fn flatten(self) -> <Self::Output as Future>::Output
    where
        Self::Output: Future + Send,
        Self: Sized,
    {
        let nested_future = self.await;
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
    /// # futures::executor::block_on(async {
    /// use futures_async_combinators::future::{ready, FutureExt};
    ///
    /// let future = ready(1);
    /// let new_future = future.inspect(|&x| println!("about to resolve: {}", x));
    /// assert_eq!(new_future.await, 1);
    /// # });
    /// ```
    async fn inspect<F>(self, f: F) -> Self::Output
    where
        F: FnOnce(&Self::Output) + Send,
        Self: Sized,
    {
        let future_result = self.await;
        f(&future_result);
        future_result
    }
}

impl<T, E, Fut: ?Sized> TryFutureExt<T, E> for Fut where Fut: Future<Output = Result<T, E>> {}

#[async_trait]
pub trait TryFutureExt<T, E>: Future<Output = Result<T, E>> {
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
    /// use futures_async_combinators::future::{ready, TryFutureExt};
    ///
    /// # futures::executor::block_on(async {
    /// let future = ready(Ok::<i32, i32>(1));
    /// let future = future.and_then(|x| ready(Ok::<i32, i32>(x + 3)));
    /// assert_eq!(future.await, Ok(4));
    /// # });
    /// ```
    ///
    /// Calling [`TryFutureExt::and_then`] on an errored future has no
    /// effect:
    ///
    /// ```
    /// use futures_async_combinators::future::{ready, TryFutureExt};
    ///
    /// # futures::executor::block_on(async {
    /// let future = ready(Err::<i32, i32>(1));
    /// let future = future.and_then(|x| ready(Err::<i32, i32>(x + 3)));
    /// assert_eq!(future.await, Err(1));
    /// # });
    /// ```
    async fn and_then<U, F, FutB>(self, f: F) -> Result<U, E>
    where
        F: FnOnce(T) -> FutB + Send,
        FutB: Future<Output = Result<U, E>> + Send,
        Self: Sized,
        T: Send + 'async_trait,
        E: Send + 'async_trait,
    {
        match self.await {
            Ok(ok) => {
                let new_future = f(ok);
                new_future.await
            }
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
    /// use futures_async_combinators::future::{ready, TryFutureExt};
    ///
    /// # futures::executor::block_on(async {
    /// let future = ready(Err::<i32, i32>(1));
    /// let future = future.or_else(|x| ready(Err::<i32, i32>(x + 3)));
    /// assert_eq!(future.await, Err(4));
    /// # });
    /// ```
    ///
    /// Calling [`TryFutureExt::or_else`] on a successful future has
    /// no effect:
    ///
    /// ```
    /// use futures_async_combinators::future::{ready, TryFutureExt};
    ///
    /// # futures::executor::block_on(async {
    /// let future = ready(Ok::<i32, i32>(1));
    /// let future = future.or_else(|x| ready(Ok::<i32, i32>(x + 3)));
    /// assert_eq!(future.await, Ok(1));
    /// # });
    /// ```
    async fn or_else<U, F, FutB>(self, f: F) -> Result<T, U>
    where
        F: FnOnce(E) -> FutB + Send,
        FutB: Future<Output = Result<T, U>> + Send,
        Self: Sized,
        T: Send + 'async_trait,
        E: Send + 'async_trait,
    {
        match self.await {
            Ok(ok) => Ok(ok),
            Err(err) => {
                let new_future = f(err);
                new_future.await
            }
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
    /// use futures_async_combinators::future::{ready, TryFutureExt};
    ///
    /// # futures::executor::block_on(async {
    /// let future = ready(Ok::<i32, i32>(1));
    /// let future = future.map_ok(|x| x + 3);
    /// assert_eq!(future.await, Ok(4));
    /// # });
    /// ```
    ///
    /// Calling [`TryFutureExt::map_ok`] on an errored future has no
    /// effect:
    ///
    /// ```
    /// use futures_async_combinators::future::{ready, TryFutureExt};
    ///
    /// # futures::executor::block_on(async {
    /// let future = ready(Err::<i32, i32>(1));
    /// let future = future.map_ok(|x| x + 3);
    /// assert_eq!(future.await, Err(1));
    /// # });
    /// ```
    async fn map_ok<U, F>(self, f: F) -> Result<U, E>
    where
        F: FnOnce(T) -> U + Send,
        Self: Sized,
        T: Send + 'async_trait,
        E: Send + 'async_trait,
    {
        self.await.map(f)
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
    /// use futures_async_combinators::future::{ready, TryFutureExt};
    ///
    /// # futures::executor::block_on(async {
    /// let future = ready(Err::<i32, i32>(1));
    /// let future = future.map_err(|x| x + 3);
    /// assert_eq!(future.await, Err(4));
    /// # });
    /// ```
    ///
    /// Calling [`TryFutureExt::map_err`] on a successful future has
    /// no effect:
    ///
    /// ```
    /// use futures_async_combinators::future::{ready, TryFutureExt};
    ///
    /// # futures::executor::block_on(async {
    /// let future = ready(Ok::<i32, i32>(1));
    /// let future = future.map_err(|x| x + 3);
    /// assert_eq!(future.await, Ok(1));
    /// # });
    /// ```
    async fn map_err<U, F>(self, f: F) -> Result<T, U>
    where
        F: FnOnce(E) -> U + Send,
        Self: Sized,
        T: Send + 'async_trait,
        E: Send + 'async_trait,
    {
        self.await.map_err(f)
    }

    /// Maps this future's `Error` to a new error type
    /// using the [`Into`](std::convert::Into) trait.
    ///
    /// This method does for futures what the `?`-operator does for
    /// [`Result`]: It lets the compiler infer the type of the resulting
    /// error. Just as [`TryFutureExt::map_err`], this is useful for
    /// example to ensure that futures have the same `Error`
    /// type when using `select!` or `join!`.
    ///
    /// Note that this method consumes the future it is called on and returns a
    /// wrapped version of it.
    ///
    /// # Examples
    ///
    /// ```
    /// use futures_async_combinators::future::{ready, TryFutureExt};
    ///
    /// # futures::executor::block_on(async {
    /// let future_err_u8 = ready(Err::<(), u8>(1));
    /// let future_err_i32 = future_err_u8.err_into::<i32>();
    /// # });
    /// ```
    async fn err_into<U>(self) -> Result<T, U>
    where
        E: Into<U>,
        Self: Sized,
        T: Send + 'async_trait,
        E: Send + 'async_trait,
    {
        self.await.map_err(Into::into)
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
    /// use futures_async_combinators::future::{ready, TryFutureExt};
    ///
    /// # futures::executor::block_on(async {
    /// let future = ready(Err::<(), &str>("Boom!"));
    /// let future = future.unwrap_or_else(|_| ());
    /// assert_eq!(future.await, ());
    /// # });
    /// ```
    async fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce(E) -> T + Send,
        Self: Sized,
        T: Send + 'async_trait,
        E: Send + 'async_trait,
    {
        self.await.unwrap_or_else(f)
    }
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
where
    Fut: Future<Output = St>,
    St: Stream<Item = T>,
{
    use crate::stream::next;
    futures::stream::unfold((Some(future), None), async move |(future, stream)| {
        match (future, stream) {
            (Some(future), None) => {
                let stream = future.await;
                let mut stream = Box::pin(stream);
                let item = next(&mut stream).await;
                item.map(|item| (item, (None, Some(stream))))
            }
            (None, Some(mut stream)) => {
                let item = next(&mut stream).await;
                item.map(|item| (item, (None, Some(stream))))
            }
            _ => unreachable!(),
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
where
    Fut: Future,
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
where
    F: FnMut(&mut Context) -> Poll<T>,
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
    use crate::future::*;
    use futures::executor;

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
            let new_future = future.map(|x| x + 3);
            assert_eq!(new_future.await, 4);
        });
    }

    #[test]
    fn test_then() {
        executor::block_on(async {
            let future = ready(1);
            let new_future = future.then(|x| ready(x + 3));
            assert_eq!(new_future.await, 4);
        });
    }

    #[test]
    fn test_and_then_ok() {
        executor::block_on(async {
            let future = ready(Ok::<i32, i32>(1));
            let new_future = future.and_then(|x| ready(Ok::<i32, i32>(x + 3)));
            assert_eq!(new_future.await, Ok(4));
        });
    }

    #[test]
    fn test_and_then_err() {
        executor::block_on(async {
            let future = ready(Err::<i32, i32>(1));
            let new_future = future.and_then(|x| ready(Ok(x + 3)));
            assert_eq!(new_future.await, Err(1));
        });
    }

    #[test]
    fn test_or_else() {
        executor::block_on(async {
            let future = ready(Err::<i32, i32>(1));
            let new_future = future.or_else(|x| ready(Err(x + 3)));
            assert_eq!(new_future.await, Err(4));
        });
    }

    #[test]
    fn test_map_ok() {
        executor::block_on(async {
            let future = ready(Ok::<i32, i32>(1));
            let new_future = future.map_ok(|x| x + 3);
            assert_eq!(new_future.await, Ok(4));
        });
    }

    #[test]
    fn test_map_err() {
        executor::block_on(async {
            let future = ready(Err::<i32, i32>(1));
            let new_future = future.map_err(|x| x + 3);
            assert_eq!(new_future.await, Err(4));
        });
    }

    #[test]
    fn test_flatten() {
        executor::block_on(async {
            let nested_future = ready(ready(1));
            let future = nested_future.flatten();
            assert_eq!(future.await, 1);
        });
    }

    #[test]
    fn test_inspect() {
        executor::block_on(async {
            let future = ready(1);
            let new_future = future.inspect(|&x| assert_eq!(x, 1));
            assert_eq!(new_future.await, 1);
        });
    }

    #[test]
    fn test_err_into() {
        executor::block_on(async {
            let future_err_u8 = ready(Err::<(), u8>(1));
            let future_err_i32 = future_err_u8.err_into();

            assert_eq!(future_err_i32.await, Err::<(), i32>(1));
        });
    }

    #[test]
    fn test_unwrap_or_else() {
        executor::block_on(async {
            let future = ready(Err::<(), &str>("Boom!"));
            let new_future = future.unwrap_or_else(|_| ());
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
            let read_line =
                |_context: &mut Context| -> Poll<String> { Poll::Ready("Hello, World!".into()) };

            let read_future = poll_fn(read_line);
            assert_eq!(read_future.await, "Hello, World!".to_owned());
        });
    }
}
