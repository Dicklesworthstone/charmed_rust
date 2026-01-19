use bubbles::textinput::TextInput;
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, Program, quit};

#[derive(Debug)]
struct InitMsg;

#[derive(Debug)]
struct App {
    input: TextInput,
}

impl App {
    fn new() -> Self {
        let mut input = TextInput::new();
        input.set_prompt("› ");
        input.set_placeholder("Type something...");

        Self { input }
    }

    fn value(&self) -> String {
        self.input.value()
    }
}

impl Model for App {
    fn init(&self) -> Option<Cmd> {
        Some(Cmd::new(|| Message::new(InitMsg)))
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if msg.is::<InitMsg>() {
            return self.input.focus();
        }

        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::Enter | KeyType::CtrlC | KeyType::Esc => return Some(quit()),
                KeyType::Runes => {
                    if key.runes.len() == 1 && key.runes[0] == 'q' {
                        return Some(quit());
                    }
                }
                _ => {}
            }
        }

        self.input.update(msg)
    }

    fn view(&self) -> String {
        format!(
            "Enter some text:\n\n{}\n\nPress Enter to accept (q to quit).",
            self.input.view()
        )
    }
}

fn main() -> Result<(), bubbletea::Error> {
    let app = App::new();
    let final_model = Program::new(app).run()?;
    println!("You entered: {}", final_model.value());
    Ok(())
}
