# futures-async-combinators

[![Build Status](https://travis-ci.com/kpp/futures-async-combinators.svg?branch=master)](https://travis-ci.com/kpp/futures-async-combinators)

FOR LEARNING PURPOSES ONLY

This is a greatly simplified implementation of [https://docs.rs/futures-preview/0.3.0-alpha.13/futures/](Future combinators)
like `FutureExt::map`, `TryFutureExt::and_then`...

# Requirements

Rust nightly-2019-02-19 for async_await, await_macro...

# Why

To understand how combinators work by looking at clean source code. Compare:

```rust
pub async fn then<...>(future, f) -> ...
{
    let future_result = await!(future);
    let new_future = f(future_result);
    await!(new_future)
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
