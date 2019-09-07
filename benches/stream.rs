#![feature(async_closure)]

use criterion::*;
use futures::executor;

fn bench_stream_iter(c: &mut Criterion) {
    executor::block_on(async {
        let mut group = c.benchmark_group("stream::iter");

        group.bench_function("futures", |b| {
            b.iter(async move || {
                use futures::stream::{iter, StreamExt};
                let mut stream = iter(1..=1000);
                while let Some(item) = stream.next().await {
                    black_box(item);
                }
            })
        });
        group.bench_function("async_combinators", |b| {
            b.iter(async move || {
                use futures::stream::StreamExt;
                use futures_async_combinators::stream::iter;
                let mut stream = iter(1..=1000);
                while let Some(item) = stream.next().await {
                    black_box(item);
                }
            })
        });

        group.finish();
    });
}

fn bench_stream_next(c: &mut Criterion) {
    executor::block_on(async {
        let mut group = c.benchmark_group("stream::next");

        group.bench_function("futures", |b| {
            b.iter(async move || {
                use futures::stream::{iter, StreamExt};
                let mut stream = iter(1..=1000);
                while let Some(item) = stream.next().await {
                    black_box(item);
                }
            })
        });
        group.bench_function("async_combinators", |b| {
            b.iter(async move || {
                use futures::stream::iter;
                use futures_async_combinators::stream::next;
                let mut stream = iter(1..=1000);
                while let Some(item) = next(&mut stream).await {
                    black_box(item);
                }
            })
        });

        group.finish();
    });
}

fn bench_stream_collect(c: &mut Criterion) {
    executor::block_on(async {
        let mut group = c.benchmark_group("stream::collect");

        group.bench_function("futures", |b| {
            b.iter(async move || {
                use futures::stream::{iter, StreamExt};
                let stream = iter(1..=1000);
                let vec: Vec<_> = stream.collect().await;
                black_box(vec)
            })
        });
        group.bench_function("async_combinators", |b| {
            b.iter(async move || {
                use futures::stream::iter;
                use futures_async_combinators::stream::collect;
                let stream = iter(1..=1000);
                let vec: Vec<_> = collect(stream).await;
                black_box(vec)
            })
        });

        group.finish();
    });
}

fn bench_stream_map(c: &mut Criterion) {
    executor::block_on(async {
        let mut group = c.benchmark_group("stream::map");

        group.bench_function("futures", |b| {
            b.iter(async move || {
                use futures::stream::{iter, StreamExt};
                let stream = iter(1..=1000);
                let stream = stream.map(|x| x + 42);
                let vec: Vec<_> = stream.collect().await;
                black_box(vec)
            })
        });
        group.bench_function("async_combinators", |b| {
            b.iter(async move || {
                use futures::stream::{iter, StreamExt};
                use futures_async_combinators::stream::map;
                let stream = iter(1..=1000);
                let stream = map(stream, |x| x + 42);
                let vec: Vec<_> = stream.collect().await;
                black_box(vec)
            })
        });

        group.finish();
    });
}

fn bench_stream_fold(c: &mut Criterion) {
    executor::block_on(async {
        let mut group = c.benchmark_group("stream::fold");

        group.bench_function("futures", |b| {
            b.iter(async move || {
                use futures::stream::{iter, StreamExt};
                let stream = iter(1..=1000);
                let acc = stream.fold(0, async move |acc, x| acc + x);
                black_box(acc).await
            })
        });
        group.bench_function("async_combinators", |b| {
            b.iter(async move || {
                use futures::stream::iter;
                use futures_async_combinators::stream::fold;
                let stream = iter(1..=1000);
                let acc = fold(stream, 0, async move |acc, x| acc + x);
                black_box(acc).await
            })
        });

        group.finish();
    });
}

criterion_group!(
    benches,
    bench_stream_iter,
    bench_stream_next,
    bench_stream_collect,
    bench_stream_map,
    bench_stream_fold,
);
criterion_main!(benches);
