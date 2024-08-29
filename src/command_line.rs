use crate::{
    task::keyboard,
    print,
    println,
    vga_buffer::WRITER,
    base64,
    randomness,
    pci,
};
use core::{mem::drop,future::Future};
use alloc::{string::{String, ToString}, vec::Vec};
use futures_util::stream::StreamExt;
use pc_keyboard::{DecodedKey, Keyboard, ScancodeSet1};

static SHSH_VERSION: &str = "b0.1";

pub async fn run_command_line() {
    println!("Made by SniverDaBest\nSHSH {}", SHSH_VERSION);
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
    // println!("Command received: {}", command); // debug statement that shouldn't always run

    // Example: simple echo command
    if command.trim().contains("echo") {
        println!("{}", command.replace("echo ", ""));
    } else if command.trim().contains("clear") {
        WRITER.lock().clear_screen();
    } else if command.trim().contains("ver") {
        println!("SHSH Version {}", SHSH_VERSION);
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
    } else if command.trim().contains("pci") {
        for arg in command.split_whitespace() {
            if arg == "-h" || arg == "--help" {
                println!("PCI Utility");
                println!("-l/--list   -- Lists PCI devices.");
                println!("-h/--help   -- Shows this message.");
                println!("-d/--device -- Only shows devices with provided device ID.");
                println!("-v/--vendor -- Only shows devices with provided vendor ID.");
                println!("-c/--class  -- Only shows devices with provided class code. -- UNIMPLEMENTED!!");
                println!("-s/--sub    -- Only shows devices with provided subclass. -- UNIMPLEMENTED!!");

                break;
            } else if arg == "-l" || arg == "--list" {
                let bus = pci::scan_pci_bus();
                for x in bus { println!("Device ID '{}' | Vendor ID '{}' | Class Code '{}' | Subclass '{}'", x[1], x[0], x[2], x[3]); }
            } else if arg == "-d" || arg == "--device" {
                let mut t = 0;
                for a in command.split_whitespace() {
                    if a == "-d" || a == "--device" {
                        t += 1;
                        break;
                    }
                    t += 1;
                }
                let bus = pci::scan_pci_bus();
                for x in bus {
                    if x[1] == command.split_whitespace().nth(t).unwrap().parse::<u32>().unwrap() {
                        println!("Device ID '{}' | Vendor ID '{}' | Class Code '{}' | Subclass '{}'", x[1], x[0], x[2], x[3]);
                    }
                }
            } else if arg == "-v" || arg == "--vendor" {
                let mut t = 0;
                for a in command.split_whitespace() {
                    if a == "-v" || a == "--vendor" {
                        t += 1;
                        break;
                    }
                    t += 1;
                }
                let bus = pci::scan_pci_bus();
                for x in bus {
                    if x[0] == command.split_whitespace().nth(t).unwrap().parse::<u32>().unwrap() {
                        println!("Device ID '{}' | Vendor ID '{}' | Class Code '{}' | Subclass '{}'", x[1], x[0], x[2], x[3]);
                    }
                }
            } else if arg == "-c" || arg == "--class" {
                println!("-_-  This isn't implemented yet.");
            } else if arg == "-s" || arg == "--sub" {
                println!("-_-  This isn't implemented yet.")
            }
        }
    } else if command.trim().contains("help") {
        println!("SHSH Version {}.", SHSH_VERSION);
        println!("help -- Shows this message.");
        println!("echo [input] -- Echos user input.");
        println!("clear -- Clears the screen.");
        println!("ver -- Shows the version of SHSH. (currently running version {})", SHSH_VERSION);
        println!("b64encode [input] -- Encodes user input into Base64");
        println!("b64decode [base64] -- Decodes Base64 user input into normal text.");
        println!("randint [seed] -- Generates a random number based on a seed.");
        println!("pci -- The PCI utility.");
    } else {
        println!("Unknown command: {}", command);
    }
}
