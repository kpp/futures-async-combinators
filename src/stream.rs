use futures::stream::Stream;
use core::pin::Pin;

pub async fn next<St>(stream: &mut St) -> Option<St::Item>
    where St: Stream + Unpin,
{
    use futures::future::poll_fn;
    use futures::task::Waker;
    let poll_next = |waker: &Waker| Pin::new(&mut *stream).poll_next(waker);
    let future_next = poll_fn(poll_next);
    await!(future_next)
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

}
