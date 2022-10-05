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
use std::time::Instant;
#[cfg(feature = "with-bench")]
use wordbreaker::Dictionary;

#[cfg(feature = "with-bench")]
macro_rules! consuming_method_call_benches {
    ($c:ident, $iter:ident, $($method:ident),+) => {
        $(
            $c.bench_function(concat!("segmentations_", stringify!($method)), |b| {
                b.iter_custom(|iters| {
                    let iters = (0..iters)
                        .map(|_| $iter.clone())
                        .collect::<Vec<_>>();

                    let start = Instant::now();
                    for iter in iters.into_iter() {
                        black_box(black_box(iter).$method());
                    }
                    start.elapsed()
                })
            });
        )*
    };
}

#[cfg(feature = "with-bench")]
macro_rules! mut_ref_method_call_benches {
    ($c:ident, $iter:ident, $($method:ident),+) => {
        $(
            $c.bench_function(concat!("segmentations_", stringify!($method)), |b| {
                b.iter_custom(|iters| {
                    let mut iters = (0..iters)
                        .map(|_| $iter.clone())
                        .collect::<Vec<_>>();

                    let start = Instant::now();
                    for iter in iters.iter_mut() {
                        black_box(black_box(iter).$method());
                    }
                    start.elapsed()
                })
            });
        )*
    };
}

#[cfg(feature = "with-bench")]
fn criterion_benchmark(c: &mut Criterion) {
    let words_iter = include_str!("../american-english-dictionary.txt").lines();
    let words = words_iter.clone().collect::<Vec<_>>();
    let dictionary = words_iter.clone().collect::<Dictionary<_>>();
    let dictionary_bytes = dictionary.as_bytes();
    let target = "thequickbrownfoxjumpsoverthelazydog";
    let segmentations_iter = dictionary.word_segmentations(target);

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
    let easy_no_solutions_segmentations_iter =
        no_solutions_dictionary.word_segmentations(&easy_no_solutions_target);
    let hard_no_solutions_target = target.chars().map(|_| 'z').collect::<String>();
    let hard_no_solutions_segmentations_iter =
        no_solutions_dictionary.word_segmentations(&hard_no_solutions_target);

    c.bench_function("dictionary_new", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(Dictionary::new(black_box(&words)));
            }
            start.elapsed()
        })
    });

    c.bench_function("dictionary_from_iter", |b| {
        b.iter_custom(|iters| {
            let mut iters = (0..iters).map(|_| words_iter.clone()).collect::<Vec<_>>();

            let start = Instant::now();
            for iter in iters.iter_mut() {
                black_box(black_box(iter).collect::<Vec<_>>());
            }
            start.elapsed()
        })
    });

    c.bench_function("dictionary_from_bytes", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                #[allow(unused_must_use)]
                {
                    black_box(Dictionary::from_bytes(black_box(dictionary_bytes)));
                }
            }
            start.elapsed()
        })
    });

    c.bench_function("dictionary_from_bytes_verified", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                #[allow(unused_must_use)]
                {
                    black_box(Dictionary::from_bytes_verified(black_box(dictionary_bytes)));
                }
            }
            start.elapsed()
        })
    });

    c.bench_function("segmentations_new", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(black_box(&dictionary).word_segmentations(black_box(target)));
            }
            start.elapsed()
        })
    });

    c.bench_function("segmentations_clone", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(black_box(&segmentations_iter).clone());
            }
            start.elapsed()
        })
    });

    c.bench_function("segmentations_collect", |b| {
        b.iter_custom(|iters| {
            let iters = (0..iters)
                .map(|_| segmentations_iter.clone())
                .collect::<Vec<_>>();

            let start = Instant::now();
            let solutions = iters
                .into_iter()
                .map(|iter| black_box(black_box(iter).collect::<Vec<_>>()))
                .collect::<Vec<_>>();
            let result = start.elapsed();
            drop(solutions);
            result
        })
    });

    consuming_method_call_benches!(c, segmentations_iter, count, last, max, min);
    mut_ref_method_call_benches!(c, segmentations_iter, next, next_back, size_hint);

    c.bench_function("segmentations_next_100", |b| {
        b.iter_custom(|iters| {
            let mut iters = (0..iters)
                .map(|_| segmentations_iter.clone())
                .collect::<Vec<_>>();

            let start = Instant::now();
            for iter in iters.iter_mut() {
                let iter = black_box(iter);
                for _ in 0..100 {
                    black_box(iter.next());
                }
            }
            start.elapsed()
        })
    });

    c.bench_function("segmentations_next_back_100", |b| {
        b.iter_custom(|iters| {
            let mut iters = (0..iters)
                .map(|_| segmentations_iter.clone())
                .collect::<Vec<_>>();

            let start = Instant::now();
            for iter in iters.iter_mut() {
                let iter = black_box(iter);
                for _ in 0..100 {
                    black_box(iter.next_back());
                }
            }
            start.elapsed()
        })
    });

    c.bench_function("segmentations_next_all", |b| {
        b.iter_custom(|iters| {
            let mut iters = (0..iters)
                .map(|_| segmentations_iter.clone())
                .collect::<Vec<_>>();

            let start = Instant::now();
            for iter in iters.iter_mut() {
                let iter = black_box(iter);
                while let Some(_) = black_box(iter.next()) {}
            }
            start.elapsed()
        })
    });

    c.bench_function("segmentations_next_back_all", |b| {
        b.iter_custom(|iters| {
            let mut iters = (0..iters)
                .map(|_| segmentations_iter.clone())
                .collect::<Vec<_>>();

            let start = Instant::now();
            for iter in iters.iter_mut() {
                let iter = black_box(iter);
                while let Some(_) = black_box(iter.next_back()) {}
            }
            start.elapsed()
        })
    });

    c.bench_function("segmentations_nth", |b| {
        b.iter_custom(|iters| {
            let mut iters = (0..iters)
                .map(|_| segmentations_iter.clone())
                .collect::<Vec<_>>();

            let start = Instant::now();
            for iter in iters.iter_mut() {
                black_box(black_box(iter).nth(black_box(71257)));
            }
            start.elapsed()
        })
    });

    c.bench_function("easy_no_solutions_next", |b| {
        b.iter_custom(|iters| {
            let mut iters = (0..iters)
                .map(|_| easy_no_solutions_segmentations_iter.clone())
                .collect::<Vec<_>>();

            let start = Instant::now();
            for iter in iters.iter_mut() {
                black_box(black_box(iter).next());
            }
            start.elapsed()
        })
    });

    c.bench_function("hard_no_solutions_next", |b| {
        b.iter_custom(|iters| {
            let mut iters = (0..iters)
                .map(|_| hard_no_solutions_segmentations_iter.clone())
                .collect::<Vec<_>>();

            let start = Instant::now();
            for iter in iters.iter_mut() {
                black_box(black_box(iter).next());
            }
            start.elapsed()
        })
    });
}

#[cfg(feature = "with-bench")]
criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(15));
    targets = criterion_benchmark
}

#[cfg(feature = "with-bench")]
criterion_main!(benches);
