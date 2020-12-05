# futures-async-combinators

[![Build Status][ci-badge]][ci-url]
[![Coverage Status][cov-badge]][cov-url]
[![Docs][doc-badge]][doc-url]

[ci-badge]: https://github.com/kpp/futures-async-combinators/workflows/CI/badge.svg?branch=master
[ci-url]: https://github.com/kpp/futures-async-combinators/actions
[cov-badge]: https://codecov.io/gh/kpp/futures-async-combinators/branch/master/graph/badge.svg?token=X29P76ICLS
[cov-url]: https://codecov.io/gh/kpp/futures-async-combinators
[doc-badge]: https://docs.rs/futures-async-combinators/badge.svg
[doc-url]: https://docs.rs/futures-async-combinators

FOR LEARNING PURPOSES ONLY

This is a greatly simplified implementation of [Future combinators](https://docs.rs/futures/0.3.8/futures/)
like `FutureExt::map`, `TryFutureExt::and_then`...

# Requirements

Rust 1.39.0 for async/await.

# State

Future

- [x] future::and_then
- [x] future::err_into
- [x] future::flatten
- [x] future::flatten_stream
- [x] future::inspect
- [x] future::into_stream
- [x] future::map
- [x] future::map_err
- [x] future::map_ok
- [x] future::or_else
- [x] future::poll_fn
- [x] future::ready
- [x] future::then
- [x] future::unwrap_or_else

Stream

- [x] stream::chain
- [x] stream::collect
- [x] stream::concat
- [x] stream::filter
- [x] stream::filter_map
- [x] stream::flatten
- [x] stream::fold
- [x] stream::for_each
- [x] stream::inspect
- [x] stream::into_future
- [x] stream::iter
- [x] stream::map
- [x] stream::next
- [x] stream::once
- [x] stream::poll_fn
- [x] stream::repeat
- [x] stream::skip
- [x] stream::skip_while
- [x] stream::take
- [x] stream::take_while
- [x] stream::then
- [x] stream::unfold
- [x] stream::zip


# Why

To understand how combinators work by looking at clean source code. Compare:

```rust
pub async fn then<...>(future, f) -> ...
{
    let new_future = f(future.await);
    new_future.await
}
```

with original

```rust
impl Then
{
    fn poll(self, waker) -> Poll<...> {
        self.as_mut().chain().poll(waker, |output, f| f(output))
    }
}

impl Chain
{
    fn poll(self, waker, f) -> Poll<...>
    {
        let mut f = Some(f);

        // Safe to call `get_unchecked_mut` because we won't move the futures.
        let this = unsafe { Pin::get_unchecked_mut(self) };

        loop {
            let (output, data) = match this {
                Chain::First(fut1, data) => {
                    match unsafe { Pin::new_unchecked(fut1) }.poll(waker) {
                        Poll::Pending => return Poll::Pending,
                        Poll::Ready(output) => (output, data.take().unwrap()),
                    }
                }
                Chain::Second(fut2) => {
                    return unsafe { Pin::new_unchecked(fut2) }.poll(waker);
                }
                Chain::Empty => unreachable!()
            };

            *this = Chain::Empty; // Drop fut1
            let fut2 = (f.take().unwrap())(output, data);
            *this = Chain::Second(fut2)
        }
    }
}
```
