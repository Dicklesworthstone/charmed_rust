//! Benchmarks for bubbletea framework components.

use bubbletea::{Cmd, KeyMsg, KeyType, Message};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_message_creation(c: &mut Criterion) {
    c.bench_function("Message::new simple", |b| {
        b.iter(|| black_box(Message::new(42i32)));
    });

    c.bench_function("Message::new string", |b| {
        b.iter(|| black_box(Message::new(String::from("hello"))));
    });
}

fn benchmark_message_downcast(c: &mut Criterion) {
    let msg = Message::new(42i32);

    c.bench_function("Message::is check", |b| {
        b.iter(|| black_box(msg.is::<i32>()));
    });

    c.bench_function("Message::downcast_ref hit", |b| {
        b.iter(|| black_box(msg.downcast_ref::<i32>()));
    });

    c.bench_function("Message::downcast_ref miss", |b| {
        b.iter(|| black_box(msg.downcast_ref::<String>()));
    });
}

fn benchmark_keymsg_creation(c: &mut Criterion) {
    c.bench_function("KeyMsg::from_type", |b| {
        b.iter(|| black_box(KeyMsg::from_type(KeyType::Enter)));
    });

    c.bench_function("KeyMsg::from_char", |b| {
        b.iter(|| black_box(KeyMsg::from_char('a')));
    });
}

fn benchmark_keymsg_to_string(c: &mut Criterion) {
    let key = KeyMsg::from_type(KeyType::Enter);

    c.bench_function("KeyMsg::to_string", |b| {
        b.iter(|| black_box(key.to_string()));
    });
}

fn benchmark_cmd_creation(c: &mut Criterion) {
    c.bench_function("Cmd::new", |b| {
        b.iter(|| black_box(Cmd::new(|| Message::new(42))));
    });
}

criterion_group!(
    benches,
    benchmark_message_creation,
    benchmark_message_downcast,
    benchmark_keymsg_creation,
    benchmark_keymsg_to_string,
    benchmark_cmd_creation,
);

criterion_main!(benches);
