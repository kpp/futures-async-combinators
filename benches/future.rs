#![feature(async_closure)]

#[macro_use]
extern crate criterion;
use criterion::{black_box, Benchmark, Criterion};

use futures::executor;

fn bench_ready(c: &mut Criterion) {
    executor::block_on(async {
        c.bench(
            "future::ready",
            Benchmark::new("futures", |b| {
                b.iter(async move || black_box(futures::future::ready(42)).await)
            })
            .with_function("async_combinators", |b| {
                b.iter(async move || black_box(futures_async_combinators::future::ready(42)).await)
            }),
        );
    });
}

fn bench_poll_fn(c: &mut Criterion) {
    use core::task::{Context, Poll};

    fn ret_42(_cx: &mut Context<'_>) -> Poll<i32> {
        Poll::Ready(42)
    }

    executor::block_on(async {
        c.bench(
            "future::poll_fn",
            Benchmark::new("futures", |b| {
                b.iter(async move || black_box(futures::future::poll_fn(ret_42)).await)
            })
            .with_function("async_combinators", |b| {
                b.iter(async move || {
                    black_box(futures_async_combinators::future::poll_fn(ret_42)).await
                })
            }),
        );
    });
}

fn bench_map(c: &mut Criterion) {
    executor::block_on(async {
        c.bench(
            "future::map",
            Benchmark::new("futures", |b| {
                b.iter(async move || {
                    use futures::future::*;
                    let fut = ready(40);
                    let fut = fut.map(|x| x + 2);
                    black_box(fut).await
                })
            })
            .with_function("async_combinators", |b| {
                b.iter(async move || {
                    use futures_async_combinators::future::*;
                    let fut = ready(40);
                    let fut = map(fut, |x| x + 2);
                    black_box(fut).await
                })
            }),
        );
    });
}

criterion_group!(benches, bench_ready, bench_poll_fn, bench_map);
criterion_main!(benches);
