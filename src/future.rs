use futures::future::Future;

pub async fn ready<T>(value: T) -> T {
    value
}

pub async fn map<Fut, U, F>(future: Fut, f: F) -> U
    where F: FnOnce(Fut::Output) -> U,
          Fut: Future,
{
    let future_result = await!(future);
    f(future_result)
}

pub async fn then<FutA, FutB, F>(future: FutA, f: F) -> FutB::Output
    where F: FnOnce(FutA::Output) -> FutB,
          FutA: Future,
          FutB: Future,
{
    let future_result = await!(future);
    let new_future = f(future_result);
    await!(new_future)
}

pub async fn and_then<FutA, FutB, F, T, U, E>(future: FutA, f: F) -> Result<U, E>
    where F: FnOnce(T) -> FutB,
          FutA: Future<Output = Result<T,E>>,
          FutB: Future<Output = Result<U,E>>,
{
    let future_result = await!(future);
    match future_result {
        Ok(ok) => {
            let new_future = f(ok);
            await!(new_future)
        },
        Err(err) => Err(err),
    }
}

pub async fn or_else<FutA, FutB, F, T, E, U>(future: FutA, f: F) -> Result<T, U>
    where F: FnOnce(E) -> FutB,
          FutA: Future<Output = Result<T,E>>,
          FutB: Future<Output = Result<T,U>>,
{
    let future_result = await!(future);
    match future_result {
        Ok(ok) => Ok(ok),
        Err(err) => {
            let new_future = f(err);
            await!(new_future)
        },
    }
}

pub async fn map_ok<Fut, F, T, U, E>(future: Fut, f: F) -> Result<U, E>
    where F: FnOnce(T) -> U,
          Fut: Future<Output = Result<T,E>>,
{
    let future_result = await!(future);
    future_result.map(f)
}

pub async fn map_err<Fut, F, T, E, U>(future: Fut, f: F) -> Result<T, U>
    where F: FnOnce(E) -> U,
          Fut: Future<Output = Result<T,E>>,
{
    let future_result = await!(future);
    future_result.map_err(f)
}

pub async fn flatten<FutA, FutB>(future: FutA) -> FutB::Output
    where FutA: Future<Output = FutB>,
          FutB: Future,
{
    let nested_future = await!(future);
    await!(nested_future)
}

pub async fn inspect<Fut, F>(future: Fut, f: F) -> Fut::Output
    where Fut: Future,
          F: FnOnce(&Fut::Output),
{
    let future_result = await!(future);
    f(&future_result);
    future_result
}


#[cfg(test)]
mod tests {
    use futures::executor;
    use crate::future::*;

    #[test]
    fn test_ready() {
        executor::block_on(async {
            let future = ready(1);
            assert_eq!(await!(future), 1);
        });
    }

    #[test]
    fn test_map() {
        executor::block_on(async {
            let future = ready(1);
            let new_future = map(future, |x| x + 3);
            assert_eq!(await!(new_future), 4);
        });
    }

    #[test]
    fn test_then() {
        executor::block_on(async {
            let future = ready(1);
            let new_future = then(future, |x| ready(x + 3));
            assert_eq!(await!(new_future), 4);
        });
    }

    #[test]
    fn test_and_then() {
        executor::block_on(async {
            let future = ready(Ok::<i32, i32>(1));
            let new_future = and_then(future, |x| ready(Ok::<i32, i32>(x + 3)));
            assert_eq!(await!(new_future), Ok(4));
        });
    }

    #[test]
    fn test_or_else() {
        executor::block_on(async {
            let future = ready(Err::<i32, i32>(1));
            let new_future = or_else(future, |x| ready(Err::<i32, i32>(x + 3)));
            assert_eq!(await!(new_future), Err(4));
        });
    }

    #[test]
    fn test_map_ok() {
        executor::block_on(async {
            let future = ready(Ok::<i32, i32>(1));
            let new_future = map_ok(future, |x| x + 3);
            assert_eq!(await!(new_future), Ok(4));
        });
    }

    #[test]
    fn test_map_err() {
        executor::block_on(async {
            let future = ready(Err::<i32, i32>(1));
            let new_future = map_err(future, |x| x + 3);
            assert_eq!(await!(new_future), Err(4));
        });
    }

    #[test]
    fn test_flatten() {
        executor::block_on(async {
            let nested_future = ready(ready(1));
            let future = flatten(nested_future);
            assert_eq!(await!(future), 1);
        });
    }

    #[test]
    fn test_inspect() {
        executor::block_on(async {
            let future = ready(1);
            let new_future = inspect(future, |&x| assert_eq!(x, 1));
            assert_eq!(await!(new_future), 1);
        });
    }
}
