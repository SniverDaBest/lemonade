use crate::{
    base64, disks::ahci::{self, scan_for_used_ports, AHCIController, AHCIDevice}, pci::{self, PCIDevice}, print, println, randomness, task::keyboard, vga_buffer::WRITER
};
use alloc::{string::{String, ToString}, vec::Vec};
use futures_util::stream::StreamExt;
use pc_keyboard::{DecodedKey, Keyboard, ScancodeSet1};

static SHSH_VERSION: &str = "b0.3";

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
    print!("\r{}", input_buffer);
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
        match randomness::rand_u64() {
            Ok(val) => println!("{}", val.unwrap()),
            Err(e) => println!("0_0  [randomness]: {}", e),
        }
    } else if command.trim().contains("pci") {
        for arg in command.split_whitespace() {
            if arg == "-h" || arg == "--help" {
                println!("PCI(e) Utility");
                println!("-l/--list -- Lists PCI devices.");
                println!("-h/--help -- Shows this message.");
                println!("-d/--device -- Only shows devices with provided device ID.");
                println!("-v/--vendor -- Only shows devices with provided vendor ID.");
                println!("-c/--class -- Only shows devices with provided class code. -- UNIMPLEMENTED!!");
                println!("-s/--sub -- Only shows devices with provided subclass. -- UNIMPLEMENTED!!");
                println!("-le/--list-pcie -- Lists PCIe devices. -- UNIMPLEMENTED!!");
                println!("-de/--device-pcie -- Only shows PCIe devices with provided device ID.");
                println!("-ve/--vendor-pcie -- Only shows PCIe devices with provided vendor ID.");
                println!("-ce/--class-pcie -- Only shows PCIe devices with provided class code. UNIMPLEMENTED!!");
                println!("-se/--sub-pcie -- Only shows PCIe devices with provided subclass. UNIMPLEMENTED!!");
                println!("NOTE: PCIe is broken :(");
                break;
            } else if arg == "-l" || arg == "--list" {
                let bus = pci::scan_pci_bus();
                for x in bus { println!("{}", x); }
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
                    if x.device_id == command.split_whitespace().nth(t).unwrap().parse::<u32>().unwrap() {
                        println!("{}", x);
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
                    if x.vendor_id == command.split_whitespace().nth(t).unwrap().parse::<u32>().unwrap() {
                        println!("{}", x);
                    }
                }
            } else if arg == "-c" || arg == "--class" {
                let mut t = 0;
                for a in command.split_whitespace() {
                    if a == "-c" || a == "--class" {
                        t += 1;
                        break;
                    }
                    t += 1;
                }
                let bus = pci::scan_pci_bus();
                for x in bus {
                    if x.class_code == command.split_whitespace().nth(t).unwrap().parse::<u32>().unwrap() {
                        println!("{}", x);
                    }
                }
            } else if arg == "-s" || arg == "--sub" {
                let mut t = 0;
                for a in command.split_whitespace() {
                    if a == "-s" || a == "--sub" {
                        t += 1;
                        break;
                    }
                    t += 1;
                }
                let bus = pci::scan_pci_bus();
                for x in bus {
                    if x.subclass == command.split_whitespace().nth(t).unwrap().parse::<u32>().unwrap() {
                        println!("{}", x);
                    }
                }
            } else if arg == "-de" || arg == "--device-pcie" {
                let mut t = 0;
                for a in command.split_whitespace() {
                    if a == "-de" || a == "--device-pcie" {
                        t += 1;
                        break;
                    }
                    t += 1;
                }
                let bus = pci::scan_pcie_bus();
                for x in bus {
                    if x[1] == command.split_whitespace().nth(t).unwrap().parse::<u32>().unwrap() {
                        println!("Device ID '{}' | Vendor ID '{}' | Class Code '{}' | Subclass '{}'", x[1], x[0], x[2], x[3]);
                    }
                }
            } else if arg == "-ve" || arg == "--vendor-pcie" {
                let mut t = 0;
                for a in command.split_whitespace() {
                    if a == "-ve" || a == "--vendor-pcie" {
                        t += 1;
                        break;
                    }
                    t += 1;
                }
                let bus = pci::scan_pcie_bus();
                for x in bus {
                    if x[0] == command.split_whitespace().nth(t).unwrap().parse::<u32>().unwrap() {
                        println!("Device ID '{}' | Vendor ID '{}' | Class Code '{}' | Subclass '{}'", x[1], x[0], x[2], x[3]);
                    }
                }
            }
        }
    } else if command.trim().contains("ahci") {
        if command.trim().contains("-h") {
            println!("AHCI Utility");
            println!("-h -- Shows this help message.");
            println!("-l -- Lists connected AHCI devices.");
            println!("-r -- Reads sectors from an AHCI device.");
            println!("-t -- Tests a little thing.");
        } else if command.trim().contains("-l") {
            ahci::scan_for_ahci_controllers(true);
        } else if command.contains("-r") {
            todo!();
        } else if command.contains("-t") {
            let ahci_controllers = ahci::scan_for_ahci_controllers(false);
            if ahci_controllers.is_empty() {
                println!("No AHCI Controllers found.");
                return;
            }
            let ahci_devices = scan_for_used_ports(&ahci_controllers[0], false);
            if ahci_devices.is_empty() {
                println!("No AHCI Devices found.");
                return;
            }

            let mut sectors: Vec<u64> = Vec::new();
            sectors.push(0);
            
            ahci::ahci_read(&ahci_devices[0], sectors);
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
        println!("pci -- The PCI(e) utility.");
        println!("ahci -- The AHCI utility.");
    } else {
        println!("Unknown command: {}", command);
    }
}
