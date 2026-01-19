use bubbles::spinner::{SpinnerModel, spinners};
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, Program, quit};
use lipgloss::Style;

#[derive(Debug, Default)]
struct App {
    spinner: SpinnerModel,
    quitting: bool,
}

impl App {
    fn new() -> Self {
        let spinner =
            SpinnerModel::with_spinner(spinners::dot()).style(Style::new().foreground("#7D56F4"));
        Self {
            spinner,
            quitting: false,
        }
    }
}

impl Model for App {
    fn init(&self) -> Option<Cmd> {
        self.spinner.init()
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::CtrlC | KeyType::Esc => {
                    self.quitting = true;
                    return Some(quit());
                }
                KeyType::Runes => {
                    if key.runes.len() == 1 && key.runes[0] == 'q' {
                        self.quitting = true;
                        return Some(quit());
                    }
                }
                _ => {}
            }
        }

        self.spinner.update(msg)
    }

    fn view(&self) -> String {
        if self.quitting {
            return "Goodbye.\n".to_string();
        }

        format!("{} Loading...\n\nPress q to quit.", self.spinner.view())
    }
}

fn main() -> Result<(), bubbletea::Error> {
    let app = App::new();
    Program::new(app).run()?;
    Ok(())
}
