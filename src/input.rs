use std::io::{self, Stdin, Write};

pub struct Input {
    stdin: Stdin,
}

impl Input {
    pub fn new() -> Input {
        Input { stdin: io::stdin() }
    }

    pub fn confirm(&self, prompt: &str, default: impl Into<Option<bool>>) -> bool {
        let default = default.into();

        let choices = match default {
            Some(true) => "[Y/n]",
            Some(false) => "[y/N]",
            None => "[y/n]",
        };

        loop {
            print!("{} {}: ", prompt, choices);

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

    pub fn choose(&self, prompt: &str, choices: &[(char, &str)], default: impl Into<Option<char>>) -> char {
        let default = default.into();

        if let Some(default) = default {
            if !choices.iter().any(|&(code, _)| code == default) {
                panic!("Default option '{}' is not a valid choice!", default);
            }
        }

        loop {
            println!("{}", prompt);

            for (code, text) in choices.iter() {
                println!("  -> [{}] {}", code, text);
            }

            match default {
                Some(code) => print!("Select [{}]: ", code),
                None => print!("Select: "),
            }

            let line = self.read();

            if let Some(answer) = line.chars().next() {
                for &(code, _) in choices.iter() {
                    if answer == code {
                        return code;
                    }
                }
            } else if let Some(default) = default {
                return default;
            }
        }
    }

    pub fn prompt(&self, prompt: &str) -> String {
        print!("{} : ", prompt);
        self.read()
    }

    fn read(&self) -> String {
        self.flush(); // always flush stdout before reading from stdin
        let mut line = String::new();
        self.stdin.read_line(&mut line).unwrap();
        line.truncate(line.trim_end().len());
        line
    }

    fn flush(&self) {
        let _ = io::stdout().flush();
    }
}
