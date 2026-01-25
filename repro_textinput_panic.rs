
#[cfg(test)]
mod tests {
    use bubbles::textinput::TextInput;
    use bubbletea::Model;

    #[test]
    fn test_delete_word_backward_panic() {
        let mut input = TextInput::new();
        input.set_value("hello world");
        input.cursor_end(); // Cursor at 11
        
        // Ensure offset logic is populated
        let _ = input.view();

        // This should delete "world"
        input.key_map.delete_word_backward.match_key(bubbletea::KeyMsg {
             key_type: bubbletea::KeyType::Runes,
             runes: vec![],
             alt: true,
             paste: false,
        }); 
        
        // Direct call to simulate the key press effect
        // We can't access private methods, but we can access `update` if we construct a message
        // Or since `delete_word_backward` is private, we rely on `update` or the public method if exposed?
        // `delete_word_backward` is private.
        // We must use `update` with a matching key.
        // The default key for delete_word_backward is Alt+Backspace or Ctrl+W.
        
        use bubbletea::KeyMsg;
        use bubbletea::KeyType;
        
        let msg = bubbletea::Message::new(KeyMsg {
            key_type: KeyType::Backspace,
            runes: vec![],
            alt: true,
            paste: false,
        });
        
        let _ = Model::update(&mut input, msg);
        
        // This view call should panic if offset_right wasn't updated
        let _ = input.view();
    }
}
