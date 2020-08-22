use std::env;
use std::fmt::{self, Display};
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::process;

fn main() {
    let (path, max_sequence_length) = match process_args() {
        Ok(args) => args,
        Err(err) => {
            eprintln!(
                "Usage: program <path to file> <maximum sequence length>\nError: {}",
                err
            );
            process::exit(1);
        }
    };

    let mut sequence_counters = Vec::with_capacity(max_sequence_length);
    for sequence_length in 1..=max_sequence_length {
        sequence_counters.push(DigitSequenceCounter::new(sequence_length));
    }

    let file = match File::open(&path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Error opening file path: {}", err);
            process::exit(2);
        }
    };
    BufReader::with_capacity(65536, file)
        .bytes()
        .map(|byte| byte.expect("Error reading file path"))
        .skip_while(|byte| *byte != b'.')
        .skip(1)
        .for_each(|byte| {
            sequence_counters.iter_mut().for_each(|sequence_counter| {
                sequence_counter.process_character(byte);
            })
        });

    sequence_counters.into_iter().for_each(|sequence_counter| {
        println!("{}", sequence_counter);
    })
}

#[derive(Debug)]
struct DigitSequenceCounter {
    sequence_length: usize,
    current_sequence: usize,
    sequence_counts: Vec<u128>,
    bitmask: usize,
    stalled_for: usize,
}

impl DigitSequenceCounter {
    pub const LARGEST_SEQUENCE_LENGTH: usize = std::mem::size_of::<usize>() << 1;

    pub fn new(sequence_length: usize) -> DigitSequenceCounter {
        if sequence_length > Self::LARGEST_SEQUENCE_LENGTH {
            panic!(
                "Cannot create a DigitSequenceCounter for a sequence length greater than {}.",
                Self::LARGEST_SEQUENCE_LENGTH
            );
        }
        let modulus = 1 << (sequence_length << 2);
        DigitSequenceCounter {
            sequence_length,
            current_sequence: 0,
            sequence_counts: vec![0; modulus],
            bitmask: modulus.wrapping_sub(1),
            stalled_for: sequence_length,
        }
    }

    pub fn process_character(&mut self, character: u8) {
        let digit = match character {
            b'0'..=b'9' => character & 0b1111,
            b'A'..=b'F' => character - b'A' + 10,
            b'a'..=b'f' => character - b'a' + 10,
            _ => {
                self.current_sequence = 0;
                self.stalled_for = self.sequence_length;
                return;
            }
        } as usize;
        self.current_sequence = ((self.current_sequence << 4) | digit) & self.bitmask;

        if self.stalled_for == 0 {
            self.sequence_counts[self.current_sequence] += 1;
        } else {
            self.stalled_for -= 1;
        }
    }
}

impl Display for DigitSequenceCounter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let nonzero_sequence_counts = self
            .sequence_counts
            .iter()
            .filter(|x| **x != 0)
            .collect::<Vec<_>>();
        write!(
            f,
            "{} {:?}",
            nonzero_sequence_counts.len(),
            nonzero_sequence_counts.as_slice()
        )
    }
}

fn process_args() -> Result<(PathBuf, usize), String> {
    let mut args = env::args_os();
    args.next();

    let path = match args.next() {
        Some(path) => match fs::canonicalize(&path) {
            Ok(path) => path,
            Err(err) => return Err(format!("bad given file path: {}", err)),
        },
        None => return Err(String::from("no given file path")),
    };

    let max_sequence_length = match args.next() {
        Some(max_sequence_length) => match max_sequence_length.into_string() {
            Ok(max_sequence_length) => match max_sequence_length.parse() {
                Ok(max_sequence_length) => {
                    if max_sequence_length <= DigitSequenceCounter::LARGEST_SEQUENCE_LENGTH {
                        max_sequence_length
                    } else {
                        return Err(format!(
                            "maximum sequence length exceeds {}",
                            DigitSequenceCounter::LARGEST_SEQUENCE_LENGTH
                        ));
                    }
                }
                Err(err) => return Err(format!("bad maximum sequence length: {}", err)),
            },
            Err(_) => return Err(String::from("bad maximum sequence length: non-UTF8 bytes")),
        },
        None => return Err(String::from("no maximum sequence length")),
    };

    if args.next().is_some() {
        Err(String::from("too many arguments"))
    } else {
        Ok((path, max_sequence_length))
    }
}
