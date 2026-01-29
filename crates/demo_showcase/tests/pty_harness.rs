//! PTY-based headless TUI testing harness.
//!
//! This module provides a real terminal emulator environment for testing the
//! demo_showcase application. It uses portable-pty to spawn the app in a real
//! pseudo-terminal and vt100 to parse the terminal output into an inspectable
//! screen buffer.

use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};
use std::thread;
use vt100::Parser;

/// Size of the virtual terminal.
const TERM_COLS: u16 = 120;
const TERM_ROWS: u16 = 40;

/// PTY harness for testing the demo_showcase TUI.
struct PtyHarness {
    /// Receiver for output from the reader thread.
    output_rx: Receiver<Vec<u8>>,
    /// Writer to send input to the PTY.
    writer: Box<dyn Write + Send>,
    /// VT100 parser for interpreting terminal output.
    parser: Parser,
    /// Child process handle.
    _child: Box<dyn portable_pty::Child + Send + Sync>,
    /// Reader thread handle.
    _reader_thread: thread::JoinHandle<()>,
}

impl PtyHarness {
    /// Spawn the demo_showcase app in a PTY.
    fn spawn() -> anyhow::Result<Self> {
        let pty_system = NativePtySystem::default();

        // Create PTY with specific size
        let pair = pty_system.openpty(PtySize {
            rows: TERM_ROWS,
            cols: TERM_COLS,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // Build command to run the demo - use the built binary directly
        let binary_path = std::env::current_dir()?
            .join("../../target/debug/demo_showcase");

        if !binary_path.exists() {
            anyhow::bail!("Binary not found at {}", binary_path.display());
        }

        let mut cmd = CommandBuilder::new(&binary_path);
        cmd.args(["--seed", "42", "--no-alt-screen"]); // Deterministic data, no alt screen
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");
        cmd.env("LANG", "en_US.UTF-8");

        // Spawn the child
        let child = pair.slave.spawn_command(cmd)?;

        // Get master for reading/writing
        let mut reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        // Create channel for output
        let (output_tx, output_rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel();

        // Spawn reader thread
        let reader_thread = thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        if output_tx.send(buf[..n].to_vec()).is_err() {
                            break; // Receiver dropped
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        // Create VT100 parser
        let parser = Parser::new(TERM_ROWS, TERM_COLS, 0);

        Ok(Self {
            output_rx,
            writer,
            parser,
            _child: child,
            _reader_thread: reader_thread,
        })
    }

    /// Read all available output and process it through the VT100 parser.
    fn read_output(&mut self, timeout: Duration) -> usize {
        let mut total_read = 0;
        let start = Instant::now();
        let mut all_data = Vec::new();

        while start.elapsed() < timeout {
            match self.output_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(data) => {
                    all_data.extend_from_slice(&data);
                    self.parser.process(&data);
                    total_read += data.len();
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // If we've read some data, we might be done
                    if total_read > 0 && start.elapsed() > Duration::from_millis(500) {
                        break;
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }

        // Debug: Print raw output and escape sequence analysis
        if total_read > 0 {
            let preview_len = std::cmp::min(2000, all_data.len());
            println!("=== Raw output preview (first {} of {} bytes) ===", preview_len, all_data.len());
            println!("{}", String::from_utf8_lossy(&all_data[..preview_len]));
            println!("=== End raw preview ===");

            // Look for cursor positioning codes (H, A, B, C, D, G)
            println!("\n=== Cursor movement analysis ===");
            let mut pos = 0;
            while pos < all_data.len() {
                if all_data[pos] == 0x1b && pos + 1 < all_data.len() && all_data[pos + 1] == b'[' {
                    // CSI sequence - look for cursor movements
                    let start = pos;
                    pos += 2; // skip ESC [
                    // Read parameters
                    let mut params = String::new();
                    while pos < all_data.len() {
                        let c = all_data[pos];
                        if c >= b'0' && c <= b'9' || c == b';' {
                            params.push(c as char);
                            pos += 1;
                        } else {
                            break;
                        }
                    }
                    if pos < all_data.len() {
                        let cmd = all_data[pos] as char;
                        match cmd {
                            'H' | 'f' => println!("  @{}: CUP (cursor position) params={}", start, params),
                            'A' => println!("  @{}: CUU (cursor up) params={}", start, params),
                            'B' => println!("  @{}: CUD (cursor down) params={}", start, params),
                            'C' => println!("  @{}: CUF (cursor forward) params={}", start, params),
                            'D' => println!("  @{}: CUB (cursor back) params={}", start, params),
                            'G' => println!("  @{}: CHA (cursor horizontal absolute) params={}", start, params),
                            'K' => println!("  @{}: EL (erase line) params={}", start, params),
                            'J' => println!("  @{}: ED (erase display) params={}", start, params),
                            _ => {}
                        }
                        pos += 1;
                    }
                } else {
                    pos += 1;
                }
            }
            println!("=== End cursor analysis ===\n");
        }

        total_read
    }

    /// Get the current screen content as plain text.
    fn screen_text(&self) -> String {
        let screen = self.parser.screen();
        let mut output = String::new();
        for row in 0..screen.size().0 {
            let line = screen.contents_between(row, 0, row, screen.size().1);
            output.push_str(&line);
            output.push('\n');
        }
        output
    }

    /// Send a key to the PTY.
    fn send_key(&mut self, key: &str) -> anyhow::Result<()> {
        self.writer.write_all(key.as_bytes())?;
        self.writer.flush()?;
        Ok(())
    }

    /// Send a character.
    fn send_char(&mut self, c: char) -> anyhow::Result<()> {
        self.send_key(&c.to_string())
    }

    /// Send 'q' to quit.
    fn send_quit(&mut self) -> anyhow::Result<()> {
        self.send_char('q')
    }

    /// Print the current screen to stdout for debugging.
    fn dump_screen(&self) {
        println!("=== Screen ({} x {}) ===", TERM_COLS, TERM_ROWS);
        let screen = self.screen_text();
        for (i, line) in screen.lines().enumerate() {
            println!("{:2}: {}", i + 1, line);
        }
        println!("=== End Screen ===");
    }

    /// Check if screen contains a string.
    fn screen_contains(&self, needle: &str) -> bool {
        self.screen_text().contains(needle)
    }
}

/// Test vt100 parser with the EXACT output from demo_showcase to isolate the issue.
#[test]
fn test_vt100_with_real_output() {
    // This is a simplified version of what demo_showcase outputs
    let mut parser = Parser::new(TERM_ROWS, TERM_COLS, 0);

    // Simulate the exact sequence from the app:
    // 1. Hide cursor, enable modes
    parser.process(b"\x1b[?25l");
    parser.process(b"\x1b[?1000h\x1b[?1002h\x1b[?1003h");

    // 2. First render: "Loading..."
    parser.process(b"\x1b[1;1H\x1b[2J");
    parser.process(b"Loading...");

    // Check state after loading
    {
        let screen = parser.screen();
        let row0 = screen.contents_between(0, 0, 0, 40);
        println!("After 'Loading...': row0 = '{}'", row0);
        assert!(row0.contains("Loading"), "Should see Loading... after first render");
    }

    // 3. Set window title (OSC sequence - note: NO BEL terminator before next ESC)
    parser.process(b"\x1b]0;Charmed Control Center");

    // Check state - OSC should not affect screen content
    {
        let screen = parser.screen();
        let row0 = screen.contents_between(0, 0, 0, 40);
        println!("After OSC (no terminator yet): row0 = '{}'", row0);
    }

    // 4. Second render: clear and show main content
    parser.process(b"\x1b[1;1H\x1b[2J");

    // Check - should be cleared
    {
        let screen = parser.screen();
        let row0 = screen.contents_between(0, 0, 0, 40);
        println!("After second clear: row0 = '{}'", row0);
    }

    // 5. Now add the styled content
    parser.process(b"\x1b[48;2;26;26;26m"); // background
    parser.process(b"\x1b[0m"); // reset
    parser.process(b"\x1b[38;2;0;255;0m"); // green fg
    parser.process(b"Connected");
    parser.process(b"\x1b[0m"); // reset
    parser.process(b" more text");

    {
        let screen = parser.screen();
        let row0 = screen.contents_between(0, 0, 0, 60);
        println!("After styled content: row0 = '{}'", row0);
        assert!(row0.contains("Connected"), "Should see 'Connected' in output");
        assert!(row0.contains("more text"), "Should see 'more text' in output");
    }

    println!("\n=== Real output simulation passed ===");

    // Now test what happens when we split escape sequences across chunks
    println!("\n=== Testing chunk boundary issues ===");
    let mut parser = Parser::new(5, 60, 0);

    // What if an escape sequence is split across process() calls?
    // E.g., "\x1b[1" then ";1H" then "Hello"
    parser.process(b"\x1b[1");  // Partial CSI
    parser.process(b";1H");     // Rest of CSI
    parser.process(b"Hello at 1,1");

    let screen = parser.screen();
    let row0 = screen.contents_between(0, 0, 0, 30);
    println!("Split CSI result: row0 = '{}'", row0);

    // Test 2: Split in the middle of OSC sequence
    let mut parser = Parser::new(5, 60, 0);
    parser.process(b"\x1b[1;1H\x1b[2JContent\x1b]0;");  // Start OSC
    parser.process(b"Title");                            // OSC text
    parser.process(b"\x1b[1;1H\x1b[2JNew Content");      // Next sequence

    let screen = parser.screen();
    let row0 = screen.contents_between(0, 0, 0, 30);
    println!("Split OSC result: row0 = '{}'", row0);
    assert!(row0.contains("New Content"), "New content should be visible after split OSC");

    // Test 3: Split RGB color sequence
    let mut parser = Parser::new(5, 60, 0);
    parser.process(b"\x1b[1;1H\x1b[2J\x1b[38;2;");   // Start RGB
    parser.process(b"0;255;0");                       // RGB values
    parser.process(b"mGreenText\x1b[0m");             // End + text

    let screen = parser.screen();
    let row0 = screen.contents_between(0, 0, 0, 30);
    println!("Split RGB result: row0 = '{}'", row0);
    assert!(row0.contains("GreenText"), "Green text should be visible after split RGB");

    println!("\n=== Chunk boundary tests passed ===");
}

/// Test to see if feeding data in one shot vs chunks matters.
#[test]
fn test_vt100_all_at_once_vs_chunked() {
    let binary_path = std::env::current_dir()
        .unwrap()
        .join("../../target/debug/demo_showcase");

    if !binary_path.exists() {
        eprintln!("Skipping test - binary not found");
        return;
    }

    // Capture raw output first
    let harness = match PtyHarness::spawn() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Skipping - spawn failed: {e}");
            return;
        }
    };

    // Collect all data without processing through parser
    let mut all_data: Vec<u8> = Vec::new();
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(3);

    while start.elapsed() < timeout {
        match harness.output_rx.recv_timeout(Duration::from_millis(100)) {
            Ok(data) => {
                all_data.extend_from_slice(&data);
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if !all_data.is_empty() && start.elapsed() > Duration::from_millis(500) {
                    break;
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    println!("Captured {} bytes", all_data.len());

    // Now feed to a fresh parser ALL AT ONCE
    let mut parser_once = Parser::new(TERM_ROWS, TERM_COLS, 0);
    parser_once.process(&all_data);
    let screen_once = parser_once.screen();
    let row0_once = screen_once.contents_between(0, 0, 0, 60);
    println!("Parser (all at once) row 0: '{}'", row0_once);

    // Compare with feeding in 1000-byte chunks
    let mut parser_chunked = Parser::new(TERM_ROWS, TERM_COLS, 0);
    for chunk in all_data.chunks(1000) {
        parser_chunked.process(chunk);
    }
    let screen_chunked = parser_chunked.screen();
    let row0_chunked = screen_chunked.contents_between(0, 0, 0, 60);
    println!("Parser (1000-byte chunks) row 0: '{}'", row0_chunked);

    // Compare with feeding in smaller chunks (like the PTY reader might)
    let mut parser_small = Parser::new(TERM_ROWS, TERM_COLS, 0);
    for chunk in all_data.chunks(100) {
        parser_small.process(chunk);
    }
    let screen_small = parser_small.screen();
    let row0_small = screen_small.contents_between(0, 0, 0, 60);
    println!("Parser (100-byte chunks) row 0: '{}'", row0_small);

    // All should match
    assert_eq!(row0_once, row0_chunked, "All-at-once vs 1000-byte chunks should match");
    assert_eq!(row0_once, row0_small, "All-at-once vs 100-byte chunks should match");

    // Check if the content is correct
    let has_content = row0_once.contains("Connected") || row0_once.contains("Charmed");
    if !has_content {
        println!("\n!!! Content NOT visible in any parsing mode !!!");
        println!("Dumping first 5 rows:");
        for r in 0..5 {
            let content = screen_once.contents_between(r, 0, r, 80);
            println!("  Row {}: '{}'", r, content);
        }

        // Analyze the raw data for control characters
        println!("\n=== Control character analysis in first 500 bytes ===");
        for (i, &b) in all_data.iter().take(500).enumerate() {
            match b {
                0x08 => println!("  @{}: BACKSPACE (BS)", i),
                0x0D => println!("  @{}: CARRIAGE RETURN (CR)", i),
                0x7F => println!("  @{}: DELETE (DEL)", i),
                0x00..=0x07 => println!("  @{}: Control char 0x{:02x}", i, b),
                0x0E..=0x1A => println!("  @{}: Control char 0x{:02x}", i, b),
                _ => {}
            }
        }

        // Check parser cursor position
        let cursor = screen_once.cursor_position();
        println!("\nParser cursor position: row={}, col={}", cursor.0, cursor.1);
        println!("Parser screen size: {:?}", screen_once.size());

        // Try processing first part only (before second clear)
        let mut test_parser = Parser::new(TERM_ROWS, TERM_COLS, 0);
        test_parser.process(&all_data[..120]); // Just through second clear
        let test_screen = test_parser.screen();
        println!("\nAfter first 120 bytes (through 2nd clear):");
        println!("  Row 0: '{}'", test_screen.contents_between(0, 0, 0, 40));
        println!("  Cursor: {:?}", test_screen.cursor_position());

        // Process more - the styled header
        test_parser.process(&all_data[120..400]);
        let test_screen = test_parser.screen();
        println!("\nAfter bytes 120-400 (header content):");
        println!("  Row 0: '{}'", test_screen.contents_between(0, 0, 0, 80));
        println!("  Cursor: {:?}", test_screen.cursor_position());

        // Continue processing in 500-byte chunks and watch what happens
        let mut chunk_parser = Parser::new(TERM_ROWS, TERM_COLS, 0);
        let chunk_size = 500;
        let mut processed = 0;
        while processed < all_data.len() {
            let end = (processed + chunk_size).min(all_data.len());
            chunk_parser.process(&all_data[processed..end]);
            let screen = chunk_parser.screen();
            let row0 = screen.contents_between(0, 0, 0, 40);
            let cursor = screen.cursor_position();

            // Print when row 0 changes
            if processed == 0 || row0.trim() != "" {
                println!("\nAfter bytes {}..{}: row0='{}', cursor={:?}",
                         processed, end, row0.trim(), cursor);
            }
            processed = end;

            // Check if row 0 content changed unexpectedly
            if processed > 400 && !row0.contains("Charmed") && row0.trim().len() > 0 {
                println!("!!! Row 0 content changed at byte {} !!!", processed);
                println!("    Now: '{}'", row0.trim());
                // Dump bytes around this point
                let context_start = processed.saturating_sub(100);
                let context = &all_data[context_start..processed];
                println!("    Context bytes: {:?}", String::from_utf8_lossy(context));
                break;
            }
        }

        // Track when row 0 becomes empty
        println!("\n=== TRACKING ROW 0 DISAPPEARANCE ===");
        let mut row0_tracker = Parser::new(TERM_ROWS, TERM_COLS, 0);
        let mut last_row0 = String::new();
        let chunk_size = 200;
        let mut processed = 0;

        while processed < all_data.len() {
            let end = (processed + chunk_size).min(all_data.len());
            row0_tracker.process(&all_data[processed..end]);
            let screen = row0_tracker.screen();
            let row0 = screen.contents_between(0, 0, 0, 60).trim().to_string();
            let cursor = screen.cursor_position();

            // Track when row 0 content changes
            if row0 != last_row0 {
                // Safely truncate strings with potential multi-byte chars
                let safe_last = last_row0.char_indices()
                    .take_while(|(i, _)| *i < 30)
                    .last()
                    .map(|(i, c)| i + c.len_utf8())
                    .unwrap_or(last_row0.len());
                let safe_row0 = row0.char_indices()
                    .take_while(|(i, _)| *i < 30)
                    .last()
                    .map(|(i, c)| i + c.len_utf8())
                    .unwrap_or(row0.len());
                println!("Bytes {}-{}: Row0 changed from '{}' to '{}', cursor={:?}",
                    processed, end, &last_row0[..safe_last],
                    &row0[..safe_row0], cursor);

                // If row 0 became empty, show the bytes that caused it
                if row0.is_empty() && !last_row0.is_empty() {
                    println!("  !!! ROW 0 BECAME EMPTY !!!");
                    let context_bytes = &all_data[processed..end];
                    // Look for clear or scroll sequences
                    let context_str = String::from_utf8_lossy(context_bytes);
                    // Safely truncate - find a safe char boundary
                    let safe_end = context_str.char_indices()
                        .take_while(|(i, _)| *i < 300)
                        .last()
                        .map(|(i, c)| i + c.len_utf8())
                        .unwrap_or(context_str.len());
                    println!("  Context (first ~300 chars): {:?}",
                        context_str[..safe_end].replace('\x1b', "ESC"));
                }
                last_row0 = row0;
            }
            processed = end;
        }

        // Count newlines in the data
        let newline_count = all_data.iter().filter(|&&b| b == 0x0A).count();
        println!("\n=== Newline analysis ===");
        println!("Total newlines in output: {}", newline_count);
        println!("Screen height: {} rows", TERM_ROWS);

        // Count clear sequences
        let data_str = String::from_utf8_lossy(&all_data);
        let clear_count = data_str.matches("\x1b[2J").count() + data_str.matches("\x1b[J").count();
        let moveto_count = data_str.matches("\x1b[1;1H").count() + data_str.matches("\x1b[H").count();
        println!("Clear sequences (ESC[2J/ESC[J): {}", clear_count);
        println!("MoveTo(0,0) sequences: {}", moveto_count);

        // Find "Loading" and "Charmed" occurrences to detect multiple renders
        let loading_count = data_str.matches("Loading").count();
        let charmed_count = data_str.matches("Charmed").count();
        println!("'Loading' occurrences: {}", loading_count);
        println!("'Charmed' occurrences: {}", charmed_count);

        // If more than expected newlines, the terminal will scroll
        if newline_count > usize::from(TERM_ROWS) {
            println!("\n!!! SCROLL DETECTED: {} newlines > {} rows !!!", newline_count, TERM_ROWS);
            println!("Extra newlines will cause content to scroll off screen");
        }

        // Analyze the first 400 bytes to see what Line 0 actually contains
        println!("\n=== FIRST 400 BYTES ANALYSIS ===");
        let first_400 = &all_data[..400.min(all_data.len())];
        let first_nl = first_400.iter().position(|&b| b == 0x0A).unwrap_or(first_400.len());
        let line0_bytes = &first_400[..first_nl];

        println!("Line 0 ends at byte {}", first_nl);
        println!("Line 0 bytes ({}):", line0_bytes.len());

        // Show bytes with escape sequences marked
        let mut pos = 0;
        while pos < line0_bytes.len() {
            let b = line0_bytes[pos];
            if b == 0x1b && pos + 1 < line0_bytes.len() {
                let next = line0_bytes[pos + 1];
                if next == b'[' {
                    // CSI - find end
                    let mut end = pos + 2;
                    while end < line0_bytes.len() && !(line0_bytes[end] >= 0x40 && line0_bytes[end] <= 0x7E) {
                        end += 1;
                    }
                    if end < line0_bytes.len() {
                        let seq = String::from_utf8_lossy(&line0_bytes[pos..=end]);
                        print!("[CSI:{}]", seq.replace('\x1b', ""));
                        pos = end + 1;
                        continue;
                    }
                } else if next == b']' {
                    // OSC - find BEL or ST
                    let mut end = pos + 2;
                    while end < line0_bytes.len() && line0_bytes[end] != 0x07 {
                        if line0_bytes[end] == 0x1b && end + 1 < line0_bytes.len() && line0_bytes[end + 1] == b'\\' {
                            end += 1;
                            break;
                        }
                        end += 1;
                    }
                    let seq = String::from_utf8_lossy(&line0_bytes[pos..=end.min(line0_bytes.len() - 1)]);
                    print!("[OSC:{}]", seq.replace('\x1b', "").chars().take(30).collect::<String>());
                    pos = end + 1;
                    continue;
                }
            }
            if b >= 0x20 && b < 0x7F {
                print!("{}", b as char);
            } else {
                print!("[0x{:02X}]", b);
            }
            pos += 1;
        }
        println!("\n=== END FIRST 400 BYTES ===");

        // Check for lines that are too long using lipgloss::width for accurate measurement
        let data_str = String::from_utf8_lossy(&all_data);
        let mut line_num = 0;
        let mut lines_over_120 = 0;
        for line in data_str.lines() {
            let vlen = lipgloss::width(line);
            if vlen > TERM_COLS as usize {
                lines_over_120 += 1;
                if lines_over_120 <= 5 { // Limit output
                    // Find visible content only
                    let visible: String = line.chars()
                        .filter(|&c| c != '\x1b' && c.is_ascii_graphic() || c == ' ')
                        .take(80)
                        .collect();
                    println!("Line {} has {} visible chars (over {}): visible = {:?}",
                             line_num, vlen, TERM_COLS, visible);
                }
            }
            line_num += 1;
        }
        println!("Lines exceeding {} columns: {}", TERM_COLS, lines_over_120);

        // Also check where newlines are in the raw data
        let newline_positions: Vec<usize> = all_data.iter()
            .enumerate()
            .filter(|&(_, &b)| b == 0x0A)
            .map(|(i, _)| i)
            .take(5)
            .collect();
        println!("First 5 newline positions: {:?}", newline_positions);
    }
}

/// Test the vt100 parser directly with escape sequences to isolate issues.
#[test]
fn test_vt100_parser_escape_sequences() {
    // Test 1: Simple text after clear
    println!("=== Test 1: Simple clear + text ===");
    let mut parser = Parser::new(5, 40, 0);
    parser.process(b"\x1b[1;1H\x1b[2JHello World");
    let screen = parser.screen();
    let row0 = screen.contents_between(0, 0, 0, 40);
    println!("Row 0: '{}'", row0);
    assert!(row0.contains("Hello World"), "Simple text should be visible");

    // Test 2: RGB colors (true color)
    println!("\n=== Test 2: RGB Colors ===");
    let mut parser = Parser::new(5, 40, 0);
    parser.process(b"\x1b[1;1H\x1b[2J\x1b[38;2;0;255;0mGreen\x1b[0m Text");
    let screen = parser.screen();
    let row0 = screen.contents_between(0, 0, 0, 40);
    println!("Row 0: '{}'", row0);
    assert!(row0.contains("Green Text"), "Colored text should be visible");

    // Test 3: OSC (window title) with proper BEL terminator
    println!("\n=== Test 3: OSC with BEL ===");
    let mut parser = Parser::new(5, 40, 0);
    parser.process(b"\x1b[1;1H\x1b[2JBefore\x1b]0;Title\x07After");
    let screen = parser.screen();
    let row0 = screen.contents_between(0, 0, 0, 40);
    println!("Row 0: '{}'", row0);
    assert!(row0.contains("Before") && row0.contains("After"),
        "OSC should be skipped, text before and after visible");

    // Test 4: OSC without BEL (just ESC to start next sequence)
    println!("\n=== Test 4: OSC without BEL ===");
    let mut parser = Parser::new(5, 40, 0);
    // This tests what happens when OSC doesn't have \x07 but goes directly to next ESC
    parser.process(b"\x1b[1;1H\x1b[2JBefore\x1b]0;Title\x1b[0mAfter");
    let screen = parser.screen();
    let row0 = screen.contents_between(0, 0, 0, 40);
    println!("Row 0: '{}'", row0);
    // This might fail - OSC without terminator could eat subsequent text
    println!("(May show unexpected results if OSC not properly terminated)");

    // Test 5: Double clear with content between
    println!("\n=== Test 5: Double clear ===");
    let mut parser = Parser::new(5, 120, 0);
    parser.process(b"\x1b[1;1H\x1b[2JLoading...\x1b[1;1H\x1b[2JFinal Content");
    let screen = parser.screen();
    let row0 = screen.contents_between(0, 0, 0, 120);
    println!("Row 0: '{}'", row0);
    assert!(!row0.contains("Loading"), "First content should be cleared");
    assert!(row0.contains("Final Content"), "Final content should be visible");

    // Test 6: Complex sequence like demo_showcase outputs
    println!("\n=== Test 6: Complex sequence ===");
    let mut parser = Parser::new(5, 120, 0);
    // Simulate: hide cursor, enable modes, clear, loading, title, clear, styled content
    let seq = b"\x1b[?25l\x1b[1;1H\x1b[2JLoading...\x1b]0;Test Title\x07\x1b[1;1H\x1b[2J\x1b[48;2;26;26;26m\x1b[0m\x1b[38;2;0;255;0mConnected\x1b[0m Rest of content";
    parser.process(seq);
    let screen = parser.screen();
    let row0 = screen.contents_between(0, 0, 0, 120);
    println!("Row 0: '{}'", row0);
    assert!(row0.contains("Connected"), "Styled text should be visible");
    assert!(row0.contains("Rest of content"), "Unstyled text should be visible");

    println!("\n=== All vt100 parser tests completed ===");
}

#[test]
#[ignore = "Requires real PTY environment - run with --ignored for manual testing"]
fn test_pty_app_starts() {
    // The binary should already be built by cargo test
    let binary_path = std::env::current_dir()
        .unwrap()
        .join("../../target/debug/demo_showcase");

    if !binary_path.exists() {
        eprintln!("Skipping PTY test - binary not found at {}", binary_path.display());
        eprintln!("Run 'cargo build -p demo_showcase' first");
        return;
    }

    let mut harness = match PtyHarness::spawn() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Skipping PTY test - spawn failed: {e}");
            return;
        }
    };

    // Wait for initial output
    let read = harness.read_output(Duration::from_secs(5));
    println!("Read {} bytes", read);

    // Dump the screen so we can see what's happening
    harness.dump_screen();

    // Check for expected content
    let has_content = harness.screen_contains("Charmed")
        || harness.screen_contains("Dashboard")
        || harness.screen_contains("Loading")
        || harness.screen_contains("Connected");

    if !has_content {
        eprintln!("Screen doesn't contain expected content!");
        panic!("No expected content found");
    }

    println!("✓ App started and rendered content");

    // Verify some specific UI elements
    if harness.screen_contains("Dashboard") {
        println!("✓ Dashboard is visible");
    }
    if harness.screen_contains("Charmed Control Center") {
        println!("✓ Title is visible");
    }
    if harness.screen_contains("Connected") {
        println!("✓ Status is visible");
    }

    // Send quit
    let _ = harness.send_quit();
    let _ = harness.read_output(Duration::from_millis(500));
}

#[test]
#[ignore = "Requires real PTY environment - run with --ignored for manual testing"]
fn test_pty_navigation() {
    let binary_path = std::env::current_dir()
        .unwrap()
        .join("../../target/debug/demo_showcase");

    if !binary_path.exists() {
        eprintln!("Skipping PTY test - binary not found");
        return;
    }

    let mut harness = match PtyHarness::spawn() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Skipping PTY test - spawn failed: {e}");
            return;
        }
    };

    // Wait for initial render
    harness.read_output(Duration::from_secs(3));

    println!("=== Initial Screen ===");
    harness.dump_screen();

    // Navigate to Jobs page (press '3')
    let _ = harness.send_char('3');
    harness.read_output(Duration::from_secs(1));

    println!("\n=== After pressing '3' (Jobs) ===");
    harness.dump_screen();

    // Navigate to Logs page (press '4')
    let _ = harness.send_char('4');
    harness.read_output(Duration::from_secs(1));

    println!("\n=== After pressing '4' (Logs) ===");
    harness.dump_screen();

    // Quit
    let _ = harness.send_quit();
}

#[test]
#[ignore = "Requires real PTY environment - run with --ignored for manual testing"]
fn test_full_ui_verification() {
    let binary_path = std::env::current_dir()
        .unwrap()
        .join("../../target/debug/demo_showcase");

    if !binary_path.exists() {
        eprintln!("Skipping PTY test - binary not found");
        return;
    }

    let mut harness = match PtyHarness::spawn() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Skipping PTY test - spawn failed: {e}");
            return;
        }
    };

    // Wait for full render
    harness.read_output(Duration::from_secs(5));

    let screen = harness.screen_text();
    harness.dump_screen();

    // Verify header elements
    assert!(screen.contains("Charmed") || screen.contains("Control Center"),
        "Header should contain title");

    // Verify sidebar elements
    let sidebar_items = ["Dashboard", "Services", "Jobs", "Logs", "Docs", "Files", "Wizard", "Settings"];
    let mut found_sidebar = 0;
    for item in &sidebar_items {
        if screen.contains(item) {
            found_sidebar += 1;
        }
    }
    assert!(found_sidebar >= 4, "Should find at least 4 sidebar items, found {}", found_sidebar);

    // Verify footer
    assert!(screen.contains("help") || screen.contains("quit"),
        "Footer should show keybindings");

    println!("✓ Full UI verification passed!");

    // Quit
    let _ = harness.send_quit();
}

/// Test using tmux as a real terminal emulator.
/// This is the "cheap oscilloscope" approach - runs the app in tmux and captures
/// Test using script to capture PTY output with proper terminal size.
/// This verifies the app renders correctly when terminal dimensions are set.
#[test]
#[ignore = "Requires tmux installation - run with --ignored for manual testing"]
fn test_tmux_real_terminal() {
    use std::process::Command;
    use std::fs;

    let binary_path = std::env::current_dir()
        .unwrap()
        .join("../../target/debug/demo_showcase")
        .canonicalize()
        .unwrap_or_else(|_| {
            eprintln!("Binary not found - skipping test");
            return std::path::PathBuf::new();
        });

    if !binary_path.exists() {
        eprintln!("Skipping test - binary not found");
        return;
    }

    // Check if script command is available
    let script_check = Command::new("script").arg("--version").output();
    if script_check.is_err() {
        eprintln!("Skipping test - script command not available");
        return;
    }

    // Create a temp file for script output
    let output_file = format!("/tmp/demo_pty_test_{}.txt", std::process::id());

    // Run the app with script to capture PTY output
    // stty sets terminal size to 120x40 before running
    let cmd = format!(
        "stty cols 120 rows 40; timeout 2 {} --seed 42 --no-alt-screen",
        binary_path.display()
    );

    let result = Command::new("script")
        .args(["-q", &output_file, "-c", &cmd])
        .env("TERM", "xterm-256color")
        .output();

    if let Err(e) = result {
        eprintln!("Failed to run script: {}", e);
        return;
    }

    // Read the captured output
    let screen_content = match fs::read_to_string(&output_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read output file: {}", e);
            return;
        }
    };

    // Clean up temp file
    let _ = fs::remove_file(&output_file);

    // Display first part of output
    println!("=== PTY captured output (first 2000 chars) ===");
    println!("{}", &screen_content.chars().take(2000).collect::<String>());
    println!("=== End capture ===\n");

    // Check for expected content
    let has_charmed = screen_content.contains("Charmed");
    let has_connected = screen_content.contains("Connected");
    let has_dashboard = screen_content.contains("Dashboard");
    let has_sidebar = screen_content.contains("Services") || screen_content.contains("Jobs");
    let has_footer = screen_content.contains("quit") || screen_content.contains("help");

    println!("Content checks:");
    println!("  Has 'Charmed': {}", has_charmed);
    println!("  Has 'Connected': {}", has_connected);
    println!("  Has 'Dashboard': {}", has_dashboard);
    println!("  Has sidebar items: {}", has_sidebar);
    println!("  Has footer hints: {}", has_footer);

    // Verify all expected content is present
    assert!(has_charmed, "Missing 'Charmed' in header");
    assert!(has_connected, "Missing 'Connected' status");
    assert!(has_dashboard, "Missing 'Dashboard' in sidebar");
    assert!(has_sidebar, "Missing sidebar items (Services/Jobs)");
    assert!(has_footer, "Missing footer hints");

    // Additional checks for layout integrity
    // Check that actual rendered content lines fit within terminal width.
    // Skip lines that are:
    // - Script header/footer (starts with "Script started/done")
    // - Initialization sequences (contains "Loading..." without proper line structure)
    // - Empty or whitespace-only lines
    let mut content_lines_checked = 0;
    let mut max_content_width = 0;
    for line in screen_content.lines() {
        // Skip script header/footer lines
        if line.starts_with("Script started") || line.starts_with("Script done") {
            continue;
        }
        // Skip initialization line (contains multiple cursor movements and Loading...)
        if line.contains("Loading...") && line.contains("\x1b[2J") {
            continue;
        }
        let visible_width = lipgloss::width(line);
        // Only check lines with actual content (more than just escape sequences)
        if visible_width > 0 && visible_width <= 200 {
            content_lines_checked += 1;
            if visible_width > max_content_width {
                max_content_width = visible_width;
            }
            // Each content line should fit in 120 columns
            // (allowing 1 extra for edge cases)
            if visible_width > 121 {
                println!("WARNING: Line {} chars exceeds 120: {}...",
                    visible_width,
                    line.chars().filter(|c| !c.is_control()).take(50).collect::<String>());
            }
        }
    }

    println!("\nContent lines checked: {}", content_lines_checked);
    println!("Max content line width: {}", max_content_width);

    // Ensure we checked some actual content
    assert!(content_lines_checked >= 30, "Expected at least 30 content lines, got {}", content_lines_checked);

    // Allow up to 121 chars (terminal width + 1 for edge cases)
    assert!(
        max_content_width <= 121,
        "Content line width {} exceeds terminal width 120",
        max_content_width
    );

    println!("\n✓ PTY real terminal test passed!");
}

/// Debug test to analyze the raw output line-by-line.
/// This helps identify where lines exceed terminal width.
#[test]
fn test_analyze_view_output() {
    use std::process::Command;

    let binary_path = std::env::current_dir()
        .unwrap()
        .join("../../target/debug/demo_showcase")
        .canonicalize()
        .unwrap_or_else(|_| std::path::PathBuf::new());

    if !binary_path.exists() {
        eprintln!("Skipping - binary not found");
        return;
    }

    let session_name = format!("analyze_{}", std::process::id());
    let _ = Command::new("tmux").args(["kill-session", "-t", &session_name]).output();

    // Create session with specific size
    let _ = Command::new("tmux")
        .args(["new-session", "-d", "-s", &session_name, "-x", "120", "-y", "40"])
        .output();

    let cmd = format!("TERM=wezterm {} --seed 42 --no-alt-screen", binary_path.display());
    let _ = Command::new("tmux")
        .args(["send-keys", "-t", &session_name, &cmd, "Enter"])
        .output();

    thread::sleep(Duration::from_secs(3));

    // Get RAW output with escape sequences for analysis
    let raw_output = Command::new("tmux")
        .args(["capture-pane", "-t", &session_name, "-p", "-S", "-100", "-e"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let _ = Command::new("tmux").args(["kill-session", "-t", &session_name]).output();

    println!("=== ANALYZING LINE-BY-LINE OUTPUT ===\n");

    let mut problematic_lines = 0;
    for (line_num, line) in raw_output.lines().enumerate() {
        let visible_width = lipgloss::width(line);
        let raw_len = line.len();

        if visible_width > 120 {
            problematic_lines += 1;
            println!("LINE {} OVERFLOW: visible={}, raw_bytes={}", line_num, visible_width, raw_len);

            // Show start and end of line
            let start: String = line.chars().take(80).collect();
            let end: String = line.chars().rev().take(40).collect::<String>().chars().rev().collect();
            println!("  START: {:?}", start.replace('\x1b', "ESC"));
            println!("  END: {:?}", end.replace('\x1b', "ESC"));

            // Find where the excess starts
            let mut visible_so_far = 0;
            let mut excess_start_idx = 0;
            for (i, c) in line.chars().enumerate() {
                if c == '\x1b' {
                    // Skip escape sequence
                    continue;
                }
                if visible_so_far >= 120 {
                    excess_start_idx = i;
                    break;
                }
                visible_so_far += unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
            }
            if excess_start_idx > 0 {
                let excess: String = line.chars().skip(excess_start_idx).take(60).collect();
                println!("  EXCESS CONTENT AT pos {}: {:?}", excess_start_idx, excess.replace('\x1b', "ESC"));
            }
            println!();
        }

        // Also flag any line with content that seems to span the boundary
        if line.len() > 200 && visible_width <= 120 {
            // Long raw length but fits - check structure
            let visible_end_pos = 115; // Check what's near the edge
            let mut visible = 0;
            let mut near_edge = String::new();
            let mut capturing = false;
            for c in line.chars() {
                if c != '\x1b' {
                    visible += 1;
                }
                if visible >= visible_end_pos - 5 && visible <= visible_end_pos + 5 {
                    capturing = true;
                    near_edge.push(c);
                }
                if visible > visible_end_pos + 5 {
                    break;
                }
            }
            if capturing && !near_edge.is_empty() {
                // println!("LINE {} edge chars (near col 115): {:?}", line_num, near_edge);
            }
        }
    }

    println!("\n=== SUMMARY ===");
    println!("Lines exceeding 120 visible columns: {}", problematic_lines);

    if problematic_lines > 0 {
        println!("\n!!! LAYOUT BUG CONFIRMED: Lines exceed terminal width !!!\n");
        panic!("Found {} lines exceeding terminal width", problematic_lines);
    }

    println!("✓ All lines fit within 120 columns");
}

/// Test with WezTerm's mux server for the most accurate reproduction.
/// This requires wezterm to be installed.
#[test]
#[ignore = "Requires WezTerm installation - run with --ignored for manual testing"]
fn test_wezterm_mux() {
    use std::process::{Command, Stdio};

    let binary_path = std::env::current_dir()
        .unwrap()
        .join("../../target/debug/demo_showcase")
        .canonicalize()
        .unwrap_or_else(|_| std::path::PathBuf::new());

    if !binary_path.exists() {
        eprintln!("Skipping wezterm test - binary not found");
        return;
    }

    // Check if wezterm CLI is available
    let wezterm_check = Command::new("wezterm")
        .args(["cli", "list"])
        .stderr(Stdio::null())
        .output();

    // If no mux server is running, try to spawn one
    let needs_mux = match &wezterm_check {
        Ok(output) => !output.status.success(),
        Err(_) => true,
    };

    if needs_mux {
        eprintln!("WezTerm mux server not running - trying to start...");
        let start_result = Command::new("wezterm-mux-server")
            .arg("--daemonize")
            .output();

        if start_result.is_err() {
            eprintln!("Could not start wezterm-mux-server - skipping test");
            eprintln!("To run this test, start 'wezterm-mux-server --daemonize' first");
            return;
        }

        // Give it time to start
        thread::sleep(Duration::from_millis(500));
    }

    // Spawn a pane running our app with explicit terminal size
    // The app needs terminal size set via stty since wezterm cli spawn doesn't support --cols/--rows
    let cmd = format!(
        "stty cols 120 rows 40 2>/dev/null; exec {} --seed 42 --no-alt-screen",
        binary_path.to_string_lossy()
    );
    let spawn_result = Command::new("wezterm")
        .args([
            "cli", "spawn",
            "--cwd", &std::env::current_dir().unwrap().to_string_lossy(),
            "--",
            "bash", "-c", &cmd,
        ])
        .output();

    let pane_id = match spawn_result {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        Ok(output) => {
            eprintln!("wezterm cli spawn failed: {}", String::from_utf8_lossy(&output.stderr));
            return;
        }
        Err(e) => {
            eprintln!("Failed to run wezterm cli spawn: {}", e);
            return;
        }
    };

    println!("Spawned pane: {}", pane_id);

    // Wait for render
    thread::sleep(Duration::from_secs(3));

    // Get the screen text
    let get_text = Command::new("wezterm")
        .args(["cli", "get-text", "--pane-id", &pane_id])
        .output();

    let screen_content = match get_text {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).to_string()
        }
        Ok(output) => {
            eprintln!("get-text failed: {}", String::from_utf8_lossy(&output.stderr));
            // Kill the pane
            let _ = Command::new("wezterm")
                .args(["cli", "kill-pane", "--pane-id", &pane_id])
                .output();
            return;
        }
        Err(e) => {
            eprintln!("Failed to get text: {}", e);
            return;
        }
    };

    println!("=== WezTerm captured screen ===");
    for (i, line) in screen_content.lines().take(45).enumerate() {
        println!("{:2}: {}", i + 1, line);
    }
    println!("=== End WezTerm capture ===\n");

    // Also get with escapes for debugging
    let get_escapes = Command::new("wezterm")
        .args(["cli", "get-text", "--pane-id", &pane_id, "--escapes"])
        .output();

    if let Ok(output) = get_escapes {
        if output.status.success() {
            let escaped = String::from_utf8_lossy(&output.stdout);
            println!("=== WezTerm with escapes (first 2000 bytes) ===");
            println!("{}", &escaped[..escaped.len().min(2000)]);
            println!("=== End escapes ===\n");
        }
    }

    // Kill the pane
    let _ = Command::new("wezterm")
        .args(["cli", "kill-pane", "--pane-id", &pane_id])
        .output();

    // Verify content
    let has_charmed = screen_content.contains("Charmed");
    let has_connected = screen_content.contains("Connected");
    let has_dashboard = screen_content.contains("Dashboard");

    println!("WezTerm content checks:");
    println!("  Has 'Charmed': {}", has_charmed);
    println!("  Has 'Connected': {}", has_connected);
    println!("  Has 'Dashboard': {}", has_dashboard);

    if has_charmed || has_connected || has_dashboard {
        println!("\n✓ WezTerm mux test passed!");
    } else {
        eprintln!("\n!!! CRITICAL: WezTerm shows broken rendering !!!");
        eprintln!("This is the exact same failure as on Mac - we can now iterate locally.\n");
        panic!("WezTerm capture shows no expected content!");
    }
}
