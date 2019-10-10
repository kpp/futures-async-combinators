use criterion::*;
use futures::executor;

fn bench_ready(c: &mut Criterion) {
    executor::block_on(async {
        let mut group = c.benchmark_group("future::ready");

        group.bench_function("futures", |b| {
            b.iter(move || async {
                black_box(futures::future::ready(42)).await
            })
        });
        group.bench_function("async_combinators", |b| {
            b.iter(move || async {
                black_box(futures_async_combinators::future::ready(42)).await
            })
        });

        group.finish();
    });
}

fn bench_poll_fn(c: &mut Criterion) {
    use core::task::{Context, Poll};

    fn ret_42(_cx: &mut Context<'_>) -> Poll<i32> {
        Poll::Ready(42)
    }

    executor::block_on(async {
        let mut group = c.benchmark_group("future::poll_fn");

        group.bench_function("futures", |b| {
            b.iter(move || async {
                black_box(futures::future::poll_fn(ret_42)).await
            })
        });
        group.bench_function("async_combinators", |b| {
            b.iter(move || async {
                black_box(futures_async_combinators::future::poll_fn(ret_42)).await
            })
        });

        group.finish();
    });
}

fn bench_map(c: &mut Criterion) {
    executor::block_on(async {
        let mut group = c.benchmark_group("future::map");

        group.bench_function("futures", |b| {
            b.iter(move || async {
                use futures::future::*;
                let fut = ready(40);
                let fut = fut.map(|x| x + 2);
                black_box(fut).await
            })
        });
        group.bench_function("async_combinators", |b| {
            b.iter(move || async {
                use futures_async_combinators::future::*;
                let fut = ready(40);
                let fut = map(fut, |x| x + 2);
                black_box(fut).await
            })
        });

        group.finish();
    });
}

criterion_group!(benches, bench_ready, bench_poll_fn, bench_map);
criterion_main!(benches);
