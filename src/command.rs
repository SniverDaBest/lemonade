

pub struct Command {
    name: &'static str,
    args: &'static [&'static str],
}

impl Command {
    pub fn execute(&self) {
        match self.name {
            "helloworld" => helloworld(),
            _ => println!("Unknown Command. :("),
        }
    }
}
