use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, Program, quit};

#[derive(Default)]
struct InputModel {
    seen: String,
}

impl Model for InputModel {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast::<KeyMsg>() {
            if key.key_type == KeyType::Runes {
                for c in key.runes {
                    if c == 'q' {
                        return Some(quit());
                    }
                    self.seen.push(c);
                }
            } else if key.key_type == KeyType::Up {
                self.seen.push('^');
            }
        }
        None
    }

    fn view(&self) -> String {
        "ok".to_string()
    }
}

#[test]
fn custom_input_reader_parses_keys() {
    let input = std::io::Cursor::new(b"ab\x1b[Aq".to_vec());
    let output = Vec::new();

    let model = InputModel::default();
    let final_model = Program::new(model)
        .with_input(input)
        .with_output(output)
        .run()
        .expect("program should run to completion");

    assert_eq!(final_model.seen, "ab^");
}

struct ExecDone(i32);

#[derive(Default)]
struct ExecModel {
    results: Vec<i32>,
    exec_first: bool,
}

impl Model for ExecModel {
    fn init(&self) -> Option<Cmd> {
        let mut command = std::process::Command::new(if cfg!(windows) { "cmd" } else { "true" });
        if cfg!(windows) {
            command.args(["/C", "exit 0"]);
        }
        let first = bubbletea::exec(|| Some(Message::new(ExecDone(1))));
        let second = bubbletea::exec_process(command, |status| {
            Some(Message::new(ExecDone(
                i32::from(status.is_ok_and(|s| s.success())) * 10,
            )))
        });
        if self.exec_first {
            Some(first)
        } else {
            Some(second)
        }
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(ExecDone(n)) = msg.downcast::<ExecDone>() {
            self.results.push(n);
            return Some(quit());
        }
        None
    }

    fn view(&self) -> String {
        format!("{:?}", self.results)
    }
}

#[test]
fn exec_runs_function_and_delivers_result() {
    let model = ExecModel {
        exec_first: true,
        ..ExecModel::default()
    };
    let final_model = Program::new(model)
        .with_input(std::io::Cursor::new(Vec::new()))
        .with_output(Vec::new())
        .run()
        .expect("program should run to completion");
    assert_eq!(final_model.results, vec![1]);
}

#[test]
fn exec_process_reports_exit_status() {
    let final_model = Program::new(ExecModel::default())
        .with_input(std::io::Cursor::new(Vec::new()))
        .with_output(Vec::new())
        .run()
        .expect("program should run to completion");
    assert_eq!(final_model.results, vec![10]);
}

#[test]
fn exec_returning_none_delivers_nothing() {
    struct NoneModel(bool);
    impl Model for NoneModel {
        fn init(&self) -> Option<Cmd> {
            bubbletea::sequence(vec![
                Some(bubbletea::exec(|| None)),
                Some(Cmd::new(|| Message::new(KeyMsg::from_char('x')))),
            ])
        }
        fn update(&mut self, msg: Message) -> Option<Cmd> {
            if msg.is::<KeyMsg>() {
                self.0 = true;
                return Some(quit());
            }
            None
        }
        fn view(&self) -> String {
            String::new()
        }
    }
    let final_model = Program::new(NoneModel(false))
        .with_input(std::io::Cursor::new(Vec::new()))
        .with_output(Vec::new())
        .run()
        .expect("program should run to completion");
    assert!(final_model.0);
}

#[cfg(feature = "async")]
#[tokio::test(flavor = "multi_thread")]
async fn exec_runs_in_async_event_loop() {
    let model = ExecModel {
        exec_first: true,
        ..ExecModel::default()
    };
    let final_model = Program::new(model)
        .with_input(std::io::Cursor::new(Vec::new()))
        .with_output(Vec::new())
        .run_async()
        .await
        .expect("program should run to completion");
    assert_eq!(final_model.results, vec![1]);
}

#[test]
fn suspend_is_a_noop_with_custom_io() {
    struct SuspendModel {
        resumed: bool,
    }
    impl Model for SuspendModel {
        fn init(&self) -> Option<Cmd> {
            bubbletea::sequence(vec![
                Some(bubbletea::suspend()),
                Some(Cmd::new(|| Message::new(KeyMsg::from_char('x')))),
            ])
        }
        fn update(&mut self, msg: Message) -> Option<Cmd> {
            if msg.is::<bubbletea::ResumeMsg>() {
                self.resumed = true;
            }
            if msg.is::<KeyMsg>() {
                return Some(quit());
            }
            None
        }
        fn view(&self) -> String {
            String::new()
        }
    }
    let final_model = Program::new(SuspendModel { resumed: false })
        .with_input(std::io::Cursor::new(Vec::new()))
        .with_output(Vec::new())
        .run()
        .expect("program should run to completion");
    assert!(!final_model.resumed, "custom I/O programs must not suspend");
}
