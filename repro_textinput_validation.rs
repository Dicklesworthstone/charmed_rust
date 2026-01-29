use bubbles::textinput::TextInput;

fn main() {
    let mut input = TextInput::new();
    input.char_limit = 3;
    
    // Validator fails if length > 3 (which is impossible if truncated, but let's say it fails if content is "123")
    // actually let's say validator fails if content contains "4".
    input.set_validate(|s| {
        if s.contains("4") {
            Some("Contains 4".to_string())
        } else {
            None
        }
    });

    // Input "1234".
    // Truncation should make it "123".
    // Validation on "1234" -> Error "Contains 4".
    // Stored value "123".
    // Stored error "Contains 4".
    // Actual state: Value "123" (valid), but Error "Contains 4" (invalid).
    
    input.set_value("1234");
    
    println!("Value: '{}'", input.value());
    println!("Error: {:?}", input.err);
    
    if input.value() == "123" && input.err.is_some() {
        println!("BUG REPRODUCED: Value is valid '123' but error is present.");
    } else {
        println!("No bug?");
    }
}
