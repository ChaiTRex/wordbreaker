#[cfg(not(feature = "with-bench"))]
fn main() {
    compile_error!(r#"Please run `cargo bench --features="with-bench"`"#);
}

#[cfg(feature = "with-bench")]
use with_bench as criterion;

#[cfg(feature = "with-bench")]
use core::time::Duration;
#[cfg(feature = "with-bench")]
use criterion::{black_box, criterion_group, criterion_main, Criterion};
#[cfg(feature = "with-bench")]
use wordbreaker::Dictionary;

#[cfg(feature = "with-bench")]
fn criterion_benchmark(c: &mut Criterion) {
    let words_iter = include_str!("../american-english-dictionary.txt").lines();
    let words = words_iter.clone().collect::<Vec<_>>();
    let dictionary = words_iter.clone().collect::<Dictionary<_>>();
    let dictionary_bytes = dictionary.as_bytes();
    let target = "thequickbrownfoxjumpsoverthelazydog";
    let concatenations_iter = dictionary.concatenations_for(target);

    let coprime_length = (2..=target.len())
        .filter_map(|n| if target.len() % n != 0 { Some(n) } else { None })
        .next()
        .unwrap();
    let no_solutions_dictionary = ('a'..='z')
        .flat_map(|ch| {
            (coprime_length..coprime_length + target.len())
                .step_by(coprime_length)
                .map(move |length| core::iter::repeat(ch).take(length).collect::<String>())
        })
        .collect::<Dictionary<_>>();
    let easy_no_solutions_target = target
        .chars()
        .enumerate()
        .map(|(i, ch)| if i == 2 * coprime_length + 1 { ' ' } else { ch })
        .collect::<String>();
    let easy_no_solutions_concatenations_iter =
        no_solutions_dictionary.concatenations_for(&easy_no_solutions_target);
    let hard_no_solutions_target = target.chars().map(|_| 'z').collect::<String>();
    let hard_no_solutions_concatenations_iter =
        no_solutions_dictionary.concatenations_for(&hard_no_solutions_target);

    c.bench_function("dictionary_new", |b| {
        b.iter(|| Dictionary::new(black_box(&words)))
    });

    c.bench_function("dictionary_from_iter", |b| {
        b.iter(|| black_box(words_iter.clone()).collect::<Dictionary<_>>())
    });

    c.bench_function("dictionary_from_bytes", |b| {
        b.iter(|| Dictionary::from_bytes(black_box(dictionary_bytes)))
    });

    c.bench_function("dictionary_from_bytes_verified", |b| {
        b.iter(|| Dictionary::from_bytes_verified(black_box(dictionary_bytes)))
    });

    c.bench_function("concatenations_new", |b| {
        b.iter(|| dictionary.concatenations_for(black_box(target)))
    });

    c.bench_function("concatenations_clone", |b| {
        b.iter(|| black_box(concatenations_iter.clone()))
    });

    c.bench_function("concatenations_collect", |b| {
        b.iter(|| black_box(concatenations_iter.clone()).collect::<Vec<_>>())
    });

    c.bench_function("concatenations_count", |b| {
        b.iter(|| black_box(concatenations_iter.clone()).count())
    });

    c.bench_function("concatenations_last", |b| {
        b.iter(|| black_box(concatenations_iter.clone()).last())
    });

    c.bench_function("concatenations_max", |b| {
        b.iter(|| black_box(concatenations_iter.clone()).max())
    });

    c.bench_function("concatenations_min", |b| {
        b.iter(|| black_box(concatenations_iter.clone()).min())
    });

    c.bench_function("concatenations_next", |b| {
        b.iter(|| black_box(concatenations_iter.clone()).next())
    });

    c.bench_function("concatenations_next_100", |b| {
        b.iter(|| {
            let mut iter = black_box(concatenations_iter.clone());
            for _ in 0..99 {
                iter.next();
            }
            iter.next()
        })
    });

    c.bench_function("concatenations_nth", |b| {
        b.iter(|| black_box(concatenations_iter.clone()).nth(71257))
    });

    c.bench_function("concatenations_size_hint", |b| {
        b.iter(|| black_box(concatenations_iter.clone()).size_hint())
    });

    c.bench_function("easy_no_solutions_next", |b| {
        b.iter(|| black_box(easy_no_solutions_concatenations_iter.clone()).next())
    });

    c.bench_function("hard_no_solutions_next", |b| {
        b.iter(|| black_box(hard_no_solutions_concatenations_iter.clone()).next())
    });
}

#[cfg(feature = "with-bench")]
criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = criterion_benchmark
}

#[cfg(feature = "with-bench")]
criterion_main!(benches);
