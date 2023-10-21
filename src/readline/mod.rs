use std::io::{self, Write, BufWriter};
use std::fs::{self, File};
use std::process;

use console::{Term, Key};

use crate::completion;


#[derive(PartialEq)]
enum Direction {
    Right,
    Left,
}

#[derive(Debug)]
pub enum ReadLineError {
    Flush(String),
    Clear(String),
    Read(String),
    Cursor(String),
    Completion(String),
}

impl ReadLineError {
    pub fn to_string(&self) -> String {
        return match self {
            Self::Flush(message) =>      message.to_string(),
            Self::Clear(message) =>      message.to_string(),
            Self::Read(message) =>       message.to_string(),
            Self::Cursor(message) =>     message.to_string(),
            Self::Completion(message) => message.to_string(),
        };
    }
}

pub struct ReadLine {
    pub buffer: String,
    term: Term,
    cursor: usize,
    history: Vec<String>,
}

impl ReadLine {
    pub fn new() -> ReadLine {
        ReadLine {
            buffer: String::new(),
            term: Term::stdout(),
            cursor: 0,
            history: Vec::new(),
        }
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    pub fn save_history(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(path)?;
        let mut buf = BufWriter::new(file);

        for line in &self.history {
            buf.write((line.to_owned() + "\n").as_bytes())?;
        }

        Ok(())
    }

    pub fn load_history(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        for line in fs::read_to_string(path)?.lines() {
            self.history.push(line.to_string());
        }

        Ok(())
    }

    fn flush(&self) -> Result<(), ReadLineError> {
        return match io::stdout().flush() {
            Ok(_) => Ok(()),
            Err(err) => Err(ReadLineError::Flush(err.to_string())),
        }
    }

    fn output(&self, prompt: &str) -> Result<(), ReadLineError> {
        if let Err(err) = self.term.clear_line() {
            return Err(ReadLineError::Clear(err.to_string()));
        }

        print!("{prompt}{}", self.buffer);

        self.flush()
    }

    fn backspace(&mut self) {
        self.cursor -= 1;
        self.buffer.remove(self.cursor);
    }

    fn insert(&mut self, character: &char) {
        self.buffer.insert(self.cursor, *character);
    }

    fn move_cursor(&mut self, direction: Direction) {
        if direction == Direction::Left {
            if self.cursor != 0 {
                self.cursor -= 1;
            }
        } else {
            if self.cursor != self.buffer.len() {
                self.cursor += 1;
            }
        }
    }

    fn render_cursor(&mut self) -> Result<(), ReadLineError> {
        if self.cursor != self.buffer.len() {
            return match self.term.move_cursor_left(self.buffer.len() - self.cursor) {
                Ok(_) => Ok(()),
                Err(err) => Err(ReadLineError::Cursor(err.to_string())),
            };
        }

        Ok(())
    }

    fn history_get(&mut self, history_index: usize) {
        if !self.history.is_empty() {
            self.buffer = self.history[history_index].clone();
            self.cursor = self.history[history_index].len();
        }
    }

    pub fn input(&mut self, prompt: &str) -> Result<(), ReadLineError> {
        self.buffer = String::new();
        self.cursor = 0;

        let mut history_index = self.history.len();

        loop {
            self.output(prompt)?;

            self.render_cursor()?;

            let key = match self.term.read_key() {
                Ok(character) => character,
                Err(err) => {
                    match err.kind() {
                        io::ErrorKind::Interrupted => {
                            return Ok(());
                        },
                        _ => {
                            return Err(ReadLineError::Read(err.to_string()));
                        },
                    }
                },
            };

            match key {
                Key::Tab => {
                    let mut path = self.buffer.split(" ").collect::<Vec<&str>>();
                    if !path.is_empty() {
                        let completed = match completion::complete(path[path.len() - 1]) {
                            Ok(buf) => buf,
                            Err(err) => {
                                return Err(ReadLineError::Completion(err.to_string()));
                            },
                        };

                        path.pop();
                        path.push(&completed);

                        self.buffer = path.join(" ");
                        self.cursor = self.buffer.len();
                    }
                },
                Key::ArrowUp => {
                    if history_index != 0 {
                        history_index -= 1;
                    }
                    self.history_get(history_index);
                },
                Key::ArrowDown => {
                    if self.history.len() > history_index + 1 {
                        history_index += 1;
                        self.history_get(history_index);
                    } else {
                        history_index = self.history.len();
                        self.buffer = String::new();
                        self.cursor = 0;
                    }
                },
                Key::ArrowLeft => {
                    self.move_cursor(Direction::Left);
                },
                Key::ArrowRight => {
                    self.move_cursor(Direction::Right);
                },
                Key::Backspace => {
                    if !self.buffer.is_empty() {
                        self.backspace();
                    }
                },
                Key::Enter => {
                    self.history.push(self.buffer.clone());
                    println!(""); // Newline
                    break;
                },
                Key::Char(character) => {
                    self.insert(&character);

                    match character {
                        '\u{4}' => {
                            println!("beta ^D");
                            process::exit(1);
                        },
                        '\u{1a}' => {
                            println!("beta ^Z");
                            process::exit(1);
                        },
                        _ => {},
                    }
                    self.cursor += 1;
                },
                _ => {},
            }
        }

        Ok(())
    }
}


