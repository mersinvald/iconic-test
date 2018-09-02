extern crate test_project;
extern crate rand;
#[macro_use]
extern crate criterion;

use rand::prelude::*;
use test_project::*;
use criterion::Criterion;
use std::{u32, i32};

fn random_meta() -> impl Iterator<Item = (Size, Meta)> {
    let mut rng = thread_rng();
    std::iter::repeat_with(move || {
        let size = rng.gen_range(1, 1000);
        let meta_0 = rng.gen::<u64>() as u128;
        let meta_1 = rng.gen::<u64>() as u128;
        let meta = meta_0 << 64 | meta_1;
        (size, meta)
    })
}

fn random_items(meta_items: usize) -> impl Iterator<Item = (Price, Container<(Size, Meta)>)> {
    let mut rng = thread_rng();
    std::iter::repeat_with(move || {
        let price = rng.gen_range(1, 1000);
        let meta: Vec<_> = random_meta().take(meta_items).collect();
        (price, Container::from(meta))
    })
}

fn bench_populating_push(c: &mut Criterion) {
    c.bench_function_over_inputs("populating items", |b, (items, metas)| {
        let items = random_items(*metas).take(*items).collect::<Vec<_>>();
        let mut store = Store::new();
        b.iter_with_setup(|| items.clone(), |items| {
            for item in items {
                store.insert(item)
            }
        })
    }, &[
        (100, 10),
        (100, 100),
        (1000, 10),
        (1000, 100),
    ]);
}

fn bench_single_push(c: &mut Criterion) {
    c.bench_function_over_inputs("single push", |b, (items, metas)| {
        let mut items = random_items(*metas).take(*items).collect::<Vec<_>>();
        items.sort_by_key(|item| item.0);
        let mut store = Store::from(Container::from(items.clone()));
        b.iter_with_setup(|| (store.clone(), items.clone()), |(mut store, items)| {
            store.insert((430, Container::from(vec![(99, 43), (201, 33)])))
        })
    }, &[
        (100, 10),
        (100, 100),
        (1000, 10),
        (1000, 100),
    ]);
}

fn bench_append_meta(c: &mut Criterion) {
    c.bench_function_over_inputs("append meta", |b, (items, metas)| {
        let mut items = random_items(*metas).take(*items).collect::<Vec<_>>();
        let target = items[items.len() / 2].0;
        items.sort_by_key(|item| item.0);
        let mut store = Store::from(Container::from(items.clone()));
        b.iter_with_setup(|| (store.clone(), items.clone()), |(mut store, items)| {
            store.append_size_and_meta_to_price(target, (55, 0));
        })
    }, &[
        (100, 10),
        (100, 100),
        (1000, 10),
        (1000, 100),
    ]);
}

fn bench_split_median_price(c: &mut Criterion) {
    c.bench_function_over_inputs("split by median price", |b, (items, metas)| {
        let mut items = random_items(*metas).take(*items).collect::<Vec<_>>();;
        items.sort_by_key(|item| item.0);
        let target_price = items[items.len() / 2].0;
        let target_size = u32::max_value();
        let mut store = Store::from(Container::from(items.clone()));
        b.iter_with_setup(|| store.clone(), |mut store| {
            store.split(target_price, target_size);
        })
    }, &[
        (100, 10),
        (100, 100),
        (1000, 10),
        (1000, 100),
    ]);
}

fn bench_split_half_size(c: &mut Criterion) {
    c.bench_function_over_inputs("split by half size", |b, (items, metas)| {
        let mut items = random_items(*metas).take(*items).collect::<Vec<_>>();;
        items.sort_by_key(|item| item.0);

        let target_price = i32::max_value();
        let target_size: u32 = items.iter()
            .map(|(_, sizes)|
                sizes.iter()
                    .map(|(size, _)| *size)
                    .sum::<u32>()
            ).sum::<u32>() / 2;

        let mut store = Store::from(Container::from(items.clone()));
        b.iter_with_setup(|| store.clone(), |mut store| {
            store.split(target_price, target_size);
        })
    }, &[
        (100, 10),
        (100, 100),
        (1000, 10),
        (1000, 100),
    ]);
}

criterion_group!(benches, bench_populating_push, bench_append_meta, bench_single_push, bench_split_median_price, bench_split_half_size);
//criterion_group!(benches, bench_split_median_price, bench_split_half_size);
criterion_main!(benches);
