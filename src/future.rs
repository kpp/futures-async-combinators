use futures::future::Future;
use futures::stream::Stream;

use core::task::{Poll, Context};

pub async fn ready<T>(value: T) -> T {
    value
}

pub async fn map<Fut, U, F>(future: Fut, f: F) -> U
    where F: FnOnce(Fut::Output) -> U,
          Fut: Future,
{
    f(future.await)
}

pub async fn then<FutA, FutB, F>(future: FutA, f: F) -> FutB::Output
    where F: FnOnce(FutA::Output) -> FutB,
          FutA: Future,
          FutB: Future,
{
    let new_future = f(future.await);
    new_future.await
}

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

pub async fn map_ok<Fut, F, T, U, E>(future: Fut, f: F) -> Result<U, E>
    where F: FnOnce(T) -> U,
          Fut: Future<Output = Result<T,E>>,
{
    future.await.map(f)
}

pub async fn map_err<Fut, F, T, E, U>(future: Fut, f: F) -> Result<T, U>
    where F: FnOnce(E) -> U,
          Fut: Future<Output = Result<T,E>>,
{
    future.await.map_err(f)
}

pub async fn flatten<FutA, FutB>(future: FutA) -> FutB::Output
    where FutA: Future<Output = FutB>,
          FutB: Future,
{
    let nested_future = future.await;
    nested_future.await
}

pub async fn inspect<Fut, F>(future: Fut, f: F) -> Fut::Output
    where Fut: Future,
          F: FnOnce(&Fut::Output),
{
    let future_result = future.await;
    f(&future_result);
    future_result
}

pub async fn err_into<Fut, T, E, U>(future: Fut) -> Result<T,U>
    where Fut: Future<Output = Result<T,E>>,
          E: Into<U>,
{
    future.await.map_err(Into::into)
}

pub async fn unwrap_or_else<Fut, T, E, F>(future: Fut, f: F) -> T
    where Fut: Future<Output = Result<T,E>>,
          F: FnOnce(E) -> T,
{
    future.await.unwrap_or_else(f)
}

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
            let future_err_i32 = err_into::<_, _, _, i32>(future_err_u8);

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
