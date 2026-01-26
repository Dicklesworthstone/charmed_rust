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
