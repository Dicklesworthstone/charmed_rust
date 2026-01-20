//! Benchmark comparing async vs thread-based command execution.
//!
//! This benchmark suite compares the performance of synchronous (thread-based)
//! and asynchronous (tokio-based) command execution patterns.

#![forbid(unsafe_code)]

use bubbletea::{Cmd, Message};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

#[cfg(feature = "async")]
use bubbletea::AsyncCmd;

// =============================================================================
// Helper Messages
// =============================================================================

#[allow(dead_code)]
struct ResultMsg(i32);

#[allow(dead_code)]
struct CounterMsg(usize);

// =============================================================================
// Synchronous (Thread-based) Benchmarks
// =============================================================================

fn bench_sync_single_command(c: &mut Criterion) {
    let mut group = c.benchmark_group("command/single");

    group.bench_function("sync_immediate", |b| {
        b.iter(|| {
            let cmd = Cmd::new(|| Message::new(ResultMsg(42)));
            cmd.execute()
        });
    });

    group.bench_function("sync_with_work", |b| {
        b.iter(|| {
            let cmd = Cmd::new(|| {
                // Simulate some minimal work
                let mut sum = 0i32;
                for i in 0..100 {
                    sum = sum.wrapping_add(i);
                }
                Message::new(ResultMsg(sum))
            });
            cmd.execute()
        });
    });

    group.finish();
}

fn bench_sync_many_commands_sequential(c: &mut Criterion) {
    let mut group = c.benchmark_group("command/many_sequential");

    for count in [10, 100, 1000] {
        group.bench_with_input(BenchmarkId::new("sync", count), &count, |b, &count| {
            b.iter(|| {
                for i in 0..count {
                    let cmd = Cmd::new(move || Message::new(ResultMsg(i)));
                    let _ = cmd.execute();
                }
            });
        });
    }

    group.finish();
}

fn bench_sync_concurrent_threads(c: &mut Criterion) {
    let mut group = c.benchmark_group("command/concurrent");

    for count in [10, 50, 100] {
        group.bench_with_input(BenchmarkId::new("threads", count), &count, |b, &count| {
            b.iter(|| {
                let counter = Arc::new(AtomicUsize::new(0));
                let handles: Vec<_> = (0..count)
                    .map(|i| {
                        let counter = Arc::clone(&counter);
                        std::thread::spawn(move || {
                            let cmd = Cmd::new(move || {
                                counter.fetch_add(1, Ordering::SeqCst);
                                Message::new(ResultMsg(i))
                            });
                            cmd.execute()
                        })
                    })
                    .collect();

                for handle in handles {
                    let _ = handle.join();
                }
            });
        });
    }

    group.finish();
}

// =============================================================================
// Asynchronous (Tokio-based) Benchmarks
// =============================================================================

#[cfg(feature = "async")]
fn bench_async_single_command(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    let mut group = c.benchmark_group("command/single");

    group.bench_function("async_immediate", |b| {
        b.iter(|| {
            rt.block_on(async {
                let cmd = AsyncCmd::new(|| async { Message::new(ResultMsg(42)) });
                cmd.execute().await
            })
        });
    });

    group.bench_function("async_with_work", |b| {
        b.iter(|| {
            rt.block_on(async {
                let cmd = AsyncCmd::new(|| async {
                    // Simulate some minimal work
                    let mut sum = 0i32;
                    for i in 0..100 {
                        sum = sum.wrapping_add(i);
                    }
                    Message::new(ResultMsg(sum))
                });
                cmd.execute().await
            })
        });
    });

    group.finish();
}

#[cfg(feature = "async")]
fn bench_async_many_commands_sequential(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    let mut group = c.benchmark_group("command/many_sequential");

    for count in [10, 100, 1000] {
        group.bench_with_input(BenchmarkId::new("async", count), &count, |b, &count| {
            b.iter(|| {
                rt.block_on(async {
                    for i in 0..count {
                        let cmd = AsyncCmd::new(move || async move { Message::new(ResultMsg(i)) });
                        let _ = cmd.execute().await;
                    }
                });
            });
        });
    }

    group.finish();
}

#[cfg(feature = "async")]
fn bench_async_concurrent_tasks(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    let mut group = c.benchmark_group("command/concurrent");

    for count in [10, 50, 100] {
        group.bench_with_input(
            BenchmarkId::new("async_tasks", count),
            &count,
            |b, &count| {
                b.iter(|| {
                    rt.block_on(async {
                        let counter = Arc::new(AtomicUsize::new(0));
                        let handles: Vec<_> = (0..count)
                            .map(|i| {
                                let counter = Arc::clone(&counter);
                                let cmd = AsyncCmd::new(move || async move {
                                    counter.fetch_add(1, Ordering::SeqCst);
                                    Message::new(ResultMsg(i))
                                });
                                tokio::spawn(async move { cmd.execute().await })
                            })
                            .collect();

                        for handle in handles {
                            let _ = handle.await;
                        }
                    });
                });
            },
        );
    }

    group.finish();
}

// =============================================================================
// Mixed Sync/Async Benchmarks (spawn_blocking comparison)
// =============================================================================

#[cfg(feature = "async")]
fn bench_spawn_blocking_comparison(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    let mut group = c.benchmark_group("command/spawn_blocking");

    // Compare sync command execution via spawn_blocking
    group.bench_function("sync_via_spawn_blocking", |b| {
        b.iter(|| {
            rt.block_on(async {
                tokio::task::spawn_blocking(|| {
                    let cmd = Cmd::new(|| Message::new(ResultMsg(42)));
                    cmd.execute()
                })
                .await
            })
        });
    });

    // Native async command
    group.bench_function("async_native", |b| {
        b.iter(|| {
            rt.block_on(async {
                let cmd = AsyncCmd::new(|| async { Message::new(ResultMsg(42)) });
                cmd.execute().await
            })
        });
    });

    // Mixed concurrent execution: spawn_blocking for sync, native async
    for count in [10, 50] {
        group.bench_with_input(
            BenchmarkId::new("mixed_concurrent", count),
            &count,
            |b, &count| {
                b.iter(|| {
                    rt.block_on(async {
                        let handles: Vec<_> = (0..count)
                            .map(|i| {
                                if i % 2 == 0 {
                                    // Sync command via spawn_blocking
                                    tokio::spawn(async move {
                                        tokio::task::spawn_blocking(move || {
                                            let cmd = Cmd::new(move || Message::new(ResultMsg(i)));
                                            cmd.execute()
                                        })
                                        .await
                                    })
                                } else {
                                    // Async command
                                    tokio::spawn(async move {
                                        let cmd = AsyncCmd::new(move || async move {
                                            Message::new(ResultMsg(i))
                                        });
                                        Ok(cmd.execute().await)
                                    })
                                }
                            })
                            .collect();

                        for handle in handles {
                            let _ = handle.await;
                        }
                    });
                });
            },
        );
    }

    group.finish();
}

// =============================================================================
// Blocking Command Benchmarks
// =============================================================================

#[cfg(feature = "async")]
#[allow(clippy::too_many_lines)]
fn bench_blocking_commands(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    let mut group = c.benchmark_group("command/blocking");
    group.sample_size(20); // Reduce sample size for slower tests

    // Compare blocking execution via threads vs spawn_blocking
    group.bench_function("thread_sleep_10ms", |b| {
        b.iter(|| {
            let cmd = Cmd::blocking(|| {
                std::thread::sleep(Duration::from_millis(10));
                Message::new(ResultMsg(1))
            });
            cmd.execute()
        });
    });

    group.bench_function("spawn_blocking_sleep_10ms", |b| {
        b.iter(|| {
            rt.block_on(async {
                tokio::task::spawn_blocking(|| {
                    let cmd = Cmd::blocking(|| {
                        std::thread::sleep(Duration::from_millis(10));
                        Message::new(ResultMsg(1))
                    });
                    cmd.execute()
                })
                .await
            })
        });
    });

    group.bench_function("async_sleep_10ms", |b| {
        b.iter(|| {
            rt.block_on(async {
                let cmd = AsyncCmd::new(|| async {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    Message::new(ResultMsg(1))
                });
                cmd.execute().await
            })
        });
    });

    // Concurrent blocking operations
    group.bench_function("concurrent_blocking_5x10ms_threads", |b| {
        b.iter(|| {
            let handles: Vec<_> = (0..5)
                .map(|i| {
                    std::thread::spawn(move || {
                        let cmd = Cmd::blocking(move || {
                            std::thread::sleep(Duration::from_millis(10));
                            Message::new(ResultMsg(i))
                        });
                        cmd.execute()
                    })
                })
                .collect();

            for handle in handles {
                let _ = handle.join();
            }
        });
    });

    group.bench_function("concurrent_blocking_5x10ms_spawn_blocking", |b| {
        b.iter(|| {
            rt.block_on(async {
                let handles: Vec<_> = (0..5)
                    .map(|i| {
                        tokio::spawn(async move {
                            tokio::task::spawn_blocking(move || {
                                let cmd = Cmd::blocking(move || {
                                    std::thread::sleep(Duration::from_millis(10));
                                    Message::new(ResultMsg(i))
                                });
                                cmd.execute()
                            })
                            .await
                        })
                    })
                    .collect();

                for handle in handles {
                    let _ = handle.await;
                }
            });
        });
    });

    group.bench_function("concurrent_async_5x10ms", |b| {
        b.iter(|| {
            rt.block_on(async {
                let handles: Vec<_> = (0..5)
                    .map(|i| {
                        let cmd = AsyncCmd::new(move || async move {
                            tokio::time::sleep(Duration::from_millis(10)).await;
                            Message::new(ResultMsg(i))
                        });
                        tokio::spawn(async move { cmd.execute().await })
                    })
                    .collect();

                for handle in handles {
                    let _ = handle.await;
                }
            });
        });
    });

    group.finish();
}

// =============================================================================
// Rapid Command Spawning Benchmark
// =============================================================================

#[cfg(feature = "async")]
fn bench_rapid_spawning(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    let mut group = c.benchmark_group("command/rapid_spawn");

    group.bench_function("threads_100_rapid", |b| {
        b.iter(|| {
            let handles: Vec<_> = (0..100)
                .map(|i| {
                    std::thread::spawn(move || {
                        let cmd = Cmd::new(move || Message::new(ResultMsg(i)));
                        cmd.execute()
                    })
                })
                .collect();

            for handle in handles {
                let _ = handle.join();
            }
        });
    });

    group.bench_function("async_tasks_100_rapid", |b| {
        b.iter(|| {
            rt.block_on(async {
                let handles: Vec<_> = (0..100)
                    .map(|i| {
                        let cmd = AsyncCmd::new(move || async move { Message::new(ResultMsg(i)) });
                        tokio::spawn(async move { cmd.execute().await })
                    })
                    .collect();

                for handle in handles {
                    let _ = handle.await;
                }
            });
        });
    });

    group.bench_function("threads_500_rapid", |b| {
        b.iter(|| {
            let handles: Vec<_> = (0..500)
                .map(|i| {
                    std::thread::spawn(move || {
                        let cmd = Cmd::new(move || Message::new(ResultMsg(i)));
                        cmd.execute()
                    })
                })
                .collect();

            for handle in handles {
                let _ = handle.join();
            }
        });
    });

    group.bench_function("async_tasks_500_rapid", |b| {
        b.iter(|| {
            rt.block_on(async {
                let handles: Vec<_> = (0..500)
                    .map(|i| {
                        let cmd = AsyncCmd::new(move || async move { Message::new(ResultMsg(i)) });
                        tokio::spawn(async move { cmd.execute().await })
                    })
                    .collect();

                for handle in handles {
                    let _ = handle.await;
                }
            });
        });
    });

    group.finish();
}

// =============================================================================
// Criterion Configuration
// =============================================================================

#[cfg(not(feature = "async"))]
criterion_group!(
    benches,
    bench_sync_single_command,
    bench_sync_many_commands_sequential,
    bench_sync_concurrent_threads,
);

#[cfg(feature = "async")]
criterion_group!(
    benches,
    bench_sync_single_command,
    bench_sync_many_commands_sequential,
    bench_sync_concurrent_threads,
    bench_async_single_command,
    bench_async_many_commands_sequential,
    bench_async_concurrent_tasks,
    bench_spawn_blocking_comparison,
    bench_blocking_commands,
    bench_rapid_spawning,
);

criterion_main!(benches);
