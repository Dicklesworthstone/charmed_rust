
use bubbles::textinput::TextInput;
use bubbletea::{Message, KeyMsg, KeyType};

#[test]
fn test_delete_word_backward_behavior() {
    let mut input = TextInput::new();
    input.set_value("abc   def");
    
    // Case 1: Cursor at end of "def" (index 9)
    input.set_cursor(9);
    // Simulate Ctrl+W
    input.update(Message::new(KeyMsg {
        key_type: KeyType::Char('w'),
        runes: vec!['w'],
        alt: false,
        paste: false,
        // Using direct method for testing logic
    }));
    // Since we can't easily synthesize KeyMsg with ctrl (it depends on how KeyMap interprets strings),
    // I will call the internal method if I can, or use the public method if exposed?
    // delete_word_backward is private.
    // But update uses KeyMap matches.
    
    // Actually, I can just write a unit test inside the crate if I want to test private methods.
    // Or I can use the existing test structure.
}

// I will write a standalone rust file that imports bubbles and runs a test.
// But bubbles is a library. I can't access private methods from outside.
// I will rely on the `update` method and standard key bindings.
// Ctrl+W is default for delete_word_backward.

fn main() {
    // This is just a script to be read, I'll put the actual test logic in the tool call
}
