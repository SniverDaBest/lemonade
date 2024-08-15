use crate::{
    task::keyboard,
    print,
    println,
    vga_buffer::WRITER,
    base64,
    randomness,
};
use core::future::Future;
use alloc::string::{String, ToString};
use futures_util::stream::StreamExt;
use pc_keyboard::{DecodedKey, Keyboard, ScancodeSet1};

static LSH_VERSION: &str = "b0.1";

pub async fn run_command_line() {
    println!("Made by SniverDaBest\nLSH {}", LSH_VERSION);
    let mut scancodes = keyboard::ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        pc_keyboard::layouts::Us104Key,
        pc_keyboard::HandleControl::Ignore,
    );

    let mut input_buffer = String::new();
    let mut prompt = "$ ".to_string();

    // Initial prompt display
    print!("{}", prompt);

    loop {
        while let Some(scancode) = scancodes.next().await {
            if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
                if let Some(key) = keyboard.process_keyevent(key_event) {
                    match key {
                        DecodedKey::Unicode(character) => {
                            if character == '\n' {
                                // Process command
                                process_command(&input_buffer);
                                input_buffer.clear();
                                prompt = "$ ".to_string();
                                // Move to the next line and show the new prompt
                                print!("\n{}", prompt);
                            } else {
                                input_buffer.push(character);
                                // Redraw input buffer
                                redraw_input_buffer(&input_buffer);
                            }
                        },
                        DecodedKey::RawKey(_) => {
                            // Handle special keys if needed
                        }
                    }
                }
            }
        }
    }
}

fn redraw_input_buffer(input_buffer: &str) {
    // Move the cursor to the start of the line
    print!("\r");
    // Print the prompt and the current input buffer
    print!("{}", input_buffer);
}

fn process_command(command: &str) {
    // Implement command processing logic here
    println!("Command received: {}", command);

    // Example: simple echo command
    if command.trim().contains("echo") {
        println!("{}", command.replace("echo ", ""));
    } else if command.trim().contains("clear") {
        WRITER.lock().clear_screen();
    } else if command.trim().contains("ver") {
        println!("LSH Version {}", LSH_VERSION);
    } else if command.trim().contains("b64encode") {
        let input_str = command.split_whitespace().nth(1).unwrap_or("").as_bytes();
        println!("{}", base64::encode(input_str));
    } else if command.trim().contains("b64decode") {
        let input_str = command.split_whitespace().nth(1).unwrap_or("").as_bytes();
        println!("{}", base64::decode(input_str));
    } else if command.trim().contains("randint") {
        if let Some(seed_str) = command.split_whitespace().nth(1) {
            let seed: u32 = seed_str.parse().expect("-_-  Expected a number for seed");
            println!("{}", randomness::gen_number(seed));
        } else {
            println!("-_-  No seed provided");
        }
    } else if command.trim().contains("help") {
        println!("LSH Version {}.", LSH_VERSION);
        println!("help -- Shows this message.");
        println!("echo [input] -- Echos user input.");
        println!("clear -- Clears the screen.");
        println!("ver -- Shows the version of LSH. (currently running version {})", LSH_VERSION);
        println!("b64encode [input] -- Encodes user input into Base64");
        println!("b64decode [base64] -- Decodes Base64 user input into normal text.");
        println!("randint [seed] -- Generates a random number based on a seed.");
    } else {
        println!("Unknown command: {}", command);
    }
}
