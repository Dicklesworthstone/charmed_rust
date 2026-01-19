#![forbid(unsafe_code)]

use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, Program, quit};

struct Counter {
    count: i32,
}

impl Counter {
    const fn new() -> Self {
        Self { count: 0 }
    }
}

impl Model for Counter {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::Runes => {
                    if let Some(&ch) = key.runes.first() {
                        match ch {
                            '+' => self.count += 1,
                            '-' => self.count -= 1,
                            'q' | 'Q' => return Some(quit()),
                            _ => {}
                        }
                    }
                }
                KeyType::CtrlC | KeyType::Esc => return Some(quit()),
                _ => {}
            }
        }
        None
    }

    fn view(&self) -> String {
        format!("Count: {}\n\nPress + / - to change, q to quit.", self.count)
    }
}

fn main() -> Result<(), bubbletea::Error> {
    let model = Counter::new();
    Program::new(model).run()?;
    Ok(())
}
