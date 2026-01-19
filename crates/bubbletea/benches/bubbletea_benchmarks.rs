#![forbid(unsafe_code)]

use bubbletea::{Cmd, Message, Model, batch, parse_sequence, sequence};
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use crossterm::event::{KeyCode, KeyModifiers};

#[derive(Clone, Copy, Debug)]
enum BenchMsg {
    Increment,
    Decrement,
    NoOp,
}

#[derive(Clone, Debug)]
struct BenchModel {
    count: i64,
}

impl Model for BenchModel {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(msg) = msg.downcast::<BenchMsg>() {
            match msg {
                BenchMsg::Increment => self.count += 1,
                BenchMsg::Decrement => self.count -= 1,
                BenchMsg::NoOp => {}
            }
        }
        None
    }

    fn view(&self) -> String {
        let count = self.count;
        format!("Count: {count}")
    }
}

#[derive(Clone, Debug)]
struct ComplexModel {
    items: Vec<String>,
    selected: usize,
}

impl Model for ComplexModel {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, _msg: Message) -> Option<Cmd> {
        None
    }

    fn view(&self) -> String {
        let mut output = String::with_capacity(self.items.len() * 24);
        for (index, item) in self.items.iter().enumerate() {
            if index == self.selected {
                output.push_str("> ");
            } else {
                output.push_str("  ");
            }
            output.push_str(item);
            output.push('\n');
        }
        output
    }
}

fn bench_message_dispatch(c: &mut Criterion) {
    let mut group = c.benchmark_group("bubbletea/message_dispatch");

    group.bench_function("single_message", |b| {
        b.iter(|| {
            let mut model = BenchModel { count: 0 };
            black_box(model.update(Message::new(BenchMsg::Increment)));
            black_box(model.count)
        });
    });

    group.throughput(Throughput::Elements(1_000));
    group.bench_function("1000_messages", |b| {
        b.iter(|| {
            let mut model = BenchModel { count: 0 };
            for _ in 0..1_000 {
                black_box(model.update(Message::new(BenchMsg::Increment)));
            }
            black_box(model.count)
        });
    });

    group.throughput(Throughput::Elements(1_000));
    group.bench_function("1000_messages_mixed", |b| {
        b.iter(|| {
            let mut model = BenchModel { count: 0 };
            for _ in 0..500 {
                black_box(model.update(Message::new(BenchMsg::Increment)));
                black_box(model.update(Message::new(BenchMsg::Decrement)));
            }
            black_box(model.count)
        });
    });

    group.finish();
}

fn bench_view_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("bubbletea/view_rendering");

    group.bench_function("simple_view", |b| {
        let model = BenchModel { count: 42 };
        b.iter(|| black_box(model.view()));
    });

    let complex = ComplexModel {
        items: (0..100).map(|i| format!("Item {i}")).collect(),
        selected: 50,
    };

    group.bench_function("list_100_items", |b| {
        b.iter(|| black_box(complex.view()));
    });

    group.finish();
}

fn bench_key_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("bubbletea/key_parsing");

    let sequences: &[(&str, &[u8])] = &[
        ("arrow_up", b"\x1b[A"),
        ("arrow_down", b"\x1b[B"),
        ("arrow_right", b"\x1b[C"),
        ("arrow_left", b"\x1b[D"),
        ("alt_arrow_up", b"\x1b[1;3A"),
        ("ctrl_arrow_up", b"\x1b[1;5A"),
        ("shift_tab", b"\x1b[Z"),
        ("function_f1", b"\x1bOP"),
        ("unknown", b"\x1b[999~"),
    ];

    for (name, seq) in sequences {
        group.bench_with_input(BenchmarkId::new("parse_sequence", name), seq, |b, seq| {
            b.iter(|| black_box(parse_sequence(seq)));
        });
    }

    group.bench_function("crossterm_char", |b| {
        b.iter(|| {
            black_box(bubbletea::key::from_crossterm_key(
                KeyCode::Char('a'),
                KeyModifiers::NONE,
            ))
        });
    });

    group.bench_function("crossterm_ctrl_c", |b| {
        b.iter(|| {
            black_box(bubbletea::key::from_crossterm_key(
                KeyCode::Char('c'),
                KeyModifiers::CONTROL,
            ))
        });
    });

    group.bench_function("crossterm_shift_tab", |b| {
        b.iter(|| {
            black_box(bubbletea::key::from_crossterm_key(
                KeyCode::Tab,
                KeyModifiers::SHIFT,
            ))
        });
    });

    group.finish();
}

fn bench_commands(c: &mut Criterion) {
    let mut group = c.benchmark_group("bubbletea/commands");

    group.bench_function("Cmd::none", |b| {
        b.iter(|| black_box(Cmd::none()));
    });

    group.bench_function("Cmd::message", |b| {
        b.iter(|| {
            black_box(Cmd::new(|| Message::new(BenchMsg::NoOp)));
        });
    });

    group.bench_function("Cmd::execute", |b| {
        b.iter(|| {
            let cmd = Cmd::new(|| Message::new(BenchMsg::NoOp));
            black_box(cmd.execute());
        });
    });

    group.bench_function("Cmd::batch_10", |b| {
        b.iter(|| {
            let cmds = (0..10)
                .map(|_| Some(Cmd::new(|| Message::new(BenchMsg::NoOp))))
                .collect::<Vec<_>>();
            black_box(batch(cmds));
        });
    });

    group.bench_function("Cmd::sequence_10", |b| {
        b.iter(|| {
            let cmds = (0..10)
                .map(|_| Some(Cmd::new(|| Message::new(BenchMsg::NoOp))))
                .collect::<Vec<_>>();
            black_box(sequence(cmds));
        });
    });

    group.finish();
}

fn bench_event_loop_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("bubbletea/event_loop");

    group.bench_function("frame_cycle", |b| {
        b.iter(|| {
            let mut model = BenchModel { count: 0 };
            black_box(model.update(Message::new(BenchMsg::Increment)));
            black_box(model.view());
        });
    });

    group.throughput(Throughput::Elements(60));
    group.bench_function("60fps_1sec", |b| {
        b.iter(|| {
            let mut model = BenchModel { count: 0 };
            for _ in 0..60 {
                black_box(model.update(Message::new(BenchMsg::Increment)));
                black_box(model.view());
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_message_dispatch,
    bench_view_rendering,
    bench_key_parsing,
    bench_commands,
    bench_event_loop_simulation
);
criterion_main!(benches);
