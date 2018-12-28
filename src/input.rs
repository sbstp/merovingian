use std::io::{self, Stdin, Write};

macro_rules! flush {
    () => {
        io::stdout().flush().unwrap();
    };
}

pub struct Input {
    stdin: Stdin,
}

impl Input {
    pub fn new() -> Input {
        Input { stdin: io::stdin() }
    }

    fn read(&self) -> String {
        let mut line = String::new();
        self.stdin.read_line(&mut line).unwrap();
        line.truncate(line.trim_end().len());
        line
    }

    pub fn confirm(&self, prompt: &str, default: impl Into<Option<bool>>) -> bool {
        let default = default.into();
        let choices = match default {
            Some(true) => "[Y/n]",
            Some(false) => "[y/N]",
            None => "[y/n]",
        };

        loop {
            print!("{} {} :", prompt, choices);
            flush!();
            let answer = self.read();
            match &answer[..] {
                "" => {
                    if let Some(default) = default {
                        return default;
                    }
                }
                "y" | "Y" => return true,
                "n" | "N" => return false,
                _ => {}
            }
        }
    }
}
