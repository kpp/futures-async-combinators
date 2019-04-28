#![feature(async_closure)]

#[macro_use]
extern crate criterion;
use criterion::{black_box, Benchmark, Criterion};

use futures::executor;

fn bench_stream_iter(c: &mut Criterion) {
    executor::block_on(async {
        c.bench(
            "stream::iter",
            Benchmark::new("futures", |b| {
                b.iter(async move || {
                    use futures::stream::{iter, StreamExt};
                    let mut stream = iter(1..=1000);
                    while let Some(item) = stream.next().await {
                        black_box(item);
                    }
                })
            })
            .with_function("async_combinators", |b| {
                b.iter(async move || {
                    use futures::stream::StreamExt;
                    use futures_async_combinators::stream::iter;
                    let mut stream = iter(1..=1000);
                    while let Some(item) = stream.next().await {
                        black_box(item);
                    }
                })
            }),
        );
    });
}

fn bench_stream_next(c: &mut Criterion) {
    executor::block_on(async {
        c.bench(
            "stream::next",
            Benchmark::new("futures", |b| {
                b.iter(async move || {
                    use futures::stream::{iter, StreamExt};
                    let mut stream = iter(1..=1000);
                    while let Some(item) = stream.next().await {
                        black_box(item);
                    }
                })
            })
            .with_function("async_combinators", |b| {
                b.iter(async move || {
                    use futures::stream::iter;
                    use futures_async_combinators::stream::next;
                    let mut stream = iter(1..=1000);
                    while let Some(item) = next(&mut stream).await {
                        black_box(item);
                    }
                })
            }),
        );
    });
}

fn bench_stream_collect(c: &mut Criterion) {
    executor::block_on(async {
        c.bench(
            "stream::collect",
            Benchmark::new("futures", |b| {
                b.iter(async move || {
                    use futures::stream::{iter, StreamExt};
                    let stream = iter(1..=1000);
                    let vec: Vec<_> = stream.collect().await;
                    black_box(vec)
                })
            })
            .with_function("async_combinators", |b| {
                b.iter(async move || {
                    use futures::stream::iter;
                    use futures_async_combinators::stream::collect;
                    let stream = iter(1..=1000);
                    let vec: Vec<_> = collect(stream).await;
                    black_box(vec)
                })
            }),
        );
    });
}

fn bench_stream_map(c: &mut Criterion) {
    executor::block_on(async {
        c.bench(
            "stream::map",
            Benchmark::new("futures", |b| {
                b.iter(async move || {
                    use futures::stream::{iter, StreamExt};
                    let stream = iter(1..=1000);
                    let stream = stream.map(|x| x + 42);
                    let vec: Vec<_> = stream.collect().await;
                    black_box(vec)
                })
            })
            .with_function("async_combinators", |b| {
                b.iter(async move || {
                    use futures::stream::{iter, StreamExt};
                    use futures_async_combinators::stream::map;
                    let stream = iter(1..=1000);
                    let stream = map(stream, |x| x + 42);
                    let vec: Vec<_> = stream.collect().await;
                    black_box(vec)
                })
            }),
        );
    });
}

criterion_group!(
    benches,
    bench_stream_iter,
    bench_stream_next,
    bench_stream_collect,
    bench_stream_map,
);
criterion_main!(benches);
