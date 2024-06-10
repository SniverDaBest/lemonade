use blog_os::{println, print};

fn print_cmdlin() { // TODO: Make the fs work with the cmdline
    print!(") ")
}

// Kernel-level CLI setup
fn cli_setup() {
    println!("Running SHSH v1.0.0.");
    print_cmdline();
    
}

fn cli_read_char() -> Option<char> {
    // Read a single character from UART
    // Return None if no character is available
    //...
}

fn cli_write_str(s: &str) {
    // Write a string to UART
    //...
}

fn cli_process_input() {
    let mut input = String::new();
    loop {
        let c = cli_read_char()?;
        if c == '\n' {
            // End of command, process it
            break;
        }
        input.push(c);
    }

    // Now `input` contains the entire command, process it
    //...
}

fn cli_execute_command(input: &str) {
    // Parse the input and execute the corresponding command
    //...
}
